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
| Delivered | `crates/mt-store` (SQLite schema v1, migrations, two importers, queries, `mt-data` CLI), `.github/workflows/player-data.yml`, the draft-first plan, `design/draft-generator-higgsfield-brief.md`, `research/wav-to-ovr-contract.md` |

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
  - 95 AFL/NFL dual-draft pairs from 1960 to 1966 remain separate players (DG04a).
- Re-importing nflverse: 12,927 unchanged. JSON query output for 1936, 1965, 1987, 2020 and 2026 was byte-identical after re-import, reopen and an independent rebuild.

Not verified: CI, macOS and Linux runtime, any rating or calibration, any M27 file, any game or MMC import. No data was taken from Pro Football Reference directly; it refuses automated requests and was not worked around.
