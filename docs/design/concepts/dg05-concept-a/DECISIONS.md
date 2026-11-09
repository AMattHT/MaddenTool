# DG05-CONCEPT-A: draft board directions, decision record

Written 2026-11-02 by Claude Opus 5.5 (High), the design agent. This is a concept package for choosing a style. It does not implement the app, and it does not show a working generator or export. It follows [the Higgsfield brief](../../draft-generator-higgsfield-brief.md) and [the wAV contract](../../../research/wav-to-ovr-contract.md). Open `comparison.html` (self-contained, no server) to see everything side by side. The review history is in [`docs/reviews/dg05-concept-a-sol-loop.md`](../../../reviews/dg05-concept-a-sol-loop.md).

- **Base:** `origin/main` at `2f1da5728b1d058f5273020ee6515941df509899`, which includes DG01.
- **Branch:** `t3/draft-board-higgsfield-concepts`.

## Decision: direction A is selected

**The user chose A, Scouting Desk:** a table with a docked player inspector. Both light and dark themes are wanted; this package draws A dark, and A's light theme comes next.

B, Class Ledger, stays documented as the explored alternative. Two of its ideas are available to A without being settled:

- its ledger palette, as a starting point for A's light theme;
- its round bands, as a possible "Group by round" view.

Choosing A settles the board structure only. The generator preferences listed under [open preferences](#assumptions-for-open-preferences-all-reversible) remain open.

| | A · Scouting Desk (selected) | B · Class Ledger (explored) |
|---|---|---|
| Appearance drawn | Dark (light still wanted) | Light |
| Hierarchy | Player first: table, then a docked 392px inspector. Settings are one chip line; counts sit in the status bar | Class first: a permanent 262px "Class recipe" column, a table grouped by round, and inspection inline under the row |
| Rows at 1440×900 | 33: 2 review rows and 31 board rows, inspector open | 28 board rows with the inline band open (about 37 when it is closed) |
| Scales to the DG05b editor | Yes; the inspector becomes the profile | No; a 150px band cannot hold seven rating groups, persona DNA and history |
| 800×600 | Same structure: fewer columns, a 300px inspector, 19 visible rows (17 board and 2 review) | The recipe column has to collapse, so the layout changes |

## Outputs

| File | Origin | Notes |
|---|---|---|
| `higgsfield/A-scouting-desk-raw-1440x900.png` | **Higgsfield** | Raw job output, resized with a caption strip. Not authoritative |
| `higgsfield/B-class-ledger-raw-1440x900.png` | **Higgsfield** | Same |
| `assembled/A-scouting-desk-dark-1440x900.png` | **Assembled** | Direction A, spoilers on |
| `assembled/B-class-ledger-light-1440x900.png` | **Assembled** | Direction B, spoilers on |
| `assembled/A-scouting-desk-spoilers-off-1440x900.png` | **Assembled** | Direction A, spoilers off |
| `assembled/A-scouting-desk-dark-800x600.png` | **Assembled** | Direction A at 800×600 |
| `comparison.html` | **Assembled** | Self-contained page: all images inlined as data URIs, with captions, icon legend, states and tradeoffs |
| `source/higgsfield-prompts.json` | Record | Exact prompts and parameters submitted |
| `source/build_frames.py`, `extract_rows.py`, `label_raw.py`, `build_comparison.py` | Scripts | How everything was assembled |

"Assembled" means a static HTML/CSS layout I wrote from the two directions, screenshotted by headless Chrome (`--headless=new`, device scale 1) at the exact frame size. Rebuilding produced byte-identical PNGs. None of these frames is app code.

`build_comparison.py` can also write a small inline presentation for an HTML preview host. Because it references images by absolute local path, it is written outside the repository and never committed.

## Higgsfield record

Capability was discovered with `models_explore`. `gpt_image_2_5` (OpenAI GPT Image 2.5, tagged `text-rendering`) has no 16:10 ratio, so 27:16 was the closest. Both jobs were submitted in one `generate_image_batch` call, followed with `jobs_wait` to `completed`, and shown with `show_generation_by_ids`. Nothing was resubmitted. No reference media was uploaded, so the EA icons were not sent to Higgsfield.

| Direction | Job ID | Model / params | Output |
|---|---|---|---|
| A | `13e40b61-90fa-4047-af36-9ea70d9921b5` | `gpt_image_2_5`, variant `flare`, quality `high`, resolution `2k`, aspect `27:16` | 2592×1536 PNG, sha256 `7acbc18e668f2d935916bad756034599056391aadbd9cab145306def1ae00542`, created 2026-11-02 17:54:20 UTC |
| B | `243530a7-97c1-4272-a970-d818f2647e6c` | same | 2592×1536 PNG, sha256 `349fada15c7116c48b1d2ba97a20292264bb8d904d6058eb49975eb1e42b549a`, created 2026-11-02 17:54:20 UTC |

The result URLs contain the account's user ID, so they are not recorded here. The job IDs retrieve the outputs.

- **Cost:** a `get_cost` preflight returned an estimate of 2.75 credits per image.
- **Earlier jobs:** the parent's jobs `f3ace4f9-…` and `e6522c5c-…` and their files under the parent's `artifacts/concepts/` were read as references only. None of their images appears here.
- **Post-processing of the raw images:** Lanczos resize to 1440×853 (aspect kept), plus a 47px caption strip to reach 1440×900 (`label_raw.py`). Nothing inside them was edited.
- **What the model got wrong,** which is why the raw images are not content:
  - **A:** invented tiers (Elite/High/Average/Developing/Limited, with A–E letters); decimal wAV; "Source: Internal model v1.0"; a 2025 "Generated" timestamp; "Estimated: 2nd round".
  - **B:** invented tiers (Star/Impact/Solid/Project/Late); counts drawn as "/ 500" bars; invented reasons ("+2 scheme fit", "Higher grade from pro day / combine"); invented timestamps.
  - **Both:** no real icons.
- **What was kept from them:** density, the proportion of A's docked inspector, and the structure of B's recipe column, round bands and three-part inline band.

## Development-trait icons

Bytes were read from the read-only legacy clone `AMattHT/Madden-Draft-Generator` at `9a4ac69f46846d33f8d084948a85a47e4267f01b`, `server/data/dev-icons/`. They were added in `f94ce3bc` (2026-08-30). Every hash matched the brief before use, and the build scripts assert them again.

| Displayed | File | SHA-256 | Bytes |
|---|---|---|---|
| Normal | `slow.png` | `bbd4fd7a9492b74859cc88aff4c32704af88b2c7d70a2a193fcdadabcb910a87` | 11,247 |
| Star | `normal.png` | `d66cddd97512c5ec224c56a7f44f5db879c7719479e0f33a60a0c7f8a0bd5d34` | 10,681 |
| Superstar | `quick.png` | `3f0cab1c56ac6e777f68491547476a92f28cb17e772e1e289499141faecec428` | 13,948 |
| X-Factor | `superstar.png` | `c484dc2c92981295046962fb225621e18496a232451f9acfa8cef72afd3ad038` | 12,326 |
| Unscouted | `hidden.png` | `07abd50c1bb48d2b925d41617ad5d1f0d2b47b1d51134e72c3248631eeb15b2c` | 12,690 |

- **Mapping:** taken from `web/src/components/ui.tsx` lines 231 and 264 (`DEV_ART_NAME = ['slow','normal','quick','superstar']`, `DEV_LABELS = ['Normal','Star','Superstar','X-Factor']`), never from file names.
- **How they appear:** the frames inline the original bytes as data URIs. The browser only scales them, to 18, 22, 26 or 40px. They are never redrawn or recoloured.
- **Labels:** each has visible text, plus `alt`/`title` set to "<Tier> development", or "Hidden — turn on Spoilers to reveal" for `hidden.png`.
- **Rights:** this is EA artwork, used at the user's explicit request. The five standalone files are **not** committed. Their bytes are inside the four assembled PNGs and inlined in `comparison.html`. The UI goal still decides whether the app ships them or loads them from a user folder.

## Data shown

The package follows the publication contract in `crates/mt-store/README.md`: it never redistributes PFR-derived values. Real data is limited to identity and draft facts. Every statistic is a labelled sample.

**Real identity data.** It comes from a local import on this laptop:

- `mt-data import --format legacy-player-lookup --as-of 2026-09-15`, with the DG01 binary built from `2f1da57`.
- File: the legacy `ALL_PLAYER_LOOKUP.csv`, sha256 `b06f3e261c0eaffc34905c101bae84a30c89adfba1d5d1ee2625477dced70d7e`.
- Database in a temporary folder, never committed. The as-of label is the one DG01 recorded for this file.
- `mt-data query --year 1964 --json` returned 334 people, which is the "Active 334" shown.
- `mt-data conflicts` gave the 1964 unsettled pairs: Jack Adams / Jake Adams (Virginia Tech, alias unconfirmed) and Bob Young × 2 (college conflict).
- `player_merge` confirmed the `dual_draft_name_college` rule for Bob Brown and Pat Crain.

**What is real:**

- names, draft selections, colleges and source position labels (LT, SE, HB…; mapping to Madden positions is DG04a);
- the Bob Brown correction and its evidence, which are already in `crates/mt-store/data/corrections.json`;
- legacy portrait IDs.

`extract_rows.py` copies only these identity fields. It never copies wAV, evidence, career span or honours.

**Sample inputs,** marked ◇ in column headers, chips, inspector rows and banners:

- **wAV, by board position:** `sample_wav` in `build_frames.py`. Rows 7 and 24 are a sample Missing state; row 19 is a sample Observed 0.
- **Evidence state:** follows the sample wAV.
- **Inspector career record:** `SAMPLE_CAREER`.
- **Independence from real records:** every sample is chosen independently of the real record. A check against the local factual excerpt found no visible row whose sample wAV number equals its real value. Row 24's sample Missing happens to coincide with the source also having no value; an absent value carries no statistic.

**Illustrative values:** OVR, dev tier, archetype and group percentile. They are computed only from the sample inputs by a hand-shaped function. B uses a different blend so that its example settings (Realistic, hindsight 0.35) do not repeat A's numbers. They are not calibrated results.

**Example settings,** labelled "example": the seed `7F3A-21C9`, the lens and hindsight choices, and class strength "Auto".

**Committed data.** No dataset is committed. The identity-only excerpt is rebuilt from a local import by `extract_rows.py` and stays out of Git. An earlier unpublished draft showed real statistics; it was replaced before publication, and a factual copy is kept only outside the repository for the user's reference.

## Requirements satisfied

- Two directions at 1440×900 that differ in hierarchy and layout, not just colour, covering light and dark.
- One spoilers-off frame and one 800×600 frame for the selected direction.
- 30 or more readable rows: A shows 33. B shows 28 with inspection open, recorded as B's cost. At 800×600, A shows 19.
- Actual icons, verified against the hashes and mapped from `ui.tsx`, each with text and an accessible name.
- **Spoilers off is a distinct state.** OVR, wAV and evidence columns are absent rather than covered. Rating sorts and the evidence filter are removed. The career record, the rating explanation and the correction citations (which mention outcomes) are withheld. The rendered HTML was scanned: no tier names, OVR, wAV, evidence or career text, and the only embedded image is `hidden.png`.
- The three counts are always separate. 402 carries "in-game import of 402 not yet verified", and validation reads "not run · concept".
- **Bob Brown:**
  - shown once, as NFL r1 p2 · AFL r1 p4;
  - the AFL correction (from Ark-Pine Bluff, DT to Nebraska, T) is shown with its ID and 4 citations;
  - Robert Brown (NFL r13 p169) is named as a different person with his own row.
- The unsettled identities are listed, not merged.
- **wAV states,** all shown as labelled samples:
  - Missing appears as "—" with the word Missing, and the rating is marked est.
  - Observed 0 reads "Observed 0 · ended".
  - Nothing says "in progress".
- The career record is labelled hindsight and sample. "Why this rating" lists only lens, hindsight, wAV, evidence, comparison group, percentile and estimated, with a banner that the inputs are samples and the model is not built.
- Seed, lens, hindsight, offline status and as-of are visible.
- Nothing from the brief's exclusions appears: no logos, team marks, photos, mobile or franchise screens, or invented tiers.

## Assumptions for open preferences (all reversible)

| Open question | What the frames show | Status |
|---|---|---|
| Board direction | A, Scouting Desk | **Selected by the user** |
| Default lens (contract Q1) | A: Career; B: Realistic | Both marked example; **undecided** |
| Career cap (Q2) | Not shown | Undecided |
| Export size, 389 or 402 | Limit 402, plus the unverified-in-game note | Undecided; DG03 gate stands |
| Seed visible by default | Visible in both | Undecided |
| Window arrangement | Single window with panels | Undecided |
| Product name | "MaddenTool" working name | Undecided |
| Board order for combined AFL/NFL years | Round, then NFL before AFL, then pick | Provisional; DG04a |
| Where unsettled identities appear | A pinned "Needs review" group at the top of the board | Design proposal |

## Findings for later goals

- **Data gap:** the legacy file has no 1964 NFL rows for picks 5 or 6. Pete Beathard and Carl Eller therefore appear only with AFL selections. DG04a should check this before showing such players as single-league.
- **Field meaning:** `wav_evidence.completed_seasons` reads 62 for 1964 players. It counts seasons elapsed since the draft, not seasons played, and the UI must never label it "seasons played".
- **Icon contrast:** `normal.png` (Star) is a dark silver that reads weakly at 18px on the dark theme. Its text label carries the meaning. DG05 should test a subtle backing chip.
- **No completed season:** this state cannot occur in a 1964 class, so no frame shows it. It needs a 2026-class frame.

## Remaining design work, for A

1. A's light theme, A at 1024×768, and A at 200% zoom.
2. Focus, hover and keyboard states.
3. Board states: empty filter result, overflow panel, a 2026 class showing "No completed season".
4. Column chooser.
5. Flow for resolving an unsettled identity.
6. The other brief screens:
   - data and sources (first run, offline degraded);
   - new class setup;
   - generation job;
   - full player profile and editor;
   - Class Studio;
   - open existing class;
   - validate and export (success and blocked).
7. An accessibility pass (contrast and screen-reader names).
