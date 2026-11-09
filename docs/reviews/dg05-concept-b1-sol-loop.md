# DG05-CONCEPT-B1 review ledger (sol-loop)

This ledger covers the design package `docs/design/concepts/dg05-concept-b1/`: direction A in a light theme, at 1024×768 in light and dark, at 200% zoom, and a keyboard-focus state concept.

- **Branch:** `t3/scouting-desk-responsive-themes`.
- **Base:** `main` at `0025e7feecc8d01aaac93888c6071ceb5722189c` (includes PR #12).
- **Roles:**
  - Claude Opus 5.5 (High) built the package and wrote this record.
  - The parent coordinates the reviews. As GPT-6 Astra (xhigh) in Codex, it applied the small round 1 corrections.
  - GPT-6.1 Sol (High) was the sole reviewer in both rounds.
- **Status:** published as a ready pull request after the round 2 approval.

**Approval status: approved by Sol at `6af344dc3427bbe1018dc199f1b39fc95c6eb144`.** This is a GPT-6.1 Sol review, not a GitHub human approval.

- Round 1 audited the whole package and asked for changes.
- Round 2 was a scoped delta review of `45845ea..6af344d` (S1, S2 and S3 and the artifacts they changed), not a repeat audit of the whole package. It approved with no new findings.
- The only later change is a documentation-only commit, which updates the status wording in this ledger, `DECISIONS.md` and the brief. The reviewed scripts, PNGs, `checks.json` and `comparison.html` are byte-identical to `6af344d`.

## Rounds

| Round | Reviewer | Head reviewed | Verdict |
|---|---|---|---|
| 1 | GPT-6.1 Sol, High (full package) | `45845ea461627a55dff76692f1978d06215852e1` | **CHANGES:** S1 and S2 (should), S3 (nit). No blocker |
| 2 | GPT-6.1 Sol, High (delta, `45845ea..6af344d`) | `6af344dc3427bbe1018dc199f1b39fc95c6eb144` | **APPROVE.** S1, S2 and S3 resolved; no new findings |

**Round 1** verified the complete B1 package: two identical rebuilds, 44-row sample parity, twelve rejected statistic fields, the original icons, contrast, row counts and publication content.

**Round 2** independently verified:

- **S1:** the 2600×650 focus composite carries a legible in-image disclaimer. Two rebuilds are identical to the committed PNG and its SHA-256. The comparison embeds it exactly. Its first 580 rows and the five board PNGs are unchanged.
- **S2:** in Chrome at DPR 1, with Fit unchecked, all six images display at their native dimensions. With Fit on, they stay within the page width, and the zoom frame fits at 960×540. The fit/native/fit round trip is correct, and both stored preference values initialize correctly. Two comparison rebuilds are identical.
- **S3:** the branch labels are correct, and the issue records are accurate.
- **Publication checks:**
  - the six frames and five icons are embedded exactly;
  - the PNG metadata is empty;
  - only the focus-detail hash changed in `checks.json`;
  - Python compiles, and the privacy scan and diff checks pass.

Neither round claimed any Rust, product, keyboard, assistive-technology or game testing. Usage was not exposed.

## Findings

| ID | Severity | Confirmed cause | Disposition and record |
|---|---|---|---|
| S1 | should | The focus crop removed every sample marker and disclaimer, so the standalone PNG could present sample values as factual statistics | **Fixed on this branch; verified by Sol in round 2.** [#13](https://github.com/AMattHT/MaddenTool/issues/13), fix `3e45abd60b61418481452574a759ec2a870c2a41`. The issue body holds the Opus verification prompt. It stays open until the PR merges |
| S2 | should | Removing `.fit` left the zoom PNG's explicit half-size dimensions in place | **Fixed on this branch; verified by Sol in round 2.** [#14](https://github.com/AMattHT/MaddenTool/issues/14), fix `3e45abd60b61418481452574a759ec2a870c2a41`. The issue body holds the Opus verification prompt. It stays open until the PR merges |
| S3 | nit | The decision record and ledger used the checkout suffix as the Git branch name | **Corrected** to `t3/scouting-desk-responsive-themes` in the fix and in this ledger. A documentation typo, so no separate bug issue |

## Round 1 correction evidence

The parent independently inspected the focus composite and its source before editing.

- **S1.** The composite's values were already samples; only their labels were missing.
  - The new in-image disclaimer identifies sample wAV and evidence, and illustrative OVR and dev tiers.
  - The composite is 2600×650. All original crop and caption pixels are unchanged in the first 580 rows, and the disclaimer occupies the added space below them.
  - No history rewrite was needed: neither real statistics nor private account values were introduced.
- **S2.** A scratch headless-Chrome check at DPR 1 loaded the comparison, turned Fit off, and compared the natural and displayed dimensions of all six images.
  - Before the fix, it failed for the zoom PNG only: natural 1920×1080, displayed 960×540.
  - After the fix, all six match, and a fit/native/fit round trip restores the original fitted layout.
  - Both saved preference values initialize the checkbox and the image dimensions correctly.
  - The handler uses the native `data-w`/`data-h` when unchecked and the stored fitted dimensions when checked.

Checks after the corrections:

- Two independent frame builds agree on all six PNGs and `checks.json`, and the comparison rebuild is byte-identical.
- All five board PNGs are byte-identical to `45845ea`, and their measured values are unchanged. Only the focus-detail hash changes in `checks.json`.
- The comparison embeds each of the six exact frame PNGs once, plus the five unchanged original icons. PNG metadata is empty.
- The disclaimer was inspected visually. The existing scan and an additional comparison scan found no private paths or email addresses. Python syntax and `git diff --check` pass.
- Sample parity still covers 44 board rows, with zero mismatches and an equal career sample.
- No production code, Rust tests, keyboard behaviour, game files or other concept package changed. Browser reproductions and build scratch stay outside Git.

Issues #13 and #14 stay open until the fixes reach `main`, and their original evidence and prompts are kept. The existing #11 has a different cause (real-statistic inputs in concept A) and was not reopened.

## Limits that remain

The package is static design frames. It does not show or test working UI behaviour, keyboard handling, screen-reader output or game import. Those wait for product work. Default lens, Career cap, 389 vs 402, seed visibility, window arrangement and the product name stay open.
