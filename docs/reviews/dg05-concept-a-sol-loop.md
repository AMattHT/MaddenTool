# DG05-CONCEPT-A review ledger (sol-loop)

This ledger covers the design concept package `docs/design/concepts/dg05-concept-a/`.

- **Branch:** `t3/draft-board-higgsfield-concepts`.
- **Base:** `main` at `2f1da5728b1d058f5273020ee6515941df509899`.
- **Roles:** the parent coordinates Sol reviews. The design agent, Claude Opus 5.5 (High), records findings and dispositions here.
- **Status:** nothing is pushed and no PR exists.

**Approval status: not approved.** Round 1 asked for changes. The fixes below await a Sol delta review before publication.

## User decision

The user selected **direction A, Scouting Desk** (a table with a docked player inspector) and wants both light and dark themes. B, Class Ledger, stays documented as the explored alternative. The selection settles the board structure only; the generator's open preferences stay open. Details are in [`DECISIONS.md`](../design/concepts/dg05-concept-a/DECISIONS.md).

## Rounds

| Round | Reviewer | Head reviewed | Verdict |
|---|---|---|---|
| 1 | GPT-6.1 Sol, **High** (sole reviewer) | `7e7021859f6f9c0506badfbce68660aeefd744a6` (unpublished, since replaced) | **CHANGES:** S1 and S2 (should), S3 (nit). No blocker. Sol confirmed the original-icon mapping and hashes, spoiler-off handling, the identity examples, legibility, source and output hashes, and byte-identical rebuilds |

## Findings

All locations are at `7e70218`.

| ID | Severity | Location | Claim | Disposition | Issue |
|---|---|---|---|---|---|
| S1 | should | `DECISIONS.md:47` | The cost note recorded the design-tool account's balance and subscription plan, which is private account state in a public repo | **Agreed, fixed.** The note keeps only the per-image cost estimate. The values are not repeated anywhere public | [#10](https://github.com/AMattHT/MaddenTool/issues/10) |
| S2 | should | `DECISIONS.md:90`, the three spoilers-on PNGs and their copies in `comparison.html` | Real PFR-derived wAV and career statistics for named players, contradicting `crates/mt-store/README.md` ("Nothing in MaddenTool redistributes them.") | **Agreed, fixed.** `extract_rows.py` is identity-only. `build_frames.py` asserts that and renders labelled ◇ sample inputs, with illustrative ratings derived only from them. Frames, `comparison.html` and `DECISIONS.md` are rebuilt. A factual copy is kept outside the repository for the user only | [#11](https://github.com/AMattHT/MaddenTool/issues/11) |
| S3 | nit | `source/build_comparison.py:42`, `comparison.html:55` | The 800×600 caption said 24 rows; the frame shows 17 board rows plus 2 review rows | **Agreed, caption corrected** to "19 visible rows: 17 board + 2 review". The layout is unchanged. A presentation nit, so no issue | — |

The parent independently verified S1 and S2 against the publication contract, the scripts and the notes. No finding was declined.

## History sanitization

`7e70218` was never pushed. The branch was rebuilt as a single commit on the base, so no commit reachable from the branch head contains the account values or the factual images. The old commit is not an ancestor and has no ref; it remains only in the local object store and reflog, for comparison.

The replacement commit hash is recorded on #10 and #11, because a commit cannot record its own hash.

## For the delta reviewer

Changed since `7e70218`, all under `docs/`:

| Path | Change |
|---|---|
| `design/concepts/dg05-concept-a/source/extract_rows.py` | Copies identity and selection fields only |
| `design/concepts/dg05-concept-a/source/build_frames.py` | Asserts identity-only input. Explicit sample inputs (`sample_wav`, `SAMPLE_CAREER`) feed the illustrative OVR, tier and percentile. ◇ sample labels on headers, chips, inspector rows and banners. Robert Brown's statistic removed. Provenance distinguishes identity sources from sample stats |
| `design/concepts/dg05-concept-a/source/build_comparison.py` | Records the user's selection, rewords the copy for sample stats, corrects the 800×600 caption, optionally writes an inline presentation outside the repository, and writes LF line endings |
| `design/concepts/dg05-concept-a/assembled/*.png` | Rebuilt. The spoilers-off frame changed only in its provenance labels |
| `design/concepts/dg05-concept-a/comparison.html` | Rebuilt from the above |
| `design/concepts/dg05-concept-a/DECISIONS.md` | Records the selected direction, removes the account state, and rewrites the data section around identity facts and sample inputs |
| `design/draft-generator-higgsfield-brief.md` | Link line notes the selected direction |
| `reviews/dg05-concept-a-sol-loop.md` | This ledger (new) |

The Higgsfield images, prompts, icon bytes and `label_raw.py` are unchanged.
