//! Opt-in differential test against the legacy JavaScript parser (development oracle only).
//!
//! Needs `MADDENTOOL_FIXTURES` (game classes) and `MADDENTOOL_LEGACY_MDG` (a checkout of
//! Madden-Draft-Generator; its `M27Parser.js` must match the pinned SHA-256), plus `node`
//! on PATH or `MADDENTOOL_NODE`. Either variable unset: visible skip. Set but broken: failure.
//!
//! For every fixture it compares:
//! - the legacy field map's declared offsets and widths with [`FIELDS`];
//! - which slots each side treats as players;
//! - every mapped field of every active record, through `mt-draft inspect --json`;
//! - every visual object, as parsed JSON (numbers compared as f64);
//! - SHA-256 of all 455 records: the oracle hashes the file, the Rust side hashes the
//!   bytes its model kept.
//!
//! Deliberate differences, normalized and counted, never silently:
//! - `assetName`: legacy returns `null` for an empty field; Rust returns `""`.
//! - `personaDNA`: legacy drops zero slots; Rust keeps all five positionally. Compared as
//!   Rust's non-zero slots in order.
//! - `const6b`, `const7d`: legacy declares them with an expected value but never reads them.
//!   Rust's values are compared with the declared values.

mod support;

use std::path::{Path, PathBuf};
use std::process::Command;

use mt_draft_format::report::sha256_hex;
use mt_draft_format::{FIELDS, read_file};
use serde_json::Value;
use support::fixtures::{fixture_dir, present_fixtures};

const PINNED_PARSER_SHA256: &str =
    "ce24b0063c24e1ca7ce8be8a94717dc96716a49073fa4f0ed6f0e4430c1f067b";
const PARSER_PATH: &str = "server/src/vendor/draft-class/M27Parser.js";

fn legacy_parser() -> Option<PathBuf> {
    let Some(root) = std::env::var_os("MADDENTOOL_LEGACY_MDG") else {
        eprintln!(
            "SKIPPED differential: MADDENTOOL_LEGACY_MDG is not set; the legacy comparison did not run"
        );
        return None;
    };
    let parser = Path::new(&root).join(PARSER_PATH);
    assert!(
        parser.is_file(),
        "MADDENTOOL_LEGACY_MDG: {} does not exist",
        parser.display()
    );
    let sha = sha256_hex(&std::fs::read(&parser).unwrap());
    assert_eq!(
        sha,
        PINNED_PARSER_SHA256,
        "{} is not the pinned legacy parser",
        parser.display()
    );
    Some(parser)
}

fn run_json(cmd: &mut Command, what: &str) -> Value {
    let out = cmd
        .output()
        .unwrap_or_else(|e| panic!("{what}: cannot run: {e}"));
    assert!(
        out.status.success(),
        "{what} failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    serde_json::from_slice(&out.stdout).unwrap_or_else(|e| panic!("{what}: bad JSON: {e}"))
}

/// Parsed-JSON equality with numbers compared by value, so `1.0` equals `1`.
fn same_json(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Number(x), Value::Number(y)) => x.as_f64() == y.as_f64(),
        (Value::Array(x), Value::Array(y)) => {
            x.len() == y.len() && x.iter().zip(y).all(|(p, q)| same_json(p, q))
        }
        (Value::Object(x), Value::Object(y)) => {
            x.len() == y.len()
                && x.iter()
                    .all(|(k, v)| y.get(k).is_some_and(|w| same_json(v, w)))
        }
        _ => a == b,
    }
}

#[derive(Default)]
struct Totals {
    fixtures: usize,
    records: usize,
    field_comparisons: usize,
    const_comparisons: usize,
    visuals: usize,
    record_hashes: usize,
    asset_null: usize,
    dna_with_zero_slots: usize,
    discrepancies: Vec<String>,
}

#[test]
fn rust_reader_agrees_with_the_legacy_parser() {
    let Some(dir) = fixture_dir("differential") else {
        return;
    };
    let fixtures = present_fixtures("differential", &dir);
    let Some(parser) = legacy_parser() else {
        return;
    };
    let node = std::env::var("MADDENTOOL_NODE").unwrap_or_else(|_| "node".to_string());
    let oracle = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/oracle/legacy-m27-oracle.cjs");
    let mut t = Totals::default();

    for (fx, path) in fixtures {
        let legacy = run_json(
            Command::new(&node).arg(&oracle).arg(&parser).arg(&path),
            "legacy oracle",
        );
        let ours = run_json(
            Command::new(env!("CARGO_BIN_EXE_mt-draft")).args([
                "inspect",
                path.to_str().unwrap(),
                "--json",
            ]),
            "mt-draft inspect --json",
        );
        let class = read_file(&path).unwrap();
        let mut issues: Vec<String> = Vec::new();
        let mut bad = |msg: String| issues.push(format!("{}: {msg}", fx.name));
        let mut fields = 0;
        let mut consts = 0;

        assert_eq!(legacy["parserSha256"], PINNED_PARSER_SHA256);
        assert_eq!(legacy["fileSha256"], fx.sha256);

        // 1. Field map: offsets and widths as legacy declares them.
        let map = legacy["fieldMap"].as_object().unwrap();
        if map.len() != FIELDS.len() {
            bad(format!(
                "legacy maps {} fields, Rust {}",
                map.len(),
                FIELDS.len()
            ));
        }
        for f in &FIELDS {
            match map.get(f.name) {
                None => bad(format!("legacy has no field {}", f.name)),
                Some(l) if l["offset"] != f.offset || l["width"] != f.width => bad(format!(
                    "{}: legacy offset {} width {}, Rust offset {} width {}",
                    f.name, l["offset"], l["width"], f.offset, f.width
                )),
                Some(_) => {}
            }
        }

        // 2. Which slots are players.
        let records = legacy["records"].as_array().unwrap();
        assert_eq!(records.len(), 455, "legacy walks all physical slots");
        let legacy_used: Vec<usize> = records
            .iter()
            .filter(|r| r["used"] == true)
            .map(|r| r["index"].as_u64().unwrap() as usize)
            .collect();
        let rust_active: Vec<usize> = (0..class.active_count()).collect();
        if legacy_used != rust_active {
            bad(format!(
                "legacy used slots {legacy_used:?} differ from Rust active slots 0..{}",
                class.active_count()
            ));
        }

        // 3. Raw bytes: the model's record bytes against the oracle's hash of the file.
        for (slot, r) in records.iter().enumerate() {
            t.record_hashes += 1;
            if r["recordSha256"] != sha256_hex(class.record_bytes(slot)) {
                bad(format!("slot {slot}: record bytes differ from the file"));
            }
            if slot >= class.active_count() && !r["visuals"].is_null() {
                bad(format!(
                    "slot {slot}: legacy found visuals in an inactive slot"
                ));
            }
        }

        // 4. Every mapped field and visual object of every active record.
        let players = ours["players"].as_array().unwrap();
        for p in players {
            let slot = p["slot"].as_u64().unwrap() as usize;
            let r = &records[slot];
            if p["recordSha256"] != r["recordSha256"] {
                bad(format!(
                    "slot {slot}: inspect --json record hash differs from the file"
                ));
            }
            let la = &r["attrs"];
            for (key, expect) in [("index", slot), ("draftPosition", slot)] {
                if la[key] != expect {
                    bad(format!("slot {slot}: legacy {key} {}", la[key]));
                }
            }
            for f in &FIELDS {
                let rv = &p["fields"][f.name];
                match f.name {
                    "const6b" | "const7d" => {
                        consts += 1;
                        if *rv != map[f.name]["value"] {
                            bad(format!(
                                "slot {slot} {}: Rust {rv}, legacy declares {}",
                                f.name, map[f.name]["value"]
                            ));
                        }
                    }
                    "assetName" => {
                        fields += 1;
                        let lv = &la[f.name];
                        let ok = if lv.is_null() { *rv == "" } else { lv == rv };
                        if lv.is_null() {
                            t.asset_null += 1;
                        }
                        if !ok {
                            bad(format!("slot {slot} assetName: legacy {lv}, Rust {rv}"));
                        }
                    }
                    "personaDNA" => {
                        fields += 1;
                        let all: Vec<u64> = rv
                            .as_array()
                            .unwrap()
                            .iter()
                            .map(|v| v.as_u64().unwrap())
                            .collect();
                        if all.contains(&0) {
                            t.dna_with_zero_slots += 1;
                        }
                        let nonzero: Vec<u64> = all.into_iter().filter(|&v| v != 0).collect();
                        if la[f.name] != serde_json::json!(nonzero) {
                            bad(format!(
                                "slot {slot} personaDNA: legacy {}, Rust {rv}",
                                la[f.name]
                            ));
                        }
                    }
                    _ => {
                        fields += 1;
                        if la[f.name] != *rv {
                            bad(format!(
                                "slot {slot} {}: legacy {}, Rust {rv}",
                                f.name, la[f.name]
                            ));
                        }
                    }
                }
            }
            t.visuals += 1;
            if !same_json(&r["visuals"], &p["visuals"]) {
                bad(format!("slot {slot}: visual JSON content differs"));
            }
        }
        println!(
            "DIFF {}: {} active records (legacy used {}), {} field comparisons, {} declared-constant checks, \
             {} visual objects, {} record hashes; assetName null->\"\" {}, DNA records with zero slots {}",
            fx.name,
            players.len(),
            legacy_used.len(),
            fields,
            consts,
            players.len(),
            records.len(),
            players
                .iter()
                .filter(|p| p["fields"]["assetName"] == "")
                .count(),
            players
                .iter()
                .filter(|p| p["fields"]["personaDNA"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|v| v == 0))
                .count(),
        );
        t.discrepancies.extend(issues);
        t.fixtures += 1;
        t.records += players.len();
        t.field_comparisons += fields;
        t.const_comparisons += consts;
    }
    println!(
        "DIFF TOTAL: {} fixtures, {} records, {} field comparisons, {} declared-constant checks, {} visual objects, \
         {} record hashes, {} discrepancies; normalized: assetName null {}, DNA zero-slot records {}",
        t.fixtures,
        t.records,
        t.field_comparisons,
        t.const_comparisons,
        t.visuals,
        t.record_hashes,
        t.discrepancies.len(),
        t.asset_null,
        t.dna_with_zero_slots
    );
    for d in t.discrepancies.iter().take(50) {
        println!("DISCREPANCY {d}");
    }
    assert!(
        t.discrepancies.is_empty(),
        "{} discrepancies",
        t.discrepancies.len()
    );
}
