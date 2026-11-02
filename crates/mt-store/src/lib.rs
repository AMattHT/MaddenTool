//! MaddenTool's historical player database (goal DG01).
//!
//! A SQLite file holding real players imported from documented datasets, with stable
//! identity, cross-source IDs and per-import provenance. It does not generate ratings
//! or read game files; see `docs/superpowers/plans/2026-11-02-draft-first.md`.

pub mod error;
pub mod formats;
pub mod import;
pub mod names;
pub mod query;
pub mod schema;
pub mod wav;

use std::path::Path;

use rusqlite::Connection;

pub use error::{Result, StoreError};
pub use formats::Format;
pub use import::{ImportOptions, ImportSummary, Issue};
pub use query::{ImportRecord, Info, PlayerRow};
pub use wav::{AsOf, CareerState, WavEvidence};

pub struct Store {
    conn: Connection,
}

impl Store {
    /// Open or create a database and bring its schema up to date.
    pub fn open(path: &Path) -> Result<Store> {
        let mut conn = Connection::open(path)?;
        conn.pragma_update(None, "foreign_keys", true)?;
        conn.busy_timeout(std::time::Duration::from_secs(5))?;
        schema::migrate(&mut conn)?;
        Ok(Store { conn })
    }

    /// Open a database that must already exist, so a mistyped path is not created empty.
    pub fn open_existing(path: &Path) -> Result<Store> {
        if !path.is_file() {
            return Err(StoreError::InvalidInput(format!(
                "no database at {}; create one with `mt-data migrate` or `mt-data import`",
                path.display()
            )));
        }
        Store::open(path)
    }

    pub fn schema_version(&self) -> Result<i64> {
        schema::schema_version(&self.conn)
    }

    /// Import a dataset file. All rows land, or (on any error) none do.
    pub fn import(&mut self, options: &ImportOptions) -> Result<ImportSummary> {
        let tx = self.conn.transaction()?;
        let summary = import::run(&tx, options)?;
        tx.commit()?;
        Ok(summary)
    }

    pub fn players_by_year(&self, year: i64, position: Option<&str>) -> Result<Vec<PlayerRow>> {
        query::players_by_year(&self.conn, year, position)
    }

    pub fn imports(&self) -> Result<Vec<ImportRecord>> {
        query::imports(&self.conn)
    }

    pub fn info(&self) -> Result<Info> {
        query::info(&self.conn, self.schema_version()?)
    }

    /// Issues recorded by one import, in source line order.
    pub fn issues(&self, import_id: i64) -> Result<Vec<Issue>> {
        let mut stmt = self.conn.prepare(
            "SELECT line, kind, detail FROM import_issue WHERE import_id = ?1 ORDER BY line, id",
        )?;
        let rows = stmt
            .query_map([import_id], |r| {
                Ok(Issue {
                    line: r.get::<_, Option<i64>>(0)?.map(|l| l as u64),
                    kind: r.get(1)?,
                    detail: r.get(2)?,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(rows)
    }

    /// Direct read access for tests and diagnostics.
    pub fn connection(&self) -> &Connection {
        &self.conn
    }
}
