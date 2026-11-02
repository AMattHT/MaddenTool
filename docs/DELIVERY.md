# Delivery ledger

## Authorization

The user approved the reviewed design and instructed implementation to begin. Approved scope: Madden 27 only; full MyFranchise/draft-tool feature parity; Rust + Tauri/React; responsive/reactive UI; Windows/macOS/Linux; private AMattHT/MaddenTool development; Opus 5.5 implementation; tstack PR workflow. PR merging is not authorized.

- Design: `superpowers/specs/2026-11-01-maddentool-design.md`
- Approved design SHA-256: `b73b95d9e3c8086fadb69987ba789e4199ada98e7c4daf8bdeefb9433a7b0411`
- First execution plan: `superpowers/plans/2026-11-01-foundation.md`
- First prompt: `prompts/01a-foundation.md`

## WP01a

Status: assigned for implementation; no PR or product results yet.

Provider/model: `claudeAgent` / `claude-opus-5-5`.

Destination: private `AMattHT/MaddenTool`. Intended product branch: `feat/cross-platform-foundation`, based on the minimal main baseline. The implementer records the actual base/head and PR after setup.

## 2026-11-02 direction change: draft generator first

The user made a proper draft generator the first milestone, authorized full-ship (merging after proof, Sol review, CI and normal protections; the parent owns review and merge), and moved the MyFranchise reference to 2.0.4. These supersede the "merging is not authorized" and "2.0.1 is current" lines above. Order: `superpowers/plans/2026-11-02-draft-first.md`.

Observed on 2026-11-02:

- `AMattHT/MaddenTool` is **PUBLIC**, not private. Visibility was not changed.
- The local `gh` has two accounts. `AOTamatthews` is active by default. The user asked for `AMattHT`; DG01 used its token per command, without switching the machine-wide account.
- PR #1 (WP01a, `feat/cross-platform-foundation`) is open and unmerged.

## DG01 Player data foundation

Status: **implemented and locally proven; committed, not pushed.** No PR yet. The parent runs Sol review and shipping.

| Field | Value |
|---|---|
| Provider/model | `claudeAgent` / `claude-opus-5-5`, high reasoning, in T3 Code |
| Branch | `t3/reusable-player-database`, based on `main` at `def9039`. Independent of PR #1 |
| Parity rows | None closed. Prepares `DG-DATA-01` to `05` (DG04a) and `DG-RATE-*` (DG04b) |
| Delivered | `crates/mt-store` (SQLite schema v2 with migrations from v1, two importers, identity reconciliation, verified corrections, queries, `mt-data` CLI), `.github/workflows/player-data.yml`, the draft-first plan, `design/draft-generator-higgsfield-brief.md`, `research/wav-to-ovr-contract.md` |

Rebasing onto PR #1 will conflict in `Cargo.toml` (workspace members: keep both), `Cargo.lock` (regenerate) and this file. PR #1's provenance check will also need manifest entries for the three fixture `.csv` files and the crate's `Cargo.toml`. Its `.rs`, `.md` and `.yml` globs already cover the rest.

### Evidence (Windows 11 x64, Rust 1.99.0, local only)

| Check | Result |
|---|---|
| `cargo fmt --all -- --check`; `cargo clippy --workspace --all-targets --locked -- -D warnings` | Pass |
| `cargo test -p mt-store --locked` | 24 pass: 9 unit, 15 integration including the CLI |
| Mutations: blanks read as 0; external-ID linking off; time-dependent UIDs | Caught: 8, 4 and 1 failing tests. Restored |
| CI smoke script, run locally in Git Bash | Pass. **CI itself has not run**; macOS and Linux are untested |
| Synthetic fixtures through `mt-data` | 9 rows: 6 inserted, 3 rejected with line numbers. Legacy fixture: 7 rows, 6 inserted, 1 slot link, 1 mismatch recorded |

Real reference sample: local scratch only, never committed.

- **nflverse `draft_picks.csv`** (sha256 `cc35472c…5f9f`, `timestamp.json` 2026-10-28):
  - 12,927 rows, 1980 to 2026, all imported.
  - 12,924 players: three people are drafted twice under one PFR ID, Bo Jackson included.
  - wAV missing 2,128, zero 1,613, matching an independent Python count.
  - 2026 class: 197 placeholder zeros classified `no_completed_season`, 60 missing.
  - `car_av` is blank in every row.
- **Legacy `ALL_PLAYER_LOOKUP.csv`** (master `9a4ac69f`, sha256 `b06f3e26…d7e`, as-of 2026-09-15 by commit date):
  - 32,246 rows: 32,245 imported, 1 rejected (round `AT`).
  - 12,923 of 12,926 1980+ NFL selections linked to nflverse by slot and surname.
  - 2 recorded mismatches (a Klosterman/Klostermann spelling; a 2022 pick 140 name conflict), plus 1 supplemental pick correctly kept apart.
  - 95 AFL/NFL dual-draft pairs from 1960 to 1966 remained separate players. Superseded by the identity package below.
- Re-importing nflverse: 12,927 unchanged. JSON query output for 1936, 1965, 1987, 2020 and 2026 was byte-identical after re-import, reopen and an independent rebuild.

Not verified: CI, macOS and Linux runtime, any rating or calibration, any M27 file, any game or MMC import. No data was taken from Pro Football Reference directly; it refuses automated requests and was not worked around.

### DG01 identity reconciliation and round 1 findings (2026-11-02)

Status: **committed on the branch at `cd6f6e0` (S1 and U1 fix; S2 fixed in `472a4c3`), not pushed; no Sol approval yet.** Round 1 (GPT-6.1 Sol, xhigh) asked for changes; the ledger is `reviews/dg01-sol-loop.md`. Issues [#2](https://github.com/AMattHT/MaddenTool/issues/2) (S1), [#3](https://github.com/AMattHT/MaddenTool/issues/3) (S2) and [#4](https://github.com/AMattHT/MaddenTool/issues/4) (AFL/NFL duplicates, user-reported) stay open until their fixes reach `main`.

AFL/NFL identity is now part of DG01, not DG04a. Schema v2 adds:

- player aliases (`merged_into`), so every uid ever issued resolves;
- a merge log with evidence;
- verified row corrections.

Reconciliation runs after every import, and `mt-data reconcile` repairs a v1 database. Rules and refusals are in `../crates/mt-store/README.md`.

PR #1 interplay: its provenance manifest will also need exact entries for `crates/mt-store/data/corrections.json` and the two new fixture CSVs.

| Check (Windows 11 x64, Rust 1.99.0) | Result |
|---|---|
| Regressions before the fix | S1 and AFL/NFL tests failed at `472a4c3`'s behaviour (`left: 2, right: 1` each). The namesake guard passed, as intended |
| fmt, clippy `-D warnings`, `cargo test -p mt-store --locked` | Pass: 36 tests (10 unit, 10 identity, 16 store) |
| Mutations | All caught, then restored: no reconciliation; alias merged without career; same name across colleges merged; ambiguous partners merged; merge refusal off; alias not kept |

Real data, both tools, scratch only:

| Database | People | Person-years | 1960-66 | 1980+ re-drafts split |
|---|---|---|---|---|
| `472a4c3`, legacy then nflverse | 32,249 | 32,252 | 2,837 | 3 |
| `472a4c3`, nflverse then legacy | 32,246 | 32,249 | 2,837 | 0 |
| Fixed, either order (also after repeat imports and reopen) | 32,128 | 32,131 | 2,719 | 0 |
| v1 database from `472a4c3`, `reconcile` then re-import | 32,128 | 32,131 | 2,719 | 0 |

- **Order independence:** query output for all 91 years (1936-2026), uids excluded, is identical between the two import orders.
- **v1 upgrade:** re-importing reports only the corrected Bob Brown row as updated (legacy: 1 updated, 32,244 unchanged; nflverse: 12,927 unchanged).
- **Dual-draft merges:** 118 in all. 89 have the same name and college, including Bob Brown through the correction; 29 are given-name variants. None merged on career footprint alone.
- **Bob Brown, 1964:** before, the Nebraska tackle appeared twice (NFL r1 p2 Nebraska LT; AFL r1 p4 "Ark-Pine Bluff" DT). After, he is one person with both selections, and the AFL selection is marked `corrected: 1964-afl-r1p4-bob-brown`. Robert Brown (Ark-Pine Bluff, NFL r13 p169) stays separate with his own wAV 51. Sources are cited in `crates/mt-store/data/corrections.json` and #4.
- **Unsettled identities, listed and not merged:** 13 name aliases (Billy/Johnny Brewer, Dave/Dale Chamberlain, Kent/George Horne, C.B./Carlton Simons and others), 6 career conflicts (Irv Cross, Mike Lucci and others) and 6 college conflicts (Tom Brown, Larry Campbell and others). `mt-data conflicts` prints all 25. They are missing evidence, not supported merges.

Dev-trait icons: provenance, hashes and the tier mapping are recorded in `design/draft-generator-higgsfield-brief.md`. They are not committed and no UI is built.

Still not verified: CI, macOS and Linux, any rating, M27 file, or game or MMC import.

### Sol round 2 (GPT-6.1 Sol, High) and S3

Round 2 reviewed `bc4c963` and returned CHANGES.

- **Resolved:** S1, S2 and U1, independently checked by the reviewer: both import orders, repeat imports and reopen; all 91 years; v1 repair; missing-only wAV; Bob Brown and his correction; SQLite integrity and foreign keys; a three-player alias-flattening case.
- **New, S3 (should):** `mt-data conflicts` omitted merges that reconciliation refused ([#5](https://github.com/AMattHT/MaddenTool/issues/5)).
- **Fixed on the branch, not pushed, no approval:** `conflicts()` now appends `merge_refused` for every proposal the existing `refusal()` rejects, worded as `reconcile()` words it, and stays read-only.

| Check (Windows 11 x64, Rust 1.99.0) | Result |
|---|---|
| CLI at `bc4c963` (Alpha's two 2001 selections, legacy first) | Import and `reconcile` reported `merge_refused: 1`; `conflicts --json` printed `[]`; two Alpha rows |
| Regressions at `bc4c963` behaviour | `conflicts_lists_merges_reconciliation_refuses` failed (`left: 0, right: 1`). The extended `merge_is_refused_when_external_ids_disagree` failed (`[]` against the `different pfr ids` refusal) |
| After the fix: fmt, clippy `-D warnings`, `cargo test -p mt-store --locked` | Pass: 37 tests (10 unit, 11 identity, 16 store) |
| CLI after the fix, same database | `conflicts --json` lists the `merge_refused` entry ("one person would have two selections in legacy-player-lookup 2001 NFL"), matching `reconcile`. The database file SHA-256 is identical before and after `conflicts`. Still two players |
| Real-data scratch databases (both orders) | `conflicts` still 25 (13 alias, 6 career, 6 college); no refusals exist in the real data |

The next review is round 3, GPT-6.1 Sol on High, scoped to S3 and its delta since `bc4c963`. Issues #2 to #5 stay open until their fixes reach `main`.
