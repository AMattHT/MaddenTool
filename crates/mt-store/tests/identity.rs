//! Identity reconciliation: one person per draft year, never a namesake merge.

use std::path::{Path, PathBuf};

use mt_store::{AsOf, Corrections, Format, ImportOptions, PlayerRow, Store};

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

const NFL_V1: &str = "synthetic-nflverse-draft-picks-v1.csv";
const REDRAFT: &str = "synthetic-legacy-redraft.csv";
const DUAL: &str = "synthetic-legacy-dual-draft.csv";

/// A verified correction for the synthetic "Boomer" AFL row, mirroring the real
/// 1964 Bob Brown case: the AFL row carries a namesake's college and position.
const BOOMER_FIX: &str = r#"{
  "version": 1,
  "corrections": [{
    "id": "synthetic-1963-afl-r1p4-boomer",
    "format": "legacy-player-lookup",
    "natural_key": "1963|AFL|1|4|syntheticboomer|deltatech",
    "expect": { "college": "Delta Tech", "position": "DT" },
    "set": { "college": "Plains State", "position": "T" },
    "evidence": ["synthetic test evidence"],
    "note": "synthetic"
  }]
}"#;

fn options(format: Format, file: &str, corrections: Corrections) -> ImportOptions {
    ImportOptions {
        format,
        file: fixture(file),
        as_of: AsOf::parse("2026-10-28").unwrap(),
        source_url: None,
        label: None,
        synthetic: true,
        corrections,
    }
}

fn import(store: &mut Store, format: Format, file: &str) -> mt_store::ImportSummary {
    store
        .import(&options(format, file, Corrections::none()))
        .unwrap()
}

fn with_surname<'a>(rows: &'a [PlayerRow], surname: &str) -> Vec<&'a PlayerRow> {
    rows.iter()
        .filter(|r| r.name.ends_with(&format!(" {surname}")))
        .collect()
}

#[test]
fn redrafted_player_is_one_person_in_either_import_order() {
    for legacy_first in [true, false] {
        let dir = tempfile::tempdir().unwrap();
        let mut store = Store::open(&dir.path().join("p.db")).unwrap();
        if legacy_first {
            import(&mut store, Format::LegacyPlayerLookup, REDRAFT);
            import(&mut store, Format::NflverseDraftPicks, NFL_V1);
        } else {
            import(&mut store, Format::NflverseDraftPicks, NFL_V1);
            import(&mut store, Format::LegacyPlayerLookup, REDRAFT);
        }
        // Repeat both imports, then reopen: still one person.
        import(&mut store, Format::LegacyPlayerLookup, REDRAFT);
        import(&mut store, Format::NflverseDraftPicks, NFL_V1);
        drop(store);
        let store = Store::open_existing(&dir.path().join("p.db")).unwrap();

        let y2001 = store.players_by_year(2001, None).unwrap();
        let y2002 = store.players_by_year(2002, None).unwrap();
        let a2001 = with_surname(&y2001, "Alpha");
        let a2002 = with_surname(&y2002, "Alpha");
        assert_eq!(a2002.len(), 1, "legacy_first={legacy_first}: {a2002:#?}");
        assert_eq!(a2001.len(), 1, "legacy_first={legacy_first}");
        assert_eq!(a2001[0].player_uid, a2002[0].player_uid);
        assert_eq!(
            a2002[0].sources,
            vec!["legacy-player-lookup", "nflverse-draft-picks"]
        );
        // Both sources' records of the 2002 selection are kept.
        assert_eq!(a2002[0].selections.len(), 2);
        let person = store.player(&a2002[0].player_uid).unwrap().unwrap();
        assert_eq!(person.selections.len(), 4, "two sources x two drafts");
        assert!(
            person
                .external_ids
                .contains(&("pfr".into(), "SyntAl00".into()))
        );
    }
}

#[test]
fn previously_issued_uids_still_resolve_after_a_merge() {
    let dir = tempfile::tempdir().unwrap();
    let mut store = Store::open(&dir.path().join("p.db")).unwrap();
    import(&mut store, Format::LegacyPlayerLookup, REDRAFT);
    // Before nflverse arrives, the 2002 legacy row is its own provisional player.
    let provisional = store.players_by_year(2002, None).unwrap()[0]
        .player_uid
        .clone();
    let s = import(&mut store, Format::NflverseDraftPicks, NFL_V1);
    assert_eq!(s.reconciliation.merges.len(), 1);
    let merge = &s.reconciliation.merges[0];
    assert_eq!(merge.from_uid, provisional);
    assert_eq!(merge.reason, "draft_slot_surname");

    let canonical = store.players_by_year(2002, None).unwrap();
    let alpha = with_surname(&canonical, "Alpha")[0];
    assert_ne!(alpha.player_uid, provisional);
    let resolved = store.player(&provisional).unwrap().unwrap();
    assert_eq!(resolved.player_uid, alpha.player_uid);
    assert_eq!(resolved.requested_uid, provisional);
    assert_eq!(resolved.aliases, vec![provisional.clone()]);
    assert!(store.player("mt-0000000000000000").unwrap().is_none());
    assert_eq!(store.info().unwrap().merged_aliases, 1);
}

#[test]
fn afl_and_nfl_selections_of_one_person_count_once() {
    let dir = tempfile::tempdir().unwrap();
    let mut store = Store::open(&dir.path().join("p.db")).unwrap();
    let s = import(&mut store, Format::LegacyPlayerLookup, DUAL);
    let mut reasons: Vec<&str> = s
        .reconciliation
        .merges
        .iter()
        .map(|m| m.reason.as_str())
        .collect();
    reasons.sort();
    assert_eq!(
        reasons,
        vec![
            "dual_draft_career_footprint",
            "dual_draft_name_college",
            "dual_draft_name_college",
            "dual_draft_name_variant_college"
        ]
    );
    let rows = store.players_by_year(1963, None).unwrap();
    for surname in ["Twin", "Blank", "Kilo", "Oscar"] {
        let found = with_surname(&rows, surname);
        assert_eq!(found.len(), 1, "{surname}");
        let leagues: Vec<_> = found[0]
            .selections
            .iter()
            .map(|s| s.league.as_deref().unwrap())
            .collect();
        assert_eq!(
            leagues,
            vec!["NFL", "AFL"],
            "{surname}: both selections kept, NFL first"
        );
    }
    // The NFL row has the career; the AFL row was blank. Neither value is invented.
    let blank = with_surname(&rows, "Blank")[0];
    assert_eq!(blank.wav, Some(4));
    assert_eq!(blank.selections[1].wav, None);
}

#[test]
fn namesakes_and_unsupported_pairs_stay_separate() {
    let dir = tempfile::tempdir().unwrap();
    let mut store = Store::open(&dir.path().join("p.db")).unwrap();
    import(&mut store, Format::LegacyPlayerLookup, DUAL);
    import(&mut store, Format::LegacyPlayerLookup, DUAL);
    let rows = store.players_by_year(1963, None).unwrap();
    assert_eq!(with_surname(&rows, "Differ").len(), 2, "careers conflict");
    assert_eq!(with_surname(&rows, "Lima").len(), 2, "different men");
    assert_eq!(with_surname(&rows, "Mike").len(), 3, "ambiguous partner");
    // Without a verified correction, the AFL Boomer row's college disagrees: not merged.
    assert_eq!(with_surname(&rows, "Boomer").len(), 3);
    let robert = rows
        .iter()
        .find(|r| r.name == "Synthetic Robert Boomer")
        .unwrap();
    assert_eq!(robert.wav, Some(51), "never takes the namesake's career");

    let kinds: Vec<String> = store
        .conflicts()
        .unwrap()
        .into_iter()
        .map(|c| c.kind)
        .collect();
    for kind in [
        "dual_draft_career_conflict",
        "dual_draft_alias_unconfirmed",
        "dual_draft_ambiguous",
        "dual_draft_college_conflict",
    ] {
        assert!(kinds.iter().any(|k| k == kind), "{kind} not in {kinds:?}");
    }
}

#[test]
fn verified_correction_links_the_afl_selection_to_the_right_person() {
    let dir = tempfile::tempdir().unwrap();
    let mut store = Store::open(&dir.path().join("p.db")).unwrap();
    let fix = Corrections::from_json(BOOMER_FIX).unwrap();
    let s = store
        .import(&options(Format::LegacyPlayerLookup, DUAL, fix.clone()))
        .unwrap();
    assert_eq!(s.corrected, 1);
    let rows = store.players_by_year(1963, None).unwrap();
    let boomers = with_surname(&rows, "Boomer");
    assert_eq!(boomers.len(), 2, "the tackle once, his namesake once");
    let tackle = boomers
        .iter()
        .find(|r| r.name == "Synthetic Boomer")
        .unwrap();
    assert_eq!(tackle.college.as_deref(), Some("Plains State"));
    assert_eq!(tackle.selections.len(), 2);
    let afl = &tackle.selections[1];
    assert_eq!(
        (
            afl.league.as_deref(),
            afl.college.as_deref(),
            afl.position.as_deref()
        ),
        (Some("AFL"), Some("Plains State"), Some("T"))
    );
    assert_eq!(afl.corrections, vec!["synthetic-1963-afl-r1p4-boomer"]);
    let robert = boomers
        .iter()
        .find(|r| r.name == "Synthetic Robert Boomer")
        .unwrap();
    assert_eq!((robert.wav, robert.selections.len()), (Some(51), 1));

    // The raw row is untouched and the original values are recorded.
    let (raw, source_value): (String, String) = store
        .connection()
        .query_row(
            "SELECT sr.raw_json, rc.source_value FROM record_correction rc
             JOIN source_record sr ON sr.id = rc.source_record_id WHERE rc.field = 'college'",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    assert!(raw.contains("\"College/Univ\":\"Delta Tech\""), "{raw}");
    assert_eq!(source_value, "Delta Tech");

    // Re-importing is stable; a database first imported without the correction is
    // updated and merged once the correction exists.
    let again = store
        .import(&options(Format::LegacyPlayerLookup, DUAL, fix.clone()))
        .unwrap();
    assert_eq!((again.updated, again.reconciliation.merges.len()), (0, 0));
    let dir2 = tempfile::tempdir().unwrap();
    let mut late = Store::open(&dir2.path().join("p.db")).unwrap();
    import(&mut late, Format::LegacyPlayerLookup, DUAL);
    let s = late
        .import(&options(Format::LegacyPlayerLookup, DUAL, fix))
        .unwrap();
    assert_eq!((s.updated, s.reconciliation.merges.len()), (1, 1));
    assert_eq!(
        with_surname(&late.players_by_year(1963, None).unwrap(), "Boomer").len(),
        2
    );
}

#[test]
fn uncorrected_rows_keep_their_schema_v1_content_hash() {
    use sha2::{Digest, Sha256};
    let dir = tempfile::tempdir().unwrap();
    let mut store = Store::open(&dir.path().join("p.db")).unwrap();
    import(&mut store, Format::LegacyPlayerLookup, REDRAFT);
    let (raw, stored): (String, String) = store
        .connection()
        .query_row(
            "SELECT raw_json, content_sha256 FROM source_record LIMIT 1",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    // Schema v1 hashed MAPPING_VERSION, NUL, raw row. Upgrading must not mark every row changed.
    let v1: String = Sha256::digest(format!("1\0{raw}").as_bytes())
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    assert_eq!(stored, v1);
}

#[test]
fn stale_correction_is_reported_and_not_applied() {
    let dir = tempfile::tempdir().unwrap();
    let mut store = Store::open(&dir.path().join("p.db")).unwrap();
    let stale = BOOMER_FIX.replace(r#""position": "DT" }"#, r#""position": "LB" }"#);
    let s = store
        .import(&options(
            Format::LegacyPlayerLookup,
            DUAL,
            Corrections::from_json(&stale).unwrap(),
        ))
        .unwrap();
    assert_eq!(s.corrected, 0);
    assert!(s.issues.iter().any(|i| i.kind == "correction_stale"));
    assert_eq!(
        with_surname(&store.players_by_year(1963, None).unwrap(), "Boomer").len(),
        3
    );
}

#[test]
fn corrections_may_only_change_descriptive_fields() {
    let bad = BOOMER_FIX.replace(r#""set": { "college""#, r#""set": { "wav": "1", "college""#);
    assert!(Corrections::from_json(&bad).is_err());
    // The shipped corrections load.
    assert!(!Corrections::builtin().corrections.is_empty());
}

#[test]
fn schema_v1_database_is_migrated_and_repaired() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("v1.db");
    {
        let mut conn = rusqlite::Connection::open(&path).unwrap();
        mt_store::schema::migrate_to(&mut conn, 1).unwrap();
        // The S1 duplicate as schema v1 stored it: one person split over two players.
        conn.execute_batch(
            "INSERT INTO dataset_import (id, format, as_of, file_name, file_sha256, synthetic,
                attribution, imported_at, tool_version)
             VALUES (1, 'legacy-player-lookup', '2026-10-28', 'f.csv', 'x', 1, 'a', 't', '0.1.0'),
                    (2, 'nflverse-draft-picks', '2026-10-28', 'g.csv', 'y', 1, 'a', 't', '0.1.0');
             INSERT INTO player (id, uid, created_import_id) VALUES (1, 'mt-aaaa', 1), (2, 'mt-bbbb', 1);
             INSERT INTO player_external_id VALUES ('pfr', 'SyntAl00', 1, 2);
             INSERT INTO source_record (format, natural_key, player_id, link_method, first_import_id,
                last_import_id, content_sha256, full_name, first_name, last_name, draft_year,
                draft_status, draft_round, draft_pick, league, wav, raw_json)
             VALUES ('legacy-player-lookup', 'k1', 1, 'new', 1, 1, 'h', 'Synthetic Alpha', 'Synthetic', 'Alpha', 2001, 'drafted', 1, 1, 'NFL', 118, '{}'),
                    ('legacy-player-lookup', 'k2', 2, 'new', 1, 1, 'h', 'Synthetic Alpha', 'Synthetic', 'Alpha', 2002, 'drafted', 1, 5, 'NFL', 118, '{}'),
                    ('nflverse-draft-picks', '2001:1', 1, 'draft_slot_surname', 2, 2, 'h', 'Synthetic Alpha', NULL, NULL, 2001, 'drafted', 1, 1, 'NFL', 120, '{}'),
                    ('nflverse-draft-picks', '2002:5', 1, 'external_id', 2, 2, 'h', 'Synthetic Alpha', NULL, NULL, 2002, 'drafted', 1, 5, 'NFL', 120, '{}');",
        )
        .unwrap();
        assert_eq!(mt_store::schema::schema_version(&conn).unwrap(), 1);
    }
    let mut store = Store::open_existing(&path).unwrap();
    assert_eq!(store.schema_version().unwrap(), 2);
    assert_eq!(
        store.players_by_year(2002, None).unwrap().len(),
        2,
        "v1 duplicate before repair"
    );
    let r = store.reconcile().unwrap();
    assert_eq!(r.merges.len(), 1);
    assert_eq!(store.players_by_year(2002, None).unwrap().len(), 1);
    assert_eq!(
        store.player("mt-bbbb").unwrap().unwrap().player_uid,
        "mt-aaaa"
    );
    assert!(store.reconcile().unwrap().merges.is_empty(), "idempotent");
}

#[test]
fn merge_is_refused_when_external_ids_disagree() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("p.db");
    let mut store = Store::open(&path).unwrap();
    import(&mut store, Format::LegacyPlayerLookup, REDRAFT);
    import(&mut store, Format::NflverseDraftPicks, NFL_V1);
    // Give the 2002 legacy row a separate player holding a different PFR id, as if a
    // later source named another man for that selection.
    store
        .connection()
        .execute_batch(
            "INSERT INTO player (uid, created_import_id) VALUES ('mt-other', 1);
             UPDATE source_record SET player_id = (SELECT id FROM player WHERE uid = 'mt-other')
              WHERE format = 'legacy-player-lookup' AND draft_year = 2002;
             INSERT INTO player_external_id VALUES
               ('pfr', 'OtheR00', (SELECT id FROM player WHERE uid = 'mt-other'), 1);",
        )
        .unwrap();
    let r = store.reconcile().unwrap();
    assert!(r.merges.is_empty());
    assert!(
        r.conflicts
            .iter()
            .any(|c| c.kind == "merge_refused" && c.detail.contains("pfr")),
        "{:?}",
        r.conflicts
    );
    assert_eq!(store.players_by_year(2002, None).unwrap().len(), 2);
}
