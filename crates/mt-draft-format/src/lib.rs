//! Read-only reader for Madden 27 draft-class files (`CAREERDRAFT-*`).
//!
//! [`DraftClass::parse`] validates the container and every active record, then exposes
//! the header, each active player's mapped fields, the 41 unknown attribute bytes and the
//! visual JSON. The original file bytes are kept unchanged, so a later patch-only writer
//! (DG03) can change exactly the bytes an edit touches. This crate has no write path.
//!
//! Layout evidence and legacy parity are recorded in this crate's README.

pub mod class;
pub mod error;
pub mod fields;
pub mod layout;
pub mod report;

pub use class::{Build, DraftClass, Header, Prospect, Timestamp, Visuals, Warning};
pub use error::ReadError;
pub use fields::{FIELDS, FieldSpec, FieldValue, UNKNOWN_OFFSETS};

use std::path::Path;

/// Read and validate a draft class. The file is only opened for reading.
pub fn read_file(path: &Path) -> Result<DraftClass, ReadError> {
    let bytes = std::fs::read(path).map_err(|source| ReadError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    DraftClass::parse(bytes)
}
