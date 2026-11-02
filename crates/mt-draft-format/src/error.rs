use std::fmt;
use std::path::PathBuf;

/// Why a file was refused. Messages are written for the person running the tool.
#[derive(Debug)]
pub enum ReadError {
    /// The file could not be read.
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    /// Shorter than the 70-byte header.
    TruncatedHeader {
        actual: usize,
    },
    /// The first eight bytes are not `FBCHUNKS`.
    BadMagic {
        found: [u8; 8],
    },
    UnsupportedVersion {
        found: u16,
    },
    UnexpectedHeaderSize {
        found: u32,
    },
    /// The payload-size field is not 455 records of 5,876 bytes.
    UnexpectedPayloadSize {
        found: u32,
    },
    /// The field at `0x12` is not the payload size plus the header size.
    SizeFieldMismatch {
        found: u32,
        expected: u32,
    },
    /// The file ends before its declared payload does.
    Truncated {
        expected: usize,
        actual: usize,
    },
    /// Bytes follow the declared payload.
    TrailingBytes {
        expected: usize,
        actual: usize,
    },
    /// The build string is not terminated ASCII.
    MalformedBuild {
        reason: &'static str,
    },
    /// A well-formed build string from another game, such as Madden 26 or College Football.
    UnsupportedGame {
        build: String,
    },
    /// A Madden 27 build string that does not have the `RL<release>-<changelist>` shape.
    UnsupportedBuild {
        build: String,
    },
    /// The active count exceeds the 455 physical slots.
    ActiveCountOutOfRange {
        found: u32,
    },
    /// A slot below the active count is entirely zero.
    EmptyActiveRecord {
        slot: usize,
    },
    /// An active record's visual region is not one JSON object followed by NUL padding.
    MalformedVisualJson {
        slot: usize,
        detail: String,
    },
}

impl fmt::Display for ReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use crate::layout::{
            CAPACITY, DATA_START, HEADER_SIZE, PAYLOAD_SIZE, RECORD_SIZE, VERSION,
        };
        match self {
            ReadError::Io { path, source } => write!(f, "cannot read {}: {source}", path.display()),
            ReadError::TruncatedHeader { actual } => write!(
                f,
                "not a Madden 27 draft class: {actual} bytes is shorter than the {DATA_START}-byte header"
            ),
            ReadError::BadMagic { found } => write!(
                f,
                "not a Madden draft class: expected magic \"FBCHUNKS\" at offset 0x00, found {}",
                printable(found)
            ),
            ReadError::UnsupportedVersion { found } => write!(
                f,
                "unsupported container version {found} at offset 0x08 (expected {VERSION})"
            ),
            ReadError::UnexpectedHeaderSize { found } => write!(
                f,
                "unexpected header size {found} at offset 0x0a (expected {HEADER_SIZE})"
            ),
            ReadError::UnexpectedPayloadSize { found } => write!(
                f,
                "not a Madden 27 draft class: payload size {found} at offset 0x0e (expected {PAYLOAD_SIZE}, {CAPACITY} records of {RECORD_SIZE} bytes)"
            ),
            ReadError::SizeFieldMismatch { found, expected } => write!(
                f,
                "inconsistent header: size field at offset 0x12 is {found}, expected payload + header = {expected}"
            ),
            ReadError::Truncated { expected, actual } => write!(
                f,
                "truncated draft class: file is {actual} bytes, header declares {expected}"
            ),
            ReadError::TrailingBytes { expected, actual } => write!(
                f,
                "draft class has {} unexpected bytes after its payload: file is {actual} bytes, header declares {expected}",
                actual - expected
            ),
            ReadError::MalformedBuild { reason } => {
                write!(f, "malformed build string at offset 0x22: {reason}")
            }
            ReadError::UnsupportedGame { build } => write!(
                f,
                "unsupported game: build \"{build}\" is not Madden 27 (expected \"Madden-27-...\")"
            ),
            ReadError::UnsupportedBuild { build } => write!(
                f,
                "unsupported Madden 27 build \"{build}\" (expected \"Madden-27-RL<release>-<changelist>\")"
            ),
            ReadError::ActiveCountOutOfRange { found } => write!(
                f,
                "active count {found} at offset 0x42 exceeds the {CAPACITY} record slots"
            ),
            ReadError::EmptyActiveRecord { slot } => write!(
                f,
                "record slot {slot} is below the active count but entirely zero"
            ),
            ReadError::MalformedVisualJson { slot, detail } => {
                write!(f, "record slot {slot} has malformed visual JSON: {detail}")
            }
        }
    }
}

impl std::error::Error for ReadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ReadError::Io { source, .. } => Some(source),
            _ => None,
        }
    }
}

fn printable(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|&b| {
            if (0x20..0x7f).contains(&b) && b != b'\\' {
                (b as char).to_string()
            } else {
                format!("\\x{b:02x}")
            }
        })
        .collect()
}
