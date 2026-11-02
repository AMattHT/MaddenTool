//! Name cleanup and the surname comparison used to link draft slots across sources.

/// PFR marker glyphs (Hall of Fame and similar). The legacy lookup also carries
/// U+FFFD where an earlier export damaged the marker.
const MARKERS: &[char] = &['‡', '†', '*', '^', '¤', '+', '\u{FFFD}'];
const SUFFIXES: &[&str] = &["jr", "sr", "ii", "iii", "iv", "v"];

/// Remove marker glyphs and any digits that directly follow one ("Sprinkle‡1" -> "Sprinkle").
pub fn strip_markers(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    let mut after_marker = false;
    for c in name.chars() {
        if MARKERS.contains(&c) {
            after_marker = true;
        } else if after_marker && c.is_ascii_digit() {
            continue;
        } else {
            after_marker = false;
            out.push(c);
        }
    }
    out.trim().to_string()
}

/// Lower-case letters only, with generational suffixes dropped.
fn key_tokens(name: &str) -> Vec<String> {
    strip_markers(name)
        .split(|c: char| c.is_whitespace() || c == '-')
        .map(|t| {
            t.chars()
                .filter(|c| c.is_alphabetic())
                .flat_map(char::to_lowercase)
                .collect::<String>()
        })
        .filter(|t| !t.is_empty() && !SUFFIXES.contains(&t.as_str()))
        .collect()
}

/// A normalized key for matching, e.g. "Kyle Van Noy" -> "kylevannoy".
pub fn name_key(name: &str) -> String {
    key_tokens(name).concat()
}

/// The names one source record carries.
pub struct NameParts<'a> {
    pub full: &'a str,
    pub last: Option<&'a str>,
}

impl NameParts<'_> {
    fn full_key(&self) -> String {
        name_key(self.full)
    }

    fn surname_key(&self) -> String {
        match self.last {
            Some(last) if !name_key(last).is_empty() => name_key(last),
            _ => key_tokens(self.full).pop().unwrap_or_default(),
        }
    }
}

/// True when both records plausibly name the same person: each full name ends with
/// the other's surname. Handles multi-word surnames and formal versus casual given
/// names ("Matthew Bosher" and "Matt" + "Bosher"); it does not handle surname changes.
pub fn surnames_compatible(a: &NameParts<'_>, b: &NameParts<'_>) -> bool {
    let (a_full, b_full) = (a.full_key(), b.full_key());
    let (a_last, b_last) = (a.surname_key(), b.surname_key());
    !a_last.is_empty()
        && !b_last.is_empty()
        && a_full.ends_with(&b_last)
        && b_full.ends_with(&a_last)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn markers_and_trailing_digits_are_removed() {
        assert_eq!(strip_markers("Sprinkle‡1"), "Sprinkle");
        assert_eq!(strip_markers("Motley\u{FFFD}"), "Motley");
        assert_eq!(strip_markers("O'Neal"), "O'Neal");
    }

    #[test]
    fn multi_word_and_casual_names_match() {
        let full = NameParts {
            full: "Kyle Van Noy",
            last: None,
        };
        let split = NameParts {
            full: "Kyle Van Noy",
            last: Some("Van Noy"),
        };
        assert!(surnames_compatible(&full, &split));
        let formal = NameParts {
            full: "Matthew Bosher",
            last: None,
        };
        let casual = NameParts {
            full: "Matt Bosher",
            last: Some("Bosher"),
        };
        assert!(surnames_compatible(&formal, &casual));
        let suffixed = NameParts {
            full: "Odell Beckham Jr.",
            last: None,
        };
        let plain = NameParts {
            full: "Odell Beckham",
            last: Some("Beckham"),
        };
        assert!(surnames_compatible(&suffixed, &plain));
    }

    #[test]
    fn different_people_do_not_match() {
        let a = NameParts {
            full: "Irving Fryar",
            last: None,
        };
        let b = NameParts {
            full: "Steve Young",
            last: Some("Young"),
        };
        assert!(!surnames_compatible(&a, &b));
    }
}
