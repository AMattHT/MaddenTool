//! Verified corrections to individual source rows.
//!
//! A correction names one source row by format and natural key, states the values it
//! expects the source to hold, the values to use instead, and the evidence. If the
//! source row no longer holds the expected values, the correction is reported stale and
//! not applied. Raw rows are never edited.

use std::collections::BTreeMap;

use serde::Deserialize;

use crate::error::{Result, StoreError};
use crate::formats::Mapped;

const BUILTIN: &str = include_str!("../data/corrections.json");

/// Fields a correction may change. Identity, draft slot and career fields are not
/// correctable here: those would change what the source says happened.
const CORRECTABLE: &[&str] = &["college", "position"];

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Correction {
    pub id: String,
    pub format: String,
    pub natural_key: String,
    pub expect: BTreeMap<String, String>,
    pub set: BTreeMap<String, String>,
    pub evidence: Vec<String>,
    pub note: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Corrections {
    pub version: u32,
    pub corrections: Vec<Correction>,
}

/// One field a correction changed on one row.
#[derive(Debug, Clone, PartialEq)]
pub struct Applied {
    pub correction_id: String,
    pub field: String,
    pub source_value: Option<String>,
    pub corrected_value: String,
    pub evidence: String,
}

pub enum Outcome {
    None,
    Applied(Vec<Applied>),
    /// The row no longer holds what the correction expects; it was left as the source says.
    Stale {
        correction_id: String,
        detail: String,
    },
}

fn field<'m>(mapped: &'m mut Mapped, name: &str) -> &'m mut Option<String> {
    match name {
        "college" => &mut mapped.college,
        "position" => &mut mapped.position,
        _ => unreachable!("validated on load"),
    }
}

impl Corrections {
    /// The corrections shipped in `data/corrections.json`.
    pub fn builtin() -> Corrections {
        Corrections::from_json(BUILTIN).expect("data/corrections.json is valid")
    }

    pub fn none() -> Corrections {
        Corrections {
            version: 1,
            corrections: Vec::new(),
        }
    }

    pub fn from_json(text: &str) -> Result<Corrections> {
        let set: Corrections = serde_json::from_str(text)?;
        for c in &set.corrections {
            let bad = c
                .set
                .keys()
                .chain(c.expect.keys())
                .find(|f| !CORRECTABLE.contains(&f.as_str()));
            if let Some(f) = bad {
                return Err(StoreError::InvalidInput(format!(
                    "correction {} names field {f:?}; correctable fields: {}",
                    c.id,
                    CORRECTABLE.join(", ")
                )));
            }
            if c.set.is_empty() || c.evidence.is_empty() {
                return Err(StoreError::InvalidInput(format!(
                    "correction {} needs at least one field and one piece of evidence",
                    c.id
                )));
            }
        }
        Ok(set)
    }

    pub fn apply(&self, format: &str, mapped: &mut Mapped) -> Outcome {
        let Some(c) = self
            .corrections
            .iter()
            .find(|c| c.format == format && c.natural_key == mapped.natural_key)
        else {
            return Outcome::None;
        };
        for (name, expected) in &c.expect {
            let actual = field(mapped, name).clone();
            if actual.as_deref() != Some(expected.as_str()) {
                return Outcome::Stale {
                    correction_id: c.id.clone(),
                    detail: format!(
                        "expected {name} {expected:?}, source now has {actual:?}; correction not applied"
                    ),
                };
            }
        }
        let evidence = serde_json::json!({ "evidence": c.evidence, "note": c.note }).to_string();
        let applied = c
            .set
            .iter()
            .map(|(name, value)| {
                let slot = field(mapped, name);
                let source_value = slot.replace(value.clone());
                Applied {
                    correction_id: c.id.clone(),
                    field: name.clone(),
                    source_value,
                    corrected_value: value.clone(),
                    evidence: evidence.clone(),
                }
            })
            .collect();
        Outcome::Applied(applied)
    }
}
