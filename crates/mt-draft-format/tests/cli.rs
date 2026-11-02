//! `mt-draft inspect` on SYNTHETIC files: output, exit status, and that it never writes.

mod support;

use std::path::Path;
use std::process::{Command, Output};

use support::fixtures::{sha256_file, sha256_hex};
use support::synthetic::{DATA_START, RECORD, Synthetic};

fn inspect(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_mt-draft"))
        .args(args)
        .output()
        .expect("run mt-draft")
}

fn write(dir: &Path, name: &str, s: Synthetic) -> std::path::PathBuf {
    let path = dir.join(name);
    std::fs::write(&path, s.bytes).expect("write synthetic file");
    path
}

/// Hash, length and modification time: what must not change.
fn fingerprint(path: &Path) -> (String, u64, std::time::SystemTime) {
    let meta = std::fs::metadata(path).expect("metadata");
    (
        sha256_file(path),
        meta.len(),
        meta.modified().expect("mtime"),
    )
}

#[test]
fn prints_metadata_and_players() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = write(dir.path(), "SYNTHETIC-class", Synthetic::new(3));
    let out = inspect(&["inspect", path.to_str().unwrap()]);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let text = String::from_utf8(out.stdout).unwrap();
    assert!(text.contains("Madden-27-RL4-9244223"), "{text}");
    assert!(
        text.contains("3 active of 455 slots; 452 inactive, 452 of them blank"),
        "{text}"
    );
    assert!(text.contains("2026-11-02 12:34:56"), "{text}");
    assert!(text.contains("Synthetic Player2"), "{text}");
    assert!(text.contains(&sha256_file(&path)), "{text}");
}

#[test]
fn slot_detail_lists_every_field() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = write(dir.path(), "SYNTHETIC-class", Synthetic::new(3));
    let out = inspect(&["inspect", path.to_str().unwrap(), "--slot", "1"]);
    assert!(out.status.success());
    let text = String::from_utf8(out.stdout).unwrap();
    for needle in [
        "firstName",
        "personaDNA",
        "longSnap",
        "unknown bytes",
        "gen_SYNTHETIC_1",
    ] {
        assert!(text.contains(needle), "missing {needle}: {text}");
    }
    let out = inspect(&["inspect", path.to_str().unwrap(), "--slot", "3"]);
    assert_eq!(
        out.status.code(),
        Some(2),
        "an inactive slot is a usage error"
    );
}

#[test]
fn json_mode_is_machine_readable() {
    let dir = tempfile::tempdir().expect("tempdir");
    let s = Synthetic::new(3);
    let bytes = s.bytes.clone();
    let path = write(dir.path(), "SYNTHETIC-class", s);
    let out = inspect(&["inspect", path.to_str().unwrap(), "--json"]);
    assert!(out.status.success());
    let doc: serde_json::Value = serde_json::from_slice(&out.stdout).expect("valid JSON");
    assert_eq!(doc["schema"], "mt-draft-inspect/1");
    assert_eq!(doc["source"]["sha256"], sha256_hex(&bytes));
    assert_eq!(doc["header"]["build"], "Madden-27-RL4-9244223");
    assert_eq!(doc["header"]["observedBuild"], true);
    assert_eq!(doc["capacity"], 455);
    assert_eq!(doc["activeCount"], 3);
    assert_eq!(doc["blankInactiveSlots"], 452);
    let players = doc["players"].as_array().unwrap();
    assert_eq!(players.len(), 3);
    let p = &players[1];
    assert_eq!(p["slot"], 1);
    assert_eq!(p["fields"].as_object().unwrap().len(), 82);
    assert_eq!(p["fields"]["lastName"], "Player1");
    assert_eq!(p["fields"]["weight"], 205);
    assert_eq!(
        p["fields"]["personaDNA"],
        serde_json::json!([11, 22, 33, 44, 55])
    );
    assert_eq!(p["unknownHex"].as_str().unwrap(), "00".repeat(41));
    let start = DATA_START + RECORD;
    assert_eq!(p["recordSha256"], sha256_hex(&bytes[start..start + RECORD]));
    assert_eq!(p["visuals"]["genericHeadName"], "gen_SYNTHETIC_1");
    assert!(p["visuals"].get("bodyType").is_none());
}

#[test]
fn refuses_invalid_input_with_a_non_zero_exit() {
    let dir = tempfile::tempdir().expect("tempdir");
    let mut truncated = Synthetic::new(3);
    truncated.bytes.truncate(1_000_000);
    let mut wrong_game = Synthetic::new(3);
    wrong_game.set_build("Madden-26-RL1-1234567");
    let mut bad_json = Synthetic::new(3);
    bad_json.set_visual(0, b"{\"a\":");
    let cases = [
        ("truncated", truncated, "truncated"),
        ("wrong-game", wrong_game, "not Madden 27"),
        ("bad-json", bad_json, "slot 0"),
    ];
    for (name, s, message) in cases {
        let path = write(dir.path(), name, s);
        for json in [false, true] {
            let mut args = vec!["inspect", path.to_str().unwrap()];
            if json {
                args.push("--json");
            }
            let out = inspect(&args);
            assert_eq!(out.status.code(), Some(1), "{name}");
            assert!(out.stdout.is_empty(), "{name}: nothing on stdout");
            let err = String::from_utf8_lossy(&out.stderr);
            assert!(err.contains(message), "{name}: {err}");
        }
    }
    let missing = dir.path().join("does-not-exist");
    let out = inspect(&["inspect", missing.to_str().unwrap()]);
    assert_eq!(out.status.code(), Some(1));
    assert_eq!(inspect(&["inspect"]).status.code(), Some(2));
    assert_eq!(inspect(&["frobnicate", "x"]).status.code(), Some(2));
    assert_eq!(inspect(&[]).status.code(), Some(2));
}

#[test]
fn inspect_never_changes_its_input() {
    let dir = tempfile::tempdir().expect("tempdir");
    let mut bad = Synthetic::new(3);
    bad.put(0, b"NOTCHUNK");
    let files = [
        write(dir.path(), "valid", Synthetic::new(389)),
        write(dir.path(), "invalid", bad),
    ];
    for path in &files {
        let before = fingerprint(path);
        for extra in [&[][..], &["--json"][..], &["--slot", "0"][..]] {
            let mut args = vec!["inspect", path.to_str().unwrap()];
            args.extend_from_slice(extra);
            inspect(&args);
        }
        assert_eq!(fingerprint(path), before, "{}", path.display());
    }
}
