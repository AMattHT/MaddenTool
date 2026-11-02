# MaddenTool discovery report

Work package 00, completed 2026-11-01 by Claude Opus 5.5.

## 1. Verdict

**The design is ready for user review.** The coordinator resolved the operational questions in section 11. The concrete design awaits user review; the chosen stack and full product scope are unchanged.

The full product is feasible, but three findings change how it should be built:

1. **Original unminified source was not found for most MyFranchise logic.** Trade value, progression, previews, tools and grades live in a minified 30.8 MB renderer bundle. No source repository was found. Proving parity needs golden output captured from a running 2.0.1, so a new package, WP00R, is added.
2. **The legacy M27 writer is not safe to port as-is.** On all four game-generated classes it changes every active visual block (389 of 389) during a no-op. It must be replaced with a patch-only writer, using the legacy code only as a reference.
3. **The franchise fixture set is too thin.** There is one M27 franchise file, from an older game build. A current RL4 save at several season stages is needed before WP05 can finish.

Nothing here has proven runtime parity. Every MyFranchise row remains static-only, and 283 of 285 feature rows stay at `discovered`.

## 2. Method and evidence limits

| Input | What was done | Limit |
|---|---|---|
| MyFranchise 2.0.1 `app.asar` | Hashes matched the inventory. Text files extracted to scratch. Route table, nav table, tool registry, FAQ text, preload and main handlers read. A component reference graph was built. | **Not run.** Minified code, so stated behaviour comes from strings and read code. |
| Installed 2.0.2 | Same extraction, then diffed. Template database read in immutable mode. | Not run. |
| Draft generator (read-only) | Routes, services, components, tests and notes read. M27 parser and writer called directly on scratch copies. | Legacy test suite and app not run. The 366 passing tests are a claim from its 1.5.0 report. |
| Game files | Probed scratch copies only: three draft classes the user exported today, one template, the saved classes, one franchise backup. | No in-game import or load was performed. |
| MMC 1.1.0.7 | ReadMe read. Strings extracted from binaries. | **Not launched.** No import was performed. No `.fbmod` file exists in the searched folders. |

Originals were not changed. All probe output went to `%TEMP%\maddentool-audit`, which also holds a copy of the real save, so delete that folder when finished. The installer and apps were not executed. The only writes were to `docs/` and that scratch folder.

Reproduction scripts are in `docs/reference/audit/`. `verify_parity.py` passes.

## 3. Corrections to the evidence ledger

| Earlier statement | Finding |
|---|---|
| 365 component symbols | 362 definitions in the main bundle plus 5 in the lazy chunk, with 2 names collapsed. `TradeBlock` is both a page and the Update Trade Block tool. `LeagueStandings` also appears twice. Both are noted in the ledger. |
| 104 IPC channels | The list is accurate but incomplete. It omits `app:get-version`, `app-close`, `app-minimize`, `app-resize`, `save-settings`, `franchise-file-changed` and `franchise:getTableRecords`. All 7 are now mapped. |
| 2.0.1 and 2.0.2 preload match, with UI and algorithms unknown | Now known. `main.js`, `preload.js` and the metadata worker are identical. The renderer differs in **one expression**: the Injured Reserve Fix adds `OverallRating>=70`. The progression chunk matches once its import name is normalized. |
| 2.0.1 is the latest context | Upstream has reached 2.0.4. The baseline stays 2.0.1 as required. `MyFranchiseRelease-2.0.4.zip` in Downloads is a 408-byte stub. |
| "Package metadata reports MIT" | True, but no source is published. The release repository has no detected license. `madden-franchise` declares MIT in `package.json`, yet GitHub detects no license and the package ships no license file. |
| `madden-franchise ^4.3.6` | The same version number differs by copy. MyFranchise bundles `M27_640_0.gz`. The legacy copy has only `M27_620_0.gz`, so it may not open newer saves. |
| Draft tool includes franchise utilities | They are **off by default**. Routes mount only with `DRAFT_TOOL_FRANCHISE=1`. M27 behaviour is unproven. |
| Saves folder holds a CAREER file (M27-PORT) | No longer. The only M27 franchise file found is a MyFranchise backup. |
| Default Madden path | `C:\Program Files\EA Games\Madden NFL 27` is absent here, though saves were written today. The game is installed elsewhere. |

Static reachability: every component is reachable from a route, the app root or the tool registry. Three registry entries are empty and never shown: `assignAbilities`, `exportSeasonData`, `motivationsGenerator`.

## 4. Parity ledger

`docs/FEATURE-PARITY.csv` has 1,008 rows: 285 features and 723 symbol mappings.

| Input | Inventory | Mapped |
|---|---:|---:|
| MyFranchise components | 365 | 365 |
| MyFranchise routes | 51 | 51 |
| MyFranchise IPC channels | 104 | 104, plus 7 found by this audit |
| Draft routes | 65 | 65 |
| Draft components | 51 | 51 |
| Draft services | 77 | 77 |

Features: 195 MyFranchise, 74 draft generator, 16 binding requirements.

- **Dead (13):** 10 unused IPC channels and the 3 empty tools. `fs:readFile` and `fs:writeFile` must not be carried over in any form, because they allow arbitrary file access.
- **Unresolved (1):** `franchise:cutPracticeSquadPlayer` is implemented and exposed, but no renderer call was found. WP00R must show how a practice-squad cut actually happens.
- **Out of scope (2):** the M26 codec and the game picker.
- **Not in the design's list, now tracked:**
  - Custom logo folders
  - User-control presets and 16 per-team toggles
  - 18 stage-based page rules
  - June 1 cuts and trade override
  - Edit import and export
  - Gear and skin tone from a photo
  - The M27 roster file format

Runtime status per row is in the `runtime_evidence` column. Section 12 summarizes it.

## 5. Format audit

### 5.1 Draft class: the 402 and 455 question

These are three different numbers:

| Number | Meaning | Evidence |
|---|---|---|
| **455** | Physical record capacity. Payload is exactly 455 × 5,876 bytes. | Header `0x0e` = 2,673,580 in every file |
| **389** | What the game itself writes: 224 drafted (32 per round for 7 rounds) and 165 undrafted. | TEMP1, TEMP2, TEMP3 (RL4) and the template (RL1) all agree |
| **402** | What this tool writes, inherited from the M26 limit. | All 16 tool-written classes probed in Saves have header 402 and 402 non-zero blocks |

The active count is the 32-bit value at `0x42`. It always matched the non-zero block count.

**Is 402 accepted by the game?** Likely, but not settled. `M27-PORT.md` records an in-game import of a class written by the tool on 2026-08-10, without stating the count, and left this exact test open on 08-22. `CAREERDRAFT-E` and `CAREERDRAFT-1977` carry a build stamp of `RL3_5` with count 402. The tool's only template is RL1, which suggests the game re-saved a 402 class, but that is an inference.

**Decision:** export defaults to a 402 limit, needs one RL4 in-game import to confirm, and blocks 403 to 455 until a test proves them. The UI shows capacity, active and drafted counts separately. See `DG-FILE-07`.

### 5.2 Draft class: structure

- `0x00` magic `FBCHUNKS`, `0x08` version 1, `0x0a` header size 52, `0x0e` payload size, `0x12` payload + 52.
- `0x16` six U16 timestamp fields, `0x22` build string, `0x3e` U32 always 7 (meaning unknown), `0x42` active count.
- Data starts at `0x46`. Each record is 5,632 bytes of plain JSON then 244 bytes of attributes.
- **No container checksum was found.** `CAREERDRAFT-2011DRAFT` and its `.bak` have identical headers with different content.
- 203 of 244 attribute bytes are mapped. The other 41 were zero on every row of all four game files, so they are recorded as unknown and must be preserved.

Game fixtures, all `Madden-27-RL4-9244223`, 389 active:

| File | SHA-256 | Timestamp |
|---|---|---|
| TEMP1 | `3e8b5a3f2634df3645351592a01b74f3454aa440dc79a4cb1d8f750fa2dfde71` | 2026-11-01 22:33:46 |
| TEMP2 | `55c91c004873d16938d3d48d032e6cf4fdb526982662953b82ef0f93612fb0e1` | 2026-11-01 22:36:05 |
| TEMP3 | `2611dda25e41f30aaa30db4a0f8038c715445907aadfbd62789df105fc31a457` | 2026-11-01 22:38:03 |

The field map held on all three: constants at `0x6b` and `0x7d`, no bytes after the JSON, valid JSON in every record, no empty PIDs. TEMP2 has one prospect with 4 DNA traits, so 5 is typical and not a rule. 272 to 280 distinct gear assets per class.

### 5.3 Legacy writer defects (do not port)

| Defect | Evidence |
|---|---|
| A no-op rewrites every active visual block | 389 of 389 on TEMP1, TEMP2, TEMP3 and the template. Attribute bytes survive; visuals do not. |
| It invents data | On the template it added `skinTone` to all 389 and `bodyType` to the 174 records that lacked it (defaults to `Muscular`). |
| Attribute section is cleared and rebuilt | Harmless on today's files, but would destroy any future non-zero unknown byte. |
| Non-ASCII JSON can be cut | `buffer.write(json, start, json.length, 'utf8')` uses a character count as a byte limit. By code reading only; not reproduced. |
| Non-ASCII names are lossy | Names are written through ASCII. By code reading only. |
| File detection uses size only | `detectGame` does not check magic, build or count. |

Whether the added `skinTone` matters in game is unknown; the code comment says it helps match body and face. New classes may keep it deliberately, but opening and editing a file must not.

**Required replacement:** patch only the changed field. A no-op returns the original bytes. The rewrite of a visual block happens only when that player's visuals change.

### 5.4 Franchise file

Probed on the one fixture and copies of it, using the legacy `madden-franchise` as the reader.

- File size 5,662,392 bytes, the same as M26 career saves, so the container looks fixed-size.
- Build `Madden-27-RL2_5-9171402`. Schema major 965 at `0x3e`, which MyFranchise maps to bundled schema 620. 987 maps to 640.
- Compressed length at `0x4a`, zlib stream at `0x52`. Unpacked is 20,305,266 bytes, starting `FrTk`, with 2,624 tables.
- **2,373,411 bytes follow the stream, and 736,880 of them are non-zero.** These look like leftovers from earlier saves. The reader keeps them, so MaddenTool must too.
- Player table: 3,960 rows. Team, Coach, SeasonGame, Injury and others all read successfully. It parsed in about 115 ms.

| Test on a copy | Result |
|---|---|
| No-op save | Byte-identical file |
| Change one player's weight, save, reopen | Value persisted. 1 unpacked byte changed. In the header, only the compressed length changed. File size unchanged. |
| Compress with Python zlib levels 1, 3, 5 to 9 | None reproduces the stored stream |

So byte-identical recompression should not be a goal. Instead: no-op returns the original bytes, and an edit must inflate to exactly the expected bytes. **No container or table checksum was updated in that edit, which suggests none exists. The game's acceptance of such a file was not tested here.**

The stored stream is close in size to this library's own output, and the file passed through MyFranchise, so the fixture **cannot be treated as an untouched game save.**

**M27 contracts** (schema 620 and 640, 26 fields each): `SalaryYear0-9`, `BonusYear0-6`, `Length`, `CurrentYear`, `VoidYears`, `IsSalaryGuaranteedYears`, `NoTradeClause`, `ExtraYearOption`, `TimesRestructured`, `SalaryDistribution`, and a `ContractIncentive[]` reference. Each incentive has `CurrentProgress`, `EarnedYears` and `IncentiveGoal`. Field names are confirmed; the cap formulas still need goldens.

**Schema drift is a live risk.** Between 620 and 640, 21 tables changed, 2 were added and 1 removed, though Player, PlayerContract, Team, Coach and SeasonInfo stayed the same. The game has moved through RL1, RL2.5, RL3.5 and RL4. MaddenTool needs an explicit version map, and it must refuse to edit an unmapped version. Adding a new schema must be a data change plus tests, not a code rewrite.

### 5.5 M27 roster file: a missing third format

The design names two codecs, but the current Rosters feature needs a third. `ROSTER-*` files are 6,291,530 bytes with a different header size (0x38) and hold TDB2 data. `madden-franchise` cannot read them. Legacy has its own `RosterFileService`, `RosterContainer` and `Tdb2Engine`, with test tooling (`roster-gate1` to `5`). Added as crate `mt-roster-format` and slice 04e.

### 5.6 Fixtures

| Fixture | Source | Privacy | Use |
|---|---|---|---|
| TEMP1, TEMP2, TEMP3 | Game export today, supplied by the user | Generated names; game data, so not committed | Primary draft codec oracle |
| `CAREERDRAFT-2027Template` | Game, RL1 | Same | Older build check |
| 16 `CAREERDRAFT-*` in Saves, including `.bak` | Tool output | The user's own work | 402 cases; copied before use |
| `CAREER-SEP15...AUTOSAVE191603` | MyFranchise backup, RL2.5 | **Real save, stays local** | Schema 620 tests |
| `ROSTER-Official`, `ROSTER-GATE*` | Game (RL4) and tool (RL2.5) | Local | Roster codec |
| M26 `CAREER-*` | Game | Local | Refusal test only |
| **Needed: RL4 franchise saves** | User | Local | See below |

**Needed from the user before WP05 closes:** a solid RL4 save at preseason, mid-season, playoffs and offseason (staff week, free agency, draft), made while MyFranchise is not touching them. Autosaves are fine if copied first. Several rows need specific stages to test at all.

Repository fixtures are synthetic only: tiny generated files, plus a local fixture path through `MADDENTOOL_FIXTURES`. Tests that need real files skip with a visible message, and CI reports what was skipped.

### 5.7 Differential tests

1. **Draft codec:** Rust against the legacy parser, field by field, on all fixtures. No-op byte equality. Mutation sweep across every field, checking only expected bytes change. Truncated, wrong-size and wrong-magic files refused.
2. **Franchise read:** every table and field against `madden-franchise`, on every fixture.
3. **Franchise write:** one-field edits compared with the oracle's unpacked bytes, plus reference checks across tables.
4. **Algorithms:** goldens from WP00R with a recorded seed where random. Where MyFranchise randomness can't be seeded, compare distributions and rule invariants, and say so in the row.
5. **Draft generation:** semantic diff against the TypeScript build for at least 1965, 1987, 2003 and 2026. Every difference gets a recorded reason.

JavaScript stays a development oracle only and never ships.

## 6. MMC audit

### 6.1 What was observed

From `PortraitManagerPlugin.dll`, `TexturePlugin.dll` and `MMCEditor.exe` strings. Observed text, not observed behaviour.

- Tools include **Import Player Portraits** and **Import Coach Portraits**, plus image library import and export.
- Input is a **folder of DDS files, each named with the numeric ID**. "Invalid filename format ... (must be numeric ID)". PNG is not accepted by this path.
- It must select `assetlibrary_playerportraits_brt` content. New assets are named `.../assets/added/plpo_ID_<id>`.
- Texture import lists `R8G8B8A8_SRGB` and `BC7_SRGB`, and says "Format must be SRGB variant".
- **There is a slot limit:** "Ran out of available portrait slots". The size of the limit is unknown.
- `.fbmod` has "tamper protection (requires re-export of mod...)". The format should not be written or relied on.
- `MMCModManager.exe` has a `-launch` argument and the message "missing profile name". That hints at launching by profile, but it is untested and undocumented. `FrostyCore.dll` has "Command line arguments to run on launch", which is the game's own arguments.
- **No editor automation API or CLI was observed.**

ReadMe facts: run as administrator, tools on the same drive as the game, and the user replaces the anti-cheat launcher. MaddenTool never automates that. Mods are not allowed online.

### 6.2 Draft tool's pipeline

- **DDS:** a sampled file is 1,048,724 bytes, with a 148-byte header (`DDS `, DX10, DXGI 29) and 512×512 RGBA. This matches the strings above. 8,673 such files sit in the legacy cache, which is 8.6 GB in total.
- **ID selection:** an M26 ID is kept when M27 dropped that image, else a custom ID from 60000 to 65535. Portrait fields are 16-bit.
- **Collision handling:** skips IDs found in `.fbmod` files in an `other-mods` folder, by searching for the text `assets/added/plpo_id_<n>`.

Issues:

| Issue | Consequence |
|---|---|
| Collision search is case-sensitive lowercase, and the plugin's string is `plpo_ID_` | It may never match. No `.fbmod` exists locally to test, and the folder is empty. Treat it as **unverified**. |
| Custom ID registry is per-machine | A pack from macOS or Linux and a class from Windows could disagree. The registry must travel in the pack manifest. |
| Shipped-ID list comes from an earlier build | Could be wrong on RL4, silently replacing or missing portraits. Must be re-extracted. |
| Custom IDs at 60000 and up | Whether the plugin and the game accept them is not shown by this audit. Same for 8,673 imports against the slot limit. |
| Older pack folders hold PNG | `CAREERDRAFT-2000DRAFT` has 9 PNGs that the importer would ignore. Only the full pack clears them. |
| `PortraitModService` | An older PNG-named-by-slot flow for Frosty. Unproven on M27. |

### 6.3 Validation procedure (WP10b, on Windows, by the user or with them)

Use a scratch MMC project and a copy of Saves.

1. Build a 3-portrait pack: a known M27-dropped ID (494), a custom ID (60000), and one replacing a shipped ID.
2. Export a small class pointing at those IDs and a fourth prospect with a default portrait.
3. In MMC Editor: Tools > Portrait Manager > Import Player Portraits, then pick the folder. Record the log lines.
4. Export the mod. Add and apply it in Mod Manager. Launch.
5. Import the class in a new franchise. Screenshot each of the four players in the draft and roster screens.
6. Negative cases: non-numeric name, PNG, wrong size, non-sRGB. Record each message.
7. Scale: import the full pack and record slots, time and errors.
8. Collision: export the mod from step 4, run the collision detector on it, confirm whether it finds the IDs.
9. Remove the mod, confirm clean behaviour, and check the fallback portrait.
10. Record game build, MMC version and results in the row. Only then mark `DG-PACK-*` runtime-verified.

Separately test the `-launch` argument only if launching is wanted. Until then the app opens the tool's folder and shows the steps.

### 6.4 macOS and Linux

Both produce the same portable output: draft class, DDS folder and manifest with ID registry, packaged as a folder or `.tar` with a transfer README. Steps then happen on a Windows MMC install. Nothing depends on launching MMC. MMC detection and launch are hidden with an explicit reason. A user-selected Wine or Proton Saves path works as any other folder, with no promise of support. There is no `.fbmod` writer on any platform.

## 7. Builder review

### 7.1 Decisions made

These are settled, with reasons. They are also in the design.

| ID | Deviation from MyFranchise | Why |
|---|---|---|
| D-1 | **No writes at load.** MF writes a file ID, reorganizes picks, repairs visuals and saves on open. MaddenTool shows a health report and writes only on commit. The file ID uses MF's field, written at first commit, and an existing ID is reused. | Required by "read originals without mutation". Keeps MF's run-once history meaningful. |
| D-2 | **Actions apply to a session, then commit.** MF saves after each of 34 actions, in place. First commit defaults to a new `CAREER-` file; later commits update that file with rolling backups. Replacing the original is an explicit choice. A "save after each action" setting matches MF's feel through the same safe commit. | Required by save integrity, while keeping speed. |
| D-3 | **Injured Reserve Fix uses the ≥70 filter.** | The tool's text and FAQ state it. 2.0.2 fixes it. Baseline otherwise stays 2.0.1. |
| D-4 | **Replace MF's blocking overlay with job progress.** | Required by "usable while large jobs run". |
| D-5 | **Resizable window replaces the scale setting.** MF is fixed, frameless and non-resizable at 1840×1000 scaled. | Required by responsive layout. MF layouts are not an oracle at small sizes. |
| D-6 | **Run-once guards stay, with MF's known caveat fixed in wording.** Restoring a backup with the same ID keeps history. The delete action remains. | FAQ documents this. Preview text must explain it. |

Other choices:

- **Stack:** Tauri 2 stable. `cargo search` lists 3.0.0-alpha as newest, and alpha is not suitable. React 19, TanStack Query 5, TanStack Virtual 3, rusqlite, flate2. Versions checked 2026-11-01; pin at scaffold.
- **M26:** removed, kept only as refusal test data.
- **Legacy franchise tools:** in scope as requested, and each must be re-proved on M27. `DG-FRAN-07` merges with the MF schedule if results match.
- **League history seeding:** script-only today, so tracked but not parity-blocking unless the user wants it.

### 7.2 Rust and Tauri boundaries

```
crates/
  mt-core              ids, errors, revision, job model, atomic commit
  mt-draft-format      M27 CAREERDRAFT
  mt-roster-format     M27 ROSTER (TDB2)
  mt-franchise-format  FBCHUNKS, FrTk, schema map, tables, references
  mt-franchise         session, queries, change sets, invariants, tools, league logic
  mt-draftgen          historical data, ratings, class building
  mt-store             SQLite, migrations, settings, history, backups index
  mt-interop           DDS, PID registry, manifests, MMC detection
app/src-tauri          thin commands, capabilities, updater, asset protocol
ui/src                 shell, query layer, features/*
```

- **Commands:** typed DTOs with generated TypeScript bindings. Errors are a closed enum with a user message and a recovery hint. No general file read or write commands; paths come only from dialogs or saved scoped entries.
- **Session:** one owner thread per open file. Commands send messages. Reads return a page plus revision. Commits return the new revision and affected IDs.
- **Jobs:** states are queued, running, cancelling, then succeeded, failed or cancelled. Rust checks a cancel token between units. Progress goes through a Tauri channel, throttled to about 10 updates a second. Each job has an ID and the revision it was based on. The UI ignores results from older ones.
- **Images:** a scoped custom protocol. MF's sync base64 calls are not copied.
- **React:** owns view state and unsaved form state only. Query keys include file ID and revision scope. Mutations show as pending until Rust confirms; failure restores and explains.
- **Commit:** check source hash, check invariants, write a sibling temp, flush, reopen and validate, atomically replace, then fsync the directory where supported. Windows uses `ReplaceFileW` with bounded retry for sharing violations. Leftover temps are detected at startup.
- **Capacity:** if franchise compressed output exceeds capacity, refuse the commit and change nothing.

### 7.3 Responsive and reactive behaviour

- **Below 1024 wide:** navigation collapses to a rail and menu. Master and detail stack. Dialogs become full-height sheets with fixed action bars.
- **Tables:** virtualized, horizontal scroll inside the table, sticky first column. Column sets are saved per view.
- **Dense screens** (weekly preview, team overview, trade center, offseason tracker): defined single-column order, not shrunk layouts.
- **Refresh:** selection, scroll and unsaved forms survive background refreshes. If an edited record changes under a form, show a conflict and don't overwrite.
- **Targets:** input feedback under 100 ms, p95 filter under 200 ms on 32,140 rows. Record hardware and build. Revise with evidence.

### 7.4 Platform gates

| Gate | Windows | macOS | Linux |
|---|---|---|---|
| CI build and tests | `windows-2025` | `macos-15` and arm64 variant | `ubuntu-24.04` |
| Package | NSIS | DMG for arm64 and x64 | AppImage, deb |
| Smoke | Install, open, export | Same, and Gatekeeper state recorded | Same, under xvfb |
| Atomic commit fault tests | Yes | Yes | Yes |
| Real-display check | Required | Required | Required |
| Game and MMC | Yes | Portable output only | Portable output only |

The runner list came from the actions/runner-images README on 2026-11-01. Intel macOS runners are shrinking, so WP01a must confirm the labels. A CI pass is not runtime proof.

## 8. Asset and data licensing

| Material | Size or count | Found rights | Handling |
|---|---|---|---|
| MF app art | 9,195 WebP, 145 MB, plus team logos | MIT label only; NFL and EA marks | **Do not ship or copy** |
| MF tuning data: trade, progression, attributes, tools | ~80 JSON files | MIT label only, no source | Track provenance before incorporating into shipped artifacts |
| MF historical data | `HistoricalSeasonStats.json` 28 MB, past awards and history | Same | Track provenance before incorporating into shipped artifacts |
| `madden-franchise` code and schemas | MIT in `package.json`, no license file; GitHub detects none | Schemas describe EA's format | Original Rust. Verify applicable terms before incorporating schemas; retain required notices. |
| Legacy code | ISC, user's own | OK | Port freely |
| Legacy portraits | `portraits` 42 MB, `retro-portraits` 177 MB (from game discs), gear images 36 MB | EA assets | Not in repo. User-supplied local folder. |
| nflverse CSVs | Downloaded | Verify at WP03a | Download at first run, as now |
| Wikipedia data | Downloaded and baked | CC BY-SA where text or data applies | Attribute; check at WP03a |
| Pro-Football-Reference images and data (`cdn.ssref.net`) | Baked lookups | Terms restrict reuse | Review at WP03a |
| ESPN and NFL headshots | Opt-in fetch | Third-party | User-initiated only; never bundled |
| Saves, draft fixtures, MMC, installers | | Not ours | Never in repo |

The current docs contain local paths and the user's account details. Sanitize before any public push.

## 9. Testing strategy

| Layer | Tooling | Required for |
|---|---|---|
| Unit and property tests | `cargo test`, `proptest` | Every codec, calculation, invariant |
| Fault injection | Test filesystem wrapper | Commit, cancel, disk full, lock, conflict |
| Differential | Node scripts against legacy and `madden-franchise`, dev only | Codecs, generation |
| Golden | WP00R captures with hashes | MF calculated views and tools |
| UI | Vitest, Testing Library, Playwright on the Vite build with a mocked IPC, plus axe | Components, a11y, viewport sweep |
| Native | `tauri-driver` on Windows and Linux; manual checklist on macOS | Dialogs, paths, jobs |
| Performance | Recorded benchmark | 32k pool, table scroll during a job |
| In-game | Written checklist run with the user | Every row that writes game data |

Rows move: `discovered`, `behavior-verified`, `implemented`, `tested`, `runtime-verified`. An in-game write check is required before `runtime-verified`. To keep this practical, batch them at the end of each package.

## 10. First useful end-to-end slice

**Open a game-generated draft class, change one player, save a new file, reopen it.**

It uses TEMP1 from a user-picked path on all three operating systems and proves the Tauri shell, responsive layout, typed commands, job progress and cancel, source hash check, patch writer and atomic commit. Its pass criteria:

- Opening shows 389 active, 455 capacity, RL4 build.
- A no-op save is byte-identical.
- A rating edit changes exactly one byte.
- Name edits outside ASCII are refused with a clear reason, until that behaviour is proven.
- The file imports in Madden.

This comes first because it carries all the safety machinery with the least format risk. The first franchise slice is read-only: open a save, list teams and players, page through 3,960 rows, and detect external change.

## 11. Coordinator resolutions and remaining gates

Updated after review by Codex on 2026-11-01.

- **Repository:** default to a private `AMattHT/MaddenTool` during development. `gh api user` and `gh api users/AMattHT` both return login `AMattHT`, ID `262685875`. The CLI's `amatthewsHT` label is stale; querying that login returns 404. This resolves the ownership question without user input. Public distribution remains a later product decision, not a prerequisite for foundation work.
- **Reference data:** keep third-party bundles, art, schemas, tuning datasets and real saves local while recording provenance and verifying applicable terms before including material in shipped artifacts. Private visibility is not evidence of permission. Build original Rust implementations and compare observable behavior; the audit does not establish that independent implementation can only approximate parity. Missing redistribution evidence is an artifact-specific dependency, not a blocker to the UI/core foundation.
- **Reference testing:** the requested implementation and compatibility work already authorizes ordinary testing with copies. No additional blanket approval is needed for WP00R. Establish actual process/profile/file isolation and updater/network controls before executing the reference; changing `%APPDATA%` alone must not be called a sandbox. Preserve hashes of originals. If suitable isolation is unavailable, report the specific limitation without executing against real data.
- **Game/MMC verification:** automation has not been attempted, so the earlier assertion that these checks cannot be automated is unproven. Assess available tools, installed game locations and safe scratch workflows before assigning manual checks to the user. Ask only for a concrete interaction that cannot be performed with available access.
- **Fixtures:** additional current-build saves remain necessary before the relevant format/stage gates can close. They do not prevent foundation or draft-codec work. Locate available copies first and request only missing stages when needed.
- **Design review:** the reviewed design is the remaining decision before product scaffolding. The implementation order starts with cross-platform Rust/Tauri/React foundations, then open/edit/export/reopen a draft class, while retaining the complete franchise and draft scope.

## 12. Rows lacking runtime proof

| Group | Rows | State |
|---|---|---|
| MyFranchise | All 195 `MF-*` and `F-*` | Static-only. Nothing run. |
| Draft generator | 56 other `DG-*` | Code and tests read, not executed here. Two of them also have the format probes noted below. |
| Legacy franchise tools | 11 `DG-FRAN-*` | Code only. Disabled by default. M27 unproven. |
| Portrait and MMC | 7 `DG-PACK-*` | No MMC import performed. `DG-PACK-06` and `07` carry extra doubt. |
| Binding requirements | 16 `XP-*` | No oracle. Implementation proof only. |

Two rows are `behavior-verified`: `DG-FILE-01` and `DG-FILE-07`. They cover observed format facts only, not in-game behaviour.

Highest risks, in order:

1. MF algorithms without source
2. Game schema changes across patches
3. In-game acceptance of writes across many tables, such as contracts and trades
4. Portrait slot and custom ID limits
5. The size of the work: 285 rows, each to runtime-verified
