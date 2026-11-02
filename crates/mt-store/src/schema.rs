//! Schema and migrations.
//!
//! `PRAGMA user_version` holds the applied migration count. `PRAGMA application_id`
//! marks the file as ours, so the tool refuses to migrate an unrelated SQLite file.
//! Migrations only ever append; an applied migration is never edited.

use rusqlite::Connection;

use crate::error::{Result, StoreError};

/// "MTDB" as a big-endian 32-bit integer.
pub const APPLICATION_ID: i64 = 0x4D54_4442;

const V1: &str = r#"
-- One row per import run: the provenance every stored value points back to.
CREATE TABLE dataset_import (
    id           INTEGER PRIMARY KEY,
    format       TEXT    NOT NULL,
    label        TEXT,
    source_url   TEXT,
    as_of        TEXT    NOT NULL,           -- YYYY-MM-DD the source data was current
    file_name    TEXT    NOT NULL,
    file_sha256  TEXT    NOT NULL,
    synthetic    INTEGER NOT NULL CHECK (synthetic IN (0, 1)),
    attribution  TEXT    NOT NULL,
    rows_read    INTEGER NOT NULL DEFAULT 0,
    inserted     INTEGER NOT NULL DEFAULT 0,
    updated      INTEGER NOT NULL DEFAULT 0,
    unchanged    INTEGER NOT NULL DEFAULT 0,
    rejected     INTEGER NOT NULL DEFAULT 0,
    imported_at  TEXT    NOT NULL,           -- UTC, RFC 3339
    tool_version TEXT    NOT NULL
);

-- A real person, independent of any source. The uid never changes once issued.
CREATE TABLE player (
    id                INTEGER PRIMARY KEY,
    uid               TEXT    NOT NULL UNIQUE,
    created_import_id INTEGER NOT NULL REFERENCES dataset_import(id)
);

-- Identifiers other systems use for the same person.
CREATE TABLE player_external_id (
    namespace       TEXT    NOT NULL CHECK (namespace IN ('pfr', 'gsis', 'cfb')),
    value           TEXT    NOT NULL,
    player_id       INTEGER NOT NULL REFERENCES player(id),
    first_import_id INTEGER NOT NULL REFERENCES dataset_import(id),
    PRIMARY KEY (namespace, value)
);
CREATE INDEX player_external_id_player ON player_external_id(player_id);

-- One row per source row (for both supported formats: one draft selection or
-- undrafted entry). Values are as the source stated them; NULL means the source
-- left the value blank, which is different from zero.
CREATE TABLE source_record (
    id              INTEGER PRIMARY KEY,
    format          TEXT    NOT NULL,
    natural_key     TEXT    NOT NULL,
    player_id       INTEGER NOT NULL REFERENCES player(id),
    link_method     TEXT    NOT NULL CHECK (link_method IN ('new', 'external_id', 'draft_slot_surname')),
    first_import_id INTEGER NOT NULL REFERENCES dataset_import(id),
    last_import_id  INTEGER NOT NULL REFERENCES dataset_import(id),
    content_sha256  TEXT    NOT NULL,
    full_name       TEXT    NOT NULL,
    first_name      TEXT,
    last_name       TEXT,
    position        TEXT,
    college         TEXT,
    draft_year      INTEGER NOT NULL,
    draft_status    TEXT    NOT NULL CHECK (draft_status IN ('drafted', 'undrafted', 'unordered')),
    draft_round     INTEGER,
    draft_pick      INTEGER,
    draft_team      TEXT,
    league          TEXT,
    career_from     INTEGER,
    career_to       INTEGER,
    games           INTEGER,
    seasons_started INTEGER,
    pro_bowls       INTEGER,
    all_pro_first   INTEGER,
    hall_of_fame    INTEGER CHECK (hall_of_fame IN (0, 1)),
    wav             INTEGER,                 -- PFR weighted career AV
    career_av       INTEGER,                 -- PFR unweighted career AV
    draft_av        INTEGER,                 -- PFR AV accrued for the drafting team
    raw_json        TEXT    NOT NULL,        -- the full source row, unknown columns included
    UNIQUE (format, natural_key)
);
CREATE INDEX source_record_year ON source_record(draft_year);
CREATE INDEX source_record_slot ON source_record(draft_year, league, draft_pick);
CREATE INDEX source_record_player ON source_record(player_id);

-- Rows an import refused or could only partly reconcile, with the source line.
CREATE TABLE import_issue (
    id        INTEGER PRIMARY KEY,
    import_id INTEGER NOT NULL REFERENCES dataset_import(id),
    line      INTEGER,
    kind      TEXT    NOT NULL,
    detail    TEXT    NOT NULL
);
CREATE INDEX import_issue_import ON import_issue(import_id);
"#;

/// Identity reconciliation. Two provisional players found to be one person are merged:
/// records and external IDs move to the surviving player, and the absorbed player stays
/// as an alias (`merged_into`) so every uid ever issued still resolves.
const V2: &str = r#"
ALTER TABLE player ADD COLUMN merged_into INTEGER REFERENCES player(id);

CREATE TABLE player_merge (
    id             INTEGER PRIMARY KEY,
    from_player_id INTEGER NOT NULL REFERENCES player(id),
    into_player_id INTEGER NOT NULL REFERENCES player(id),
    import_id      INTEGER REFERENCES dataset_import(id),  -- NULL for a standalone reconcile
    reason         TEXT    NOT NULL,
    evidence       TEXT    NOT NULL,                       -- JSON: the records and facts relied on
    moved_records  TEXT    NOT NULL                        -- JSON array of source_record ids moved
);

-- A verified fix to one source row. The row's raw_json is never changed; the mapped
-- column holds the corrected value and this table holds the original and the evidence.
CREATE TABLE record_correction (
    id               INTEGER PRIMARY KEY,
    source_record_id INTEGER NOT NULL REFERENCES source_record(id),
    correction_id    TEXT    NOT NULL,
    field            TEXT    NOT NULL,
    source_value     TEXT,
    corrected_value  TEXT,
    evidence         TEXT    NOT NULL,
    UNIQUE (source_record_id, field)
);
"#;

const MIGRATIONS: &[&str] = &[V1, V2];

/// The schema version this build writes.
pub const SCHEMA_VERSION: i64 = MIGRATIONS.len() as i64;

pub fn schema_version(conn: &Connection) -> Result<i64> {
    Ok(conn.query_row("PRAGMA user_version", [], |r| r.get(0))?)
}

/// Bring the database to [`SCHEMA_VERSION`], in one transaction per migration.
pub fn migrate(conn: &mut Connection) -> Result<i64> {
    migrate_to(conn, SCHEMA_VERSION)
}

/// Apply migrations up to `target` only. Public so tests can build an older database.
pub fn migrate_to(conn: &mut Connection, target: i64) -> Result<i64> {
    let app_id: i64 = conn.query_row("PRAGMA application_id", [], |r| r.get(0))?;
    let version = schema_version(conn)?;
    if app_id != APPLICATION_ID {
        let objects: i64 =
            conn.query_row("SELECT count(*) FROM sqlite_schema", [], |r| r.get(0))?;
        if app_id != 0 || version != 0 || objects != 0 {
            return Err(StoreError::ForeignDatabase);
        }
    }
    if version > SCHEMA_VERSION {
        return Err(StoreError::NewerSchema {
            found: version,
            supported: SCHEMA_VERSION,
        });
    }
    let target = target.clamp(0, SCHEMA_VERSION) as usize;
    for (index, sql) in MIGRATIONS
        .iter()
        .enumerate()
        .take(target)
        .skip(version as usize)
    {
        let tx = conn.transaction()?;
        tx.execute_batch(sql)?;
        tx.pragma_update(None, "application_id", APPLICATION_ID)?;
        tx.pragma_update(None, "user_version", (index + 1) as i64)?;
        tx.commit()?;
    }
    schema_version(conn)
}
