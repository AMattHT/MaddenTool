//! The draft-class model: validated header, active players, and the original bytes.
//!
//! Parsing never changes or normalizes anything. The file's bytes are kept whole; each
//! player keeps a copy of its attribute region (so the 41 unknown bytes travel with the
//! model) and the exact text of its visual JSON next to the parsed object.

use std::fmt;

use serde_json::{Map, Value};

use crate::error::ReadError;
use crate::fields::{self, FIELDS, FieldSpec, FieldValue, UNKNOWN_OFFSETS};
use crate::layout::*;

/// Six U16 values at `0x16`: year, month, day, hour, minute, second. Not range-checked.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Timestamp(pub [u16; 6]);

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let [y, mo, d, h, mi, s] = self.0;
        write!(f, "{y:04}-{mo:02}-{d:02} {h:02}:{mi:02}:{s:02}")
    }
}

/// The build string at `0x22`, for example `Madden-27-RL4-9244223`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Build {
    pub text: String,
    /// `RL4`, `RL1`, `RL3_5`.
    pub release: String,
    pub changelist: u64,
}

impl Build {
    /// Whether this exact build was seen in a game-written file.
    pub fn is_observed(&self) -> bool {
        OBSERVED_BUILDS.contains(&self.text.as_str())
    }

    fn parse(region: &[u8]) -> Result<Build, ReadError> {
        let end = region
            .iter()
            .position(|&b| b == 0)
            .ok_or(ReadError::MalformedBuild {
                reason: "no NUL terminator within its 28 bytes",
            })?;
        let raw = &region[..end];
        if raw.is_empty() {
            return Err(ReadError::MalformedBuild {
                reason: "the build string is empty",
            });
        }
        if !raw.iter().all(|b| (0x21..0x7f).contains(b)) {
            return Err(ReadError::MalformedBuild {
                reason: "the build string is not printable ASCII",
            });
        }
        let text = String::from_utf8(raw.to_vec()).expect("checked ASCII");
        let unsupported = || ReadError::UnsupportedBuild {
            build: text.clone(),
        };
        // A build stamp names its game first: `Madden-27-...`, `Madden-26-...`, `CFB-26-...`.
        if text != "Madden-27" && !text.starts_with("Madden-27-") {
            return Err(ReadError::UnsupportedGame { build: text });
        }
        let rest = text.strip_prefix("Madden-27-").ok_or_else(unsupported)?;
        let (release, changelist) = rest.split_once('-').ok_or_else(unsupported)?;
        let digits = |s: &str| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit());
        let release_ok = release
            .strip_prefix("RL")
            .is_some_and(|r| match r.split_once('_') {
                Some((major, minor)) => digits(major) && digits(minor),
                None => digits(r),
            });
        if !release_ok || !digits(changelist) {
            return Err(unsupported());
        }
        let changelist = changelist.parse().map_err(|_| unsupported())?;
        Ok(Build {
            release: release.to_string(),
            changelist,
            text,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Header {
    pub version: u16,
    pub header_size: u32,
    pub payload_size: u32,
    /// The U32 at `0x12`: payload size plus header size.
    pub size_field: u32,
    pub timestamp: Timestamp,
    pub build: Build,
    /// The U32 at `0x3e`, observed as 7.
    pub constant_3e: u32,
    pub active_count: u32,
}

/// Something unusual that does not stop the file from being read. The bytes involved
/// are always kept.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Warning {
    /// A well-formed Madden 27 build that no game file has shown yet.
    UnobservedBuild { build: String },
    /// The U32 at `0x3e` is not 7.
    UnexpectedConstant { found: u32 },
    /// A slot at or above the active count holds non-zero bytes. It is not a player.
    InactiveSlotNotBlank { slot: usize },
    /// Non-zero bytes follow an active record's visual JSON.
    VisualPaddingNotZero { slot: usize },
    /// Non-zero bytes follow a text field's terminator.
    TextPaddingNotZero { slot: usize, field: &'static str },
}

impl fmt::Display for Warning {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Warning::UnobservedBuild { build } => {
                write!(f, "build {build} has not been seen in a game file")
            }
            Warning::UnexpectedConstant { found } => write!(
                f,
                "U32 at 0x3e is {found}; every game file has {OBSERVED_CONSTANT}"
            ),
            Warning::InactiveSlotNotBlank { slot } => {
                write!(f, "inactive slot {slot} is not blank; its bytes are kept")
            }
            Warning::VisualPaddingNotZero { slot } => write!(
                f,
                "slot {slot}: bytes after the visual JSON are not zero; they are kept"
            ),
            Warning::TextPaddingNotZero { slot, field } => write!(
                f,
                "slot {slot}: {field} has bytes after its terminator; they are kept"
            ),
        }
    }
}

/// An active record's visual JSON: the exact text from the file and its parsed object.
/// Nothing is added: a key the file lacks (`bodyType`, `skinTone`) stays absent.
#[derive(Clone, Debug, PartialEq)]
pub struct Visuals {
    json: String,
    object: Map<String, Value>,
}

impl Visuals {
    /// The exact JSON text from the file, without the padding after it.
    pub fn json(&self) -> &str {
        &self.json
    }

    /// The parsed object. Use [`Visuals::json`] for key order and number formatting.
    pub fn object(&self) -> &Map<String, Value> {
        &self.object
    }

    /// A top-level key, or `None` when the file does not have it.
    pub fn get(&self, key: &str) -> Option<&Value> {
        self.object.get(key)
    }

    pub fn body_type(&self) -> Option<&str> {
        self.get("bodyType").and_then(Value::as_str)
    }

    pub fn generic_head_name(&self) -> Option<&str> {
        self.get("genericHeadName").and_then(Value::as_str)
    }

    /// Split a visual region into its JSON object and padding. The JSON is the bytes
    /// before the first NUL, or the whole region when there is none.
    fn parse(region: &[u8]) -> Result<(Visuals, bool), String> {
        let end = region.iter().position(|&b| b == 0).unwrap_or(region.len());
        let text = &region[..end];
        if text.is_empty() {
            return Err("the visual region is empty".to_string());
        }
        let json = std::str::from_utf8(text)
            .map_err(|e| format!("not UTF-8 ({e})"))?
            .to_string();
        let value: Value = serde_json::from_str(&json).map_err(|e| e.to_string())?;
        let Value::Object(object) = value else {
            return Err("the JSON value is not an object".to_string());
        };
        let padding_is_zero = region[end..].iter().all(|&b| b == 0);
        Ok((Visuals { json, object }, padding_is_zero))
    }
}

/// One active record.
#[derive(Clone, Debug, PartialEq)]
pub struct Prospect {
    slot: usize,
    attributes: [u8; ATTRIBUTE_SIZE],
    values: Vec<FieldValue>,
    visuals: Visuals,
}

impl Prospect {
    /// Record slot, 0-based. In game files slot `i` is draft position `i`.
    pub fn slot(&self) -> usize {
        self.slot
    }

    pub fn field(&self, name: &str) -> Option<&FieldValue> {
        FIELDS
            .iter()
            .position(|f| f.name == name)
            .map(|i| &self.values[i])
    }

    /// Every mapped field with its value, in offset order.
    pub fn fields(&self) -> impl Iterator<Item = (&'static FieldSpec, &FieldValue)> {
        FIELDS.iter().zip(self.values.iter())
    }

    pub fn number(&self, name: &str) -> Option<u32> {
        self.field(name).and_then(FieldValue::as_number)
    }

    /// A text field, when it is printable ASCII.
    pub fn text(&self, name: &str) -> Option<&str> {
        self.field(name).and_then(FieldValue::as_text)
    }

    /// The record's 244 attribute bytes exactly as read.
    pub fn attribute_bytes(&self) -> &[u8; ATTRIBUTE_SIZE] {
        &self.attributes
    }

    /// The 41 unknown attribute bytes, in [`UNKNOWN_OFFSETS`] order.
    pub fn unknown_bytes(&self) -> [u8; UNKNOWN_OFFSETS.len()] {
        UNKNOWN_OFFSETS.map(|o| self.attributes[o])
    }

    pub fn visuals(&self) -> &Visuals {
        &self.visuals
    }
}

/// A validated draft class with its original bytes.
#[derive(Clone, Debug)]
pub struct DraftClass {
    source: Vec<u8>,
    header: Header,
    players: Vec<Prospect>,
    warnings: Vec<Warning>,
}

fn u16_at(b: &[u8], o: usize) -> u16 {
    u16::from_le_bytes([b[o], b[o + 1]])
}

fn u32_at(b: &[u8], o: usize) -> u32 {
    u32::from_le_bytes([b[o], b[o + 1], b[o + 2], b[o + 3]])
}

fn parse_header(source: &[u8]) -> Result<Header, ReadError> {
    if source.len() < DATA_START {
        return Err(ReadError::TruncatedHeader {
            actual: source.len(),
        });
    }
    let mut magic = [0u8; 8];
    magic.copy_from_slice(&source[..8]);
    if magic != MAGIC {
        return Err(ReadError::BadMagic { found: magic });
    }
    let version = u16_at(source, OFFSET_VERSION);
    if version != VERSION {
        return Err(ReadError::UnsupportedVersion { found: version });
    }
    let header_size = u32_at(source, OFFSET_HEADER_SIZE);
    if header_size != HEADER_SIZE {
        return Err(ReadError::UnexpectedHeaderSize { found: header_size });
    }
    let payload_size = u32_at(source, OFFSET_PAYLOAD_SIZE);
    if payload_size as usize != PAYLOAD_SIZE {
        return Err(ReadError::UnexpectedPayloadSize {
            found: payload_size,
        });
    }
    let size_field = u32_at(source, OFFSET_SIZE_FIELD);
    let expected = payload_size + header_size;
    if size_field != expected {
        return Err(ReadError::SizeFieldMismatch {
            found: size_field,
            expected,
        });
    }
    if source.len() < FILE_SIZE {
        return Err(ReadError::Truncated {
            expected: FILE_SIZE,
            actual: source.len(),
        });
    }
    if source.len() > FILE_SIZE {
        return Err(ReadError::TrailingBytes {
            expected: FILE_SIZE,
            actual: source.len(),
        });
    }
    let build = Build::parse(&source[OFFSET_BUILD..OFFSET_BUILD + BUILD_LEN])?;
    let active_count = u32_at(source, OFFSET_ACTIVE_COUNT);
    if active_count as usize > CAPACITY {
        return Err(ReadError::ActiveCountOutOfRange {
            found: active_count,
        });
    }
    let mut timestamp = [0u16; 6];
    for (i, t) in timestamp.iter_mut().enumerate() {
        *t = u16_at(source, OFFSET_TIMESTAMP + 2 * i);
    }
    Ok(Header {
        version,
        header_size,
        payload_size,
        size_field,
        timestamp: Timestamp(timestamp),
        build,
        constant_3e: u32_at(source, OFFSET_CONSTANT),
        active_count,
    })
}

fn parse_player(
    slot: usize,
    record: &[u8],
    warnings: &mut Vec<Warning>,
) -> Result<Prospect, ReadError> {
    if record.iter().all(|&b| b == 0) {
        return Err(ReadError::EmptyActiveRecord { slot });
    }
    let (visuals, padding_is_zero) = Visuals::parse(&record[..VISUAL_SIZE])
        .map_err(|detail| ReadError::MalformedVisualJson { slot, detail })?;
    if !padding_is_zero {
        warnings.push(Warning::VisualPaddingNotZero { slot });
    }
    let mut attributes = [0u8; ATTRIBUTE_SIZE];
    attributes.copy_from_slice(&record[VISUAL_SIZE..]);
    for spec in FIELDS
        .iter()
        .filter(|f| f.encoding == fields::Encoding::Text)
    {
        if !fields::text_padding_is_zero(spec, &attributes) {
            warnings.push(Warning::TextPaddingNotZero {
                slot,
                field: spec.name,
            });
        }
    }
    Ok(Prospect {
        slot,
        values: FIELDS
            .iter()
            .map(|f| fields::decode(f, &attributes))
            .collect(),
        attributes,
        visuals,
    })
}

impl DraftClass {
    /// Validate and parse a whole file. Takes ownership of the bytes and keeps them unchanged.
    pub fn parse(source: Vec<u8>) -> Result<Self, ReadError> {
        let header = parse_header(&source)?;
        let mut warnings = Vec::new();
        if !header.build.is_observed() {
            warnings.push(Warning::UnobservedBuild {
                build: header.build.text.clone(),
            });
        }
        if header.constant_3e != OBSERVED_CONSTANT {
            warnings.push(Warning::UnexpectedConstant {
                found: header.constant_3e,
            });
        }
        let active = header.active_count as usize;
        let mut players = Vec::with_capacity(active);
        for slot in 0..active {
            players.push(parse_player(
                slot,
                &source[record_range(slot)],
                &mut warnings,
            )?);
        }
        for slot in active..CAPACITY {
            if source[record_range(slot)].iter().any(|&b| b != 0) {
                warnings.push(Warning::InactiveSlotNotBlank { slot });
            }
        }
        Ok(DraftClass {
            source,
            header,
            players,
            warnings,
        })
    }

    /// The file exactly as read.
    pub fn source_bytes(&self) -> &[u8] {
        &self.source
    }

    pub fn header(&self) -> &Header {
        &self.header
    }

    /// Physical record slots: always 455. Not an export limit.
    pub fn capacity(&self) -> usize {
        CAPACITY
    }

    /// The header's active count. Slots below it are players; slots from it up are inactive.
    pub fn active_count(&self) -> usize {
        self.header.active_count as usize
    }

    /// Active players, in slot order.
    pub fn players(&self) -> &[Prospect] {
        &self.players
    }

    /// The original 5,876 bytes of any slot, active or not.
    ///
    /// # Panics
    /// If `slot` is 455 or more.
    pub fn record_bytes(&self, slot: usize) -> &[u8] {
        assert!(
            slot < CAPACITY,
            "slot {slot} is beyond the {CAPACITY} slots"
        );
        &self.source[record_range(slot)]
    }

    /// Inactive slots holding only zeros.
    pub fn blank_inactive_slots(&self) -> usize {
        (self.active_count()..CAPACITY)
            .filter(|&s| self.record_bytes(s).iter().all(|&b| b == 0))
            .count()
    }

    pub fn warnings(&self) -> &[Warning] {
        &self.warnings
    }
}
