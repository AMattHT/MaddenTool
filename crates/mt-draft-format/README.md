# mt-draft-format: Madden 27 draft-class reader (DG02)

A read-only reader and validator for Madden 27 draft-class files (`CAREERDRAFT-*`), plus the `mt-draft inspect` CLI. It keeps every original byte, so a later patch-only writer can change exactly the bytes an edit touches.

**Not in this crate:** saving, patching, exporting, generating, or any UI. There is no public write API; DG03 owns the writer. The synthetic builder in `tests/support` is test code only.

## Run it

```sh
cargo build --release -p mt-draft-format       # builds target/release/mt-draft
cargo test -p mt-draft-format                  # synthetic tests; real-file tests skip

mt-draft inspect CAREERDRAFT-TEMP1             # metadata and a player table
mt-draft inspect CAREERDRAFT-TEMP1 --slot 0    # every field of one player, unknown bytes, visual JSON
mt-draft inspect CAREERDRAFT-TEMP1 --json      # mt-draft-inspect/1 document (all players)
```

Exit status: 0 for a valid file, 1 for a refused or unreadable file (the reason goes to stderr), and 2 for a usage error, such as `--slot` naming an inactive slot. The file is opened for reading only.

## Format

All integers are little-endian. Evidence comes from the game's own files: TEMP1 to TEMP3 (RL4), recorded in `docs/DISCOVERY-REPORT.md` 5.2, and the RL1 template, re-checked here.

| Offset | Field | Rule |
|---|---|---|
| `0x00` | magic | `FBCHUNKS`, else refused |
| `0x08` | U16 version | 1, else refused |
| `0x0a` | U32 header size | 52 (bytes from `0x12` to the data), else refused |
| `0x0e` | U32 payload size | 2,673,580 = 455 × 5,876, else refused |
| `0x12` | U32 size field | payload + 52, else refused |
| `0x16` | 6 × U16 timestamp | year, month, day, hour, minute, second; not range-checked |
| `0x22` | build string, 28 bytes | printable ASCII, NUL-terminated, `Madden-27-RL<n>[_<n>]-<digits>`. Another game is refused, and so is a malformed string. A well-formed M27 build not seen before opens with a warning. Seen so far: `Madden-27-RL1-9081279`, `Madden-27-RL4-9244223` |
| `0x3e` | U32 | 7 in every file, meaning unknown. Any other value is a warning |
| `0x42` | U32 active count | 0 to 455; more is refused |
| `0x46` | records | 455 slots of 5,876 bytes. The file is exactly 2,673,650 bytes: shorter is `truncated`, longer is `trailing bytes` |

Each record holds 5,632 bytes of visual JSON (UTF-8 object, then NUL padding) followed by 244 attribute bytes.

**Counts are kept apart.** Capacity is the 455 physical slots. The active count is the header value, and slots below it are players. The game writes 389; the reader accepts any count from 0 to 455. **The 402 export limit is a DG03 export policy, not a reader rule.**

Each record is checked:

- **Active slot:** an all-zero record is refused (`EmptyActiveRecord`). The visual region must be one JSON object: the bytes before the first NUL, or the whole region if there is no NUL. Empty, cut-short, non-object, two-value or non-UTF-8 JSON is refused, naming the slot. Nothing about the content is checked: keys, gear, DNA count and so on are not restricted.
- **Inactive slot:** padding, never a player. Game files have all-zero inactive slots. A non-zero one is kept byte for byte and reported as a warning.
- **Warnings, never refusals:** non-zero bytes after the visual JSON or after a text field's terminator. The bytes are kept.

### Field map

There are 82 fields covering 203 bytes, in `src/fields.rs`. Names follow the legacy keys. Offsets are relative to the attribute region:

- Text, ASCII and NUL-terminated: `firstName` `0x00` (21), `lastName` `0x15` (21), `homeTown` `0x2b` (27), `assetName` `0xa0` (42).
- U16: `college` `0x46`, `birthdate` `0x48`, `draftPick` `0x52`, `PID` `0x94`, `commentaryId` `0x9e`.
- `weight` `0x4c` is pounds minus 160. The value reported is pounds.
- `personaDNA` `0xca` is five U16 slots, kept positionally with zeros included.
- U8: `homeState` `0x2a`, `age` `0x4a`, `heightInches` `0x4b`, `position` `0x4e`, `archetype` `0x4f`, `jerseyNum` `0x50`, `draftRound` `0x54`, `overall` `0x55`, `devTrait` `0x90`, `bodyTypeId` `0x91`, `qbStyle` `0x96`, `hidden9c` `0x9c`, `focus` `0xf2`.
- The 54 ratings plus `const6b`, `personalityRating` `0x70`, `const7d` and `hidden87` fill `0x56` to `0x8f`.

**The 41 unknown bytes** are `0x4d`, `0x51`, `0x92`, `0x93`, `0x97`–`0x9b`, `0x9d`, `0xd4`–`0xf1` and `0xf3`. A compile-time check derives them as the complement of the map, and a unit test pins them. They are kept in each `Prospect`, so `attribute_bytes()` and `unknown_bytes()` hold the exact bytes, non-zero values included.

### Model

`DraftClass::parse(Vec<u8>)` (or `read_file`) keeps the whole file (`source_bytes()`, `record_bytes(slot)` for any slot).

Each active `Prospect` has:

- its decoded fields;
- a copy of its 244 attribute bytes;
- `Visuals`: the exact JSON text (`json()`) beside the parsed object.

Nothing is normalized or filled in. A key the file lacks, such as `bodyType` or `skinTone`, stays absent. Text that is not printable ASCII is kept as bytes and not decoded (`as_text()` returns `None`), because no game file has shown its encoding.

## Legacy parity

The oracle is `server/src/vendor/draft-class/M27Parser.js` from [Madden-Draft-Generator](https://github.com/AMattHT/Madden-Draft-Generator) `master` at `9a4ac69f46846d33f8d084948a85a47e4267f01b`, pinned by SHA-256 `ce24b0063c24e1ca7ce8be8a94717dc96716a49073fa4f0ed6f0e4430c1f067b`. It runs only in the opt-in differential test, through `tests/oracle/legacy-m27-oracle.cjs`. JavaScript is never a runtime dependency.

The differential test (`tests/differential.rs`) first requires the `inspect --json` report to list every active slot exactly once, in order: `0..active_count`, nothing missing, repeated, reordered or extra. A matching player count is not enough (issue [#8](https://github.com/AMattHT/MaddenTool/issues/8)). The inspect real-file test applies the same check and compares every reported record hash with the file. Then the differential compares, per fixture:

1. the legacy field map's declared offsets and widths with all 82 Rust fields;
2. the slots each side treats as players;
3. every mapped field of every active record, read from `mt-draft inspect --json`;
4. every visual object as parsed JSON, with numbers compared by value;
5. SHA-256 of all 455 records: the oracle hashes the file, Rust hashes the bytes its model kept.

Deliberate differences are normalized and counted, never hidden:

| Legacy behaviour | Rust | Why |
|---|---|---|
| Empty `assetName` becomes `null` | `""` | Missing is not a different value |
| `personaDNA` drops zero slots | All five slots, positionally | Zeros carry position. Compared as Rust's non-zero slots |
| `const6b` and `const7d` are declared with an expected value (127, 1) but never read | Read like any byte | Compared with the declared values |
| Detects M27 by file size only (`detectGame`) | Magic, version, sizes, build, count | `DISCOVERY-REPORT` 5.3 |
| Treats a slot as used when its attributes are non-zero, whatever the header says | The header count decides, and contradictions are refused or warned | — |
| Text: strips every NUL and trims whitespace, decodes as `ascii` | Bytes before the first NUL, exact | Keeps trailing data visible |

The legacy writer's defects (rewriting visuals, inventing `skinTone` and `bodyType`, clearing the attribute section) are out of scope here, and nothing in this crate writes.

## Tests and what they prove

| Suite | Runs | Proves |
|---|---|---|
| `src/fields.rs` unit tests | Always | 203 mapped and 41 unknown bytes, no overlaps, 54 ratings |
| `tests/read.rs` (26) | Always, offline, three OSes | Every refusal, count boundaries 0, 1, 389, 402, 454 and 455 (456 and above refused), inactive slots, non-zero unknown bytes surviving, every source byte kept, verbatim visual JSON, absent keys staying absent |
| `tests/cli.rs` (5) | Always | Human and JSON output, `--slot`, exit codes 0/1/2, input unchanged (SHA-256, length, mtime) |
| `tests/smoke.sh` | CI | The built binary on a shell-built synthetic class, plus a truncated copy |
| `tests/real_fixtures.rs` | 2 opt-in, 2 always | Opt-in: game files: hash, build, count, every player and field, observed invariants, inspect leaves the file unchanged and reports every slot once. Always: required and optional mode, run in child processes (#7) |
| `tests/differential.rs` | 1 opt-in, 4 always | Opt-in: legacy parity, as above. Always: required mode for both variables, the optional skip (#7), and the slot check rejecting omitted, repeated, reordered or extra players (#8) |

Synthetic files are labelled `SYNTHETIC` and built in memory at full size; no binary fixture is committed. **They test the reader, not the game: nothing here shows Madden accepts a file.**

### Real files (opt-in)

Copy the classes into a folder outside the repository; never point at live saves. Then:

```sh
MADDENTOOL_FIXTURES=/path/to/copies \
MADDENTOOL_LEGACY_MDG=/path/to/Madden-Draft-Generator \
MADDENTOOL_REQUIRE_FIXTURES=1 \
  cargo test -p mt-draft-format --test real_fixtures --test differential -- --nocapture
```

| Situation | Optional (default) | `MADDENTOOL_REQUIRE_FIXTURES=1` |
|---|---|---|
| `MADDENTOOL_FIXTURES` unset | Visible `SKIPPED` line; the test passes | Fails, naming the variable |
| Set, but not a directory, or holding none of the four fixtures | Fails | Fails |
| A fixture with a different SHA-256 | Fails | Fails |
| A fixture missing | `SKIPPED` line | Fails |
| `MADDENTOOL_LEGACY_MDG` unset | The differential test skips visibly. Fixtures are still checked | Fails, naming the variable. Checked before the fixture files |
| Set to a wrong path, a parser that is not the pinned one, or no `node` | Fails | Fails |

Use required mode for any run that claims real-file or legacy proof; a pass then means all four fixtures and the oracle ran. `MADDENTOOL_NODE` overrides `node`. `cargo test` hides `SKIPPED` lines unless you pass `--nocapture`.

| Fixture | SHA-256 | Build | Active | Source |
|---|---|---|---|---|
| `CAREERDRAFT-TEMP1` | `3e8b5a3f…2fde71` | RL4-9244223 | 389 | Game export supplied by the user (DISCOVERY 5.2) |
| `CAREERDRAFT-TEMP2` | `55c91c00…0fb0e1` | RL4-9244223 | 389 | Same |
| `CAREERDRAFT-TEMP3` | `2611dda2…31a457` | RL4-9244223 | 389 | Same |
| `CAREERDRAFT-2027Template` | `b62bbe814303b2329cde7b124c1e3da99a8ec9d4d796cfdd2204f8f702557ab3` | RL1-9081279, saved 2026-08-10 13:28:56 | 389 | Madden-Draft-Generator `server/data/Templates/CAREERDRAFT-2027Template` at `9a4ac69f` (git blob `93bbadf2`). Hash first recorded by DG02 |

The template carries no `skinTone` in any record and no `bodyType` in 174, the opposite of what the legacy writer adds. So it has not been through that writer. It is still a game file inside the legacy repository, not a fresh export.

### CI

`.github/workflows/draft-format.yml` runs on Windows, Linux and macOS: rustfmt, clippy `-D warnings`, the synthetic tests, a check that the opt-in tests printed `SKIPPED`, and the CLI smoke test.

**CI never sees a game file.** It cannot verify the real-file results, legacy parity, or anything about the game.

## Results (DG02, Windows 11, Rust 1.99.0, Node 24.17.0)

- **Template:** valid with no warnings.
  - 389 active of 455; the 66 inactive slots are blank.
  - 389 players × 82 fields = 31,898 values.
  - All 15,949 unknown bytes are zero.
  - `const6b` = 127 and `const7d` = 1 on every record.
  - Every player has five DNA traits.
  - Rounds 1 to 7 have 32 players each; 165 have round 63.
  - `inspect` left the SHA-256 unchanged.
- **Differential on the template:** 0 discrepancies.
  - 389 records; legacy treats the same 389 slots as used.
  - 31,120 field comparisons (389 × 80) and 778 declared-constant checks.
  - 389 visual objects and 455 record hashes.
  - Normalized: `assetName` null 389 times; no DNA zero slots.
- **Sensitivity:** swapping `speed` and `spinMove` offsets for one run produced 780 discrepancies (2 map, 778 values). Reverted.
- **TEMP1 to TEMP3 were not run.** DG02 was built on the user's laptop, which has no Madden 27 and no copies of them; the originals are on the user's desktop PC. Their rows above come from the discovery report. **The required-mode command above, with all four files, is a pending acceptance gate.** The RL1 template and synthetic tests do not stand in for RL4.

## Open evidence

- **TEMP1 to TEMP3 (RL4)** need the required-mode real-file and differential run, on the desktop PC or with copies from it.
- **`birthdate` encoding.** The legacy decode (`day<<11 | (month-1)<<7 | year-1940`, `M27Fields.ts`) does not fit the template: decoded years spread evenly over 1940 to 2067 for players aged 21 to 24. The reader reports the raw U16 only. The generator's encoding (DG04c) must be re-derived before it writes birthdates.
- Meaning of the `0x3e` constant, `const6b`, `const7d`, `hidden87`, `hidden9c` and the 41 unknown bytes.
- The encoding of non-ASCII names.
- Whether round 63 always means undrafted. It does on the template.
- Game acceptance of any file: DG03 and the in-game gate.
