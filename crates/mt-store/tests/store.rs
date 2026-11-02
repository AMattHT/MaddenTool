//! Behaviour of the player database against the synthetic fixtures.

use std::path::{Path, PathBuf};

use mt_store::{
    AsOf, CareerState, Format, ImportOptions, PlayerRow, Store, StoreError, WavEvidence,
};

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

const NFL_V1: &str = "synthetic-nflverse-draft-picks-v1.csv";
const NFL_V2: &str = "synthetic-nflverse-draft-picks-v2.csv";
const LEGACY: &str = "synthetic-legacy-player-lookup.csv";

fn options(format: Format, file: &str, as_of: &str) -> ImportOptions {
    ImportOptions {
        format,
        file: fixture(file),
        as_of: AsOf::parse(as_of).unwrap(),
        source_url: None,
        label: Some("synthetic test fixture".into()),
        synthetic: true,
    }
}

fn nfl(file: &str, as_of: &str) -> ImportOptions {
    options(Format::NflverseDraftPicks, file, as_of)
}

fn legacy() -> ImportOptions {
    options(Format::LegacyPlayerLookup, LEGACY, "2026-09-28")
}

fn find<'a>(rows: &'a [PlayerRow], name: &str) -> &'a PlayerRow {
    rows.iter()
        .find(|r| r.name == name)
        .unwrap_or_else(|| panic!("{name} not in {rows:#?}"))
}

fn names(rows: &[PlayerRow]) -> Vec<&str> {
    rows.iter().map(|r| r.name.as_str()).collect()
}

#[test]
fn creates_migrates_and_reopens_a_database() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("players.db");
    assert_eq!(Store::open(&path).unwrap().schema_version().unwrap(), 1);
    let store = Store::open_existing(&path).unwrap();
    assert_eq!(store.schema_version().unwrap(), 1);
    let app_id: i64 = store
        .connection()
        .query_row("PRAGMA application_id", [], |r| r.get(0))
        .unwrap();
    assert_eq!(app_id, mt_store::schema::APPLICATION_ID);
}

#[test]
fn query_does_not_create_a_missing_database() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("typo.db");
    assert!(matches!(
        Store::open_existing(&path),
        Err(StoreError::InvalidInput(_))
    ));
    assert!(!path.exists());
}

#[test]
fn refuses_newer_schema_and_foreign_databases() {
    let dir = tempfile::tempdir().unwrap();
    let newer = dir.path().join("newer.db");
    drop(Store::open(&newer).unwrap());
    rusqlite::Connection::open(&newer)
        .unwrap()
        .pragma_update(None, "user_version", 99)
        .unwrap();
    assert!(matches!(
        Store::open(&newer),
        Err(StoreError::NewerSchema {
            found: 99,
            supported: 1
        })
    ));

    let foreign = dir.path().join("foreign.db");
    rusqlite::Connection::open(&foreign)
        .unwrap()
        .execute_batch("CREATE TABLE notes (body TEXT)")
        .unwrap();
    assert!(matches!(
        Store::open(&foreign),
        Err(StoreError::ForeignDatabase)
    ));
}

#[test]
fn import_validates_rows_and_reports_them_by_line() {
    let dir = tempfile::tempdir().unwrap();
    let mut store = Store::open(&dir.path().join("p.db")).unwrap();
    let s = store.import(&nfl(NFL_V1, "2026-10-28")).unwrap();
    assert_eq!(
        (s.rows_read, s.inserted, s.updated, s.unchanged, s.rejected),
        (9, 6, 0, 0, 3)
    );
    let issues: Vec<(Option<u64>, &str)> =
        s.issues.iter().map(|i| (i.line, i.kind.as_str())).collect();
    assert_eq!(
        issues,
        vec![
            (Some(8), "invalid_row"),
            (Some(9), "invalid_row"),
            (Some(10), "duplicate_in_file")
        ]
    );
    assert!(
        s.issues[1].detail.contains("w_av is not a whole number"),
        "{:?}",
        s.issues[1]
    );
    // The same issues are stored with the import.
    assert_eq!(store.issues(s.import_id).unwrap(), s.issues);
    // Rejected rows are not stored: slot 2001:2 still belongs to the first row.
    let rows = store.players_by_year(2001, None).unwrap();
    assert_eq!(
        names(&rows),
        vec![
            "Synthetic Alpha",
            "Synthetic Bravo Jr.",
            "Synthetic Charlie",
            "Synthétic Délta"
        ]
    );
}

#[test]
fn wrong_format_is_refused_and_writes_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let mut store = Store::open(&dir.path().join("p.db")).unwrap();
    let err = store
        .import(&options(Format::NflverseDraftPicks, LEGACY, "2026-10-28"))
        .unwrap_err();
    assert!(matches!(err, StoreError::MissingColumns { .. }), "{err}");
    assert!(store.imports().unwrap().is_empty());
    assert_eq!(store.info().unwrap().players, 0);
}

#[test]
fn missing_zero_and_placeholder_wav_stay_distinct() {
    let dir = tempfile::tempdir().unwrap();
    let mut store = Store::open(&dir.path().join("p.db")).unwrap();
    store.import(&nfl(NFL_V1, "2026-10-28")).unwrap();
    let rows = store.players_by_year(2001, None).unwrap();

    let charlie = find(&rows, "Synthetic Charlie");
    assert_eq!(charlie.wav, None);
    assert_eq!(charlie.wav_evidence, WavEvidence::Missing);

    let bravo = find(&rows, "Synthetic Bravo Jr.");
    assert_eq!(bravo.wav, Some(0));
    assert_eq!(
        bravo.wav_evidence,
        WavEvidence::Observed {
            wav: 0,
            completed_seasons: 25,
            career: CareerState::Ended,
            pre_av_seasons: false
        }
    );

    let echo = &store.players_by_year(2026, None).unwrap()[0];
    assert_eq!(echo.wav, Some(0));
    assert_eq!(
        echo.wav_evidence,
        WavEvidence::NoCompletedSeason { reported: 0 }
    );

    let info = store.info().unwrap();
    assert_eq!(
        (info.formats[0].wav_missing, info.formats[0].wav_zero),
        (1, 2)
    );
}

#[test]
fn one_person_drafted_twice_is_one_player() {
    let dir = tempfile::tempdir().unwrap();
    let mut store = Store::open(&dir.path().join("p.db")).unwrap();
    let s = store.import(&nfl(NFL_V1, "2026-10-28")).unwrap();
    assert_eq!(s.linked_by_external_id, 1);
    let first = find(
        &store.players_by_year(2001, None).unwrap(),
        "Synthetic Alpha",
    )
    .clone();
    let second = find(
        &store.players_by_year(2002, None).unwrap(),
        "Synthetic Alpha",
    )
    .clone();
    assert_eq!(first.player_uid, second.player_uid);
    assert_eq!(
        (first.draft_team.as_deref(), second.draft_team.as_deref()),
        (Some("AAA"), Some("DDD"))
    );
    assert_eq!(store.info().unwrap().players, 5);
}

#[test]
fn reimport_keeps_identity_updates_values_and_keeps_unseen_rows() {
    let dir = tempfile::tempdir().unwrap();
    let mut store = Store::open(&dir.path().join("p.db")).unwrap();
    store.import(&nfl(NFL_V1, "2026-10-28")).unwrap();
    let before = store.players_by_year(2001, None).unwrap();

    let s = store.import(&nfl(NFL_V2, "2026-11-01")).unwrap();
    assert_eq!(
        (s.rows_read, s.inserted, s.updated, s.unchanged, s.rejected),
        (6, 1, 2, 3, 0)
    );
    let after = store.players_by_year(2001, None).unwrap();
    for b in &before {
        assert_eq!(
            find(&after, &b.name).player_uid,
            b.player_uid,
            "{} changed identity",
            b.name
        );
    }

    let alpha = find(&after, "Synthetic Alpha");
    assert_eq!((alpha.wav, alpha.as_of.as_str()), (Some(125), "2026-11-01"));
    let charlie = find(&after, "Synthetic Charlie");
    assert_eq!(charlie.wav, Some(7));
    assert!(matches!(
        charlie.wav_evidence,
        WavEvidence::Observed { wav: 7, .. }
    ));
    // Unchanged rows are re-confirmed by the newer import.
    assert_eq!(find(&after, "Synthetic Bravo Jr.").as_of, "2026-11-01");
    // A row absent from v2 is kept, still dated by the import that last had it.
    let echo = &store.players_by_year(2026, None).unwrap()[0];
    assert_eq!(
        (echo.name.as_str(), echo.as_of.as_str()),
        ("Synthetic Echo", "2026-10-28")
    );
}

#[test]
fn identical_reimport_changes_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let mut store = Store::open(&dir.path().join("p.db")).unwrap();
    let first = store.import(&nfl(NFL_V1, "2026-10-28")).unwrap();
    let before = store.players_by_year(2001, None).unwrap();
    let again = store.import(&nfl(NFL_V1, "2026-10-28")).unwrap();
    assert_eq!(
        (
            again.inserted,
            again.updated,
            again.unchanged,
            again.rejected
        ),
        (0, 0, 6, 3)
    );
    assert_eq!(first.file_sha256, again.file_sha256);
    assert_eq!(store.players_by_year(2001, None).unwrap(), before);
    assert_eq!(store.info().unwrap().players, 5);
}

#[test]
fn legacy_rows_link_to_nflverse_by_draft_slot_and_surname() {
    let dir = tempfile::tempdir().unwrap();
    let mut store = Store::open(&dir.path().join("p.db")).unwrap();
    store.import(&nfl(NFL_V1, "2026-10-28")).unwrap();
    let s = store.import(&legacy()).unwrap();
    assert_eq!(
        (s.rows_read, s.inserted, s.rejected, s.linked_by_draft_slot),
        (7, 6, 1, 1)
    );
    let kinds: Vec<&str> = s.issues.iter().map(|i| i.kind.as_str()).collect();
    assert_eq!(kinds, vec!["slot_name_mismatch", "invalid_row"]);

    let rows = store.players_by_year(2001, None).unwrap();
    let alpha = find(&rows, "Synthetic Alpha");
    assert_eq!(
        alpha.sources,
        vec!["legacy-player-lookup", "nflverse-draft-picks"]
    );
    // nflverse wins precedence, so its values are shown.
    assert_eq!(
        (alpha.source_format.as_str(), alpha.wav),
        ("nflverse-draft-picks", Some(120))
    );
    // Same slot, different surname: a separate person.
    let zulu = find(&rows, "Synthetic Zulu");
    assert_ne!(
        zulu.player_uid,
        find(&rows, "Synthetic Bravo Jr.").player_uid
    );
    // Undrafted players sort after drafted ones and carry no round or pick.
    let golf = rows.last().unwrap();
    assert_eq!(
        (
            golf.name.as_str(),
            golf.draft_status.as_str(),
            golf.draft_pick
        ),
        ("Synthetic Golf", "undrafted", None)
    );
    assert_eq!(golf.wav_evidence, WavEvidence::Missing);
}

#[test]
fn legacy_special_cases_are_mapped_explicitly() {
    let dir = tempfile::tempdir().unwrap();
    let mut store = Store::open(&dir.path().join("p.db")).unwrap();
    store.import(&legacy()).unwrap();

    let hotel = &store.players_by_year(1960, Some("fb")).unwrap()[0];
    assert_eq!(hotel.name, "Synthetic Hotel");
    assert_eq!(
        (
            hotel.draft_status.as_str(),
            hotel.draft_round,
            hotel.draft_pick
        ),
        ("unordered", Some(1), None)
    );
    assert_eq!(
        (hotel.league.as_deref(), hotel.hall_of_fame),
        (Some("AFL"), Some(true))
    );

    let india = &store.players_by_year(1955, None).unwrap()[0];
    assert!(matches!(
        india.wav_evidence,
        WavEvidence::Observed {
            wav: 3,
            pre_av_seasons: true,
            ..
        }
    ));

    // The common draft began in 1967, so a 1968 "AFL" label is the NFL draft.
    let juliet = &store.players_by_year(1968, None).unwrap()[0];
    assert_eq!(
        (juliet.league.as_deref(), juliet.wav),
        (Some("NFL"), Some(0))
    );
}

#[test]
fn position_filter_uses_the_source_vocabulary() {
    let dir = tempfile::tempdir().unwrap();
    let mut store = Store::open(&dir.path().join("p.db")).unwrap();
    store.import(&nfl(NFL_V1, "2026-10-28")).unwrap();
    assert_eq!(
        names(&store.players_by_year(2001, Some("qb")).unwrap()),
        vec!["Synthetic Alpha", "Synthétic Délta"]
    );
    assert_eq!(
        names(&store.players_by_year(2001, Some("DB")).unwrap()),
        vec!["Synthetic Bravo Jr."]
    );
    assert!(store.players_by_year(2001, Some("CB")).unwrap().is_empty());
}

#[test]
fn provenance_is_recorded_per_import() {
    let dir = tempfile::tempdir().unwrap();
    let mut store = Store::open(&dir.path().join("p.db")).unwrap();
    let mut opts = nfl(NFL_V1, "2026-10-28");
    opts.source_url = Some("https://example.invalid/synthetic.csv".into());
    let s = store.import(&opts).unwrap();
    let imports = store.imports().unwrap();
    let record = &imports[0];
    assert_eq!(record.format, "nflverse-draft-picks");
    assert_eq!(record.as_of, "2026-10-28");
    assert_eq!(
        record.file_name, NFL_V1,
        "only the file name is stored, never the full path"
    );
    assert_eq!(record.file_sha256, s.file_sha256);
    assert_eq!(record.file_sha256.len(), 64);
    assert!(record.synthetic);
    assert!(record.attribution.contains("Pro Football Reference"));
    assert_eq!(
        record.source_url.as_deref(),
        Some("https://example.invalid/synthetic.csv")
    );
    assert_eq!((record.inserted, record.rejected, record.issues), (6, 3, 3));
    assert!(
        store
            .players_by_year(2001, None)
            .unwrap()
            .iter()
            .all(|r| r.synthetic)
    );
}

#[test]
fn results_survive_reopen_and_rebuild_identically() {
    let dir = tempfile::tempdir().unwrap();
    let build = |name: &str| {
        let path = dir.path().join(name);
        let mut store = Store::open(&path).unwrap();
        store.import(&nfl(NFL_V1, "2026-10-28")).unwrap();
        store.import(&legacy()).unwrap();
        store.import(&nfl(NFL_V2, "2026-11-01")).unwrap();
        (path, store.players_by_year(2001, None).unwrap())
    };
    let (path, original) = build("a.db");
    let reopened = Store::open_existing(&path)
        .unwrap()
        .players_by_year(2001, None)
        .unwrap();
    assert_eq!(reopened, original);
    let (_, rebuilt) = build("b.db");
    assert_eq!(
        rebuilt, original,
        "same imports in the same order give the same uids and rows"
    );
}

#[test]
fn cli_imports_and_queries() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("cli.db");
    let bin = env!("CARGO_BIN_EXE_mt-data");
    let run = |args: &[&str]| std::process::Command::new(bin).args(args).output().unwrap();

    let out = run(&[
        "import",
        "--db",
        db.to_str().unwrap(),
        "--format",
        "nflverse-draft-picks",
        "--file",
        fixture(NFL_V1).to_str().unwrap(),
        "--as-of",
        "2026-10-28",
        "--synthetic",
    ]);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let text = String::from_utf8(out.stdout).unwrap();
    assert!(
        text.contains("SYNTHETIC") && text.contains("inserted 6"),
        "{text}"
    );

    let out = run(&[
        "query",
        "--db",
        db.to_str().unwrap(),
        "--year",
        "2001",
        "--position",
        "QB",
        "--json",
    ]);
    assert!(out.status.success());
    let rows: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(rows.as_array().unwrap().len(), 2);
    assert_eq!(rows[0]["wav_evidence"]["status"], "observed");

    let missing_as_of = run(&[
        "import",
        "--db",
        db.to_str().unwrap(),
        "--format",
        "nflverse-draft-picks",
        "--file",
        "x.csv",
    ]);
    assert_eq!(missing_as_of.status.code(), Some(2));
}
