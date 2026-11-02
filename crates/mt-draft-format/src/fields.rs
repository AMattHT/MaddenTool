//! The 244-byte attribute region: mapped fields and the unknown bytes between them.
//!
//! Offsets are relative to the start of the attribute region (record offset `0x1600`).
//! The map is the one the legacy parser (`M27Parser.js`, Madden-Draft-Generator
//! `9a4ac69f`) decoded from game files, checked here against the game's own classes.
//! Field names follow the legacy keys so the differential test can compare them directly.
//! 203 bytes are mapped; the other 41 are [`UNKNOWN_OFFSETS`], kept verbatim.

use crate::layout::ATTRIBUTE_SIZE;

/// How a field's bytes become a value.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Encoding {
    /// One unsigned byte.
    U8,
    /// Unsigned 16-bit little-endian.
    U16Le,
    /// One byte holding pounds minus 160. The value is the weight in pounds.
    WeightMinus160,
    /// ASCII text, NUL-terminated within the field width. The value is the bytes
    /// before the first NUL; the bytes after it stay in the record.
    Text,
    /// Five U16 LE persona DNA slots, kept positionally, zeros included.
    U16LeArray,
}

/// One mapped field of the attribute region.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FieldSpec {
    pub name: &'static str,
    /// Offset within the attribute region.
    pub offset: usize,
    /// Width in bytes.
    pub width: usize,
    pub encoding: Encoding,
    /// What is known about the field, for people reading `inspect`.
    pub note: &'static str,
}

const fn field(
    name: &'static str,
    offset: usize,
    width: usize,
    encoding: Encoding,
    note: &'static str,
) -> FieldSpec {
    FieldSpec {
        name,
        offset,
        width,
        encoding,
        note,
    }
}

const fn byte(name: &'static str, offset: usize, note: &'static str) -> FieldSpec {
    field(name, offset, 1, Encoding::U8, note)
}

const fn rating(name: &'static str, offset: usize) -> FieldSpec {
    field(name, offset, 1, Encoding::U8, "rating")
}

/// Number of persona DNA slots at `0xca`.
pub const DNA_SLOTS: usize = 5;

/// Every mapped field, in offset order.
pub const FIELDS: [FieldSpec; 82] = [
    field(
        "firstName",
        0x00,
        21,
        Encoding::Text,
        "ASCII, NUL-terminated",
    ),
    field(
        "lastName",
        0x15,
        21,
        Encoding::Text,
        "ASCII, NUL-terminated",
    ),
    byte("homeState", 0x2a, "state id"),
    field(
        "homeTown",
        0x2b,
        27,
        Encoding::Text,
        "ASCII, NUL-terminated",
    ),
    field("college", 0x46, 2, Encoding::U16Le, "college id"),
    field(
        "birthdate",
        0x48,
        2,
        Encoding::U16Le,
        "packed birthdate; encoding unverified (see README)",
    ),
    byte("age", 0x4a, "years"),
    byte("heightInches", 0x4b, "inches"),
    field(
        "weight",
        0x4c,
        1,
        Encoding::WeightMinus160,
        "pounds; stored as pounds-160",
    ),
    byte("position", 0x4e, "position id"),
    byte("archetype", 0x4f, "archetype id"),
    byte("jerseyNum", 0x50, "jersey number"),
    field(
        "draftPick",
        0x52,
        2,
        Encoding::U16Le,
        "pick within round when drafted; slot index for undrafted",
    ),
    byte(
        "draftRound",
        0x54,
        "round; 63 marks undrafted in game files",
    ),
    byte("overall", 0x55, "overall rating"),
    rating("acceleration", 0x56),
    rating("agility", 0x57),
    rating("awareness", 0x58),
    rating("ballCarrierVision", 0x59),
    rating("blockShedding", 0x5a),
    rating("breakSack", 0x5b),
    rating("breakTackle", 0x5c),
    rating("carrying", 0x5d),
    rating("catching", 0x5e),
    rating("catchInTraffic", 0x5f),
    rating("changeOfDirection", 0x60),
    rating("finesseMoves", 0x61),
    rating("hitPower", 0x62),
    rating("impactBlocking", 0x63),
    rating("injury", 0x64),
    rating("jukeMove", 0x65),
    rating("jumping", 0x66),
    rating("kickAccuracy", 0x67),
    rating("kickPower", 0x68),
    rating("kickReturn", 0x69),
    rating("leadBlock", 0x6a),
    byte(
        "const6b",
        0x6b,
        "observed 127 on every game record; meaning unknown",
    ),
    rating("manCoverage", 0x6c),
    rating("passBlockPower", 0x6d),
    rating("passBlockFinesse", 0x6e),
    rating("passBlock", 0x6f),
    byte(
        "personalityRating",
        0x70,
        "personality, 10-98 in game files",
    ),
    rating("playAction", 0x71),
    rating("playRecognition", 0x72),
    rating("powerMoves", 0x73),
    rating("pressCoverage", 0x74),
    rating("pursuit", 0x75),
    rating("release", 0x76),
    rating("deepRouteRunning", 0x77),
    rating("mediumRouteRunning", 0x78),
    rating("shortRouteRunning", 0x79),
    rating("runBlockFinesse", 0x7a),
    rating("runBlockPower", 0x7b),
    rating("runBlock", 0x7c),
    byte(
        "const7d",
        0x7d,
        "observed 1 on every game record; meaning unknown",
    ),
    rating("spectacularCatch", 0x7e),
    rating("speed", 0x7f),
    rating("spinMove", 0x80),
    rating("stamina", 0x81),
    rating("stiffArm", 0x82),
    rating("strength", 0x83),
    rating("tackle", 0x84),
    rating("throwAccuracyDeep", 0x85),
    rating("throwAccuracyMid", 0x86),
    byte("hidden87", 0x87, "always set by the game; meaning unknown"),
    rating("throwAccuracyShort", 0x88),
    rating("throwOnTheRun", 0x89),
    rating("throwPower", 0x8a),
    rating("throwUnderPressure", 0x8b),
    rating("toughness", 0x8c),
    rating("trucking", 0x8d),
    rating("zoneCoverage", 0x8e),
    rating("longSnap", 0x8f),
    byte(
        "devTrait",
        0x90,
        "0 Normal, 1 Star, 2 Superstar, 3 X-Factor",
    ),
    byte(
        "bodyTypeId",
        0x91,
        "0 Standard, 1 Thin, 2 Muscular, 3 Heavy",
    ),
    field("PID", 0x94, 2, Encoding::U16Le, "menu portrait id"),
    byte("qbStyle", 0x96, "QB style; QBs only"),
    byte("hidden9c", 0x9c, "always set by the game; meaning unknown"),
    field(
        "commentaryId",
        0x9e,
        2,
        Encoding::U16Le,
        "announcer surname id",
    ),
    field(
        "assetName",
        0xa0,
        42,
        Encoding::Text,
        "real face asset; empty for generic heads",
    ),
    field(
        "personaDNA",
        0xca,
        2 * DNA_SLOTS,
        Encoding::U16LeArray,
        "five persona DNA trait slots",
    ),
    byte("focus", 0xf2, "mindset focus 0-3"),
];

/// Attribute-region offsets no field maps, in ascending order. Always kept verbatim.
pub const UNKNOWN_OFFSETS: [usize; 41] = unknown_offsets();

/// Number of mapped attribute bytes.
pub const MAPPED_BYTES: usize = ATTRIBUTE_SIZE - UNKNOWN_OFFSETS.len();

const fn coverage() -> [u8; ATTRIBUTE_SIZE] {
    let mut covered = [0u8; ATTRIBUTE_SIZE];
    let mut i = 0;
    while i < FIELDS.len() {
        let f = FIELDS[i];
        let mut b = f.offset;
        while b < f.offset + f.width {
            covered[b] += 1;
            b += 1;
        }
        i += 1;
    }
    covered
}

const fn unknown_offsets() -> [usize; 41] {
    let covered = coverage();
    let mut out = [0usize; 41];
    let mut n = 0;
    let mut b = 0;
    while b < ATTRIBUTE_SIZE {
        assert!(covered[b] <= 1, "two fields map the same byte");
        if covered[b] == 0 {
            out[n] = b;
            n += 1;
        }
        b += 1;
    }
    assert!(n == 41, "the field map must leave exactly 41 unknown bytes");
    out
}

/// The field named `name`, if mapped.
pub fn spec(name: &str) -> Option<&'static FieldSpec> {
    FIELDS.iter().find(|f| f.name == name)
}

/// A decoded field value. The original bytes always remain in the record.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FieldValue {
    Number(u32),
    /// Bytes before the field's first NUL. Game files hold ASCII; see [`FieldValue::as_text`].
    Text(Vec<u8>),
    /// Persona DNA slots, positionally, zeros included.
    Numbers(Vec<u16>),
}

impl FieldValue {
    pub fn as_number(&self) -> Option<u32> {
        match self {
            FieldValue::Number(n) => Some(*n),
            _ => None,
        }
    }

    /// The text, when every byte is printable ASCII. Anything else is not decoded,
    /// because no game file has shown which encoding the game uses for it.
    pub fn as_text(&self) -> Option<&str> {
        match self {
            FieldValue::Text(b) if b.iter().all(|c| (0x20..0x7f).contains(c)) => {
                std::str::from_utf8(b).ok()
            }
            _ => None,
        }
    }

    pub fn as_numbers(&self) -> Option<&[u16]> {
        match self {
            FieldValue::Numbers(v) => Some(v),
            _ => None,
        }
    }
}

/// Decode one field from an attribute region.
pub fn decode(spec: &FieldSpec, attributes: &[u8; ATTRIBUTE_SIZE]) -> FieldValue {
    let bytes = &attributes[spec.offset..spec.offset + spec.width];
    match spec.encoding {
        Encoding::U8 => FieldValue::Number(u32::from(bytes[0])),
        Encoding::U16Le => FieldValue::Number(u32::from(u16::from_le_bytes([bytes[0], bytes[1]]))),
        Encoding::WeightMinus160 => FieldValue::Number(u32::from(bytes[0]) + 160),
        Encoding::Text => {
            let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
            FieldValue::Text(bytes[..end].to_vec())
        }
        Encoding::U16LeArray => FieldValue::Numbers(
            bytes
                .as_chunks::<2>()
                .0
                .iter()
                .map(|&c| u16::from_le_bytes(c))
                .collect(),
        ),
    }
}

/// Whether a text field has non-zero bytes after its terminator.
pub fn text_padding_is_zero(spec: &FieldSpec, attributes: &[u8; ATTRIBUTE_SIZE]) -> bool {
    let bytes = &attributes[spec.offset..spec.offset + spec.width];
    match bytes.iter().position(|&b| b == 0) {
        Some(end) => bytes[end..].iter().all(|&b| b == 0),
        None => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn map_covers_203_bytes_and_leaves_the_41_unknown() {
        assert_eq!(MAPPED_BYTES, 203);
        let mut expected = vec![0x4d, 0x51, 0x92, 0x93, 0x97, 0x98, 0x99, 0x9a, 0x9b, 0x9d];
        expected.extend(0xd4..=0xf1);
        expected.push(0xf3);
        assert_eq!(UNKNOWN_OFFSETS.to_vec(), expected);
        let total: usize = FIELDS.iter().map(|f| f.width).sum();
        assert_eq!(total, 203);
    }

    #[test]
    fn field_names_are_unique_and_in_offset_order() {
        for pair in FIELDS.windows(2) {
            assert!(pair[0].offset + pair[0].width <= pair[1].offset, "{pair:?}");
        }
        let mut names: Vec<_> = FIELDS.iter().map(|f| f.name).collect();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), FIELDS.len());
        assert_eq!(FIELDS.iter().filter(|f| f.note == "rating").count(), 54);
    }
}
