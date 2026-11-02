//! `mt-data`: create, import into and query the MaddenTool player database.
//! Run `mt-data help` for usage.

use std::collections::HashMap;
use std::path::PathBuf;
use std::process::ExitCode;

use mt_store::{AsOf, Format, ImportOptions, PlayerRow, Store, StoreError, WavEvidence};

const USAGE: &str = "\
mt-data: MaddenTool historical player database

USAGE:
  mt-data migrate --db <file>
  mt-data import  --db <file> --format <format> --file <csv> --as-of <YYYY-MM-DD>
                  [--source-url <url>] [--label <text>] [--synthetic]
  mt-data query   --db <file> --year <year> [--position <pos>] [--json]
  mt-data imports --db <file> [--json]
  mt-data info    --db <file> [--json]

FORMATS:
  nflverse-draft-picks   draft_picks.csv from the nflverse-data `draft_picks` release
  legacy-player-lookup   ALL_PLAYER_LOOKUP.csv from Madden-Draft-Generator

--as-of is the date the source data was current (for nflverse, the release's
timestamp.json). It is required: wAV means nothing without it.
--synthetic marks test fixtures so they can never pass for real data.
";

struct Args {
    command: String,
    values: HashMap<String, String>,
    flags: Vec<String>,
}

fn parse_args() -> Result<Args, String> {
    let mut it = std::env::args().skip(1);
    let command = it.next().unwrap_or_else(|| "help".to_string());
    let mut values = HashMap::new();
    let mut flags = Vec::new();
    while let Some(arg) = it.next() {
        let Some(name) = arg.strip_prefix("--") else {
            return Err(format!("unexpected argument {arg:?}"));
        };
        if matches!(name, "synthetic" | "json") {
            flags.push(name.to_string());
        } else {
            let value = it.next().ok_or(format!("--{name} needs a value"))?;
            if values.insert(name.to_string(), value).is_some() {
                return Err(format!("--{name} given twice"));
            }
        }
    }
    Ok(Args {
        command,
        values,
        flags,
    })
}

impl Args {
    fn require(&self, name: &str) -> Result<&str, String> {
        self.values
            .get(name)
            .map(String::as_str)
            .ok_or(format!("{} needs --{name}", self.command))
    }

    fn flag(&self, name: &str) -> bool {
        self.flags.iter().any(|f| f == name)
    }

    fn check_known(&self, allowed: &[&str]) -> Result<(), String> {
        for name in self.values.keys().chain(self.flags.iter()) {
            if !allowed.contains(&name.as_str()) {
                return Err(format!("{} does not take --{name}", self.command));
            }
        }
        Ok(())
    }
}

enum Failure {
    Usage(String),
    Store(StoreError),
}

impl From<StoreError> for Failure {
    fn from(e: StoreError) -> Self {
        Failure::Store(e)
    }
}

impl From<String> for Failure {
    fn from(e: String) -> Self {
        Failure::Usage(e)
    }
}

fn evidence_label(e: &WavEvidence) -> String {
    match e {
        WavEvidence::Missing => "missing".to_string(),
        WavEvidence::NoCompletedSeason { .. } => "no_completed_season".to_string(),
        WavEvidence::Observed {
            career,
            pre_av_seasons,
            ..
        } => {
            let career = serde_json::to_value(career)
                .ok()
                .and_then(|v| v.as_str().map(str::to_string))
                .unwrap_or_default();
            if *pre_av_seasons {
                format!("observed:{career}:pre_av_seasons")
            } else {
                format!("observed:{career}")
            }
        }
    }
}

fn opt<T: ToString>(v: &Option<T>) -> String {
    v.as_ref().map(ToString::to_string).unwrap_or_default()
}

fn print_players(rows: &[PlayerRow]) {
    println!(
        "player_uid\tyear\tstatus\tround\tpick\tteam\tleague\tposition\tname\tcollege\twav\twav_evidence\tas_of\tsource\tsources\tsynthetic"
    );
    for r in rows {
        println!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            r.player_uid,
            r.draft_year,
            r.draft_status,
            opt(&r.draft_round),
            opt(&r.draft_pick),
            opt(&r.draft_team),
            opt(&r.league),
            opt(&r.position),
            r.name,
            opt(&r.college),
            opt(&r.wav),
            evidence_label(&r.wav_evidence),
            r.as_of,
            r.source_format,
            r.sources.join(","),
            r.synthetic
        );
    }
}

fn json<T: serde::Serialize>(value: &T) -> Result<(), Failure> {
    println!(
        "{}",
        serde_json::to_string_pretty(value).map_err(StoreError::from)?
    );
    Ok(())
}

fn run(args: Args) -> Result<(), Failure> {
    match args.command.as_str() {
        "help" | "--help" | "-h" => {
            print!("{USAGE}");
            Ok(())
        }
        "migrate" => {
            args.check_known(&["db"])?;
            let store = Store::open(&PathBuf::from(args.require("db")?))?;
            println!("schema version {}", store.schema_version()?);
            Ok(())
        }
        "import" => {
            args.check_known(&[
                "db",
                "format",
                "file",
                "as-of",
                "source-url",
                "label",
                "synthetic",
            ])?;
            let options = ImportOptions {
                format: Format::parse(args.require("format")?)?,
                file: PathBuf::from(args.require("file")?),
                as_of: AsOf::parse(args.require("as-of")?)?,
                source_url: args.values.get("source-url").cloned(),
                label: args.values.get("label").cloned(),
                synthetic: args.flag("synthetic"),
            };
            let mut store = Store::open(&PathBuf::from(args.require("db")?))?;
            let s = store.import(&options)?;
            println!(
                "import {} ({}{}) sha256 {}",
                s.import_id,
                options.format.id(),
                if options.synthetic { ", SYNTHETIC" } else { "" },
                s.file_sha256
            );
            println!(
                "rows {} | inserted {} | updated {} | unchanged {} | rejected {}",
                s.rows_read, s.inserted, s.updated, s.unchanged, s.rejected
            );
            println!(
                "linked to an existing player: {} by external id, {} by draft slot and surname",
                s.linked_by_external_id, s.linked_by_draft_slot
            );
            let mut kinds: Vec<(&str, usize)> = Vec::new();
            for issue in &s.issues {
                match kinds.iter_mut().find(|(k, _)| *k == issue.kind) {
                    Some((_, n)) => *n += 1,
                    None => kinds.push((&issue.kind, 1)),
                }
            }
            for (kind, n) in &kinds {
                println!("issues {kind}: {n}");
            }
            for issue in s.issues.iter().take(10) {
                println!(
                    "  line {}: {} - {}",
                    opt(&issue.line),
                    issue.kind,
                    issue.detail
                );
            }
            if s.issues.len() > 10 {
                println!(
                    "  ... {} more recorded in import_issue",
                    s.issues.len() - 10
                );
            }
            Ok(())
        }
        "query" => {
            args.check_known(&["db", "year", "position", "json"])?;
            let year: i64 = args
                .require("year")?
                .parse()
                .map_err(|_| "--year must be a number".to_string())?;
            let store = Store::open_existing(&PathBuf::from(args.require("db")?))?;
            let rows =
                store.players_by_year(year, args.values.get("position").map(String::as_str))?;
            if args.flag("json") {
                json(&rows)
            } else {
                print_players(&rows);
                eprintln!("{} player(s)", rows.len());
                Ok(())
            }
        }
        "imports" => {
            args.check_known(&["db", "json"])?;
            let store = Store::open_existing(&PathBuf::from(args.require("db")?))?;
            let imports = store.imports()?;
            if args.flag("json") {
                return json(&imports);
            }
            for i in imports {
                println!(
                    "#{} {} as-of {} file {} sha256 {}{}\n    rows {} inserted {} updated {} unchanged {} rejected {} issues {}\n    label: {}\n    url: {}\n    attribution: {}",
                    i.id,
                    i.format,
                    i.as_of,
                    i.file_name,
                    i.file_sha256,
                    if i.synthetic { " SYNTHETIC" } else { "" },
                    i.rows_read,
                    i.inserted,
                    i.updated,
                    i.unchanged,
                    i.rejected,
                    i.issues,
                    opt(&i.label),
                    opt(&i.source_url),
                    i.attribution
                );
            }
            Ok(())
        }
        "info" => {
            args.check_known(&["db", "json"])?;
            let store = Store::open_existing(&PathBuf::from(args.require("db")?))?;
            let info = store.info()?;
            if args.flag("json") {
                return json(&info);
            }
            println!(
                "schema version {} | players {} | external ids {} | imports {}",
                info.schema_version, info.players, info.external_ids, info.imports
            );
            for f in info.formats {
                println!(
                    "{}: {} records, {}-{}, wAV missing {}, wAV zero {}",
                    f.format, f.records, f.first_year, f.last_year, f.wav_missing, f.wav_zero
                );
            }
            Ok(())
        }
        other => Err(Failure::Usage(format!("unknown command {other:?}"))),
    }
}

fn main() -> ExitCode {
    let result = parse_args().map_err(Failure::Usage).and_then(run);
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(Failure::Usage(msg)) => {
            eprintln!("error: {msg}\n\n{USAGE}");
            ExitCode::from(2)
        }
        Err(Failure::Store(e)) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}
