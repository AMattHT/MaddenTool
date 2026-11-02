# DG01 review ledger (sol-loop)

Package DG01, branch `t3/reusable-player-database`, base `main` at `def90394db3d352d790dc4ebbd8b54dd3eef2db8`. Nothing pushed, no PR. The parent coordinates Sol reviews; the implementer records findings and dispositions here.

**Approval status: none.** Round 2 resolved S1, S2 and U1 and raised S3. S3 is fixed on the branch but awaits review. The next review is round 3: GPT-6.1 Sol on **High**, scoped to S3 and its delta since `bc4c963`. The parent dispatches it.

## Rounds

| Round | Reviewer | Head reviewed | Verdict | Usage |
|---|---|---|---|---|
| 1 | GPT-6.1 Sol, **xhigh** (run before the user required High) | `c86e66481695316a68b17142e4cbac7fab7c4480` | CHANGES | not exposed to the implementer |
| 2 | GPT-6.1 Sol, **High** | `bc4c96321ca0f7482518162a73800f4cb9369378` | CHANGES: S1, S2 and U1 resolved; S3 raised | not exposed to the implementer |
| 3 | GPT-6.1 Sol, High, scoped to S3 and its delta | pending | pending | |

## Findings

| ID | Severity | Location (at `c86e664`) | Claim | Evidence | Disposition | Issue | Fix |
|---|---|---|---|---|---|---|---|
| S1 | should | `crates/mt-store/src/import.rs:175` | Legacy-first imports duplicate re-drafted players: the second nflverse selection resolves the PFR ID and returns without reconciling that year's legacy record | Reproduced by Sol, the parent and the implementer. Synthetic Alpha appeared twice for 2002 (`mt-005366f5c024bee7`, `mt-584c3883b3b0cef1`). The regression `redrafted_player_is_one_person_in_either_import_order` failed before the fix (`left: 2, right: 1`). On real data at `472a4c3`, legacy-first left 3 re-drafted people split across two players | **Agreed, fixed.** A reconciliation pass after each import merges records that share a cross-source draft slot and a compatible surname. Merges keep the absorbed uid as an alias, refuse on conflicting external IDs, and repair schema v1 databases | [#2](https://github.com/AMattHT/MaddenTool/issues/2) | `cd6f6e0` |
| S2 | nit (confirmed bug) | `crates/mt-store/src/query.rs:229` | `sum(wav = 0)` is NULL when every wAV is missing, so `info` fails | Parent reproduced `Invalid column type Null at index: 5`; regression `info_counts_missing_wav_without_treating_it_as_zero` went red, then green | **Agreed, fixed by the parent** in `472a4c37ed75d27b05d50b22587b24ee8e4da554` (`coalesce(..., 0)`). The implementer verified the change and keeps it | [#3](https://github.com/AMattHT/MaddenTool/issues/3) | `472a4c3` |
| U1 | user-reported | identity design | AFL and NFL selections of one person (1960-66) appear twice, for example Bob Brown 1964 | `472a4c3` lists Bob Brown twice in 1964 (Nebraska; Ark-Pine Bluff/DT). The regression `afl_and_nfl_selections_of_one_person_count_once` failed before the fix (`left: 2, right: 1`). Independent sources are cited in `crates/mt-store/data/corrections.json` | **Fixed** with evidence rules for dual drafts, given-name variants, a career footprint, and a cited correction for the Bob Brown AFL row. 25 real identities stay unsettled and listed, not forced | [#4](https://github.com/AMattHT/MaddenTool/issues/4) | `cd6f6e0` |
| S3 | should (round 2) | `crates/mt-store/src/identity.rs:433` (at `bc4c963`) | `conflicts()` omits merges that reconciliation refuses: only `reconcile()` applies `refusal()`, so an import can report `merge_refused: 1` while `mt-data conflicts --json` prints `[]` | Sol's CLI reproduction (one person's two 2001 selections, legacy imported first, nflverse rows sharing PFR ID). The parent confirmed the code path; the implementer reproduced it with the CLI at `bc4c963`. Regressions `conflicts_lists_merges_reconciliation_refuses` (same-league-year refusal) and the extended `merge_is_refused_when_external_ids_disagree` (external-ID refusal) failed before the fix (`left: 0, right: 1`; `left: [], right: [merge_refused ... different pfr ids]`) | **Agreed, fixed.** `conflicts()` keeps the proposals from `propose()` and appends `merge_refused` for each one the existing `refusal()` rejects. A shared helper gives the same year and detail as `reconcile()`. It stays read-only: no call to `reconcile()`, no change to eligibility, no second refusal implementation | [#5](https://github.com/AMattHT/MaddenTool/issues/5) | `dbd32fe` |

Round 2 (High) verified S1, S2 and U1 as resolved, with independent checks: both import orders, repeat imports and reopen; all 91 years; v1 repair; missing-only wAV diagnostics; Bob Brown's selections, correction guards and sources; SQLite integrity and foreign-key checks; a three-player alias-flattening case. No finding was declined. No other bug was confirmed in this package. Two other things are data disagreements, not code bugs, and are recorded instead of filed:

- the legacy file's wrong college and position on the Bob Brown AFL row, handled by the correction and described in #4;
- the 2022 NFL pick 140 name conflict between sources.

## What the fix changed, for the next reviewer

Changed code since `472a4c3`, all under `crates/mt-store/`:

| File | Change |
|---|---|
| `src/identity.rs` (new) | The reconciliation rules and merges |
| `src/corrections.rs` (new), `data/corrections.json` (new) | Verified row corrections |
| `src/names.rs` | Given-name variants |
| `src/schema.rs` | Migration v2: `player.merged_into`, `player_merge`, `record_correction` |
| `src/import.rs` | Applies corrections, runs reconciliation, keeps the v1 content hash for uncorrected rows; the slot-link guard also checks league |
| `src/query.rs` | One row per person and year, with `selections`; `player` lookup by any uid; alias-aware counts |
| `src/lib.rs`, `src/bin/mt-data.rs` | `player`, `conflicts` and `reconcile` |
| `tests/identity.rs` (new) and two synthetic fixtures | Regressions |

S3 delta since `bc4c963`: `src/identity.rs` (`conflicts()` reports refusals; `refused_conflict` helper shared with `reconcile()`) and `tests/identity.rs` (one new regression, one extended test). Real data is unchanged: `conflicts` still lists 25 in both import orders, with no refusals present.

Out of scope: PR #1 (`feat/cross-platform-foundation`) and its CI failures.

## Commits

| Commit | What |
|---|---|
| `c86e664` | DG01 as first reviewed |
| `472a4c3` | S2 fix (parent) |
| `cd6f6e0c82f27962060f0fef324e3c73dde8d378` | S1 and U1 fix, docs |
| `bc4c963` | Ledger SHA update (round 2 reviewed head) |
| `dbd32fe72ea4b0bb7685ab8af1f8be0666cc96cb` | S3 fix and records |
