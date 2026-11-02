# Draft generator: design brief for Higgsfield concepts

For the parent's Higgsfield concept round. Written 2026-11-02 in DG01. This brief fixes the workflow, screens, states and data. It deliberately does **not** fix the visual style; that is what the concepts are for. Nothing here is implemented yet except the data behind it (DG01).

## Product and user

MaddenTool is a desktop app for Windows, macOS and Linux that builds and edits **Madden 27 draft classes**. Its user is a franchise-mode player who wants real historical classes (for example, "the 1983 draft as if it were today"), custom classes, or edits to a class the game wrote. Output is an M27 draft-class file the game imports. It succeeds the user's Madden-Draft-Generator, which users already know, and is independent of EA, the NFL, MyFranchise and MMC.

The user works in long sessions with dense tables. Desktop windows from 800×600 to 1920×1080 at up to 200% zoom must work. Mouse and keyboard; no touch or mobile layouts.

## Core workflow

1. **Data:** see which datasets are installed, their as-of dates and sources, and whether an update is available. Offline is normal.
2. **Start a class:** choose a source:
   - historical year (1936 to 2026, AFL/NFL merged for 1960 to 1969);
   - decade, all-time, by team or random;
   - Class Studio (hand-built);
   - or open an existing M27 class file.
3. **Choose the ratings approach:**
   - Lens: Realistic (Madden-shaped rookies), Career (how good they became) or Launch Day (EA's release ratings where available).
   - Hindsight slider: 0 is the draft-day board, 1 is career outcome.
   - Class strength, studs, generational #1.
   - Seed, shown, editable and rerollable.
   - Spoilers on or off (blind scouting).
4. **Generate:** a background job with progress and cancel. The app stays usable.
5. **Review the class:** board of up to 402 prospects, with filters, sorts, alternate views, counts and the overflow of dropped players.
6. **Inspect and edit a player:** identity, bio, physicals, ratings, dev trait, archetype, persona DNA, appearance and portrait ID. Every edit can be undone. A "why this rating" panel explains it.
7. **Validate:** blocking errors and warnings, each linked to the player and field.
8. **Export:** choose the folder and name (new file by default, backup when replacing), then write, reopen and verify, and show the result. Spreadsheet export is a secondary path.

## Screens and their states

| Screen | Must show | States to concept |
|---|---|---|
| **Data and sources** | Each dataset: name (nflverse draft picks, legacy lookup...), coverage years, as-of date, record count, attribution ("courtesy of Pro Football Reference via nflverse"), last check | First run with nothing installed; downloading with progress and cancel; up to date; update available; offline with data; offline without data (degraded, nothing fabricated) |
| **New class setup** | Source picker; year picker showing class size per year; league note for 1960-69; lens; hindsight slider with end labels "Draft day" and "Career"; strength, studs and generational; seed; spoilers toggle | Defaults; invalid combination explained; Launch Day lens with partial EA coverage for the year |
| **Generation job** | Phase, progress, cancel, and what is already usable | Running; cancelled; failed with retry |
| **Class board** (main screen) | Rows: pick, round, name, position, college, age, height, weight, archetype, **overall**, **dev trait** (Normal, Star, Superstar, X-Factor), **wAV with its evidence badge**, portrait or likeness status, warnings. Header: active count / 402 export limit / 455 file capacity, seed, lens, hindsight, dataset as-of | Spoilers off: overall, dev trait and wAV hidden, and sorting by them disabled. Spoilers on. Empty filter result. Overflow panel with dropped players and why. 402 rows at once without lag |
| **Player profile and editor** | Identity and bio; physicals; ratings grouped as speed/physical, passing, rushing/ball carrier, receiving, blocking, defense, kicking; dev trait; archetype; 5 persona DNA slots; face, gear and portrait ID; career facts from the data (draft slot, team, career span, Pro Bowls, All-Pros); **rating explanation** with lens, hindsight, wAV, its evidence ("missing", "no completed season", "observed, career ended" and so on), percentile in group, and whether estimated | Read-only with spoilers off; editing with unsaved changes; field-level validation errors; undo and redo history |
| **Class Studio** | Pool search over about 32k players by name, year, position and college; ordered board; drag and keyboard reorder; custom prospect creation | Empty board; search with no results; board over 402 |
| **Open existing class** | File metadata: build (for example RL4), active count (389 when the game wrote it), capacity 455, and validation | Valid M27; refused with reason (wrong game, truncated, not a draft class) |
| **Validate and export** | Checklist of errors and warnings, output path and name, backup choice, progress, then the reopen-verified result with path and hash | Blocked by errors; warnings only; success; failure with the source untouched |

## Data rules the design must respect

- **Missing is not zero.** wAV is shown with its evidence: actual PFR value, estimated, or no completed season yet. Legacy users know the badges A (actual), P (predicted) and EA (EA rating); keep that idea, but the form is open.
- **Hindsight is not draft-day knowledge.** Career wAV was never known on draft day. Labels must not imply otherwise.
- **Provenance is one click away:** source, as-of date and attribution for any player value.
- **Spoilers off means hidden,** including in tooltips, sorts, filters and exports.
- Counts are three different numbers (active, 402 export limit, 455 capacity). Never merge them into one.

## Constraints for the concepts

- Original branding. Working name "MaddenTool". No EA, Madden, NFL or team logos, no team wordmarks, no real player photos. Use neutral placeholders for portraits.
- Desktop window chrome. Frames at 1440×900 and 1024×768, plus the class board at 800×600.
- Data-dense but readable: about 30 or more board rows visible at 1440×900; overall and dev trait scannable at a glance.
- Light and dark appearance, keyboard focus visible, colour never the only signal (dev traits and evidence badges need text or shape too).
- Do not design franchise screens. They are deferred.

## Deliverables requested

One frame per screen above at 1440×900. The class board also at 1024×768 and 800×600. State variants: board with spoilers off and on, data screen first-run and offline-degraded, export success and blocked, player profile with the rating explanation open. Two to three distinct directions are enough to choose from.

## Decisions already made

Rust backend, Tauri and React. 402 export limit, 455 capacity shown. New file by default on export. Deterministic seeds. Real data only, never invented players presented as real (era fillers are labelled generated).

## Open questions

1. Default lens: Realistic or Career (wAV contract Q1)?
2. Should the Career lens stay uncapped or be Madden-shaped (contract Q2)?
3. Default export size: 402 (tool tradition) or 389 (what the game writes)?
4. Is the seed visible by default, or only in an advanced panel?
5. Single window with panels, or separate windows for profile and studio?
6. Final product name and identity.
