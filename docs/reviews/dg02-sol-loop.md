# DG02 review ledger (sol-loop)

Package DG02, branch `t3/8830e17a` (T3 displays it as `t3/m27-draft-reader`), base `main` at `2f1da5728b1d058f5273020ee6515941df509899` (DG01 merged). The parent coordinates GPT-6.1 Sol High reviews. The implementer records findings and dispositions here.

**Approval status: code approved by Sol at `fabcd3ea313b7b1c34cb17c47d886811eb729fe9`.** This is a GPT-6.1 Sol review, not a GitHub human approval, and it does not complete DG02. The approval rests on two rounds:

- Round 1 reviewed the whole reader at `6911c423` and returned CHANGES.
- Round 2 was scoped to `6911c423..fabcd3ea` and its callers, and approved it. It was not a repeat of the full reader audit.

Two acceptance gates stay open, and the reviewer preserved both explicitly:

- the TEMP1 to TEMP3 (RL4) real-file and differential run in required mode;
- three-OS CI.

## Rounds

| Round | Reviewer | Head reviewed | Verdict |
|---|---|---|---|
| 1 | GPT-6.1 Sol, High | `6911c423045a49a026f2156223defa25663c898e` | **CHANGES.** S1 and S2 (should); no reader decoding or byte-preservation bug |
| 2 | GPT-6.1 Sol, High, scoped to `6911c423..fabcd3ea` and its callers | `fabcd3ea313b7b1c34cb17c47d886811eb729fe9` | **APPROVE.** S1 and S2 resolved, no new findings. RL4 and CI gates preserved |

Round 2 checks, as the parent relayed them from the reviewer thread:

- fmt and a check of the delta diff;
- clippy on the draft crate, all targets;
- the 39 always-run tests, plus the 3 opt-in tests run explicitly unconfigured;
- 24 direct configuration and template checks:
  - S1: every opt-in test fails in required mode without fixture setup. A missing oracle gives the correct variable diagnostic before TEMP1. Optional skips are preserved. A nonexistent, empty or hash-invalid directory still fails without the oracle.
  - S2: the slot check precedes comparisons in both paths. Corrupting only slot 388's hash fails. Authentic output passes.
- 10 adapter runs: repeated, missing and reordered slots all fail;
- the child-process regressions run in parallel under an invalid parent environment;
- template totals: 389 players, 31,120 comparisons, 778 constant checks, 389 visuals, 455 record hashes, 0 discrepancies. SHA-256 and mtime unchanged.

The reviewer's checkout was clean at `fabcd3ea` afterwards.

Round 1 checks, as the parent relayed them:

- fmt, workspace clippy and tests;
- the template real-file and differential runs;
- an independent literal-offset decode of all 31,898 template field values, 389 visuals, 455 record hashes and 15,949 unknown bytes;
- scratch checks of fields, DNA, text, padding, warnings, the CLI and refusals;
- source hash and mtime checks;
- the Git Bash smoke script.

Every intended reader check passed. The reviewer reproduced both findings outside the checkout.

## Findings

| ID | Severity | Location (at `6911c423`) | Claim | Confirmation | Disposition | Issue | Fix |
|---|---|---|---|---|---|---|---|
| S1 | should | `tests/support/fixtures.rs:64` `fixture_dir`; `tests/differential.rs:37` `legacy_parser` | `MADDENTOOL_REQUIRE_FIXTURES=1` does not fail on an unset `MADDENTOOL_FIXTURES` or `MADDENTOOL_LEGACY_MDG`; the tests print SKIPPED and pass | Implementer, at `6911c423`: both variables unset, required mode on, three SKIPPED lines and `cargo` exit 0. With a template-only directory and the legacy variable unset, the run failed earlier for the wrong reason (`CAREERDRAFT-TEMP1 is missing`), so the oracle branch was unreachable without RL4 | **Agreed, fixed.** Both helpers check required mode before their unset return. The differential resolves the legacy setting before the fixture files, and still validates fixtures when the oracle is optional and unset. Optional skips are unchanged | [#7](https://github.com/AMattHT/MaddenTool/issues/7) | `84a4a3e9` |
| S2 | should | `tests/differential.rs:185` loop over `ours["players"]` | The differential never checks that reported slots are exactly the active slots, so a report repeating one slot passes with full totals | Implementer, at `6911c423`: a scratch adapter replaced every player with slot 0 (389 copies) and was copied over `target/debug/mt-draft.exe`. The unchanged test binaries passed: `DIFF TOTAL ... 31120 field comparisons ... 0 discrepancies`, and `real_fixtures` 2 passed | **Agreed, fixed.** A test-support helper `checked_players` requires the reported slots to equal `0..active_count` in order, with nothing missing, repeated, reordered or extra. It runs before any comparison or total, in the differential and in the inspect real-file test, which now also checks every record hash | [#8](https://github.com/AMattHT/MaddenTool/issues/8) | `84a4a3e9` |

Both issues are open and labelled `bug`, with status **Fixed in PR, awaiting merge**: draft [PR #9](https://github.com/AMattHT/MaddenTool/pull/9), which carries `Fixes #7` and `Fixes #8`. Each body holds the reproduction and a copyable Opus 5.5 prompt.

The missing TEMP1 to TEMP3 files are a known acceptance gate, not a bug, so no issue was filed for them.

## Red, then green

The six regressions run on every `cargo test`:

- `real_fixtures.rs`: `required_mode_fails_when_fixtures_are_not_configured` and `optional_mode_skips_visibly_when_not_configured`.
- `differential.rs`: those two, plus `required_mode_fails_when_the_legacy_parser_is_not_configured` and `slot_check_rejects_reports_that_omit_or_repeat_players`.

Each required/optional case re-runs the real opt-in test in a child copy of the test binary, with the setup variables removed and the case's variables set on that child only.

| Stage | Result |
|---|---|
| Regressions with the support code as at `6911c423`, and `checked_players` stubbed to that commit's behaviour (accepts the report as given) | 4 failed:<br>• both required-without-fixtures cases: the child passed;<br>• the missing-oracle case: the child stopped at `CAREERDRAFT-TEMP1 is missing`, not at the legacy setting;<br>• the slot check accepted all 7 corrupted reports.<br>Both optional-skip cases passed, which is the behaviour to keep |
| After the fix | All pass |
| Direct runs, required mode | Every opt-in test fails with `MADDENTOOL_FIXTURES is not set, but MADDENTOOL_REQUIRE_FIXTURES=1 requires ...`. With a template-only directory and no oracle: `MADDENTOOL_LEGACY_MDG is not set, but ... requires the legacy differential test to run`. Optional mode still prints SKIPPED and passes |
| Repeated-slot adapter, after the fix | Differential and inspect real-file tests exit 101: `reported 389 players for 389 active slots ... missing 388 [1, 2, ...]; repeated [0]`. With the genuine binary restored, both pass with unchanged totals |

## Scope for round 2

Changed since `6911c423`:

- `crates/mt-draft-format/tests/support/fixtures.rs` (`required()`, `fixture_dir`);
- `tests/support/child.rs` (new);
- `tests/support/report.rs` (new);
- `tests/support/mod.rs`;
- `tests/differential.rs` (legacy check order, slot check, four regressions);
- `tests/real_fixtures.rs` (slot and record-hash check, two regressions);
- the crate README (required mode, test table, the RL4 gate);
- this ledger and `DELIVERY.md`.

No production code (`src/`) changed. RL4 TEMP1 to TEMP3 and the three-OS CI remain separate, unresolved acceptance gates.
