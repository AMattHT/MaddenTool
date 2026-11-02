# Work packages and PR delivery

Refined by work package 00 on 2026-11-01. Each lettered slice is one or a few small PRs, delivering working backend, UI and proof. Parity rows are selected by the `work_package` column in `FEATURE-PARITY.csv`. No package removes features from the full-parity release.

Codex dispatches bounded prompts to Claude Opus 5.5 and reviews the returned evidence. Use a new delegate for each new audit, review or recheck. Once a repository exists, T3 prepares isolated child checkouts; do not shell-create worktrees. Base refs must be explicit.

## Order and dependencies

```
00 -> user review -> 01a -> 01b -> 02a -> 02b -> 02c   first useful slice
                              |
                              +-> 03a -> 03b -> 03c -> 04a -> 04b -> 04c -> 04d
                              |                                 |
                              |                                 +-> 10a -> 10b
                              +-> 04e (roster, needs 02b and 03c)
00R ------------------------------------------------+
02b -> 05a -> 05b -> 05c -> 05d -> 06a -> 06b/06c/06d/06e
                                     06c -> 07a/07b/07c
                              05d -> 08a -> 08b -> 08c -> 08d -> 08e
                              06a -> 09a -> 09b/09c/09d
everything -> 11a -> 11b -> 11c
```

- WP03 and WP05 can run in parallel after 02b.
- 00R must finish before any slice that ports an MF calculation (06b onward, 07, 08d, 09).
- Stack dependent slices on their real predecessor. Independent slices target the mainline.

## Gates before work starts

| Needed | Blocks | Owner |
|---|---|---|
| Reviewed design acceptance | 01a | User |
| GitHub owner | Resolved: authenticated `AMattHT` | Coordinator |
| Verified reference-app isolation and copied fixtures | 00R runtime execution | Implementer |
| RL4 franchise saves at several stages | Closing 05b, then stage-specific rows | User |
| Provenance and verified terms for incorporated third-party material | Affected shipped artifacts | Implementer; ask only for unresolved specifics |
| In-game and MMC proof | Relevant `runtime-verified` rows | Implementer first; user only for inaccessible interactions |

## 00. Evidence and design: complete

Delivered `FEATURE-PARITY.csv`, `DISCOVERY-REPORT.md`, a refined design and this roadmap.

## 00R. Runtime audit of MyFranchise 2.0.1

No product code. Testing copies is within the authorized task; verify real isolation before executing the reference app.

Run a copy extracted from the 2.0.1 package, never the installer, against copied saves. Verify isolation of application data and all file paths; redirecting `%APPDATA%` alone is insufficient. Prevent updater/network effects using scoped controls, and use supported automation tools. For each stage fixture, capture every routed page's values, tool previews and write results. Store goldens outside the repository, with hashes.

Done when: routes and tools are confirmed or corrected in the ledger, stage rules are checked, `cutPracticeSquadPlayer` is resolved, and rows change from static-only to observed. Real app data must be untouched, checked by hash.

## 01. Foundation

**01a Repository and CI.** New private `AMattHT/MaddenTool` repository under the verified authenticated account, retaining the brief after sanitizing. Workspace skeleton. CI for rustfmt, clippy, tests, and frontend format, lint, typecheck and tests. Tauri builds on Windows, macOS and Linux. Provenance manifest and check (`XP-015`). Root commit may establish metadata; product changes go through PRs.

**01b Shell and jobs.** Original app identity, responsive navigation, settings store, typed errors and generated bindings, job runner with progress and cancel, query invalidation, error and empty states. The first screen does real file selection and metadata inspection. Proof: viewport sweep from 800×600 to 1920×1080 and 200% zoom, axe scan, cancel test. Rows: `XP-004` to `XP-010`, `MF-SET-*`, shell rows.

## 02. M27 draft codec and safe file lifecycle

**02a Read and validate.** Detection by magic, sizes, build string and count. Field parser and unknown-byte map. Refuse truncated, wrong-size and non-M27 files. Differential test against the legacy parser on TEMP1 to 3, the template and tool classes. Rows: `DG-FILE-01`, `DG-FILE-07`, `XP-014`.

**02b Patch writer and commit.** Patch-only writes, atomic commit, conflict detection, backups, fault injection on three OSes. No-op byte-identical; one edit changes only its bytes. Rows: `XP-011` to `XP-013`.

**02c First useful slice.** Open a class, edit one player, save a new file, reopen. Save to Saves with a backup. In-game import of an edited TEMP class and a 402-count class on RL4. Rows: `DG-FILE-02` to `DG-FILE-05`.

## 03. Draft data and generation

**03a Data.** SQLite historical store, ingest, first-run acquisition with progress, degraded state, offline use, AFL merge. Review nflverse, Wikipedia and PFR terms.

**03b Ratings.** Three rating lenses, hindsight, strength options, OVR and attribute model with M27 overrides, dev traits, persona. Deterministic seeds. Semantic diff against the TypeScript oracle.

**03c Build and export.** Enrichment, era fillers, identity fields, undrafted and supplemental handling. Year-to-class-to-export path. Compare 1965, 1987, 2003 and 2026, recording each difference. In-game import of a generated class.

## 04. Draft studio, editing and rosters

**04a Board.** Tables, alternate views, overflow panel, blind scouting. No hidden value reaches the UI until spoilers are on.

**04b Profile and editing.** Profile card, all edits, undo and redo, edit JSON import and export, CSV.

**04c Studio and sources.** Random, decade, all-time and by-team classes, Class Studio, keyboard reorder, custom prospects, saved boards. p95 search under 200 ms on the 32k pool.

**04d Appearance.** Faces, gear, likeness overrides, skin tone, gear and tone from photo, display with no bundled proprietary art.

**04e Rosters.** `mt-roster-format` with no-op and edit proof, open and browse, historical builder, add-player preview, team assignment. In-game roster load.

## 05. Franchise format and read-only workspace

A major format project, not a wrapper.

**05a Container.** FBCHUNKS read, zlib, capacity, trailing bytes, schema map for 965 and 987. Refuse unmapped versions.

**05b Tables and references.** Native FrTk reader and writer, including records, strings, compressed text, references and empty-record lists. Differential read of all tables and fields against `madden-franchise`. One-field edit matches the oracle's unpacked bytes. Rows: `F-FMT-*`.

**05c File session.** Load screen, metadata streaming, autosave warning, external change, reload, close, backup with retention, file health report, file identity, data deletion. Nothing is written at load.

**05d Read workspace.** Paged queries, stage rules, player and coach lists, player card read tabs, team roster and injuries, title bar and back navigation. List 3,960 players with no UI stall.

## 06. Franchise operations

Every slice previews, commits, reopens and validates relationships. An in-game check closes each batch.

**06a Write foundation and player edits.** Change sets, invariants, undo. Bio, ratings, DNA, traits, abilities, appearance, custom portraits, file health fixes.

**06b Contracts and cap.** All 26 contract fields, incentives, void years, no-trade, cap views, team salaries, upcoming free agents. Cap goldens from 00R.

**06c Transactions.** Sign, practice squad, promote, cut, June 1 cut, retire, cut and retire, unretire, trade block toggle, transaction logging, depth regeneration setting.

**06d Team control.** User control, ownership, 32-team control, presets, 16 toggles, depth chart edit and auto reorder.

**06e Coaches.** Card, edit, talents, hire, fire, retire, unretire, portraits, staff tab.

## 07. Draft, trades and offseason inside a franchise

**07a Draft.** Order, pick owner, class with spoiler guard, prospect edit, big board, mocks, workouts.

**07b Trades.** Block page, builder, valuation, execution, override. Atomic multi-asset commit with roster, cap and history checks.

**07c Offseason.** Free agency tracker, offseason tracker sections, grades.

These stay distinct from standalone class generation.

## 08. Statistics, presentation and history

**08a Stats.** Player and team stats by scope, leaders, team ratings, stat editing, records, rookie tracker, historical seasons.

**08b Game views.** Schedule, standings, box score, gamecast, team overview and schedule.

**08c Home and help.** News, ticker, season-phase hubs, league hub, custom logos, contextual help in original wording.

**08d Media.** Season preview tabs, power rankings, bracket, weekly preview and recap.

**08e Awards and history.** Awards pages, Super Bowl and award history, records, season recap, draft history, retired numbers.

Informational screens count as parity. Verify season and week switching and empty states.

## 09. Tools

Every tool has a preview that matches its commit, seeded randomness where possible, a stage guard, and cancel safety.

**09a Framework and simple tools.** Hub, spotlight, schedule, options, results table. Injury tools, weather, wardrobe, ID fixes, jersey fix, depth reorder.

**09b Roster tools.** Trade block update, draft order, FA and trade lotteries, training camp, cap clearing, balance, playbooks.

**09c Progression.** Both progression tools, snapshots, run-once guards, report, player history.

**09d Legacy franchise tools.** Each re-proved on M27: cap reset, bulk edits, roster apply, trait realism, FA trim, pick reset, relocation, advance roster, historic season. Merge duplicates with MF tools only if results match. League history seeding only if the user wants it.

## 10. MMC and portrait interoperability

**10a Packs.** DDS writing, ID allocation, portable registry, manifests, custom sources, opt-in fetch, class and full packs, cancel and resume, portable output on all OSes. Re-extract the shipped ID list for RL4.

**10b MMC validation.** The 10-step procedure in report section 6.3. Tool detection and honest capability display. Decide the fate of the collision scan, the older PNG flow and any launch command from the results. No `.fbmod` writer unless the format and support are established. Never change launchers or anti-cheat.

## 11. Distribution and release gate

**11a Packages.** Windows x64, macOS arm64 and x64, Linux x64 AppImage and deb, each installed on a real or native-runner environment. Unsigned builds labelled.

**11b Updates and about.** Updater with supplied keys, tamper rejection, notes, notices. Absent credentials are reported, not invented.

**11c Parity gate.** Audit every row. Repeat integration tests on release candidates. Measure performance with recorded hardware and data. Confirm files in Madden and MMC. Release only when the gates pass, or deliver a precise remaining-gates report.

## tstack use and PR bookkeeping

Read `reference/tstack/skills/file-pr/SKILL.md` for every new PR. Use its problem-first descriptions and real model attribution. Use `leaving-pr-comments` when comments are authorized and needed. The parent may apply `babysit-pr`, using T3 watchers rather than shell polling. Full-send and automatic merging are not selected.

Keep `docs/DELIVERY.md` with package, parity IDs, branch, base, PR URL, head SHA, checks, reviews and runtime proof. Each dependent PR names its predecessor. After merging is separately authorized, retarget later layers and rerun affected checks. Never claim a stack layer is ready from checks on an earlier head.
