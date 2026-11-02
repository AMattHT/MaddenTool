//! Identity reconciliation: when two provisional players are one person, merge them.
//!
//! A person can have several draft selections: the AFL and the NFL in one year
//! (1960-1966), or the NFL in different years. Import links what it can row by row;
//! this pass then looks across all stored records. It merges only on evidence, never
//! on a name alone, and reports every candidate it cannot settle as a [`Conflict`].
//!
//! Rules:
//!
//! - **Draft slot across sources** (`draft_slot_surname`): one record in each of two
//!   formats at the same year, league and overall pick, with compatible surnames, and
//!   no other compatible record at that slot.
//! - **AFL/NFL dual draft, 1960-1966**, records of one format and year, one per league:
//!   - `dual_draft_name_college`: same full name and college, careers not conflicting.
//!   - `dual_draft_name_variant_college`: as above, but the given names are variants
//!     ([`given_names_compatible`]: "Bob"/"Robert", "Emery"/"Emory", "Billy Joe"/"Billy").
//!   - `dual_draft_career_footprint`: same surname and college, unrelated given names,
//!     and the same distinctive career (seasons, starts, honours, wAV) on both rows.
//!   - Each side must have exactly one such partner; otherwise the pair is ambiguous.
//!
//! A merge is refused when the two players hold different IDs in one namespace, or
//! would end up with two selections in one league and year from one source.
//!
//! The older player (lower id) survives. The absorbed one keeps its uid as an alias.

use std::collections::{BTreeMap, BTreeSet};
use std::ops::RangeInclusive;

use rusqlite::{Connection, OptionalExtension, params};
use serde::Serialize;
use serde_json::json;

use crate::error::Result;
use crate::names::{NameParts, given_names_compatible, name_key, surnames_compatible};

/// Seasons with separate AFL and NFL drafts. From 1967 there was one common draft.
pub const DUAL_DRAFT_YEARS: RangeInclusive<i64> = 1960..=1966;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Merge {
    pub from_uid: String,
    pub into_uid: String,
    pub reason: String,
    pub evidence: serde_json::Value,
}

/// A possible identity the data cannot settle. Not stored: recomputed from current data.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Conflict {
    pub kind: String,
    pub draft_year: i64,
    pub detail: String,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct Reconciliation {
    pub merges: Vec<Merge>,
    pub conflicts: Vec<Conflict>,
}

#[derive(Debug, Clone)]
struct Rec {
    id: i64,
    player_id: i64,
    format: String,
    natural_key: String,
    full_name: String,
    first_name: Option<String>,
    last_name: Option<String>,
    college: Option<String>,
    year: i64,
    league: Option<String>,
    round: Option<i64>,
    pick: Option<i64>,
    career: [Option<i64>; 6],
}

impl Rec {
    fn names(&self) -> NameParts<'_> {
        NameParts {
            full: &self.full_name,
            last: self.last_name.as_deref(),
        }
    }

    fn surname_key(&self) -> String {
        match &self.last_name {
            Some(l) => name_key(l),
            None => self
                .full_name
                .split_whitespace()
                .last()
                .map(name_key)
                .unwrap_or_default(),
        }
    }

    fn college_key(&self) -> String {
        self.college.as_deref().map(name_key).unwrap_or_default()
    }

    fn career_blank(&self) -> bool {
        self.career.iter().all(|v| matches!(v, None | Some(0)))
    }

    /// A career specific enough that two unrelated rows are unlikely to share it.
    fn career_distinctive(&self) -> bool {
        let [_, to, ap, pb, st, wav] = self.career;
        to.is_some() && [ap, pb, st, wav].iter().any(|v| v.unwrap_or(0) >= 1)
    }

    fn describe(&self) -> serde_json::Value {
        json!({
            "record_id": self.id, "format": self.format, "natural_key": self.natural_key, "name": self.full_name,
            "college": self.college, "league": self.league, "round": self.round, "pick": self.pick,
            "career": { "from": self.career[0], "to": self.career[1], "all_pro_first": self.career[2],
                        "pro_bowls": self.career[3], "seasons_started": self.career[4], "wav": self.career[5] },
        })
    }

    fn short(&self) -> String {
        format!(
            "{} ({}, {} r{} p{})",
            self.full_name,
            self.college.as_deref().unwrap_or("no college"),
            self.league.as_deref().unwrap_or("-"),
            self.round.map_or("-".into(), |r| r.to_string()),
            self.pick.map_or("-".into(), |p| p.to_string()),
        )
    }
}

fn load(conn: &Connection) -> Result<Vec<Rec>> {
    let mut stmt = conn.prepare(
        "SELECT id, player_id, format, natural_key, full_name, last_name, college, draft_year,
                first_name,
                league, draft_round, draft_pick,
                career_from, career_to, all_pro_first, pro_bowls, seasons_started, wav
         FROM source_record
         WHERE draft_status IN ('drafted', 'unordered') AND league IS NOT NULL
         ORDER BY id",
    )?;
    let recs = stmt
        .query_map([], |r| {
            Ok(Rec {
                id: r.get(0)?,
                player_id: r.get(1)?,
                format: r.get(2)?,
                natural_key: r.get(3)?,
                full_name: r.get(4)?,
                last_name: r.get(5)?,
                first_name: r.get(8)?,
                college: r.get(6)?,
                year: r.get(7)?,
                league: r.get(9)?,
                round: r.get(10)?,
                pick: r.get(11)?,
                career: [
                    r.get(12)?,
                    r.get(13)?,
                    r.get(14)?,
                    r.get(15)?,
                    r.get(16)?,
                    r.get(17)?,
                ],
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(recs)
}

struct Proposal {
    a: usize,
    b: usize,
    reason: &'static str,
}

fn conflict(out: &mut Vec<Conflict>, kind: &str, year: i64, detail: String) {
    out.push(Conflict {
        kind: kind.to_string(),
        draft_year: year,
        detail,
    });
}

/// Same slot, different formats, compatible surnames, one each.
fn slot_proposals(recs: &[Rec], proposals: &mut Vec<Proposal>, conflicts: &mut Vec<Conflict>) {
    let mut slots: BTreeMap<(i64, &str, i64), Vec<usize>> = BTreeMap::new();
    for (i, r) in recs.iter().enumerate() {
        if let (Some(league), Some(pick)) = (&r.league, r.pick) {
            slots
                .entry((r.year, league.as_str(), pick))
                .or_default()
                .push(i);
        }
    }
    for ((year, league, pick), members) in slots {
        let mut partners: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
        for (x, &i) in members.iter().enumerate() {
            for &j in &members[x + 1..] {
                let (a, b) = (&recs[i], &recs[j]);
                if a.format != b.format && surnames_compatible(&a.names(), &b.names()) {
                    partners.entry(i).or_default().push(j);
                    partners.entry(j).or_default().push(i);
                }
            }
        }
        for (&i, list) in &partners {
            for &j in list {
                if i > j || recs[i].player_id == recs[j].player_id {
                    continue;
                }
                let unique = |k: usize, other: usize| {
                    partners[&k]
                        .iter()
                        .filter(|&&p| recs[p].format == recs[other].format)
                        .count()
                        == 1
                };
                if unique(i, j) && unique(j, i) {
                    proposals.push(Proposal {
                        a: i,
                        b: j,
                        reason: "draft_slot_surname",
                    });
                } else {
                    conflict(
                        conflicts,
                        "slot_ambiguous",
                        year,
                        format!(
                            "{year} {league} pick {pick}: {} and {} share the slot with another compatible record",
                            recs[i].short(),
                            recs[j].short()
                        ),
                    );
                }
            }
        }
    }
}

enum Pair {
    Eligible(&'static str),
    Conflict(&'static str),
    Unrelated,
}

/// Records arrive grouped by surname, so only given names, college and career decide.
fn classify_dual(a: &Rec, n: &Rec) -> Pair {
    let exact = name_key(&a.full_name) == name_key(&n.full_name);
    let variant = !exact
        && given_names_compatible(
            a.first_name.as_deref().unwrap_or(""),
            n.first_name.as_deref().unwrap_or(""),
        );
    let same_college = !a.college_key().is_empty() && a.college_key() == n.college_key();
    let careers_conflict = !a.career_blank() && !n.career_blank() && a.career != n.career;
    let same_footprint = a.career == n.career && a.career_distinctive();
    match (exact, variant, same_college) {
        (true, _, true) | (_, true, true) if careers_conflict => {
            Pair::Conflict("dual_draft_career_conflict")
        }
        (true, _, true) => Pair::Eligible("dual_draft_name_college"),
        (_, true, true) => Pair::Eligible("dual_draft_name_variant_college"),
        (true, _, false) => Pair::Conflict("dual_draft_college_conflict"),
        // A name variant at another school is a different man; nothing to report.
        (_, true, false) => Pair::Unrelated,
        (false, false, true) if same_footprint => Pair::Eligible("dual_draft_career_footprint"),
        (false, false, true) => Pair::Conflict("dual_draft_alias_unconfirmed"),
        (false, false, false) => Pair::Unrelated,
    }
}

/// Records of one format and year sharing a surname: (AFL indexes, NFL indexes).
type DualGroups<'r> = BTreeMap<(&'r str, i64, String), (Vec<usize>, Vec<usize>)>;

/// AFL and NFL selections of one person in one dual-draft year, within one source.
fn dual_proposals(recs: &[Rec], proposals: &mut Vec<Proposal>, conflicts: &mut Vec<Conflict>) {
    let mut groups: DualGroups<'_> = BTreeMap::new();
    for (i, r) in recs.iter().enumerate() {
        if !DUAL_DRAFT_YEARS.contains(&r.year) {
            continue;
        }
        let entry = groups
            .entry((r.format.as_str(), r.year, r.surname_key()))
            .or_default();
        match r.league.as_deref() {
            Some("AFL") => entry.0.push(i),
            Some("NFL") => entry.1.push(i),
            _ => {}
        }
    }
    for ((_, year, _), (afl, nfl)) in groups {
        let mut eligible: BTreeMap<usize, Vec<(usize, &'static str)>> = BTreeMap::new();
        for &a in &afl {
            for &n in &nfl {
                match classify_dual(&recs[a], &recs[n]) {
                    Pair::Eligible(reason) => {
                        eligible.entry(a).or_default().push((n, reason));
                        eligible.entry(n).or_default().push((a, reason));
                    }
                    Pair::Conflict(kind) if recs[a].player_id != recs[n].player_id => conflict(
                        conflicts,
                        kind,
                        year,
                        format!("{} and {}", recs[a].short(), recs[n].short()),
                    ),
                    _ => {}
                }
            }
        }
        for &a in &afl {
            let Some(list) = eligible.get(&a) else {
                continue;
            };
            for &(n, reason) in list {
                if recs[a].player_id == recs[n].player_id {
                    continue;
                }
                if list.len() == 1 && eligible[&n].len() == 1 {
                    proposals.push(Proposal { a, b: n, reason });
                } else {
                    conflict(
                        conflicts,
                        "dual_draft_ambiguous",
                        year,
                        format!(
                            "{} has {} possible partners, {} has {}",
                            recs[a].short(),
                            list.len(),
                            recs[n].short(),
                            eligible[&n].len()
                        ),
                    );
                }
            }
        }
    }
}

fn uid(conn: &Connection, player_id: i64) -> Result<String> {
    Ok(
        conn.query_row("SELECT uid FROM player WHERE id = ?1", [player_id], |r| {
            r.get(0)
        })?,
    )
}

/// Why two players cannot be one, if anything in the store already says so.
fn refusal(conn: &Connection, a: i64, b: i64) -> Result<Option<String>> {
    let clash: Option<String> = conn
        .query_row(
            "SELECT namespace FROM player_external_id WHERE player_id IN (?1, ?2)
             GROUP BY namespace HAVING count(DISTINCT value) > 1 LIMIT 1",
            params![a, b],
            |r| r.get(0),
        )
        .optional()?;
    if let Some(ns) = clash {
        return Ok(Some(format!("the players hold different {ns} ids")));
    }
    let twice: Option<String> = conn
        .query_row(
            "SELECT format || ' ' || draft_year || ' ' || league FROM source_record
             WHERE player_id IN (?1, ?2) AND league IS NOT NULL AND draft_status <> 'undrafted'
             GROUP BY format, draft_year, league HAVING count(*) > 1 LIMIT 1",
            params![a, b],
            |r| r.get(0),
        )
        .optional()?;
    Ok(twice.map(|t| format!("one person would have two selections in {t}")))
}

fn apply(
    conn: &Connection,
    import_id: Option<i64>,
    recs: &[Rec],
    p: &Proposal,
) -> Result<std::result::Result<Merge, String>> {
    let (pa, pb) = (recs[p.a].player_id, recs[p.b].player_id);
    if let Some(why) = refusal(conn, pa, pb)? {
        return Ok(Err(why));
    }
    let (into, from) = (pa.min(pb), pa.max(pb));
    let moved: Vec<i64> = conn
        .prepare("SELECT id FROM source_record WHERE player_id = ?1 ORDER BY id")?
        .query_map([from], |r| r.get(0))?
        .collect::<rusqlite::Result<_>>()?;
    let evidence =
        json!({ "rule": p.reason, "records": [recs[p.a].describe(), recs[p.b].describe()] });
    conn.execute(
        "UPDATE source_record SET player_id = ?1 WHERE player_id = ?2",
        params![into, from],
    )?;
    conn.execute(
        "UPDATE player_external_id SET player_id = ?1 WHERE player_id = ?2",
        params![into, from],
    )?;
    conn.execute(
        "UPDATE player SET merged_into = ?1 WHERE id = ?2 OR merged_into = ?2",
        params![into, from],
    )?;
    conn.execute(
        "INSERT INTO player_merge (from_player_id, into_player_id, import_id, reason, evidence, moved_records)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            from,
            into,
            import_id,
            p.reason,
            evidence.to_string(),
            serde_json::to_string(&moved)?
        ],
    )?;
    Ok(Ok(Merge {
        from_uid: uid(conn, from)?,
        into_uid: uid(conn, into)?,
        reason: p.reason.to_string(),
        evidence,
    }))
}

fn propose(recs: &[Rec]) -> (Vec<Proposal>, Vec<Conflict>) {
    let mut proposals = Vec::new();
    let mut conflicts = Vec::new();
    slot_proposals(recs, &mut proposals, &mut conflicts);
    dual_proposals(recs, &mut proposals, &mut conflicts);
    (proposals, conflicts)
}

/// Unsettled identities in the current data, without changing anything.
pub fn conflicts(conn: &Connection) -> Result<Vec<Conflict>> {
    let (_, mut conflicts) = propose(&load(conn)?);
    dedup(&mut conflicts);
    Ok(conflicts)
}

fn dedup(conflicts: &mut Vec<Conflict>) {
    let mut seen = BTreeSet::new();
    conflicts.retain(|c| seen.insert((c.kind.clone(), c.draft_year, c.detail.clone())));
}

/// Merge every pair the rules support, repeating until nothing changes. Run inside the
/// caller's transaction. Returns the merges made and the conflicts that remain.
pub fn reconcile(conn: &Connection, import_id: Option<i64>) -> Result<Reconciliation> {
    let mut merges = Vec::new();
    let mut refused: Vec<Conflict> = Vec::new();
    loop {
        let recs = load(conn)?;
        let (proposals, _) = propose(&recs);
        let mut changed = false;
        let mut merged_away: BTreeSet<i64> = BTreeSet::new();
        for p in &proposals {
            // A player absorbed earlier in this round has moved; the next round sees it.
            if merged_away.contains(&recs[p.a].player_id)
                || merged_away.contains(&recs[p.b].player_id)
            {
                continue;
            }
            match apply(conn, import_id, &recs, p)? {
                Ok(m) => {
                    merged_away.insert(recs[p.a].player_id.max(recs[p.b].player_id));
                    merges.push(m);
                    changed = true;
                }
                Err(why) => conflict(
                    &mut refused,
                    "merge_refused",
                    recs[p.a].year,
                    format!("{} and {}: {why}", recs[p.a].short(), recs[p.b].short()),
                ),
            }
        }
        if !changed {
            let mut conflicts = conflicts(conn)?;
            conflicts.extend(refused);
            dedup(&mut conflicts);
            return Ok(Reconciliation { merges, conflicts });
        }
    }
}

/// The surviving uid for any uid ever issued, or None if it was never issued.
pub fn resolve(conn: &Connection, uid: &str) -> Result<Option<String>> {
    let mut current: Option<(i64, Option<i64>)> = conn
        .query_row(
            "SELECT id, merged_into FROM player WHERE uid = ?1",
            [uid],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?;
    // Merges keep chains flat, but follow a few hops in case an older database did not.
    for _ in 0..16 {
        match current {
            None => return Ok(None),
            Some((id, None)) => return Ok(Some(self::uid(conn, id)?)),
            Some((_, Some(next))) => {
                current = conn
                    .query_row(
                        "SELECT id, merged_into FROM player WHERE id = ?1",
                        [next],
                        |r| Ok((r.get(0)?, r.get(1)?)),
                    )
                    .optional()?;
            }
        }
    }
    Ok(None)
}
