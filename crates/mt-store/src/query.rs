//! Read queries. Results are fully ordered, so the same database always returns the
//! same rows in the same order.

use std::collections::BTreeMap;

use rusqlite::{Connection, params};
use serde::Serialize;

use crate::error::Result;
use crate::formats::Format;
use crate::identity;
use crate::wav::{AsOf, WavEvidence, WavInputs, classify};

/// One source record: a draft selection (or undrafted entry) as one source states it.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Selection {
    pub source_format: String,
    pub draft_year: i64,
    pub league: Option<String>,
    pub draft_status: String,
    pub draft_round: Option<i64>,
    pub draft_pick: Option<i64>,
    pub draft_team: Option<String>,
    pub name: String,
    pub position: Option<String>,
    pub college: Option<String>,
    pub wav: Option<i64>,
    pub as_of: String,
    /// Verified corrections applied to this record (see `data/corrections.json`).
    pub corrections: Vec<String>,
}

/// One person in one draft year. Values come from the representative selection: the
/// highest-precedence source, then the NFL, then the earliest round and pick. Every
/// selection of that person in that year is listed in `selections`.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PlayerRow {
    pub player_uid: String,
    pub name: String,
    pub position: Option<String>,
    pub college: Option<String>,
    pub draft_year: i64,
    pub draft_status: String,
    pub draft_round: Option<i64>,
    pub draft_pick: Option<i64>,
    pub draft_team: Option<String>,
    pub league: Option<String>,
    pub career_to: Option<i64>,
    pub hall_of_fame: Option<bool>,
    /// PFR weighted career AV as the source stated it. `None` is missing, not zero.
    pub wav: Option<i64>,
    pub wav_evidence: WavEvidence,
    /// When the source last confirmed these values.
    pub as_of: String,
    pub source_format: String,
    pub synthetic: bool,
    /// Every format holding a record of this player for this draft year.
    pub sources: Vec<String>,
    pub selections: Vec<Selection>,
}

struct Candidate {
    player_id: i64,
    record_id: i64,
    uid: String,
    career_from: Option<i64>,
    career_to: Option<i64>,
    hall_of_fame: Option<bool>,
    synthetic: bool,
    selection: Selection,
}

fn status_rank(s: &str) -> u8 {
    match s {
        "drafted" => 0,
        "unordered" => 1,
        _ => 2,
    }
}

fn load(conn: &Connection, filter: &str, value: &dyn rusqlite::ToSql) -> Result<Vec<Candidate>> {
    let sql = format!(
        "SELECT sr.player_id, sr.id, p.uid, sr.full_name, sr.position, sr.college, sr.draft_year,
                sr.draft_status, sr.draft_round, sr.draft_pick, sr.draft_team, sr.league,
                sr.career_from, sr.career_to, sr.hall_of_fame, sr.wav, di.as_of, sr.format, di.synthetic,
                (SELECT group_concat(DISTINCT rc.correction_id) FROM record_correction rc
                  WHERE rc.source_record_id = sr.id)
         FROM source_record sr
         JOIN player p ON p.id = sr.player_id
         JOIN dataset_import di ON di.id = sr.last_import_id
         WHERE {filter}"
    );
    let mut stmt = conn.prepare_cached(&sql)?;
    let rows = stmt
        .query_map(params![value], |r| {
            let corrections: Option<String> = r.get(19)?;
            Ok(Candidate {
                player_id: r.get(0)?,
                record_id: r.get(1)?,
                uid: r.get(2)?,
                career_from: r.get(12)?,
                career_to: r.get(13)?,
                hall_of_fame: r.get(14)?,
                synthetic: r.get(18)?,
                selection: Selection {
                    name: r.get(3)?,
                    position: r.get(4)?,
                    college: r.get(5)?,
                    draft_year: r.get(6)?,
                    draft_status: r.get(7)?,
                    draft_round: r.get(8)?,
                    draft_pick: r.get(9)?,
                    draft_team: r.get(10)?,
                    league: r.get(11)?,
                    wav: r.get(15)?,
                    as_of: r.get(16)?,
                    source_format: r.get(17)?,
                    corrections: corrections
                        .map(|c| c.split(',').map(str::to_string).collect())
                        .unwrap_or_default(),
                },
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}

/// Representative first: source precedence, drafted before undrafted, NFL before AFL
/// (AFL and NFL picks are separate sequences, so their rounds are not comparable),
/// earliest round and pick, then record id.
fn representative_order(c: &Candidate) -> impl Ord + use<> {
    let s = &c.selection;
    (
        Format::precedence(&s.source_format),
        status_rank(&s.draft_status),
        s.league.as_deref() != Some("NFL"),
        s.draft_round.unwrap_or(i64::MAX),
        s.draft_pick.unwrap_or(i64::MAX),
        c.record_id,
    )
}

/// Players of one draft year, optionally limited to one source position (case-insensitive).
///
/// A person appears once, however many selections they had that year (AFL and NFL in
/// the dual-draft years, or one per source). Positions are the source's own vocabulary
/// (nflverse files many safeties as `DB`); mapping them to Madden positions belongs to
/// class generation, not storage.
pub fn players_by_year(
    conn: &Connection,
    year: i64,
    position: Option<&str>,
) -> Result<Vec<PlayerRow>> {
    let mut by_player: BTreeMap<i64, Vec<Candidate>> = BTreeMap::new();
    for c in load(conn, "sr.draft_year = ?1", &year)? {
        by_player.entry(c.player_id).or_default().push(c);
    }

    let mut rows = Vec::new();
    for (_, mut group) in by_player {
        group.sort_by_key(representative_order);
        let mut sources: Vec<String> = group
            .iter()
            .map(|c| c.selection.source_format.clone())
            .collect();
        sources.sort();
        sources.dedup();
        let chosen = &group[0];
        let s = &chosen.selection;
        if let Some(wanted) = position
            && !s
                .position
                .as_deref()
                .is_some_and(|p| p.eq_ignore_ascii_case(wanted))
        {
            continue;
        }
        let wav_evidence = classify(
            &WavInputs {
                wav: s.wav,
                draft_year: s.draft_year as i32,
                career_from: chosen.career_from.map(|v| v as i32),
                career_to: chosen.career_to.map(|v| v as i32),
            },
            AsOf::parse(&s.as_of)?,
        );
        rows.push(PlayerRow {
            player_uid: chosen.uid.clone(),
            name: s.name.clone(),
            position: s.position.clone(),
            college: s.college.clone(),
            draft_year: s.draft_year,
            draft_status: s.draft_status.clone(),
            draft_round: s.draft_round,
            draft_pick: s.draft_pick,
            draft_team: s.draft_team.clone(),
            league: s.league.clone(),
            career_to: chosen.career_to,
            hall_of_fame: chosen.hall_of_fame,
            wav: s.wav,
            wav_evidence,
            as_of: s.as_of.clone(),
            source_format: s.source_format.clone(),
            synthetic: chosen.synthetic,
            sources,
            selections: group.iter().map(|c| c.selection.clone()).collect(),
        });
    }

    rows.sort_by(|a, b| {
        (
            status_rank(&a.draft_status),
            a.draft_pick.unwrap_or(i64::MAX),
            a.draft_round.unwrap_or(i64::MAX),
        )
            .cmp(&(
                status_rank(&b.draft_status),
                b.draft_pick.unwrap_or(i64::MAX),
                b.draft_round.unwrap_or(i64::MAX),
            ))
            .then_with(|| a.name.cmp(&b.name))
            .then_with(|| a.player_uid.cmp(&b.player_uid))
    });
    Ok(rows)
}

/// Everything known about one person, looked up by any uid they were ever issued.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PlayerDetail {
    pub requested_uid: String,
    pub player_uid: String,
    /// Uids of provisional players later merged into this one. All still resolve here.
    pub aliases: Vec<String>,
    pub external_ids: Vec<(String, String)>,
    /// Every selection, in draft-year order.
    pub selections: Vec<Selection>,
}

pub fn player(conn: &Connection, uid: &str) -> Result<Option<PlayerDetail>> {
    let Some(canonical) = identity::resolve(conn, uid)? else {
        return Ok(None);
    };
    let id: i64 = conn.query_row("SELECT id FROM player WHERE uid = ?1", [&canonical], |r| {
        r.get(0)
    })?;
    let aliases = conn
        .prepare("SELECT uid FROM player WHERE merged_into = ?1 ORDER BY uid")?
        .query_map([id], |r| r.get(0))?
        .collect::<rusqlite::Result<Vec<String>>>()?;
    let external_ids = conn
        .prepare(
            "SELECT namespace, value FROM player_external_id WHERE player_id = ?1 ORDER BY 1, 2",
        )?
        .query_map([id], |r| Ok((r.get(0)?, r.get(1)?)))?
        .collect::<rusqlite::Result<Vec<(String, String)>>>()?;
    let mut records = load(conn, "sr.player_id = ?1", &id)?;
    records.sort_by_key(|c| {
        (
            c.selection.draft_year,
            status_rank(&c.selection.draft_status),
            c.record_id,
        )
    });
    Ok(Some(PlayerDetail {
        requested_uid: uid.to_string(),
        player_uid: canonical,
        aliases,
        external_ids,
        selections: records.into_iter().map(|c| c.selection).collect(),
    }))
}

#[derive(Debug, Clone, Serialize)]
pub struct ImportRecord {
    pub id: i64,
    pub format: String,
    pub label: Option<String>,
    pub source_url: Option<String>,
    pub as_of: String,
    pub file_name: String,
    pub file_sha256: String,
    pub synthetic: bool,
    pub attribution: String,
    pub rows_read: i64,
    pub inserted: i64,
    pub updated: i64,
    pub unchanged: i64,
    pub rejected: i64,
    pub issues: i64,
    pub imported_at: String,
    pub tool_version: String,
}

pub fn imports(conn: &Connection) -> Result<Vec<ImportRecord>> {
    let mut stmt = conn.prepare(
        "SELECT di.id, format, label, source_url, as_of, file_name, file_sha256, synthetic, attribution,
                rows_read, inserted, updated, unchanged, rejected,
                (SELECT count(*) FROM import_issue ii WHERE ii.import_id = di.id),
                imported_at, tool_version
         FROM dataset_import di ORDER BY di.id",
    )?;
    let rows = stmt
        .query_map([], |r| {
            Ok(ImportRecord {
                id: r.get(0)?,
                format: r.get(1)?,
                label: r.get(2)?,
                source_url: r.get(3)?,
                as_of: r.get(4)?,
                file_name: r.get(5)?,
                file_sha256: r.get(6)?,
                synthetic: r.get(7)?,
                attribution: r.get(8)?,
                rows_read: r.get(9)?,
                inserted: r.get(10)?,
                updated: r.get(11)?,
                unchanged: r.get(12)?,
                rejected: r.get(13)?,
                issues: r.get(14)?,
                imported_at: r.get(15)?,
                tool_version: r.get(16)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}

#[derive(Debug, Clone, Serialize)]
pub struct FormatCount {
    pub format: String,
    pub records: i64,
    pub first_year: i64,
    pub last_year: i64,
    pub wav_missing: i64,
    pub wav_zero: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct Info {
    pub schema_version: i64,
    /// Distinct people (players not merged into another).
    pub players: i64,
    /// Provisional players merged into another; their uids still resolve.
    pub merged_aliases: i64,
    pub corrected_records: i64,
    pub external_ids: i64,
    pub imports: i64,
    pub formats: Vec<FormatCount>,
}

pub fn info(conn: &Connection, schema_version: i64) -> Result<Info> {
    let count = |sql: &str| -> Result<i64> { Ok(conn.query_row(sql, [], |r| r.get(0))?) };
    let mut stmt = conn.prepare(
        "SELECT format, count(*), min(draft_year), max(draft_year),
                sum(wav IS NULL), coalesce(sum(wav = 0), 0)
         FROM source_record GROUP BY format ORDER BY format",
    )?;
    let formats = stmt
        .query_map([], |r| {
            Ok(FormatCount {
                format: r.get(0)?,
                records: r.get(1)?,
                first_year: r.get(2)?,
                last_year: r.get(3)?,
                wav_missing: r.get(4)?,
                wav_zero: r.get(5)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(Info {
        schema_version,
        players: count("SELECT count(*) FROM player WHERE merged_into IS NULL")?,
        merged_aliases: count("SELECT count(*) FROM player WHERE merged_into IS NOT NULL")?,
        corrected_records: count("SELECT count(DISTINCT source_record_id) FROM record_correction")?,
        external_ids: count("SELECT count(*) FROM player_external_id")?,
        imports: count("SELECT count(*) FROM dataset_import")?,
        formats,
    })
}
