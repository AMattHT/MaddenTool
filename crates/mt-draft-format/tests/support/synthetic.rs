//! SYNTHETIC draft classes for tests. Not game data, and no proof that the game accepts
//! anything: these files only exercise the reader's validation and preservation.
//!
//! The builder writes bytes at literal offsets taken from the layout evidence, not
//! through the crate's own field map, so a wrong map entry is caught by the field
//! assertions instead of agreeing with itself. Files are built full size in memory;
//! nothing large is committed.

pub const DATA_START: usize = 0x46;
pub const CAPACITY: usize = 455;
pub const RECORD: usize = 5_876;
pub const VISUAL: usize = 5_632;
pub const FILE_SIZE: usize = DATA_START + CAPACITY * RECORD;
pub const SYNTHETIC_BUILD: &str = "Madden-27-RL4-9244223";

/// Attribute offsets (relative to the attribute region) used by the builder.
pub mod at {
    pub const FIRST_NAME: usize = 0x00;
    pub const LAST_NAME: usize = 0x15;
    pub const HOME_TOWN: usize = 0x2b;
    pub const COLLEGE: usize = 0x46;
    pub const BIRTHDATE: usize = 0x48;
    pub const AGE: usize = 0x4a;
    pub const HEIGHT: usize = 0x4b;
    pub const WEIGHT: usize = 0x4c;
    pub const POSITION: usize = 0x4e;
    pub const DRAFT_PICK: usize = 0x52;
    pub const DRAFT_ROUND: usize = 0x54;
    pub const OVERALL: usize = 0x55;
    pub const SPEED: usize = 0x7f;
    pub const LONG_SNAP: usize = 0x8f;
    pub const DEV_TRAIT: usize = 0x90;
    pub const PID: usize = 0x94;
    pub const ASSET_NAME: usize = 0xa0;
    pub const DNA: usize = 0xca;
    pub const FOCUS: usize = 0xf2;
}

/// The synthetic visual JSON for a slot. Deliberately small, with no `bodyType` or `skinTone`.
pub fn visual_json(slot: usize) -> String {
    format!(r#"{{"genericHeadName":"gen_SYNTHETIC_{slot}","loadouts":[]}}"#)
}

/// A synthetic class file under construction.
pub struct Synthetic {
    pub bytes: Vec<u8>,
}

impl Synthetic {
    /// A valid file with `active` synthetic players in slots `0..active` and blank slots after.
    pub fn new(active: u32) -> Self {
        let mut s = Synthetic {
            bytes: vec![0; FILE_SIZE],
        };
        s.put(0x00, b"FBCHUNKS");
        s.put(0x08, &1u16.to_le_bytes());
        s.put(0x0a, &52u32.to_le_bytes());
        s.put(0x0e, &((CAPACITY * RECORD) as u32).to_le_bytes());
        s.put(0x12, &((CAPACITY * RECORD + 52) as u32).to_le_bytes());
        for (i, v) in [2026u16, 11, 2, 12, 34, 56].iter().enumerate() {
            s.put(0x16 + 2 * i, &v.to_le_bytes());
        }
        s.set_build(SYNTHETIC_BUILD);
        s.put(0x3e, &7u32.to_le_bytes());
        s.set_active(active);
        for slot in 0..(active as usize).min(CAPACITY) {
            s.fill_player(slot);
        }
        s
    }

    pub fn put(&mut self, offset: usize, data: &[u8]) {
        self.bytes[offset..offset + data.len()].copy_from_slice(data);
    }

    pub fn set_active(&mut self, active: u32) {
        self.put(0x42, &active.to_le_bytes());
    }

    /// Write `text` into the 28-byte build region, NUL-padded.
    pub fn set_build(&mut self, text: &str) {
        let mut region = [0u8; 28];
        region[..text.len()].copy_from_slice(text.as_bytes());
        self.put(0x22, &region);
    }

    pub fn record_start(slot: usize) -> usize {
        DATA_START + slot * RECORD
    }

    pub fn attr_offset(slot: usize, offset: usize) -> usize {
        Self::record_start(slot) + VISUAL + offset
    }

    pub fn put_attr(&mut self, slot: usize, offset: usize, data: &[u8]) {
        self.put(Self::attr_offset(slot, offset), data);
    }

    /// Replace a slot's visual region with `json` followed by zero padding.
    pub fn set_visual(&mut self, slot: usize, json: &[u8]) {
        let start = Self::record_start(slot);
        self.bytes[start..start + VISUAL].fill(0);
        self.put(start, json);
    }

    pub fn clear_record(&mut self, slot: usize) {
        let start = Self::record_start(slot);
        self.bytes[start..start + RECORD].fill(0);
    }

    fn fill_player(&mut self, slot: usize) {
        self.set_visual(slot, visual_json(slot).as_bytes());
        self.put_attr(slot, at::FIRST_NAME, b"Synthetic");
        self.put_attr(slot, at::LAST_NAME, format!("Player{slot}").as_bytes());
        self.put_attr(slot, at::HOME_TOWN, b"Testville");
        self.put_attr(slot, at::COLLEGE, &(300 + slot as u16).to_le_bytes());
        self.put_attr(
            slot,
            at::BIRTHDATE,
            &((14u16 << 11) | (5 << 7) | 64).to_le_bytes(),
        );
        self.put_attr(slot, at::AGE, &[22]);
        self.put_attr(slot, at::HEIGHT, &[74]);
        self.put_attr(slot, at::WEIGHT, &[45]); // 205 lb
        self.put_attr(slot, at::POSITION, &[(slot % 21) as u8]);
        self.put_attr(
            slot,
            at::DRAFT_PICK,
            &((slot % 32 + 1) as u16).to_le_bytes(),
        );
        self.put_attr(slot, at::DRAFT_ROUND, &[(slot / 32 + 1).min(63) as u8]);
        self.put_attr(slot, at::OVERALL, &[50 + (slot % 40) as u8]);
        self.put_attr(slot, at::SPEED, &[88]);
        self.put_attr(slot, at::LONG_SNAP, &[12]);
        self.put_attr(slot, at::DEV_TRAIT, &[(slot % 4) as u8]);
        self.put_attr(slot, at::PID, &(60_000 + slot as u16).to_le_bytes());
        for (i, v) in [11u16, 22, 33, 44, 55].iter().enumerate() {
            self.put_attr(slot, at::DNA + 2 * i, &v.to_le_bytes());
        }
        self.put_attr(slot, at::FOCUS, &[3]);
    }
}
