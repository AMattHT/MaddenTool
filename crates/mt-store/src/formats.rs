//! Supported dataset formats and how one source row maps onto a stored record.
//!
//! A row that cannot be mapped is rejected with a reason; it is never guessed at.
//! Blank cells (and nflverse's `NA`) become NULL, never zero.

use std::collections::HashMap;

use crate::error::{Result, StoreError};
use crate::names::{name_key, strip_markers};

/// Bump when the mapping below changes meaning, so re-imports refresh stored values.
pub const MAPPING_VERSION: &str = "1";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Format {
    /// `draft_picks.csv` from the nflverse-data `draft_picks` release (1980 onward).
    NflverseDraftPicks,
    /// `ALL_PLAYER_LOOKUP.csv` from the user's Madden-Draft-Generator (1936 onward).
    LegacyPlayerLookup,
}

impl Format {
    pub const ALL: [Format; 2] = [Format::NflverseDraftPicks, Format::LegacyPlayerLookup];

    pub fn id(self) -> &'static str {
        match self {
            Format::NflverseDraftPicks => "nflverse-draft-picks",
            Format::LegacyPlayerLookup => "legacy-player-lookup",
        }
    }

    pub fn parse(text: &str) -> Result<Format> {
        Format::ALL
            .into_iter()
            .find(|f| f.id() == text)
            .ok_or_else(|| {
                let known: Vec<&str> = Format::ALL.iter().map(|f| f.id()).collect();
                StoreError::InvalidInput(format!(
                    "unknown format {text:?}; supported: {}",
                    known.join(", ")
                ))
            })
    }

    /// Lower ranks win when two sources describe the same player and draft year.
    pub fn precedence(id: &str) -> u8 {
        match id {
            "nflverse-draft-picks" => 0,
            "legacy-player-lookup" => 1,
            _ => 2,
        }
    }

    pub fn required_columns(self) -> &'static [&'static str] {
        match self {
            Format::NflverseDraftPicks => &[
                "season",
                "round",
                "pick",
                "team",
                "pfr_player_name",
                "position",
                "w_av",
            ],
            Format::LegacyPlayerLookup => &[
                "Last Name",
                "First Name",
                "College/Univ",
                "Round",
                "Pick",
                "Draft Class",
                "Position",
                "From",
                "To",
                "AP1",
                "PB",
                "St",
                "wAV",
                "League",
                "isHOF",
            ],
        }
    }

    pub fn attribution(self) -> &'static str {
        match self {
            Format::NflverseDraftPicks => {
                "nflverse-data draft_picks release (repository licence CC-BY-4.0); \
                 draft and career values courtesy of Pro Football Reference (Sports Reference LLC). \
                 Approximate Value is a Pro Football Reference metric."
            }
            Format::LegacyPlayerLookup => {
                "Madden-Draft-Generator ALL_PLAYER_LOOKUP.csv (the user's own project dataset); \
                 draft, career and wAV columns follow Pro Football Reference draft tables. \
                 The file does not record when those values were retrieved."
            }
        }
    }

    pub fn map_row(self, row: &Row<'_>) -> std::result::Result<Mapped, String> {
        match self {
            Format::NflverseDraftPicks => map_nflverse(row),
            Format::LegacyPlayerLookup => map_legacy(row),
        }
    }
}

/// One CSV row with header lookup.
pub struct Row<'a> {
    pub columns: &'a HashMap<String, usize>,
    pub values: &'a csv::StringRecord,
}

impl Row<'_> {
    /// Trimmed cell, or None when absent, blank or `NA`.
    pub fn get(&self, column: &str) -> Option<&str> {
        let value = self.values.get(*self.columns.get(column)?)?.trim();
        (!value.is_empty() && value != "NA").then_some(value)
    }
}

/// What one source row says, in the stored vocabulary.
#[derive(Debug, Clone, PartialEq)]
pub struct Mapped {
    pub natural_key: String,
    pub full_name: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub position: Option<String>,
    pub college: Option<String>,
    pub draft_year: i64,
    pub draft_status: &'static str,
    pub draft_round: Option<i64>,
    pub draft_pick: Option<i64>,
    pub draft_team: Option<String>,
    pub league: Option<String>,
    pub career_from: Option<i64>,
    pub career_to: Option<i64>,
    pub games: Option<i64>,
    pub seasons_started: Option<i64>,
    pub pro_bowls: Option<i64>,
    pub all_pro_first: Option<i64>,
    pub hall_of_fame: Option<bool>,
    pub wav: Option<i64>,
    pub career_av: Option<i64>,
    pub draft_av: Option<i64>,
    pub external_ids: Vec<(&'static str, String)>,
}

/// Whole number, accepting a `.0` float spelling. Fractions are refused.
fn int(row: &Row<'_>, column: &str) -> std::result::Result<Option<i64>, String> {
    let Some(text) = row.get(column) else {
        return Ok(None);
    };
    if let Ok(v) = text.parse::<i64>() {
        return Ok(Some(v));
    }
    match text.parse::<f64>() {
        Ok(v) if v.fract() == 0.0 && v.abs() < 1e15 => Ok(Some(v as i64)),
        _ => Err(format!("{column} is not a whole number: {text:?}")),
    }
}

fn required_int(row: &Row<'_>, column: &str) -> std::result::Result<i64, String> {
    int(row, column)?.ok_or_else(|| format!("{column} is blank"))
}

fn year(row: &Row<'_>, column: &str) -> std::result::Result<i64, String> {
    let y = required_int(row, column)?;
    if (1900..=2200).contains(&y) {
        Ok(y)
    } else {
        Err(format!("{column} is not a plausible draft year: {y}"))
    }
}

fn boolean(row: &Row<'_>, column: &str) -> std::result::Result<Option<bool>, String> {
    match row.get(column) {
        None => Ok(None),
        Some(t) if t.eq_ignore_ascii_case("true") => Ok(Some(true)),
        Some(t) if t.eq_ignore_ascii_case("false") => Ok(Some(false)),
        Some(t) => Err(format!("{column} is not TRUE or FALSE: {t:?}")),
    }
}

fn text(row: &Row<'_>, column: &str) -> Option<String> {
    row.get(column).map(str::to_string)
}

fn map_nflverse(row: &Row<'_>) -> std::result::Result<Mapped, String> {
    let season = year(row, "season")?;
    let round = required_int(row, "round")?;
    let pick = required_int(row, "pick")?;
    if round < 1 || pick < 1 {
        return Err(format!(
            "round and pick must be positive, got {round} and {pick}"
        ));
    }
    let name = row
        .get("pfr_player_name")
        .ok_or("pfr_player_name is blank")?
        .to_string();
    let external_ids = [
        ("pfr", "pfr_player_id"),
        ("gsis", "gsis_id"),
        ("cfb", "cfb_player_id"),
    ]
    .into_iter()
    .filter_map(|(ns, col)| row.get(col).map(|v| (ns, v.to_string())))
    .collect();
    Ok(Mapped {
        // The table is one row per draft selection; season and overall pick identify it.
        natural_key: format!("{season}:{pick}"),
        full_name: name,
        first_name: None,
        last_name: None,
        position: text(row, "position"),
        college: text(row, "college"),
        draft_year: season,
        draft_status: "drafted",
        draft_round: Some(round),
        draft_pick: Some(pick),
        draft_team: text(row, "team"),
        // The release covers the NFL draft only.
        league: Some("NFL".to_string()),
        career_from: None,
        career_to: int(row, "to")?,
        games: int(row, "games")?,
        seasons_started: int(row, "seasons_started")?,
        pro_bowls: int(row, "probowls")?,
        all_pro_first: int(row, "allpro")?,
        hall_of_fame: boolean(row, "hof")?,
        wav: int(row, "w_av")?,
        career_av: int(row, "car_av")?,
        draft_av: int(row, "dr_av")?,
        external_ids,
    })
}

/// Legacy round/pick cells: a number, `UD` (undrafted), or `AFL n` / `ALF n` / `NFL n`
/// (the unordered 1960 AFL selections carry a round but no overall pick).
enum Slot {
    Drafted(i64),
    Undrafted,
}

fn legacy_slot(value: Option<&str>, column: &str) -> std::result::Result<Option<Slot>, String> {
    let Some(v) = value else { return Ok(None) };
    if v.eq_ignore_ascii_case("UD") {
        return Ok(Some(Slot::Undrafted));
    }
    if let Ok(n) = v.parse::<i64>() {
        return if n >= 1 {
            Ok(Some(Slot::Drafted(n)))
        } else {
            Err(format!("{column} must be positive: {n}"))
        };
    }
    let labelled = ["AFL", "ALF", "NFL"]
        .iter()
        .find_map(|p| v.strip_prefix(p))
        .and_then(|n| n.trim().parse::<i64>().ok())
        .filter(|n| *n >= 1);
    match (labelled, column) {
        (Some(n), "Round") => Ok(Some(Slot::Drafted(n))),
        // In the Pick column the label repeats the round; there is no overall pick.
        (Some(_), _) => Ok(None),
        (None, _) => Err(format!(
            "{column} is not a number, UD or a league round: {v:?}"
        )),
    }
}

fn map_legacy(row: &Row<'_>) -> std::result::Result<Mapped, String> {
    let draft_year = year(row, "Draft Class")?;
    let first = row
        .get("First Name")
        .map(strip_markers)
        .filter(|s| !s.is_empty());
    let last = row
        .get("Last Name")
        .map(strip_markers)
        .filter(|s| !s.is_empty())
        .ok_or("Last Name is blank")?;
    let full_name = match &first {
        Some(f) => format!("{f} {last}"),
        None => last.clone(),
    };
    let round = legacy_slot(row.get("Round"), "Round")?;
    let pick = legacy_slot(row.get("Pick"), "Pick")?;
    let (status, draft_round, draft_pick) = match (round, pick) {
        (Some(Slot::Undrafted), _) => ("undrafted", None, None),
        (Some(Slot::Drafted(r)), Some(Slot::Drafted(p))) => ("drafted", Some(r), Some(p)),
        (Some(Slot::Drafted(r)), None) => ("unordered", Some(r), None),
        (Some(Slot::Drafted(_)), Some(Slot::Undrafted)) => {
            return Err("Round is a number but Pick is UD".to_string());
        }
        (None, _) => return Err("Round is blank".to_string()),
    };
    // The first common AFL-NFL draft was 1967; later rows tagged AFL are the NFL draft.
    let league = row.get("League").map(|l| {
        let l = l.to_ascii_uppercase();
        if l == "AFL" && draft_year >= 1967 {
            "NFL".to_string()
        } else {
            l
        }
    });
    let college = text(row, "College/Univ");
    let natural_key = format!(
        "{draft_year}|{}|{}|{}|{}|{}",
        league.as_deref().unwrap_or("-"),
        draft_round.map_or("UD".to_string(), |r| r.to_string()),
        draft_pick.map_or("-".to_string(), |p| p.to_string()),
        name_key(&full_name),
        college.as_deref().map(name_key).unwrap_or_default(),
    );
    Ok(Mapped {
        natural_key,
        full_name,
        first_name: first,
        last_name: Some(last),
        position: text(row, "Position"),
        college,
        draft_year,
        draft_status: status,
        draft_round,
        draft_pick,
        draft_team: None,
        league,
        career_from: int(row, "From")?,
        career_to: int(row, "To")?,
        games: None,
        seasons_started: int(row, "St")?,
        pro_bowls: int(row, "PB")?,
        all_pro_first: int(row, "AP1")?,
        // Recorded as the file states it. The legacy tool found this flag is stamped by
        // name and can sit on a namesake; consumers must not treat it as verified.
        hall_of_fame: boolean(row, "isHOF")?,
        wav: int(row, "wAV")?,
        career_av: None,
        draft_av: None,
        external_ids: Vec::new(),
    })
}
