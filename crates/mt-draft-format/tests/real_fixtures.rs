//! Opt-in: the game's own draft classes, from `MADDENTOOL_FIXTURES` (see `support::fixtures`
//! for skip and failure rules). Every active player and every mapped field is read; the
//! totals are printed. Run with `-- --nocapture` to see them.

mod support;

use std::collections::BTreeMap;
use std::process::Command;

use mt_draft_format::layout::{CAPACITY, VISUAL_SIZE};
use mt_draft_format::{FIELDS, FieldValue, UNKNOWN_OFFSETS, read_file};
use support::fixtures::{fixture_dir, present_fixtures, sha256_file, sha256_hex};

#[test]
fn game_classes_parse_completely() {
    let Some(dir) = fixture_dir("game_classes_parse_completely") else {
        return;
    };
    let mut grand = (0usize, 0usize, 0usize);
    for (fx, path) in present_fixtures("game_classes_parse_completely", &dir) {
        let bytes = std::fs::read(&path).expect("read fixture");
        let class = read_file(&path).unwrap_or_else(|e| panic!("{}: {e}", fx.name));
        let h = class.header();
        assert_eq!(h.build.text, fx.build, "{}", fx.name);
        assert!(h.build.is_observed());
        assert_eq!(class.active_count(), fx.active, "{}", fx.name);
        assert_eq!(class.capacity(), CAPACITY);
        assert_eq!(class.players().len(), fx.active);
        assert!(
            class.warnings().is_empty(),
            "{}: {:?}",
            fx.name,
            class.warnings()
        );
        assert_eq!(class.blank_inactive_slots(), CAPACITY - fx.active);
        assert_eq!(class.source_bytes(), &bytes[..], "the model keeps the file");

        let mut field_values = 0;
        let mut nonzero_unknown = 0;
        let mut missing_keys: BTreeMap<&str, usize> = BTreeMap::new();
        let mut dna_counts: BTreeMap<usize, usize> = BTreeMap::new();
        let mut rounds: BTreeMap<u32, usize> = BTreeMap::new();
        for p in class.players() {
            let record = class.record_bytes(p.slot());
            assert_eq!(&p.attribute_bytes()[..], &record[VISUAL_SIZE..]);
            let json = p.visuals().json().as_bytes();
            assert_eq!(
                json,
                &record[..json.len()],
                "slot {}: JSON text is verbatim",
                p.slot()
            );
            assert!(record[json.len()..VISUAL_SIZE].iter().all(|&b| b == 0));

            for (spec, value) in p.fields() {
                field_values += 1;
                if let FieldValue::Text(_) = value {
                    assert!(
                        value.as_text().is_some(),
                        "{} slot {} {}: not ASCII",
                        fx.name,
                        p.slot(),
                        spec.name
                    );
                }
            }
            // Recorded evidence (DISCOVERY-REPORT 5.2), checked rather than assumed.
            assert_eq!(
                p.number("const6b"),
                Some(127),
                "{} slot {}",
                fx.name,
                p.slot()
            );
            assert_eq!(
                p.number("const7d"),
                Some(1),
                "{} slot {}",
                fx.name,
                p.slot()
            );
            assert!(!p.text("firstName").unwrap_or("").is_empty());
            nonzero_unknown += p.unknown_bytes().iter().filter(|&&b| b != 0).count();
            for key in ["bodyType", "skinTone", "genericHeadName", "loadouts"] {
                if p.visuals().get(key).is_none() {
                    *missing_keys.entry(key).or_default() += 1;
                }
            }
            let dna = p
                .field("personaDNA")
                .and_then(FieldValue::as_numbers)
                .unwrap();
            *dna_counts
                .entry(dna.iter().filter(|&&v| v != 0).count())
                .or_default() += 1;
            *rounds.entry(p.number("draftRound").unwrap()).or_default() += 1;
        }
        assert_eq!(field_values, fx.active * FIELDS.len());
        println!(
            "REAL {}: sha256 ok, build {}, saved {}, {} active of {} slots, {} blank inactive; \
             {} players x {} fields = {} field values; unknown bytes {} x {} = {}, non-zero {}; \
             visual keys missing {:?}; DNA traits per player {:?}; rounds {:?}",
            fx.name,
            h.build.text,
            h.timestamp,
            class.active_count(),
            class.capacity(),
            class.blank_inactive_slots(),
            class.players().len(),
            FIELDS.len(),
            field_values,
            class.players().len(),
            UNKNOWN_OFFSETS.len(),
            class.players().len() * UNKNOWN_OFFSETS.len(),
            nonzero_unknown,
            missing_keys,
            dna_counts,
            rounds,
        );
        grand.0 += 1;
        grand.1 += class.players().len();
        grand.2 += field_values;
    }
    println!(
        "REAL TOTAL: {} fixtures, {} players, {} field values",
        grand.0, grand.1, grand.2
    );
}

#[test]
fn inspect_leaves_game_files_unchanged() {
    let Some(dir) = fixture_dir("inspect_leaves_game_files_unchanged") else {
        return;
    };
    for (fx, path) in present_fixtures("inspect_leaves_game_files_unchanged", &dir) {
        let before = sha256_file(&path);
        let out = Command::new(env!("CARGO_BIN_EXE_mt-draft"))
            .args(["inspect", path.to_str().unwrap(), "--json"])
            .output()
            .expect("run mt-draft");
        assert!(
            out.status.success(),
            "{}: {}",
            fx.name,
            String::from_utf8_lossy(&out.stderr)
        );
        let human = Command::new(env!("CARGO_BIN_EXE_mt-draft"))
            .args(["inspect", path.to_str().unwrap()])
            .output()
            .expect("run mt-draft");
        assert!(human.status.success());
        let after = sha256_file(&path);
        assert_eq!(before, fx.sha256);
        assert_eq!(after, before, "{}: inspect changed the file", fx.name);
        let doc: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
        assert_eq!(doc["source"]["sha256"], fx.sha256);
        assert_eq!(doc["players"].as_array().unwrap().len(), fx.active);
        let first = &doc["players"][0];
        let bytes = std::fs::read(&path).unwrap();
        assert_eq!(first["recordSha256"], sha256_hex(&bytes[0x46..0x46 + 5876]));
        println!("REAL {}: sha256 before and after inspect {after}", fx.name);
    }
}
