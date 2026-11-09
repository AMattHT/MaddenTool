# Draft-first execution plan

Written 2026-11-02 in DG01. The user made **a proper draft generator** the first product milestone. This plan changes the *order* of work only. The approved design (`../specs/2026-11-01-maddentool-design.md`) still governs: Rust authoritative backend, Tauri 2 with React/TypeScript, Windows, macOS and Linux, Madden 27 only, and full MyFranchise and draft-tool parity as the release goal. Franchise work is deferred, not dropped.

Each goal below is one bounded package, usually one or two PRs. A goal is done when its acceptance list is observed, not when its code exists. Goals map onto the existing roadmap (`../../PR-ROADMAP.md`) where noted.

## Milestone: a proper draft generator

A user can choose a year or source and a ratings approach, generate or open a class, review and edit it, validate it, export a real M27 draft-class file, and reopen it. Generation is deterministic for a seed, works offline once data exists, and matches or knowingly improves on the original Madden-Draft-Generator.

```
DG01 data ──────────────┐
DG02 M27 reader ── DG03 patch/export ──┐
                         DG04a data completion ── DG04b ratings ── DG04c class build+export
WP01a/01b shell ─────────┴──────────────── DG05a review ── DG05b editing ── DG05c studio
                                                                   DG06 MMC portraits
```

DG01, DG02 and WP01a/01b are independent. DG04c needs DG03. DG05 needs the shell and the Higgsfield concepts (`../../design/draft-generator-higgsfield-brief.md`).

## Goals

### DG01 Player data foundation (this package)

Outcome: a tested Rust/SQLite store that imports nflverse `draft_picks` and the legacy `ALL_PLAYER_LOOKUP.csv`, keeps stable identity and provenance, and answers "players of year Y at position P". **Identity reconciliation is part of DG01** (moved here from DG04a at the user's request). A person drafted more than once, whether by the AFL and the NFL in one year or by the NFL in different years, is one person per class. Every selection and its provenance is kept, and a merge needs evidence beyond the name. Also delivered: this plan, the Higgsfield brief and the wAV contract.

Done when: a fresh checkout builds the `mt-data` CLI, creates and migrates a database, imports both formats, queries by year and position, and gets identical results after reopening and after an independent rebuild. Both import orders give one person per class where evidence establishes identity. Tests cover validation, identity, merges and refusals, aliases, re-import, missing versus zero wAV, migrations and provenance. Evidence: `../../DELIVERY.md` and `../../reviews/dg01-sol-loop.md`.

### DG02 Real M27 draft-class reader (WP02a)

Outcome: crate `mt-draft-format` reads and validates M27 draft-class files: magic, header, build string, 455-record capacity, active count, the 5,876-byte record layout, field map and unknown-byte map. It refuses truncated, wrong-size and non-M27 input with clear reasons. A CLI `inspect` prints metadata and players.

Acceptance:
- TEMP1-3 (RL4, 389 active) and the template parse.
- A differential test against the legacy JS parser agrees on every mapped field.
- The 41 unknown attribute bytes are preserved in the model.
- Real fixtures stay outside Git (`MADDENTOOL_FIXTURES`); synthetic minimal files are in the repository.

Rows: `DG-FILE-01`, `DG-FILE-07`, `XP-014`.

### DG03 Safe patch, export and reopen (WP02b/02c)

Outcome: patch-only writer and atomic commit.

Acceptance:
- A no-op save is byte-identical.
- One rating edit changes exactly its byte.
- Edits go to a new file by default, with a backup, a source-hash conflict check, temp-write, flush, reopen-validate and platform commit.
- Fault injection covers disk full, a lock and an interrupted write.
- A 402 export limit, with 403 to 455 blocked.
- Non-ASCII names are refused until proven.
- The user imports an edited TEMP class in Madden on RL4. Until then the in-game gate is reported as pending.

Rows: `XP-011` to `XP-013`, `DG-FILE-02` to `05`.

### DG04a Data completion, updates and offline use (WP03a)

Outcome: everything the generator needs from data, before ratings.

- The AFL/NFL identity merge is done in DG01. What remains: the 25 identities DG01 leaves unsettled on the real legacy file, each resolved with cited evidence or left separate; a league filter; and combined 1960-1969 class composition (how the AFL and NFL boards interleave).
- Same-source re-drafts across years (legacy-only, before 1980) for multi-year classes.
- Undrafted careers from the nflverse `players` release.
- Supplemental picks.
- Position mapping from source labels to Madden positions, including the `DB` split.
- Year list with class sizes.
- First-run acquisition of the nflverse files with progress and cancel, ETag or timestamp checks for updates, and a degraded state when data is missing.
- **Offline use once data exists.**
- Attribution display.

Acceptance:
- Year and player counts equal the legacy oracle or differences are listed.
- With the network blocked after acquisition, everything still works.
- Without data, the app shows an honest state with no fabricated players.
- An update never changes player UIDs.

Rows: `DG-DATA-01` to `05`.

### DG04b Ratings per the wAV contract (WP03b)

Outcome: the Realistic, Career and Launch Day lenses, the hindsight control, class strength, studs and generational options, overall and dev traits. Everything is seeded and deterministic, following `../../research/wav-to-ovr-contract.md`.

Acceptance: the contract's nine tests. The M27 curve is rebuilt reproducibly from RL4 game classes. A rank and overall comparison against the TypeScript oracle for 1965, 1987, 2003 and 2026 is recorded per position.

Rows: `DG-RATE-01` to `07`.

### DG04c Class build and export (WP03c)

Outcome: attributes, archetypes, combine, persona DNA, M27 hidden and identity fields, era fillers, PIDs, and the year → class → M27 file → reopen path through DG03.

Acceptance:
- The same seed and options give a byte-identical file on all three operating systems.
- Values fall inside the observed game distributions.
- The user imports a generated class in Madden.
- Spreadsheet export.

Rows: `DG-RATE-08`, `09`, and the class-build rows.

### DG05 Draft studio UI (WP04a to 04c), after Higgsfield concepts

- **DG05a Review board:** class table with alternate views, overflow panel, counts (active, 402 limit, 455 capacity), validation summary and blind scouting. No hidden value reaches the UI while spoilers are off.
  - Dev traits use the actual Madden 27 icon files from the user's Madden-Draft-Generator, with text labels; `hidden.png` marks an unscouted trait. Provenance and mapping are in the Higgsfield brief.
  - A person with several selections that year shows once, with every selection visible.
- **DG05b Player profile and editing:** all edits, undo and redo, edit JSON and CSV.
- **DG05c Sources and Class Studio:** random, decade, all-time and by-team classes; pool search over about 32k players with p95 under 200 ms; ordered board composition; custom prospects; saved boards.

Acceptance for each: the viewport sweep from 800×600 to 1920×1080 and at 200% zoom, keyboard use, axe, and real data with no placeholder dashboards.

### DG06 MMC portrait interoperability (WP10a/10b)

DDS portrait packs, PID allocation and a portable registry, manifests, and the MMC validation procedure in `DISCOVERY-REPORT.md` 6.3. No `.fbmod` writer unless the format is established.

### Deferred

Franchise saves and the workspace (WP05 to 09), rosters (WP04e) and the legacy franchise tools stay in scope after the draft milestone. The MyFranchise reference moves to 2.0.4 (`../../reference/SOURCES.md`). Its deep audit (WP00R) is needed before any franchise calculation is ported, and not before.

## Cross-cutting rules for every draft goal

- **Roles (user direction, 2026-11-02):** Claude Opus 5.5 on High does all implementation, design-tool work (Higgsfield concepts included), testing and shipping. GPT-6.1 Sol on High reviews PRs. The parent drafts prompts and coordinates.

- **Parity:** each goal lists the legacy behaviour it reproduces and every deliberate difference.
- **Determinism:** seeds are explicit and shown. No wall-clock input to generation. The same inputs give the same bytes.
- **Data:** real datasets are never committed. Every displayed value can show its source and as-of date. PFR is never contacted by the product.
- **Export proof:** "exports" means written, reopened and validated by MaddenTool. "Imports in Madden" needs an in-game check, reported separately.
