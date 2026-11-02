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

/// Common English given-name forms. Two names are variants when they share a group.
/// Deliberately conservative: no group joins names that are routinely different people.
const NICKNAMES: &[&[&str]] = &[
    &["robert", "bob", "bobby", "rob", "robbie"],
    &["william", "bill", "billy", "will", "willie"],
    &["james", "jim", "jimmy", "jimmie"],
    &["john", "johnny", "jon"],
    &["richard", "rich", "rick", "ricky", "dick", "richie"],
    &["ronald", "ron", "ronnie"],
    &["thomas", "tom", "tommy"],
    &["charles", "charlie", "charley", "chuck"],
    &["peter", "pete"],
    &["walter", "walt"],
    &["louis", "lou"],
    &["arthur", "art"],
    &["gerald", "gerry", "jerry"],
    &["mack", "mac"],
    &["glenn", "glen"],
    &["allen", "allan", "alan"],
    &["edward", "ed", "eddie"],
    &["michael", "mike"],
    &["joseph", "joe", "joey"],
    &["daniel", "dan", "danny"],
    &["david", "dave"],
    &["donald", "don", "donnie"],
    &["kenneth", "ken", "kenny"],
    &["lawrence", "larry"],
    &["samuel", "sam", "sammy"],
    &["steven", "stephen", "steve"],
    &["timothy", "tim"],
    &["anthony", "tony"],
    &["gregory", "greg"],
    &["raymond", "ray"],
    &["frederick", "fred", "freddie"],
    &["benjamin", "ben"],
    &["douglas", "doug"],
    &["patrick", "pat"],
    &["philip", "phillip", "phil"],
    &["eugene", "gene"],
    &["nicholas", "nick"],
    &["andrew", "andy"],
    &["matthew", "matt"],
    &["jeffrey", "jeff"],
];

fn one_edit_apart(a: &str, b: &str) -> bool {
    let (a, b): (Vec<char>, Vec<char>) = (a.chars().collect(), b.chars().collect());
    let (short, long) = if a.len() <= b.len() {
        (&a, &b)
    } else {
        (&b, &a)
    };
    if long.len() - short.len() > 1 {
        return false;
    }
    let prefix = short
        .iter()
        .zip(long.iter())
        .take_while(|(x, y)| x == y)
        .count();
    if short.len() == long.len() {
        prefix < short.len() && short[prefix + 1..] == long[prefix + 1..]
    } else {
        short[prefix..] == long[prefix + 1..]
    }
}

/// True when two given names are plausibly the same person's: equal; one a leading
/// part of the other ("Billy Joe" and "Billy"); a known nickname pair ("Bob" and
/// "Robert"); or a one-letter spelling difference in names of five letters or more
/// ("Emery" and "Emory"). Never enough on its own: callers also require surname,
/// college and draft year to agree. Initials ("C.B.") match nothing.
pub fn given_names_compatible(a: &str, b: &str) -> bool {
    let (ta, tb) = (key_tokens(a), key_tokens(b));
    if ta.is_empty() || tb.is_empty() {
        return false;
    }
    // Shared leading names say nothing; compare from the first difference.
    let same = ta.iter().zip(&tb).take_while(|(x, y)| x == y).count();
    let (Some(fa), Some(fb)) = (ta.get(same), tb.get(same)) else {
        return true; // one is a leading part of the other
    };
    if NICKNAMES
        .iter()
        .any(|g| g.contains(&fa.as_str()) && g.contains(&fb.as_str()))
    {
        return true;
    }
    fa.chars().count() >= 5 && fb.chars().count() >= 5 && one_edit_apart(fa, fb)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn given_name_variants() {
        for (a, b) in [
            ("Bob", "Robert"),
            ("Bobby", "Bob"),
            ("Ronald", "Ron"),
            ("Emery", "Emory"),
            ("Emmet", "Emmett"),
            ("Billy Joe", "Billy"),
            ("Earl Ray", "Earl"),
            ("Glenn", "Glen"),
            ("Charley", "Charlie"),
        ] {
            assert!(given_names_compatible(a, b), "{a} / {b}");
        }
        for (a, b) in [
            ("Dave", "Dale"),
            ("Jim", "Jerry"),
            ("Kent", "George"),
            ("Billy", "Johnny"),
            ("C.B.", "Carlton"),
            ("Jack", "Jake"),
            ("Tom", "Terry"),
            ("", "Bob"),
        ] {
            assert!(!given_names_compatible(a, b), "{a} / {b}");
        }
    }

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
