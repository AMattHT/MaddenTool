//! Read queries. Results are fully ordered, so the same database always returns the
//! same rows in the same order.

use std::collections::BTreeMap;

use rusqlite::{Connection, params};
use serde::Serialize;

use crate::error::Result;
use crate::formats::Format;
use crate::wav::{AsOf, WavEvidence, WavInputs, classify};

/// One player in one draft year, as the highest-precedence source describes them.
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
}

struct Candidate {
    player_id: i64,
    record_id: i64,
    career_from: Option<i64>,
    row: PlayerRow,
}

/// Players of one draft year, optionally limited to one source position (case-insensitive).
///
/// Positions are the source's own vocabulary (nflverse files many safeties as `DB`).
/// Mapping them to Madden positions belongs to class generation, not storage.
pub fn players_by_year(
    conn: &Connection,
    year: i64,
    position: Option<&str>,
) -> Result<Vec<PlayerRow>> {
    let mut stmt = conn.prepare_cached(
        "SELECT sr.player_id, sr.id, p.uid, sr.full_name, sr.position, sr.college, sr.draft_year,
                sr.draft_status, sr.draft_round, sr.draft_pick, sr.draft_team, sr.league,
                sr.career_from, sr.career_to, sr.hall_of_fame, sr.wav, di.as_of, sr.format, di.synthetic
         FROM source_record sr
         JOIN player p ON p.id = sr.player_id
         JOIN dataset_import di ON di.id = sr.last_import_id
         WHERE sr.draft_year = ?1",
    )?;
    let candidates = stmt
        .query_map(params![year], |r| {
            Ok(Candidate {
                player_id: r.get(0)?,
                record_id: r.get(1)?,
                career_from: r.get(12)?,
                row: PlayerRow {
                    player_uid: r.get(2)?,
                    name: r.get(3)?,
                    position: r.get(4)?,
                    college: r.get(5)?,
                    draft_year: r.get(6)?,
                    draft_status: r.get(7)?,
                    draft_round: r.get(8)?,
                    draft_pick: r.get(9)?,
                    draft_team: r.get(10)?,
                    league: r.get(11)?,
                    career_to: r.get(13)?,
                    hall_of_fame: r.get(14)?,
                    wav: r.get(15)?,
                    wav_evidence: WavEvidence::Missing,
                    as_of: r.get(16)?,
                    source_format: r.get(17)?,
                    synthetic: r.get(18)?,
                    sources: Vec::new(),
                },
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    let mut by_player: BTreeMap<i64, Vec<Candidate>> = BTreeMap::new();
    for c in candidates {
        by_player.entry(c.player_id).or_default().push(c);
    }

    let mut rows = Vec::new();
    for (_, mut group) in by_player {
        group.sort_by_key(|c| (Format::precedence(&c.row.source_format), c.record_id));
        let mut sources: Vec<String> = group.iter().map(|c| c.row.source_format.clone()).collect();
        sources.sort();
        sources.dedup();
        let chosen = group.swap_remove(0);
        let mut row = chosen.row;
        if let Some(wanted) = position
            && !row
                .position
                .as_deref()
                .is_some_and(|p| p.eq_ignore_ascii_case(wanted))
        {
            continue;
        }
        let as_of = AsOf::parse(&row.as_of)?;
        row.wav_evidence = classify(
            &WavInputs {
                wav: row.wav,
                draft_year: row.draft_year as i32,
                career_from: chosen.career_from.map(|v| v as i32),
                career_to: row.career_to.map(|v| v as i32),
            },
            as_of,
        );
        row.sources = sources;
        rows.push(row);
    }

    let status_rank = |s: &str| match s {
        "drafted" => 0,
        "unordered" => 1,
        _ => 2,
    };
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
    pub players: i64,
    pub external_ids: i64,
    pub imports: i64,
    pub formats: Vec<FormatCount>,
}

pub fn info(conn: &Connection, schema_version: i64) -> Result<Info> {
    let count = |sql: &str| -> Result<i64> { Ok(conn.query_row(sql, [], |r| r.get(0))?) };
    let mut stmt = conn.prepare(
        "SELECT format, count(*), min(draft_year), max(draft_year),
                sum(wav IS NULL), sum(wav = 0)
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
        players: count("SELECT count(*) FROM player")?,
        external_ids: count("SELECT count(*) FROM player_external_id")?,
        imports: count("SELECT count(*) FROM dataset_import")?,
        formats,
    })
}
