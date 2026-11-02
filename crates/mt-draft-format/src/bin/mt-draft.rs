//! `mt-draft`: inspect Madden 27 draft-class files. Read-only: it never writes its input.
//! Run `mt-draft help` for usage.

use std::io::Write;
use std::path::Path;
use std::process::ExitCode;

use mt_draft_format::report::{field_json, hex, inspect_json, sha256_hex};
use mt_draft_format::{DraftClass, Prospect, read_file};

const USAGE: &str = "\
mt-draft: read-only Madden 27 draft-class inspector

USAGE:
  mt-draft inspect <file> [--json] [--slot <n>]

  --json      print the mt-draft-inspect/1 JSON document (every active player,
              every mapped field, unknown bytes, visuals, record hashes)
  --slot <n>  print every field of the active player in slot <n>

The file is opened for reading only. Exit status: 0 valid, 1 refused or
unreadable file, 2 usage error.
";

struct Options {
    path: String,
    json: bool,
    slot: Option<usize>,
}

fn parse_args(args: &[String]) -> Result<Options, String> {
    let mut it = args.iter();
    match it.next().map(String::as_str) {
        Some("inspect") => {}
        Some(other) => return Err(format!("unknown command {other:?}")),
        None => return Err("missing command".to_string()),
    }
    let mut path = None;
    let mut json = false;
    let mut slot = None;
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "--json" => json = true,
            "--slot" => {
                let v = it.next().ok_or("--slot needs a value")?;
                let n = v
                    .parse()
                    .map_err(|_| format!("--slot {v:?} is not a slot number"))?;
                if slot.replace(n).is_some() {
                    return Err("--slot given twice".to_string());
                }
            }
            a if a.starts_with("--") => return Err(format!("unknown option {a}")),
            a => {
                if path.replace(a.to_string()).is_some() {
                    return Err("inspect takes one file".to_string());
                }
            }
        }
    }
    let path = path.ok_or("inspect needs a file")?;
    Ok(Options { path, json, slot })
}

fn name_of(p: &Prospect) -> String {
    let first = p.text("firstName").unwrap_or("?");
    let last = p.text("lastName").unwrap_or("?");
    format!("{first} {last}")
}

fn summary(class: &DraftClass, path: &str) -> String {
    let h = class.header();
    let blank = class.blank_inactive_slots();
    let mut out = String::new();
    out += &format!("file      {path}\n");
    out += &format!("bytes     {}\n", class.source_bytes().len());
    out += &format!("sha256    {}\n", sha256_hex(class.source_bytes()));
    out += &format!(
        "game      Madden 27, build {} ({})\n",
        h.build.text,
        if h.build.is_observed() {
            "seen in game files"
        } else {
            "not seen before"
        }
    );
    out += &format!("saved     {} (file timestamp)\n", h.timestamp);
    out += &format!(
        "records   {} active of {} slots; {} inactive, {} of them blank\n",
        class.active_count(),
        class.capacity(),
        class.capacity() - class.active_count(),
        blank
    );
    if class.warnings().is_empty() {
        out += "warnings  none\n";
    } else {
        for w in class.warnings() {
            out += &format!("warning   {w}\n");
        }
    }
    out
}

fn table(class: &DraftClass) -> String {
    let mut out = String::from(
        "slot  name                                        pos arch ovr age  ht  wt  rd pick dev\n",
    );
    for p in class.players() {
        let n = |k| p.number(k).unwrap_or(0);
        out += &format!(
            "{:>4}  {:<42} {:>4} {:>4} {:>3} {:>3} {:>3} {:>3} {:>3} {:>4} {:>3}\n",
            p.slot(),
            name_of(p),
            n("position"),
            n("archetype"),
            n("overall"),
            n("age"),
            n("heightInches"),
            n("weight"),
            n("draftRound"),
            n("draftPick"),
            n("devTrait"),
        );
    }
    out
}

fn player_detail(class: &DraftClass, p: &Prospect) -> String {
    let mut out = format!("slot {}: {}\n", p.slot(), name_of(p));
    out += &format!(
        "record sha256  {}\n",
        sha256_hex(class.record_bytes(p.slot()))
    );
    for (spec, value) in p.fields() {
        out += &format!(
            "  0x{:02x} {:<20} {:<24} {}\n",
            spec.offset,
            spec.name,
            field_json(value).to_string(),
            spec.note
        );
    }
    out += &format!("unknown bytes  {}\n", hex(&p.unknown_bytes()));
    out += &format!(
        "visual JSON ({} bytes)\n{}\n",
        p.visuals().json().len(),
        p.visuals().json()
    );
    out
}

enum Failure {
    Read(mt_draft_format::ReadError),
    Usage(String),
}

fn run(opts: &Options) -> Result<String, Failure> {
    let class = read_file(Path::new(&opts.path)).map_err(Failure::Read)?;
    let player = match opts.slot {
        None => None,
        Some(slot) => Some(class.players().get(slot).ok_or_else(|| {
            Failure::Usage(match class.active_count() {
                0 => format!("slot {slot} is not an active player; this file has none"),
                n => format!(
                    "slot {slot} is not an active player; this file has {n} (slots 0 to {})",
                    n - 1
                ),
            })
        })?),
    };
    if opts.json {
        let mut doc = inspect_json(&class, &opts.path);
        if let Some(p) = player {
            doc["players"] = serde_json::Value::Array(vec![doc["players"][p.slot()].take()]);
        }
        return Ok(format!("{doc:#}\n"));
    }
    let body = match player {
        Some(p) => player_detail(&class, p),
        None => table(&class),
    };
    Ok(format!("{}\n{body}", summary(&class, &opts.path)))
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() || matches!(args[0].as_str(), "help" | "--help" | "-h") {
        print!("{USAGE}");
        return if args.is_empty() {
            ExitCode::from(2)
        } else {
            ExitCode::SUCCESS
        };
    }
    let opts = match parse_args(&args) {
        Ok(o) => o,
        Err(e) => {
            eprintln!("mt-draft: {e}\n\n{USAGE}");
            return ExitCode::from(2);
        }
    };
    match run(&opts) {
        Ok(text) => {
            // A reader that stops early (`| head`) is not an error.
            match std::io::stdout().lock().write_all(text.as_bytes()) {
                Ok(()) => ExitCode::SUCCESS,
                Err(e) if e.kind() == std::io::ErrorKind::BrokenPipe => ExitCode::SUCCESS,
                Err(e) => {
                    eprintln!("mt-draft: cannot write output: {e}");
                    ExitCode::from(1)
                }
            }
        }
        Err(Failure::Read(e)) => {
            eprintln!("mt-draft: {e}");
            ExitCode::from(1)
        }
        Err(Failure::Usage(e)) => {
            eprintln!("mt-draft: {e}");
            ExitCode::from(2)
        }
    }
}
