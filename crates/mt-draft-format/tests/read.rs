//! Reader behaviour on SYNTHETIC files: validation, refusals, count boundaries and
//! byte preservation. None of this is evidence that the game accepts a file.

mod support;

use mt_draft_format::{DraftClass, FieldValue, ReadError, UNKNOWN_OFFSETS, Warning};
use support::synthetic::{
    CAPACITY, DATA_START, FILE_SIZE, RECORD, SYNTHETIC_BUILD, Synthetic, VISUAL, at, visual_json,
};

fn parse(s: Synthetic) -> Result<DraftClass, ReadError> {
    DraftClass::parse(s.bytes)
}

fn refused(s: Synthetic) -> ReadError {
    match parse(s) {
        Ok(_) => panic!("the reader accepted an invalid file"),
        Err(e) => e,
    }
}

// --- reading a valid file ---------------------------------------------------------

#[test]
fn reads_header_counts_and_mapped_fields() {
    let class = parse(Synthetic::new(3)).expect("valid synthetic class");
    let h = class.header();
    assert_eq!(h.version, 1);
    assert_eq!(h.header_size, 52);
    assert_eq!(h.payload_size as usize, CAPACITY * RECORD);
    assert_eq!(h.size_field as usize, CAPACITY * RECORD + 52);
    assert_eq!(h.timestamp.to_string(), "2026-11-02 12:34:56");
    assert_eq!(h.build.text, SYNTHETIC_BUILD);
    assert_eq!(h.build.release, "RL4");
    assert_eq!(h.build.changelist, 9_244_223);
    assert!(h.build.is_observed());
    assert_eq!(h.constant_3e, 7);
    assert_eq!(class.capacity(), 455);
    assert_eq!(class.active_count(), 3);
    assert_eq!(class.players().len(), 3);
    assert!(class.warnings().is_empty(), "{:?}", class.warnings());

    let p = &class.players()[2];
    assert_eq!(p.slot(), 2);
    assert_eq!(p.text("firstName"), Some("Synthetic"));
    assert_eq!(p.text("lastName"), Some("Player2"));
    assert_eq!(p.text("homeTown"), Some("Testville"));
    assert_eq!(p.text("assetName"), Some(""));
    assert_eq!(p.number("college"), Some(302));
    assert_eq!(p.number("birthdate"), Some((14 << 11) | (5 << 7) | 64));
    assert_eq!(p.number("age"), Some(22));
    assert_eq!(p.number("heightInches"), Some(74));
    assert_eq!(p.number("weight"), Some(205), "stored as pounds - 160");
    assert_eq!(p.number("position"), Some(2));
    assert_eq!(p.number("draftPick"), Some(3));
    assert_eq!(p.number("draftRound"), Some(1));
    assert_eq!(p.number("overall"), Some(52));
    assert_eq!(p.number("speed"), Some(88));
    assert_eq!(p.number("longSnap"), Some(12));
    assert_eq!(p.number("devTrait"), Some(2));
    assert_eq!(p.number("PID"), Some(60_002));
    assert_eq!(p.number("focus"), Some(3));
    assert_eq!(
        p.field("personaDNA").and_then(FieldValue::as_numbers),
        Some(&[11u16, 22, 33, 44, 55][..])
    );
    assert_eq!(p.fields().count(), 82);
}

#[test]
fn persona_dna_keeps_zero_slots_in_place() {
    // Four traits is legal (TEMP2 has one such prospect); a gap stays a gap.
    let mut s = Synthetic::new(1);
    for (i, v) in [7u16, 0, 9, 4, 0].iter().enumerate() {
        s.put_attr(0, at::DNA + 2 * i, &v.to_le_bytes());
    }
    let class = parse(s).expect("valid");
    assert_eq!(
        class.players()[0]
            .field("personaDNA")
            .and_then(FieldValue::as_numbers),
        Some(&[7u16, 0, 9, 4, 0][..])
    );
}

// --- refusals ---------------------------------------------------------------------

#[test]
fn refuses_a_file_shorter_than_the_header() {
    for len in [0, 8, DATA_START - 1] {
        let mut s = Synthetic::new(1);
        s.bytes.truncate(len);
        let e = refused(s);
        assert!(
            matches!(e, ReadError::TruncatedHeader { actual } if actual == len),
            "{e:?}"
        );
    }
}

#[test]
fn refuses_a_truncated_payload() {
    for len in [DATA_START, DATA_START + RECORD * 100 + 17, FILE_SIZE - 1] {
        let mut s = Synthetic::new(389);
        s.bytes.truncate(len);
        let e = refused(s);
        assert!(
            matches!(e, ReadError::Truncated { expected, actual } if expected == FILE_SIZE && actual == len),
            "{e:?}"
        );
        assert!(e.to_string().contains("truncated"), "{e}");
    }
}

#[test]
fn refuses_bytes_after_the_payload() {
    let mut s = Synthetic::new(3);
    s.bytes.push(0);
    let e = refused(s);
    assert!(
        matches!(e, ReadError::TrailingBytes { expected, actual } if expected == FILE_SIZE && actual == FILE_SIZE + 1),
        "{e:?}"
    );
}

#[test]
fn refuses_bad_magic() {
    let mut s = Synthetic::new(3);
    s.put(0, b"FBCHUNKZ");
    let e = refused(s);
    assert!(
        matches!(e, ReadError::BadMagic { found } if &found == b"FBCHUNKZ"),
        "{e:?}"
    );
    assert!(e.to_string().contains("FBCHUNKZ"), "{e}");
}

#[test]
fn refuses_other_container_versions() {
    let mut s = Synthetic::new(3);
    s.put(0x08, &2u16.to_le_bytes());
    assert!(matches!(
        refused(s),
        ReadError::UnsupportedVersion { found: 2 }
    ));
}

#[test]
fn refuses_a_wrong_header_size() {
    let mut s = Synthetic::new(3);
    s.put(0x0a, &56u32.to_le_bytes());
    assert!(matches!(
        refused(s),
        ReadError::UnexpectedHeaderSize { found: 56 }
    ));
}

#[test]
fn refuses_a_payload_size_other_than_455_records() {
    // An M26 class declares 455 records of 4,296 bytes.
    let mut s = Synthetic::new(3);
    let m26 = 455u32 * 4296;
    s.put(0x0e, &m26.to_le_bytes());
    s.put(0x12, &(m26 + 52).to_le_bytes());
    assert!(matches!(refused(s), ReadError::UnexpectedPayloadSize { found } if found == m26));
}

#[test]
fn refuses_an_inconsistent_size_field() {
    let mut s = Synthetic::new(3);
    s.put(0x12, &((CAPACITY * RECORD) as u32 + 51).to_le_bytes());
    let e = refused(s);
    assert!(
        matches!(e, ReadError::SizeFieldMismatch { found, expected } if found as usize == CAPACITY * RECORD + 51 && expected as usize == CAPACITY * RECORD + 52),
        "{e:?}"
    );
}

#[test]
fn refuses_other_games_by_build_string() {
    for build in ["Madden-26-RL9-1234567", "CFB-26-RL1-1234567", "Madden-27"] {
        let mut s = Synthetic::new(3);
        s.set_build(build);
        let e = refused(s);
        match build {
            "Madden-27" => assert!(matches!(&e, ReadError::UnsupportedBuild { .. }), "{e:?}"),
            _ => assert!(
                matches!(&e, ReadError::UnsupportedGame { build: b } if b == build),
                "{e:?}"
            ),
        }
    }
}

#[test]
fn refuses_madden_27_builds_with_an_unknown_shape() {
    for build in [
        "Madden-27-RLx-9244223",
        "Madden-27-RL4-",
        "Madden-27-RL4-92a",
        "Madden-27-XX4-1",
    ] {
        let mut s = Synthetic::new(3);
        s.set_build(build);
        let e = refused(s);
        assert!(
            matches!(&e, ReadError::UnsupportedBuild { build: b } if b == build),
            "{build}: {e:?}"
        );
    }
}

#[test]
fn refuses_a_malformed_build_string() {
    let mut s = Synthetic::new(3);
    s.put(0x22, &[b'M'; 28]); // no terminator
    assert!(matches!(refused(s), ReadError::MalformedBuild { .. }));

    let mut s = Synthetic::new(3);
    s.set_build("Madden-27-RL4-9244223");
    s.put(0x22 + 3, &[0xe9]); // not ASCII
    assert!(matches!(refused(s), ReadError::MalformedBuild { .. }));

    let mut s = Synthetic::new(3);
    s.set_build("");
    assert!(matches!(refused(s), ReadError::MalformedBuild { .. }));
}

#[test]
fn opens_an_unseen_madden_27_build_with_a_warning() {
    let mut s = Synthetic::new(3);
    s.set_build("Madden-27-RL3_5-9100000");
    let class = parse(s).expect("a well-formed Madden 27 build opens");
    assert_eq!(class.header().build.release, "RL3_5");
    assert_eq!(class.header().build.changelist, 9_100_000);
    assert!(!class.header().build.is_observed());
    assert_eq!(
        class.warnings(),
        &[Warning::UnobservedBuild {
            build: "Madden-27-RL3_5-9100000".to_string()
        }]
    );
}

#[test]
fn an_unexpected_header_constant_is_a_warning_not_a_refusal() {
    let mut s = Synthetic::new(3);
    s.put(0x3e, &8u32.to_le_bytes());
    let class = parse(s).expect("opens");
    assert_eq!(
        class.warnings(),
        &[Warning::UnexpectedConstant { found: 8 }]
    );
}

// --- active count and inactive slots -------------------------------------------------

#[test]
fn active_count_boundaries() {
    for active in [0u32, 1, 389, 402, 454, 455] {
        let class = parse(Synthetic::new(active)).unwrap_or_else(|e| panic!("{active}: {e}"));
        assert_eq!(class.active_count(), active as usize);
        assert_eq!(class.players().len(), active as usize);
        assert_eq!(class.capacity(), 455);
        let last = class.players().last().map(|p| p.slot());
        assert_eq!(last, (active as usize).checked_sub(1));
    }
    for active in [456u32, 1000, u32::MAX] {
        let mut s = Synthetic::new(455);
        s.set_active(active);
        let e = refused(s);
        assert!(
            matches!(e, ReadError::ActiveCountOutOfRange { found } if found == active),
            "{e:?}"
        );
    }
}

#[test]
fn blank_inactive_slots_are_padding_not_players() {
    let class = parse(Synthetic::new(389)).expect("valid");
    assert_eq!(class.players().len(), 389);
    assert!(class.warnings().is_empty());
    for slot in 389..CAPACITY {
        assert!(
            class.record_bytes(slot).iter().all(|&b| b == 0),
            "slot {slot}"
        );
    }
}

#[test]
fn a_blank_slot_below_the_active_count_is_refused() {
    let mut s = Synthetic::new(5);
    s.clear_record(3);
    assert!(matches!(
        refused(s),
        ReadError::EmptyActiveRecord { slot: 3 }
    ));
}

#[test]
fn a_non_blank_inactive_slot_is_kept_and_reported() {
    let mut s = Synthetic::new(5);
    s.put_attr(7, 0x30, &[0x5a]);
    let original = s.bytes.clone();
    let class = parse(s).expect("opens");
    assert_eq!(class.players().len(), 5, "an inactive slot is not a player");
    assert_eq!(
        class.warnings(),
        &[Warning::InactiveSlotNotBlank { slot: 7 }]
    );
    assert_eq!(
        class.record_bytes(7),
        &original[DATA_START + 7 * RECORD..DATA_START + 8 * RECORD]
    );
}

// --- visual JSON --------------------------------------------------------------------

#[test]
fn refuses_malformed_visual_json_in_an_active_record() {
    let cases: [&[u8]; 6] = [
        br#"{"genericHeadName":"gen_1""#, // cut short
        br#"{"a":1}{"b":2}"#,             // two values
        b"[1,2,3]",                       // not an object
        b"\"text\"",                      // not an object
        b"{\"a\":\"\xff\"}",              // not UTF-8
        b"",                              // empty region, attributes present
    ];
    for json in cases {
        let mut s = Synthetic::new(4);
        s.set_visual(2, json);
        let e = refused(s);
        assert!(
            matches!(e, ReadError::MalformedVisualJson { slot: 2, .. }),
            "{:?}: {e:?}",
            String::from_utf8_lossy(json)
        );
        assert!(e.to_string().contains("slot 2"), "{e}");
    }
}

#[test]
fn visual_json_is_kept_exactly_and_absent_keys_stay_absent() {
    let odd = "{ \"loadouts\" : [ ], \"genericHeadName\":\"gen_\\u00e9\",\"zeta\":1.50 }";
    let mut s = Synthetic::new(3);
    s.set_visual(1, odd.as_bytes());
    let class = parse(s).expect("valid");
    let v = class.players()[1].visuals();
    assert_eq!(v.json(), odd, "the text is not re-serialized");
    assert_eq!(v.generic_head_name(), Some("gen_\u{e9}"));
    assert_eq!(v.body_type(), None, "bodyType is not invented");
    assert_eq!(v.get("skinTone"), None, "skinTone is not invented");
    let mut keys: Vec<_> = v.object().keys().cloned().collect();
    keys.sort();
    assert_eq!(keys, ["genericHeadName", "loadouts", "zeta"]);

    let plain = class.players()[0].visuals();
    assert_eq!(plain.json(), visual_json(0));
    assert_eq!(plain.object().len(), 2);
}

#[test]
fn visual_json_may_fill_the_whole_region() {
    let mut json = String::from("{\"pad\":\"");
    json.push_str(&"x".repeat(VISUAL - json.len() - 2));
    json.push_str("\"}");
    assert_eq!(json.len(), VISUAL);
    let mut s = Synthetic::new(2);
    s.set_visual(1, json.as_bytes());
    let class = parse(s).expect("a full region with no terminator is still one object");
    assert_eq!(class.players()[1].visuals().json().len(), VISUAL);
}

#[test]
fn non_zero_bytes_after_the_visual_json_are_kept_and_reported() {
    let mut s = Synthetic::new(3);
    let tail = Synthetic::record_start(1) + visual_json(1).len() + 10;
    s.put(tail, &[0x41]);
    let original = s.bytes.clone();
    let class = parse(s).expect("opens");
    assert_eq!(
        class.warnings(),
        &[Warning::VisualPaddingNotZero { slot: 1 }]
    );
    assert_eq!(class.source_bytes(), &original[..]);
}

// --- attribute bytes ----------------------------------------------------------------

#[test]
fn non_zero_unknown_bytes_survive_unchanged() {
    let mut s = Synthetic::new(4);
    let planted: Vec<u8> = (0..UNKNOWN_OFFSETS.len()).map(|i| 0xa0 + i as u8).collect();
    for (&off, &b) in UNKNOWN_OFFSETS.iter().zip(&planted) {
        s.put_attr(2, off, &[b]);
    }
    let original = s.bytes.clone();
    let class = parse(s).expect("non-zero unknown bytes are legal");
    let p = &class.players()[2];
    assert_eq!(p.unknown_bytes().to_vec(), planted);
    let attr_start = DATA_START + 2 * RECORD + VISUAL;
    assert_eq!(
        &p.attribute_bytes()[..],
        &original[attr_start..attr_start + 244]
    );
    // Mapped values are unaffected by their unknown neighbours.
    assert_eq!(p.number("position"), Some(2));
    assert_eq!(p.number("overall"), Some(52));
    assert_eq!(p.number("draftPick"), Some(3));
    assert_eq!(p.number("focus"), Some(3));
    // The other players' unknown bytes stay zero.
    assert_eq!(class.players()[1].unknown_bytes(), [0u8; 41]);
}

#[test]
fn the_model_keeps_every_original_byte() {
    let mut s = Synthetic::new(402);
    for (&off, i) in UNKNOWN_OFFSETS.iter().zip(0u8..) {
        s.put_attr(401, off, &[i.wrapping_mul(37) | 1]);
    }
    let original = s.bytes.clone();
    let class = parse(s).expect("valid");
    assert_eq!(class.source_bytes(), &original[..]);
    for slot in 0..CAPACITY {
        let start = DATA_START + slot * RECORD;
        assert_eq!(
            class.record_bytes(slot),
            &original[start..start + RECORD],
            "slot {slot}"
        );
    }
}

#[test]
fn text_fields_keep_exact_bytes() {
    let mut s = Synthetic::new(3);
    // Non-ASCII name byte: opened, not decoded, bytes kept.
    let mut last = [0u8; 21];
    last[..4].copy_from_slice(b"Ren\xe9");
    s.put_attr(0, at::LAST_NAME, &last);
    // Bytes after a terminator: kept and reported.
    s.put_attr(1, at::FIRST_NAME, b"Al\0x");
    // A field filled to its width with no terminator.
    s.put_attr(2, at::ASSET_NAME, &[b'A'; 42]);
    let class = parse(s).expect("opens");
    let p0 = &class.players()[0];
    assert_eq!(p0.text("lastName"), None);
    assert_eq!(
        p0.field("lastName"),
        Some(&FieldValue::Text(b"Ren\xe9".to_vec()))
    );
    assert_eq!(class.players()[1].text("firstName"), Some("Al"));
    assert_eq!(
        class.players()[2].text("assetName"),
        Some(&"A".repeat(42)[..])
    );
    assert_eq!(
        class.warnings(),
        &[Warning::TextPaddingNotZero {
            slot: 1,
            field: "firstName"
        }]
    );
}
