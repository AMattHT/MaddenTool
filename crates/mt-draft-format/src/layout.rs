//! Fixed geometry of a Madden 27 draft-class file.
//!
//! Every number here was observed in the game's own files (`CAREERDRAFT-TEMP1` to
//! `TEMP3`, build RL4, and `CAREERDRAFT-2027Template`, build RL1); see
//! `docs/DISCOVERY-REPORT.md` section 5.2 and this crate's README.

/// `FBCHUNKS` at offset `0x00`.
pub const MAGIC: [u8; 8] = *b"FBCHUNKS";
/// Container version, U16 LE at `0x08`.
pub const VERSION: u16 = 1;
/// Header-size field, U32 LE at `0x0a`. It counts the bytes from `0x12` up to the data.
pub const HEADER_SIZE: u32 = 52;

pub const OFFSET_VERSION: usize = 0x08;
pub const OFFSET_HEADER_SIZE: usize = 0x0a;
/// Payload size, U32 LE.
pub const OFFSET_PAYLOAD_SIZE: usize = 0x0e;
/// Payload size plus the header size, U32 LE: the byte count from here to the end of the file.
pub const OFFSET_SIZE_FIELD: usize = 0x12;
/// Six U16 LE values: year, month, day, hour, minute, second.
pub const OFFSET_TIMESTAMP: usize = 0x16;
/// NUL-terminated ASCII build string, for example `Madden-27-RL4-9244223`.
pub const OFFSET_BUILD: usize = 0x22;
/// Bytes reserved for the build string, `0x22` up to `0x3e`.
pub const BUILD_LEN: usize = 0x1c;
/// U32 LE observed as 7 in every file; its meaning is unknown.
pub const OFFSET_CONSTANT: usize = 0x3e;
/// Observed value of the constant at `0x3e`.
pub const OBSERVED_CONSTANT: u32 = 7;
/// Active record count, U32 LE.
pub const OFFSET_ACTIVE_COUNT: usize = 0x42;

/// First record. Also the length of the header region.
pub const DATA_START: usize = 0x46;
/// Physical record slots. The payload always holds exactly this many.
pub const CAPACITY: usize = 455;
/// One record: visual JSON, then attributes.
pub const RECORD_SIZE: usize = 5_876;
/// Visual region: UTF-8 JSON object, then NUL padding.
pub const VISUAL_SIZE: usize = 5_632;
/// Attribute region, after the visual region.
pub const ATTRIBUTE_SIZE: usize = 244;
/// `CAPACITY * RECORD_SIZE` = 2,673,580.
pub const PAYLOAD_SIZE: usize = CAPACITY * RECORD_SIZE;
/// Whole file: 2,673,650 bytes.
pub const FILE_SIZE: usize = DATA_START + PAYLOAD_SIZE;

/// Build strings seen in game-written files. Others that pass validation open with a warning.
pub const OBSERVED_BUILDS: [&str; 2] = ["Madden-27-RL1-9081279", "Madden-27-RL4-9244223"];

const _: () = assert!(VISUAL_SIZE + ATTRIBUTE_SIZE == RECORD_SIZE);
const _: () = assert!(OFFSET_SIZE_FIELD + HEADER_SIZE as usize == DATA_START);
const _: () = assert!(OFFSET_BUILD + BUILD_LEN == OFFSET_CONSTANT);

/// Byte range of record `slot` within the file.
pub fn record_range(slot: usize) -> std::ops::Range<usize> {
    let start = DATA_START + slot * RECORD_SIZE;
    start..start + RECORD_SIZE
}
