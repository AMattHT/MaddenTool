//! Import one dataset file in a single transaction.
//!
//! Identity rules, in order, for a row the database has not seen before:
//!
//! 1. Same format and natural key as an existing record: that record is updated in
//!    place and keeps its player. Identity never moves on re-import.
//! 2. An external ID (PFR, then GSIS, then CFB) already belongs to a player: link to
//!    that player (`external_id`). nflverse lists Bo Jackson's 1986 and 1987
//!    selections under one PFR ID; they become one player with two records.
//! 3. A drafted row whose year, league and overall pick match exactly one record from
//!    a *different* format with a compatible surname: link (`draft_slot_surname`).
//! 4. Otherwise a new player (`new`).
//!
//! Records are never deleted. A record missing from a later import keeps its values
//! and its `last_import_id` shows when a source last confirmed it.

use std::collections::HashMap;
use std::path::PathBuf;

use rusqlite::{OptionalExtension, Transaction, params};
use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::error::{Result, StoreError};
use crate::formats::{Format, MAPPING_VERSION, Mapped, Row};
use crate::names::{NameParts, surnames_compatible};
use crate::wav::AsOf;

pub struct ImportOptions {
    pub format: Format,
    pub file: PathBuf,
    pub as_of: AsOf,
    pub source_url: Option<String>,
    pub label: Option<String>,
    pub synthetic: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Issue {
    pub line: Option<u64>,
    pub kind: String,
    pub detail: String,
}

#[derive(Debug, Default, Clone, Serialize)]
pub struct ImportSummary {
    pub import_id: i64,
    pub file_sha256: String,
    pub rows_read: u64,
    pub inserted: u64,
    pub updated: u64,
    pub unchanged: u64,
    pub rejected: u64,
    pub linked_by_external_id: u64,
    pub linked_by_draft_slot: u64,
    pub issues: Vec<Issue>,
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn sha256_hex(data: &[u8]) -> String {
    hex(&Sha256::digest(data))
}

/// UTC RFC 3339 timestamp without external crates (civil-from-days, Howard Hinnant).
fn now_utc() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let (days, rem) = (secs.div_euclid(86_400), secs.rem_euclid(86_400));
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}Z",
        rem / 3600,
        rem % 3600 / 60,
        rem % 60
    )
}

enum Target {
    Existing(i64),
    New {
        player_id: i64,
        method: &'static str,
    },
}

struct Importer<'t, 'c> {
    tx: &'t Transaction<'c>,
    format: Format,
    import_id: i64,
    summary: ImportSummary,
}

impl Importer<'_, '_> {
    fn issue(&mut self, line: Option<u64>, kind: &str, detail: String) -> Result<()> {
        self.tx.execute(
            "INSERT INTO import_issue (import_id, line, kind, detail) VALUES (?1, ?2, ?3, ?4)",
            params![self.import_id, line.map(|l| l as i64), kind, detail],
        )?;
        self.summary.issues.push(Issue {
            line,
            kind: kind.to_string(),
            detail,
        });
        Ok(())
    }

    fn owner_of(&self, namespace: &str, value: &str) -> Result<Option<i64>> {
        Ok(self
            .tx
            .query_row(
                "SELECT player_id FROM player_external_id WHERE namespace = ?1 AND value = ?2",
                params![namespace, value],
                |r| r.get(0),
            )
            .optional()?)
    }

    fn attach_external_ids(&mut self, line: u64, player_id: i64, row: &Mapped) -> Result<()> {
        for (namespace, value) in &row.external_ids {
            match self.owner_of(namespace, value)? {
                None => {
                    self.tx.execute(
                        "INSERT INTO player_external_id (namespace, value, player_id, first_import_id)
                         VALUES (?1, ?2, ?3, ?4)",
                        params![namespace, value, player_id, self.import_id],
                    )?;
                }
                Some(owner) if owner == player_id => {}
                Some(_) => self.issue(
                    Some(line),
                    "external_id_conflict",
                    format!(
                        "{namespace} id {value} already belongs to another player; not reassigned"
                    ),
                )?,
            }
        }
        Ok(())
    }

    /// Choose the player for a record the database has not seen.
    fn link(&mut self, line: u64, row: &Mapped) -> Result<Option<(i64, &'static str)>> {
        let mut owners: Vec<i64> = Vec::new();
        for (namespace, value) in &row.external_ids {
            if let Some(owner) = self.owner_of(namespace, value)?
                && !owners.contains(&owner)
            {
                owners.push(owner);
            }
        }
        if let Some(&first) = owners.first() {
            if owners.len() > 1 {
                self.issue(
                    Some(line),
                    "external_id_conflict",
                    format!(
                        "external ids point at {} different players; linked by the first",
                        owners.len()
                    ),
                )?;
            }
            return Ok(Some((first, "external_id")));
        }

        let (Some(league), Some(pick), "drafted") = (&row.league, row.draft_pick, row.draft_status)
        else {
            return Ok(None);
        };
        let mut stmt = self.tx.prepare_cached(
            "SELECT sr.player_id, sr.full_name, sr.last_name FROM source_record sr
             WHERE sr.format <> ?1 AND sr.draft_year = ?2 AND sr.league = ?3 AND sr.draft_pick = ?4
               AND sr.draft_status = 'drafted'
               AND NOT EXISTS (SELECT 1 FROM source_record mine
                               WHERE mine.player_id = sr.player_id AND mine.format = ?1
                                 AND mine.draft_year = ?2)
             ORDER BY sr.id",
        )?;
        let candidates: Vec<(i64, String, Option<String>)> = stmt
            .query_map(
                params![self.format.id(), row.draft_year, league, pick],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )?
            .collect::<rusqlite::Result<_>>()?;
        drop(stmt);
        let me = NameParts {
            full: &row.full_name,
            last: row.last_name.as_deref(),
        };
        let matching: Vec<i64> = candidates
            .iter()
            .filter(|(_, full, last)| {
                surnames_compatible(
                    &me,
                    &NameParts {
                        full,
                        last: last.as_deref(),
                    },
                )
            })
            .map(|(id, _, _)| *id)
            .collect();
        match matching.as_slice() {
            [one] => Ok(Some((*one, "draft_slot_surname"))),
            [] if candidates.is_empty() => Ok(None),
            [] => {
                let names: Vec<&str> = candidates.iter().map(|(_, n, _)| n.as_str()).collect();
                self.issue(
                    Some(line),
                    "slot_name_mismatch",
                    format!(
                        "{} {league} pick {pick}: {:?} does not match {}; kept as a separate player",
                        row.draft_year,
                        row.full_name,
                        names.join(", ")
                    ),
                )?;
                Ok(None)
            }
            many => {
                self.issue(
                    Some(line),
                    "slot_ambiguous",
                    format!(
                        "{} candidates share this draft slot and surname; kept separate",
                        many.len()
                    ),
                )?;
                Ok(None)
            }
        }
    }

    fn new_player(&self, row: &Mapped) -> Result<i64> {
        // Derived from the first record's identity, so rebuilding a database from the
        // same imports in the same order reproduces every uid.
        let digest = sha256_hex(format!("{}\0{}", self.format.id(), row.natural_key).as_bytes());
        let uid = format!("mt-{}", &digest[..16]);
        self.tx.execute(
            "INSERT INTO player (uid, created_import_id) VALUES (?1, ?2)",
            params![uid, self.import_id],
        )?;
        Ok(self.tx.last_insert_rowid())
    }

    /// Insert a new record, or overwrite an existing one's values (never its identity).
    fn write_record(&self, target: Target, row: &Mapped, hash: &str, raw: &str) -> Result<()> {
        let id = match target {
            Target::Existing(id) => id,
            Target::New { player_id, method } => {
                // Identity columns first; the shared UPDATE below fills every value column.
                self.tx.execute(
                    "INSERT INTO source_record (format, natural_key, player_id, link_method,
                        first_import_id, last_import_id, content_sha256, raw_json, full_name,
                        draft_year, draft_status)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?5, ?6, ?7, ?8, ?9, ?10)",
                    params![
                        self.format.id(),
                        row.natural_key,
                        player_id,
                        method,
                        self.import_id,
                        hash,
                        raw,
                        row.full_name,
                        row.draft_year,
                        row.draft_status
                    ],
                )?;
                self.tx.last_insert_rowid()
            }
        };
        let values = params![
            hash,
            row.full_name,
            row.first_name,
            row.last_name,
            row.position,
            row.college,
            row.draft_year,
            row.draft_status,
            row.draft_round,
            row.draft_pick,
            row.draft_team,
            row.league,
            row.career_from,
            row.career_to,
            row.games,
            row.seasons_started,
            row.pro_bowls,
            row.all_pro_first,
            row.hall_of_fame,
            row.wav,
            row.career_av,
            row.draft_av,
            raw,
            self.import_id,
        ];
        const COLUMNS: &str =
            "content_sha256 = ?1, full_name = ?2, first_name = ?3, last_name = ?4,
            position = ?5, college = ?6, draft_year = ?7, draft_status = ?8, draft_round = ?9,
            draft_pick = ?10, draft_team = ?11, league = ?12, career_from = ?13, career_to = ?14,
            games = ?15, seasons_started = ?16, pro_bowls = ?17, all_pro_first = ?18,
            hall_of_fame = ?19, wav = ?20, career_av = ?21, draft_av = ?22, raw_json = ?23,
            last_import_id = ?24";
        self.tx.execute(
            &format!("UPDATE source_record SET {COLUMNS} WHERE id = {id}"),
            values,
        )?;
        Ok(())
    }

    fn import_row(&mut self, line: u64, row: Mapped, raw: String) -> Result<()> {
        let hash = sha256_hex(format!("{MAPPING_VERSION}\0{raw}").as_bytes());
        let existing: Option<(i64, i64, String)> = self
            .tx
            .query_row(
                "SELECT id, player_id, content_sha256 FROM source_record
                 WHERE format = ?1 AND natural_key = ?2",
                params![self.format.id(), row.natural_key],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .optional()?;
        match existing {
            Some((id, _, old_hash)) if old_hash == hash => {
                self.tx.execute(
                    "UPDATE source_record SET last_import_id = ?1 WHERE id = ?2",
                    params![self.import_id, id],
                )?;
                self.summary.unchanged += 1;
            }
            Some((id, player_id, _)) => {
                self.write_record(Target::Existing(id), &row, &hash, &raw)?;
                self.attach_external_ids(line, player_id, &row)?;
                self.summary.updated += 1;
            }
            None => {
                let (player_id, method) = match self.link(line, &row)? {
                    Some(link) => link,
                    None => (self.new_player(&row)?, "new"),
                };
                match method {
                    "external_id" => self.summary.linked_by_external_id += 1,
                    "draft_slot_surname" => self.summary.linked_by_draft_slot += 1,
                    _ => {}
                }
                self.write_record(Target::New { player_id, method }, &row, &hash, &raw)?;
                self.attach_external_ids(line, player_id, &row)?;
                self.summary.inserted += 1;
            }
        }
        Ok(())
    }
}

/// Run an import inside `tx`. The caller commits; any error leaves the database unchanged.
pub fn run(tx: &Transaction<'_>, options: &ImportOptions) -> Result<ImportSummary> {
    let bytes = std::fs::read(&options.file)?;
    let file_sha256 = sha256_hex(&bytes);
    let body = bytes.strip_prefix(b"\xEF\xBB\xBF").unwrap_or(&bytes);
    let mut reader = csv::ReaderBuilder::new().flexible(true).from_reader(body);

    let headers: Vec<String> = reader
        .headers()?
        .iter()
        .map(|h| h.trim().to_string())
        .collect();
    let mut columns = HashMap::new();
    for (i, h) in headers.iter().enumerate() {
        if columns.insert(h.clone(), i).is_some() {
            return Err(StoreError::InvalidInput(format!(
                "duplicate column {h:?}; nothing was imported"
            )));
        }
    }
    let missing: Vec<String> = options
        .format
        .required_columns()
        .iter()
        .filter(|c| !columns.contains_key(**c))
        .map(|c| c.to_string())
        .collect();
    if !missing.is_empty() {
        return Err(StoreError::MissingColumns {
            format: options.format.id(),
            missing,
        });
    }

    let file_name = options
        .file
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    tx.execute(
        "INSERT INTO dataset_import (format, label, source_url, as_of, file_name, file_sha256,
            synthetic, attribution, imported_at, tool_version)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![
            options.format.id(),
            options.label,
            options.source_url,
            options.as_of.to_string(),
            file_name,
            file_sha256,
            options.synthetic,
            options.format.attribution(),
            now_utc(),
            env!("CARGO_PKG_VERSION"),
        ],
    )?;
    let mut importer = Importer {
        tx,
        format: options.format,
        import_id: tx.last_insert_rowid(),
        summary: ImportSummary {
            file_sha256,
            ..Default::default()
        },
    };
    importer.summary.import_id = importer.import_id;

    let mut seen_keys: HashMap<String, u64> = HashMap::new();
    for result in reader.records() {
        importer.summary.rows_read += 1;
        let values = match result {
            Ok(v) => v,
            Err(e) => {
                let line = e.position().map(|p| p.line());
                importer.issue(line, "malformed_row", e.to_string())?;
                importer.summary.rejected += 1;
                continue;
            }
        };
        let line = values.position().map_or(0, |p| p.line());
        if values.len() != headers.len() {
            importer.issue(
                Some(line),
                "malformed_row",
                format!("{} cells for {} columns", values.len(), headers.len()),
            )?;
            importer.summary.rejected += 1;
            continue;
        }
        let row = Row {
            columns: &columns,
            values: &values,
        };
        let mapped = match options.format.map_row(&row) {
            Ok(m) => m,
            Err(detail) => {
                importer.issue(Some(line), "invalid_row", detail)?;
                importer.summary.rejected += 1;
                continue;
            }
        };
        if let Some(first) = seen_keys.get(&mapped.natural_key) {
            importer.issue(
                Some(line),
                "duplicate_in_file",
                format!(
                    "same identity as line {first} ({}); later row ignored",
                    mapped.natural_key
                ),
            )?;
            importer.summary.rejected += 1;
            continue;
        }
        seen_keys.insert(mapped.natural_key.clone(), line);
        let raw: serde_json::Map<String, serde_json::Value> = headers
            .iter()
            .zip(values.iter())
            .map(|(h, v)| (h.clone(), serde_json::Value::String(v.to_string())))
            .collect();
        importer.import_row(line, mapped, serde_json::to_string(&raw)?)?;
    }

    let s = &importer.summary;
    tx.execute(
        "UPDATE dataset_import SET rows_read = ?1, inserted = ?2, updated = ?3, unchanged = ?4,
            rejected = ?5 WHERE id = ?6",
        params![
            s.rows_read as i64,
            s.inserted as i64,
            s.updated as i64,
            s.unchanged as i64,
            s.rejected as i64,
            importer.import_id
        ],
    )?;
    Ok(importer.summary)
}
