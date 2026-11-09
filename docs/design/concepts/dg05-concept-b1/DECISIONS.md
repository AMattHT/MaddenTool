# DG05-CONCEPT-B1: Scouting Desk in light, compact and zoomed, decision record

Written 2026-11-02 by Claude Opus 5.5 (High), the design agent. This package extends the user's selected direction, [A · Scouting Desk](../dg05-concept-a/DECISIONS.md), into a light appearance, a 1024×768 layout in light and dark, a 200% zoom layout, and one keyboard-focus state. **These are static design frames.** They are not the app: nothing generates, imports, validates or exports, and nothing is interactive. Open `comparison.html` (self-contained, no server) to see everything together.

- **Base:** `origin/main` at `0025e7feecc8d01aaac93888c6071ceb5722189c` (includes PR #12).
- **Branch:** `t3/scouting-desk-responsive-themes`.
- **Not decided here:** the board structure was already settled by the user (A). Default lens, Career cap, 389 vs 402, seed visibility, window arrangement and the product name stay open. The frames show labelled example settings.

## Outputs

| File | What it shows |
|---|---|
| `assembled/A-light-1440x900.png` | A in light. All columns and inspector sections from the dark frame. Bob Brown selected and focused |
| `assembled/A-light-1024x768.png` | A at 1024×768, light |
| `assembled/A-dark-1024x768.png` | A at 1024×768, dark |
| `assembled/A-light-zoom200-960x540css.png` | A at 200% zoom: a 1920×1080 physical window, so the CSS viewport is 960×540. The PNG is 1920×1080 physical pixels |
| `assembled/A-light-focus-1440x900.png` | Focus on row 5 while the selection stays on Bob Brown (a state concept) |
| `assembled/A-focus-detail.png` | Crops of the focus states in colour and in luminance greyscale, enlarged 2× |
| `assembled/checks.json` | Measurements taken from the rendered frames (see [Checks](#checks)) |
| `comparison.html` | Every PNG inlined, with captions, the column priority, the tab order and the measured tables |
| `source/build_b1.py`, `source/build_b1_comparison.py` | Build scripts |
| [`next-task-dg05-concept-b2.md`](next-task-dg05-concept-b2.md) | Prompt for the next package |

Every frame is "assembled": a fixed-size static HTML/CSS layout that I wrote, then screenshotted in headless Chrome 156 (`--headless=new`) at an exact size and device scale. **No new Higgsfield generation was made.** The style comes from concept A's assembled dark frame, B's ledger palette for light, and the two existing Higgsfield jobs recorded in concept A (`13e40b61-…`, `243530a7-…`). No paid images were rerun.

## Decisions

### Light appearance

- **Palette.** B's ledger palette: paper `#F7F5F0`, ink `#1B2433` and ledger green `#2F6B4F` as the accent. A's gold works in dark, but on paper it fails as text and is easily confused with the amber warning colour. The two themes therefore have different accent hues, and the focus colour is the same blue family in both (see below). Whether to unify the accent is open.
- **Tokens raised to reach AA.** I raised two of B's tokens: the faint text colour (`#8A909B` to `#646B77`; the old value was 2.95:1 on paper) and the control outline (`#C9C4B9` to `#8C8578`).
- **Development icons.** In light they sit directly on the row, with no backing.

### Dark at 1024

- **Palette.** A's dark palette, with two tokens raised: faint text (`#6E7782` to `#8A939E`; the old value was 4.06:1 on the board and 3.84:1 in the inspector) and the control outline (`#363C44` to `#68707A`; the old value was 1.66:1).
- **Not changed:** concept A's merged dark 1440 frame is not repainted. Its old token values are a known gap, listed in [findings](#findings-for-later).
- **Icon backing chip.** Development icons sit on a neutral chip (`#A7AFB9`, 2px larger than the icon), and the icon bytes are unchanged. Several of the original icons are dark artwork. Without the chip, only 14–34% of an icon's opaque pixels reach 3:1 against the dark row; on the chip, 62–85% do. The text label still carries the tier.

### Column and inspector priority

| Width | Board columns | Moved out, and where to | Inspector | Toolbar |
|---|---|---|---|---|
| 1440×900 | # · Selections · Player · Pos · College · OVR ◇ · Dev ◇ · wAV ◇ · Evidence ◇ · Portrait · Flags | — | 392px, all sections open | Every filter is its own control |
| 1024×768 | # · Selections · Player · Pos · OVR ◇ · Dev ◇ · wAV ◇ · Evidence ◇ (short) · Flags (glyph) | College and Portrait, to the inspector header and the Columns menu ("2 hidden") | 296px. Selections and "Why this rating" open; Career record and Provenance collapse to one-line disclosures | Position, League and Evidence fold into one Filters button |
| 960×540 CSS (200%) | Selections · Player · Pos · OVR ◇ · Dev ◇ · wAV ◇ · Evidence ◇ (short) · Flags (glyph) | `#` (the pick is in Selections, and the sort order is shown on its header) | **Compact presentation:** still docked to the right of the board, 268 CSS px wide. The header (name, position, OVR, dev) stays fixed, and the sections become tabs: Selections · Rating ◇ · Career ◇ · Source | As at 1024. Validation status leaves the status bar; it is reached from Validate & export |

These never drop at any size:

- Player and Pos;
- OVR, and Dev with its icon **and** its label;
- wAV and the evidence text;
- the three separate counts (active, export limit 402 with its unverified-in-game note, file capacity 455);
- the Spoilers switch, Generate… and Validate & export….

Bob Brown's dual selection ("NFL r1 p2 · AFL r1 p4") and the longest selection in the excerpt (Pat Crain's "NFL r2 p23 · AFL r15 p118") fit without truncation at every size.

**Spoilers off,** not drawn here, follows concept A's rule at every size:

- the OVR, Dev, wAV and Evidence columns are not rendered; one Scouting column shows `hidden.png`;
- rating sorts and the evidence filter are removed;
- at 1024, the Career record and "Why this rating" sections are not rendered;
- at 200%, the Rating and Career tabs are absent rather than disabled;
- no hidden value appears in a tooltip, an accessible name or a sort key.

### Focus and selection

- **Selection:** a row tint, a 4px accent bar on the left edge and a bold player name. The inspector shows that player (`aria-selected`).
- **Focus:** a 2px ring in a blue focus colour (light `#1D5FD1`, dark `#7DB8FF`) drawn around the whole row or control.
- **Not colour alone:** selection is an edge bar and bold text, and focus is an enclosing ring. The greyscale crops in `A-focus-detail.png` show that both survive with no hue.
- **Proposed tab sequence:** one tab stop per region, with arrow keys inside a region. F6 cycles the regions.
  1. A "Skip to board" link.
  2. The view rail.
  3. The class context chips. Enter opens a chip's popover.
  4. The toolbar: Search → Filters → Sort → Columns → Spoilers → Generate… → Validate & export….
  5. The board grid, with roving focus:
     - ↑/↓ move focus and selection together;
     - Ctrl+↑/↓ move focus only;
     - Space selects the focused row;
     - Enter moves into the inspector.
  6. The inspector: tabs or disclosure headers, then links such as "View evidence". Esc returns to the selected row.
  7. The status-bar links (Overflow, unsettled IDs).
- **Status: a state concept.** No keyboard or screen-reader behaviour was exercised. The roles in the frame HTML (`grid`, `switch`, `tablist`, `aria-selected`) are annotations only.

## Checks

`build_b1.py` measures each frame's DOM inside headless Chrome, using a copy of the page with a measuring script appended; the screenshots are taken without that script. It writes the results to `assembled/checks.json`.

| Frame | CSS viewport · DPR | Fully visible rows | Clipped visible cells | Inspector sections fully shown | Lowest text contrast |
|---|---|---|---|---|---|
| Light 1440 | 1440×900 · 1 | 32: 30 board + 2 review, plus row 31 partly visible | 0 | 5/5 | 4.80:1 |
| Light 1440, focus | 1440×900 · 1 | 32: 30 + 2 | 0 | 5/5 | 4.80:1 |
| Light 1024 | 1024×768 · 1 | 26: 24 + 2 | 0 | 5/5 | 4.80:1 |
| Dark 1024 | 1024×768 · 1 | 26: 24 + 2 | 0 | 5/5 | 5.96:1 |
| 200% zoom | 960×540 · 2 | 16: 14 + 2 | 0 | 3/3 (header, tabs, Selections panel) | 4.84:1 |

How the columns were measured:

- **Fully visible rows** counts only rows whose whole height is inside the board.
  - The light 1440 frame has the same row geometry as concept A's dark frame, so that frame's row 31 is also partly visible, cut by about 3px.
  - Concept A's "31 board rows" counted that row; by this stricter rule it is 30.
- **Clipped cells** means `scrollWidth > clientWidth` on every visible header and cell.
  - It also covers the context, toolbar and status bars: nothing overflows them.
  - Below the fold, the only clipped cell in any frame is "Texas A&M-Kingsville" in the 1440 College column, a width inherited from A.
- **Text contrast** walks every visible text element against its painted background. No pair is below 4.5:1 in any frame.
- **Smallest text** is 11px (the "est." marker and flags); the body text is 12.5px.

Token contrast (light · dark):

| Pair | Light | Dark |
|---|---|---|
| Body text on board | 14.31:1 | 15.31:1 |
| Body text on the selected row | 13.11:1 | 12.79:1 |
| Lowest text pair (faint on the selected row) | 4.51:1 | 4.95:1 |
| Primary button label | 6.29:1 | 8.96:1 |
| Focus ring against the selected row (lowest ring pair) | 4.89:1 | 7.47:1 |
| Selection bar against its tint | 5.29:1 | 7.46:1 |
| Control outline against the board | 3.36:1 | 3.68:1 |

Other checks:

- **Icons:** the five SHA-256 values match the brief. The script reads the bytes from the legacy clone at `9a4ac69f`, asserts each hash, and inlines the bytes unchanged. The mapping comes from `ui.tsx` and is the same as in concept A.
- **Sample-only boundary:**
  - The renderer rejects any stat field in its input.
  - wAV, evidence and the career record are the same ◇ sample inputs as concept A, and OVR, tier and percentile are the same illustrative function. A parity check imports concept A's script and compares all 44 board rows: 0 mismatches.
  - Every frame carries the ◇ markers and the "sample inputs" banner or chip.
  - Missing ("—" plus "Missing", marked est.) and Observed 0 ("Obs 0 · ended") stay distinct samples (rows 7, 24 and 19).
- **Public-content scan:** an automated scan of every rendered frame document, `comparison.html` and `checks.json` found no machine path, user name, e-mail address, `uid` or career field.
- **Deterministic rebuild:** two clean builds produced byte-identical PNGs and an identical `checks.json`, and `comparison.html` rebuilt identically. The concept A frames also rebuilt byte-identically from the same input, which confirms the base.

### Round 1 corrections

The coordinating Codex agent (GPT-6 Astra, xhigh) applied the small review corrections after GPT-6.1 Sol High round 1. The focus-detail composite now carries its own sample-input and illustrative-rating disclaimer; the crop and caption pixels are unchanged, with the label added below them. The comparison's 1:1 toggle now uses each PNG's native dimensions and restores its fitted dimensions when checked again. The branch labels above and in the ledger were corrected.

A scratch browser check at DPR 1 failed before the viewer fix: the zoom PNG was 1920×1080 but displayed at 960×540 with Fit unchecked. After the fix, all six images match their native dimensions, a fit/native/fit round trip restores the fitted layout, and both saved preference values initialize correctly. Two builds agree, all five board PNGs and their measurements are unchanged, and the six embedded frame copies match the PNGs. These checks exercise the comparison viewer only. See [the review ledger](../../../reviews/dg05-concept-b1-sol-loop.md) and issues [#13](https://github.com/AMattHT/MaddenTool/issues/13) and [#14](https://github.com/AMattHT/MaddenTool/issues/14).

### Review

The package was reviewed in two rounds with GPT-6.1 Sol on High.

- **Round 1** audited the full package at `45845ea` and asked for changes: S1 and S2 (should) and S3 (nit).
- **Round 2** was a delta review of `45845ea..6af344d`, covering those fixes and the artifacts they changed.
  - It approved at `6af344dc3427bbe1018dc199f1b39fc95c6eb144` with no new findings.
  - It was not a repeat audit of the five unchanged board frames, and it is not a GitHub human review.

Later commits change only the status wording in the records. Static frames remain static: no keyboard, assistive-technology or game behaviour has been tested.

## Data shown

The data rules are the same as concept A's [Data shown](../dg05-concept-a/DECISIONS.md#data-shown).

- **Real:** names, draft selections, colleges, source positions, portrait IDs and the Bob Brown correction (`1964-afl-r1p4-bob-brown`, 4 citations). Bob Brown appears once, as NFL r1 p2 · AFL r1 p4; Robert Brown is a different person. The Adams pair and Bob Young × 2 are left unmerged.
- **Sample** (◇): wAV, evidence state and the career record.
- **Illustrative:** OVR, dev tier and percentile.
- **Example settings:** the lens, hindsight and seed.
- **Input:** `identity-rows.json` is the identity-only excerpt from concept A's `extract_rows.py`, sha256 `8d7397af75e20f43f84c79623a14c87314afb0a494f93533ca78a0d2156861af`. It is read from a local folder and is not committed.

## Rebuild

From the repository root, with the legacy icon folder and the identity-only excerpt available locally:

```
python -I docs/design/concepts/dg05-concept-b1/source/build_b1.py <dev-icons dir> <work dir> <identity-rows.json>
# copy <work dir>/*.png and checks.json into docs/design/concepts/dg05-concept-b1/assembled/
python -I docs/design/concepts/dg05-concept-b1/source/build_b1_comparison.py <dev-icons dir> docs/design/concepts/dg05-concept-b1 [inline.html outside the repo]
```

- **Requirements:** Python 3.12 with Pillow, and Chrome at the default Windows path (or pass its path as a fourth argument).
- **Fonts:** the frames use Segoe UI Variable. On another OS the fonts differ, so the pixels differ, although the layout rules do not.

## Findings for later

1. Concept A's dark 1440 tokens give faint text 4.06:1 on the board (3.84:1 in the inspector) and control outlines 1.66:1, and its selected row uses a gold outline that reads like focus. A dark 1440 redraw should adopt this package's dark tokens and its focus/selection split. It was left as merged evidence.
2. The accent differs between themes: green in light, gold in dark. One accent, or two by design, is a choice for the user.
3. At 200%, the inspector's tabs hide the rating explanation behind one click. If "Why this rating" must always be visible, the alternative is a narrower board with no Selections column.
4. The College column at 1440 truncates long names such as "Texas A&M-Kingsville" (below the fold). This belongs to the column chooser in B2.

## Remaining design work for A

1. **B2:** board states, namely the empty filter result, the overflow panel, a 2026 class with no completed season, the column chooser and the identity-review flow. See the [B2 prompt](next-task-dg05-concept-b2.md).
2. Spoilers-off drawn at 1024 and at 200%; the rules are fixed above.
3. A dark 1440 redraw with the B1 tokens.
4. The other brief screens (data and sources, setup, generation, profile and editor, Class Studio, open, validate and export).
5. Real keyboard and screen-reader testing, once there is a working UI.
