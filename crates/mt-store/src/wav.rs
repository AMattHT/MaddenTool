//! What a stored wAV value can and cannot say, relative to the data's as-of date.
//!
//! This is the input side of the wAV-to-OVR research contract
//! (`docs/research/wav-to-ovr-contract.md`). It does not produce a rating. It keeps
//! three cases apart that a rating model must not confuse: the source had no value,
//! the source reported a number before any season could be observed, and a real
//! observed value (including a real zero) with its career context.
//!
//! Everything is relative to the import's as-of date, never the wall clock, so the
//! same database gives the same answer next year.

use serde::Serialize;

use crate::error::{Result, StoreError};

/// PFR computes Approximate Value for seasons from 1960 on (PFR glossary). Careers
/// that began earlier have seasons with no AV, so their wAV undercounts.
pub const AV_FIRST_SEASON: i32 = 1960;

/// A calendar date in `YYYY-MM-DD` form.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AsOf {
    pub year: i32,
    pub month: u32,
    pub day: u32,
}

impl AsOf {
    pub fn parse(text: &str) -> Result<AsOf> {
        let bad =
            || StoreError::InvalidInput(format!("as-of date must be YYYY-MM-DD, got {text:?}"));
        let parts: Vec<&str> = text.split('-').collect();
        if parts.len() != 3 || parts[0].len() != 4 || parts[1].len() != 2 || parts[2].len() != 2 {
            return Err(bad());
        }
        let year: i32 = parts[0].parse().map_err(|_| bad())?;
        let month: u32 = parts[1].parse().map_err(|_| bad())?;
        let day: u32 = parts[2].parse().map_err(|_| bad())?;
        let leap = (year % 4 == 0 && year % 100 != 0) || year % 400 == 0;
        let days = match month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 if leap => 29,
            2 => 28,
            _ => return Err(bad()),
        };
        if day == 0 || day > days || !(1900..=2200).contains(&year) {
            return Err(bad());
        }
        Ok(AsOf { year, month, day })
    }

    /// The latest NFL season known to be finished on this date. Season Y runs from
    /// September Y to the Super Bowl in February Y+1, so it counts as complete from
    /// March Y+1. This is a calendar rule, not a schedule lookup.
    pub fn last_completed_season(&self) -> i32 {
        if self.month >= 3 {
            self.year - 1
        } else {
            self.year - 2
        }
    }
}

impl std::fmt::Display for AsOf {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CareerState {
    /// Last season is before the last completed season.
    Ended,
    /// Played in the last completed season or later; the total can still grow.
    PossiblyActive,
    /// The source has no final season.
    Unknown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum WavEvidence {
    /// The source left wAV blank. Not zero.
    Missing,
    /// The source reported a number, but no season since the draft had finished on
    /// the as-of date, so it carries no career evidence (nflverse lists 0 for the
    /// class being drafted).
    NoCompletedSeason { reported: i64 },
    /// An observed weighted career AV. Zero here is a real zero.
    Observed {
        wav: i64,
        /// Seasons from the draft year to the last completed season, inclusive.
        completed_seasons: i32,
        career: CareerState,
        /// Part of the career predates AV, so the value undercounts.
        pre_av_seasons: bool,
    },
}

pub struct WavInputs {
    pub wav: Option<i64>,
    pub draft_year: i32,
    pub career_from: Option<i32>,
    pub career_to: Option<i32>,
}

pub fn classify(inputs: &WavInputs, as_of: AsOf) -> WavEvidence {
    let Some(wav) = inputs.wav else {
        return WavEvidence::Missing;
    };
    let last = as_of.last_completed_season();
    if inputs.draft_year > last {
        return WavEvidence::NoCompletedSeason { reported: wav };
    }
    let career = match inputs.career_to {
        None => CareerState::Unknown,
        Some(to) if to >= last => CareerState::PossiblyActive,
        Some(_) => CareerState::Ended,
    };
    let first = inputs.career_from.unwrap_or(inputs.draft_year);
    WavEvidence::Observed {
        wav,
        completed_seasons: last - inputs.draft_year + 1,
        career,
        pre_av_seasons: first < AV_FIRST_SEASON,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(s: &str) -> AsOf {
        AsOf::parse(s).unwrap()
    }

    fn inputs(wav: Option<i64>, year: i32, to: Option<i32>) -> WavInputs {
        WavInputs {
            wav,
            draft_year: year,
            career_from: None,
            career_to: to,
        }
    }

    #[test]
    fn missing_and_zero_are_different() {
        let as_of = at("2026-10-28");
        assert_eq!(
            classify(&inputs(None, 2010, Some(2012)), as_of),
            WavEvidence::Missing
        );
        assert_eq!(
            classify(&inputs(Some(0), 2010, Some(2011)), as_of),
            WavEvidence::Observed {
                wav: 0,
                completed_seasons: 16,
                career: CareerState::Ended,
                pre_av_seasons: false
            }
        );
    }

    #[test]
    fn current_class_zero_is_not_evidence() {
        // nflverse on 2026-10-28 lists the 2026 class with w_av 0 while its first season is in progress.
        assert_eq!(
            classify(&inputs(Some(0), 2026, Some(2026)), at("2026-10-28")),
            WavEvidence::NoCompletedSeason { reported: 0 }
        );
        // In February the 2025 season is not finished yet either.
        assert_eq!(
            classify(&inputs(Some(3), 2025, Some(2025)), at("2026-02-01")),
            WavEvidence::NoCompletedSeason { reported: 3 }
        );
    }

    #[test]
    fn recent_careers_are_possibly_active_and_count_seasons() {
        let e = classify(&inputs(Some(9), 2024, Some(2026)), at("2026-10-28"));
        assert_eq!(
            e,
            WavEvidence::Observed {
                wav: 9,
                completed_seasons: 2,
                career: CareerState::PossiblyActive,
                pre_av_seasons: false
            }
        );
    }

    #[test]
    fn career_state_boundary_follows_the_as_of_season() {
        // On 2026-03-01 the last finished season is 2025: a 2025 final season may continue.
        let career = |to, as_of| match classify(&inputs(Some(40), 2020, Some(to)), at(as_of)) {
            WavEvidence::Observed { career, .. } => career,
            other => panic!("{other:?}"),
        };
        assert_eq!(career(2025, "2026-03-01"), CareerState::PossiblyActive);
        assert_eq!(career(2024, "2026-03-01"), CareerState::Ended);
        // The same row read against older data was still mid-career.
        assert_eq!(career(2024, "2025-06-01"), CareerState::PossiblyActive);
    }

    #[test]
    fn pre_1960_careers_are_flagged() {
        let e = classify(
            &WavInputs {
                wav: Some(3),
                draft_year: 1954,
                career_from: Some(1955),
                career_to: Some(1960),
            },
            at("2026-10-28"),
        );
        assert!(matches!(
            e,
            WavEvidence::Observed {
                pre_av_seasons: true,
                career: CareerState::Ended,
                ..
            }
        ));
    }

    #[test]
    fn as_of_rejects_bad_dates() {
        for bad in [
            "2026-02-30",
            "2026-13-01",
            "26-10-28",
            "2026-10-28T00:00",
            "",
            "2025-02-29",
        ] {
            assert!(AsOf::parse(bad).is_err(), "{bad}");
        }
        assert!(AsOf::parse("2024-02-29").is_ok());
    }
}
