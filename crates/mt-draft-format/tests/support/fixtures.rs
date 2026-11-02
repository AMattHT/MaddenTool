//! Real game fixtures, found through `MADDENTOOL_FIXTURES`. They never enter Git.
//!
//! - Unset: the test prints a visible SKIPPED line and passes; CI runs this way.
//! - Set to something that is not a directory, or a directory holding none of the
//!   known fixtures: the test fails, because the configuration is broken.
//! - A fixture whose SHA-256 differs from the recorded hash: the test fails.
//! - A missing fixture is skipped with a visible line.
//! - `MADDENTOOL_REQUIRE_FIXTURES=1` turns every skip into a failure: an unset
//!   `MADDENTOOL_FIXTURES` (and, for the differential test, an unset
//!   `MADDENTOOL_LEGACY_MDG`) or any missing fixture.

use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

pub struct Fixture {
    pub name: &'static str,
    pub sha256: &'static str,
    pub build: &'static str,
    pub active: usize,
}

/// Game-written classes. Hashes: `docs/DISCOVERY-REPORT.md` 5.2 for TEMP1-3; the
/// template hash is recorded in this crate's README with its source.
pub const FIXTURES: [Fixture; 4] = [
    Fixture {
        name: "CAREERDRAFT-TEMP1",
        sha256: "3e8b5a3f2634df3645351592a01b74f3454aa440dc79a4cb1d8f750fa2dfde71",
        build: "Madden-27-RL4-9244223",
        active: 389,
    },
    Fixture {
        name: "CAREERDRAFT-TEMP2",
        sha256: "55c91c004873d16938d3d48d032e6cf4fdb526982662953b82ef0f93612fb0e1",
        build: "Madden-27-RL4-9244223",
        active: 389,
    },
    Fixture {
        name: "CAREERDRAFT-TEMP3",
        sha256: "2611dda25e41f30aaa30db4a0f8038c715445907aadfbd62789df105fc31a457",
        build: "Madden-27-RL4-9244223",
        active: 389,
    },
    Fixture {
        name: "CAREERDRAFT-2027Template",
        sha256: "b62bbe814303b2329cde7b124c1e3da99a8ec9d4d796cfdd2204f8f702557ab3",
        build: "Madden-27-RL1-9081279",
        active: 389,
    },
];

pub fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

pub fn sha256_file(path: &Path) -> String {
    sha256_hex(&std::fs::read(path).unwrap_or_else(|e| panic!("read {}: {e}", path.display())))
}

/// `MADDENTOOL_REQUIRE_FIXTURES=1`: every opt-in input must be configured and present.
pub fn required() -> bool {
    std::env::var("MADDENTOOL_REQUIRE_FIXTURES").is_ok_and(|v| v == "1")
}

/// The configured fixture directory, or `None` (with a SKIPPED line) when unset.
/// In required mode an unset variable is a failure, not a skip.
pub fn fixture_dir(test: &str) -> Option<PathBuf> {
    match std::env::var_os("MADDENTOOL_FIXTURES") {
        None => {
            assert!(
                !required(),
                "{test}: MADDENTOOL_FIXTURES is not set, but MADDENTOOL_REQUIRE_FIXTURES=1 \
                 requires the real-file tests to run"
            );
            eprintln!(
                "SKIPPED {test}: MADDENTOOL_FIXTURES is not set; real-file tests did not run"
            );
            None
        }
        Some(dir) => {
            let dir = PathBuf::from(dir);
            assert!(
                dir.is_dir(),
                "MADDENTOOL_FIXTURES={} is not a directory",
                dir.display()
            );
            Some(dir)
        }
    }
}

/// The present, hash-verified fixtures in `dir`. Fails on a broken configuration.
pub fn present_fixtures(test: &str, dir: &Path) -> Vec<(&'static Fixture, PathBuf)> {
    let require = required();
    let mut found = Vec::new();
    for f in &FIXTURES {
        let path = dir.join(f.name);
        if !path.is_file() {
            assert!(
                !require,
                "{test}: {} is missing from {} and MADDENTOOL_REQUIRE_FIXTURES=1",
                f.name,
                dir.display()
            );
            eprintln!("SKIPPED {test}: {} is not in {}", f.name, dir.display());
            continue;
        }
        let actual = sha256_file(&path);
        assert_eq!(
            actual, f.sha256,
            "{test}: {} has SHA-256 {actual}, expected {}; use an unmodified copy",
            f.name, f.sha256
        );
        found.push((f, path));
    }
    assert!(
        !found.is_empty(),
        "{test}: MADDENTOOL_FIXTURES={} holds none of the known fixtures",
        dir.display()
    );
    found
}
