# DG05-CONCEPT-B1 review ledger (sol-loop)

This ledger covers the design package `docs/design/concepts/dg05-concept-b1/`: direction A in a light theme, at 1024×768 in light and dark, at 200% zoom, and a keyboard-focus state concept.

- **Branch:** `t3/scouting-desk-responsive-themes`.
- **Base:** `main` at `0025e7feecc8d01aaac93888c6071ceb5722189c` (includes PR #12).
- **Roles:** Claude Opus 5.5 (High) built the package. The parent coordinates reviews and applied the small round 1 corrections as GPT-6 Astra (xhigh) in Codex. GPT-6.1 Sol (High) is the sole reviewer each round.
- **Status:** **awaiting round 2 delta review**. Both should findings are fixed locally, with verification below. The branch is not pushed, and no PR is open.

## Rounds

| Round | Reviewer | Head reviewed | Verdict |
|---|---|---|---|
| 1 | GPT-6.1 Sol, High | `45845ea461627a55dff76692f1978d06215852e1` | CHANGES: S1 and S2 should; S3 nit |
| 2 | GPT-6.1 Sol, High | (pending) | (pending) |

Round 1 verified the complete B1 package, including two identical rebuilds, 44-row sample parity, twelve rejected statistic fields, original icons, contrast, row counts and publication content. No product interaction or assistive-technology test was claimed. Usage was not exposed.

## Findings

| ID | Severity | Confirmed cause | Disposition and record |
|---|---|---|---|
| S1 | should | The focus crop removed every sample marker and disclaimer. The standalone PNG could present sample values as factual statistics. | Fixed on branch, PR pending. [#13](https://github.com/AMattHT/MaddenTool/issues/13), fix `3e45abd60b61418481452574a759ec2a870c2a41`; the issue body contains the Opus verification prompt. Pending Sol delta verification. |
| S2 | should | Removing `.fit` left the zoom PNG's explicit half-size dimensions in place. | Fixed on branch, PR pending. [#14](https://github.com/AMattHT/MaddenTool/issues/14), fix `3e45abd60b61418481452574a759ec2a870c2a41`; the issue body contains the Opus verification prompt. Pending Sol delta verification. |
| S3 | nit | The decision record and ledger used the checkout suffix as the Git branch name. | Corrected to `t3/scouting-desk-responsive-themes` in the fix and this ledger update. Documentation typo; no separate bug issue. |

## Round 1 correction evidence

The parent independently inspected the focus composite and source before editing. Its values were already samples; only their labels were missing. The new in-image disclaimer identifies sample wAV/evidence and illustrative OVR/dev tiers. The composite is 2600×650, with all original crop and caption pixels unchanged in the first 580 rows. The disclaimer occupies the added lower space. No history rewrite was needed: neither real statistics nor private account values were introduced.

For S2, a scratch headless-Chrome check at DPR 1 loaded the comparison, toggled Fit off, and compared all six natural/displayed dimensions. Before the fix it failed for the zoom PNG only: natural 1920×1080, displayed 960×540. After the fix, all six match. A fit/native/fit round trip restores the original fit layout. Both saved preference values initialize the checkbox and image dimensions correctly. The handler uses native `data-w`/`data-h` when unchecked and the stored fitted dimensions when checked.

Checks after the corrections:

- Two independent frame builds agree on all six PNGs and `checks.json`; the comparison rebuild is byte-identical.
- All five board PNGs are byte-identical to `45845ea`; their measured values are unchanged. Only the focus-detail hash changes in `checks.json`.
- The comparison embeds each of the six exact frame PNGs once, plus the five unchanged original icons. PNG metadata is empty.
- The disclaimer was visually inspected. The existing scan and an additional comparison scan found no private paths or email addresses; Python syntax and `git diff --check` pass.
- Sample parity still covers 44 board rows with zero mismatches and an equal career sample.
- No production code, Rust tests, keyboard behavior, game files or other concept package changed. Browser reproductions and build scratch remain outside Git.

Issues #13 and #14 remain open until the fixes reach `main`. Their initial evidence is retained. The existing #11 addresses a different cause, real-statistic inputs in concept A, and was not reopened.

Review scope for round 2: `45845ea..HEAD`, S1/S2/S3 and changed artifacts only. Do not repeat the full audit of the five unchanged board frames. No push or PR before approval.
