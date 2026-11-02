//! The machine-readable `inspect --json` document (schema `mt-draft-inspect/1`).
//!
//! Visual objects are re-emitted from the parsed JSON, so their key order and number
//! formatting may differ from the file; `visualJsonSha256` and `recordSha256` identify
//! the exact original bytes.

use serde_json::{Map, Value, json};
use sha2::{Digest, Sha256};

use crate::class::{DraftClass, Prospect};
use crate::fields::FieldValue;

pub const SCHEMA: &str = "mt-draft-inspect/1";

pub fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    hex(&Sha256::digest(bytes))
}

pub fn field_json(value: &FieldValue) -> Value {
    match value {
        FieldValue::Number(n) => json!(n),
        FieldValue::Numbers(v) => json!(v),
        FieldValue::Text(bytes) => match value.as_text() {
            Some(s) => json!(s),
            None => json!({ "nonAsciiHex": hex(bytes) }),
        },
    }
}

pub fn player_json(class: &DraftClass, p: &Prospect) -> Value {
    let mut fields = Map::new();
    for (spec, value) in p.fields() {
        fields.insert(spec.name.to_string(), field_json(value));
    }
    json!({
        "slot": p.slot(),
        "recordSha256": sha256_hex(class.record_bytes(p.slot())),
        "fields": fields,
        "unknownHex": hex(&p.unknown_bytes()),
        "visualJsonBytes": p.visuals().json().len(),
        "visualJsonSha256": sha256_hex(p.visuals().json().as_bytes()),
        "visuals": Value::Object(p.visuals().object().clone()),
    })
}

/// The whole document. `path` is reported as given; nothing else about the machine is.
pub fn inspect_json(class: &DraftClass, path: &str) -> Value {
    let h = class.header();
    json!({
        "schema": SCHEMA,
        "source": {
            "path": path,
            "bytes": class.source_bytes().len(),
            "sha256": sha256_hex(class.source_bytes()),
        },
        "game": "Madden 27",
        "header": {
            "version": h.version,
            "headerSize": h.header_size,
            "payloadSize": h.payload_size,
            "sizeField": h.size_field,
            "timestamp": h.timestamp.0,
            "timestampText": h.timestamp.to_string(),
            "build": h.build.text,
            "release": h.build.release,
            "changelist": h.build.changelist,
            "observedBuild": h.build.is_observed(),
            "constant3e": h.constant_3e,
            "activeCount": h.active_count,
        },
        "capacity": class.capacity(),
        "activeCount": class.active_count(),
        "inactiveSlots": class.capacity() - class.active_count(),
        "blankInactiveSlots": class.blank_inactive_slots(),
        "warnings": class.warnings().iter().map(ToString::to_string).collect::<Vec<_>>(),
        "players": class.players().iter().map(|p| player_json(class, p)).collect::<Vec<_>>(),
    })
}
