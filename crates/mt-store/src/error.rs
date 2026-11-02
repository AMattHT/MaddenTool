use std::fmt;

/// Every failure the store reports. Messages are written for the person running the tool.
#[derive(Debug)]
pub enum StoreError {
    Sqlite(rusqlite::Error),
    Csv(csv::Error),
    Io(std::io::Error),
    Json(serde_json::Error),
    /// The database was written by a newer MaddenTool than this one.
    NewerSchema {
        found: i64,
        supported: i64,
    },
    /// The file is a SQLite database that MaddenTool did not create.
    ForeignDatabase,
    /// The file lacks columns the chosen format requires. Nothing was imported.
    MissingColumns {
        format: &'static str,
        missing: Vec<String>,
    },
    /// Bad command-line or caller input.
    InvalidInput(String),
}

impl fmt::Display for StoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StoreError::Sqlite(e) => write!(f, "database error: {e}"),
            StoreError::Csv(e) => write!(f, "CSV error: {e}"),
            StoreError::Io(e) => write!(f, "file error: {e}"),
            StoreError::Json(e) => write!(f, "JSON error: {e}"),
            StoreError::NewerSchema { found, supported } => write!(
                f,
                "database schema version {found} is newer than this tool supports ({supported}); use a newer MaddenTool"
            ),
            StoreError::ForeignDatabase => {
                write!(
                    f,
                    "this SQLite file was not created by MaddenTool; refusing to change it"
                )
            }
            StoreError::MissingColumns { format, missing } => write!(
                f,
                "file is not a {format} dataset: missing column(s) {}; nothing was imported",
                missing.join(", ")
            ),
            StoreError::InvalidInput(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for StoreError {}

impl From<rusqlite::Error> for StoreError {
    fn from(e: rusqlite::Error) -> Self {
        StoreError::Sqlite(e)
    }
}

impl From<csv::Error> for StoreError {
    fn from(e: csv::Error) -> Self {
        StoreError::Csv(e)
    }
}

impl From<std::io::Error> for StoreError {
    fn from(e: std::io::Error) -> Self {
        StoreError::Io(e)
    }
}

impl From<serde_json::Error> for StoreError {
    fn from(e: serde_json::Error) -> Self {
        StoreError::Json(e)
    }
}

pub type Result<T> = std::result::Result<T, StoreError>;
