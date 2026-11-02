# MaddenTool: Madden 27 franchise and draft companion

Status: **approved by the user for implementation on 2026-11-01**, after work package 00 audit. The user explicitly said: "Approve the design and begin implementation." Evidence is in `docs/DISCOVERY-REPORT.md`; operational questions are resolved in its section 11. The user selected the language/UI architecture, platform scope, and full-parity goal. The section "Audit revisions" records what the audit changed.

## Required outcome

Create the user's own desktop franchise application for **Madden 27 only**, encompassing all user-facing features in **MyFranchise 2.0.1** and the **current draft generator**. Integrate with the supplied MMC tools through verified file workflows. A draft editor alone does not satisfy the request.

Use **Rust for the backend and Tauri/React for the interface**, and support **Windows, macOS, and Linux**. The interface must adapt to window size and react immediately to edits, selection changes, job progress, and validation results. Codex authors execution prompts and coordinates delivery; **Claude Opus 5.5 implements**. Use Theo's tstack skills to prepare and ship reviewable PRs.

## Approaches considered

The user chose a Rust backend with Tauri/React over an all-Rust native interface. This retains useful React interaction patterns and gives the Rust data engine a desktop host across the three requested operating systems. Retaining Electron/Express as the shipped backend would not fulfill the Rust choice. Existing TypeScript implementations may run as development-only behavioral oracles.

## Product scope and parity contract

`docs/FEATURE-PARITY.csv` is the requirements ledger: 285 feature rows and 723 symbol mappings, with ID, source/version, screen or operation, evidence pointer, observable behavior, M27 applicability, dependencies, Rust owner, UI owner, test/proof, runtime evidence, status, work package and PR link. `docs/reference/audit/verify_parity.py` checks it. Classify each discovered symbol as a feature, shared implementation detail, unreachable/dead code, or unresolved. Every source feature must be accounted for; a component count is not a feature count.

Statuses: discovered, behavior-verified, implemented, tested, runtime-verified, blocked. Full parity requires every applicable feature to reach runtime-verified. A disabled control, fake data view, undocumented omission, or an arbitrary MVP is not parity. Platform-specific companion launching can be unavailable with an explicit reason while portable data work remains usable. Newly discovered features expand this ledger and the roadmap; they are not silently dropped.

### Draft generator capabilities to preserve

- Historical drafts spanning the supported 1936–2026 data, AFL/NFL merging, source/cache refresh, degraded-data disclosure, and offline use after data acquisition.
- Realistic, Career, and Launch Day ratings, hindsight controls, combine/position/archetype calibration, OVR and development traits, M27 rookie data and Persona DNA.
- Standard, random, custom, decade and all-time classes; Class Studio pool search, ordered board composition, drag/reorder, custom prospects, veteran/free-agent injection where present, era-correct fillers, overflow visibility and recovery.
- Player inspection/editing: identity, bio, physicals, all supported ratings, positions, traits, persona, faces, portrait IDs, equipment, and likeness overrides; undo/redo and persistence.
- Blind scouting and spoiler controls, including sort/filter paths that could reveal hidden ratings.
- Open existing classes, validate, edit, export M27 files, choose an output directory, spreadsheet export, and save-directory integration when available.
- Historical roster builder/editor and team assignment features present in the current UI/routes.
- Existing franchise utilities: cap reset, bulk edits, injuries, roster apply, trait realism, free-agent trimming, draft-pick reset, schedule display, relocation/rebranding, roster aging/retirement/regression, historical-season preview/playoff configuration and league-history behavior present in current code.
- M27 portrait packs, portrait-ID allocation/collision handling, custom sources, face/gear mappings and optional image acquisition with correct missing-asset feedback.

### MyFranchise capabilities to preserve

All 51 routes, 365 component symbols and 104 IPC channels, plus 7 channels the inventory missed, are mapped in the ledger, which is the acceptance checklist. The families are:

- File selection, franchise identity, open/reload/save/close, backup management, file-change detection, metadata loading, local settings/data and custom player/coach portraits.
- Home/news/ticker views, season and weekly previews/recaps, game previews/recaps, scores, gamecast/box scores and contextual league/season hubs.
- League/team standings, power rankings, team ratings, schedules, playoff picture/bracket, predictions, awards races/history, Pro Bowl voting/rosters, Super Bowl history, record books and retired numbers.
- League/team rosters, active/free-agent/practice-squad/retired players, injuries, player detail and editing, attributes, development and behavioral traits, DNA/focus/mindset, abilities, character visuals, equipment, commentary/presentation IDs.
- M27 contracts: salary, bonus, guarantees, incentives, no-trade clauses and other exposed fields; cap/finances views, restructure/release impact, sign/cut/retire/unretire and practice-squad transactions.
- Depth charts, team control/settings, coaches and staff, hire/fire/retire/unretire, talents, schemes/playbooks and coach visuals.
- Draft prospects, workout data, big board, mock drafts, draft order/pick editing/reordering and history.
- Trade block, trade center/valuation, player/pick trades, trade lottery, free-agent lottery, free-agency tracker, upcoming free agents, offseason tracker/report cards and transaction history.
- Player/team game, season and career statistics; stat editing, records, leaders, rookie tracking and historical views.
- Progression/offseason progression, snapshots/history/reporting and rerun prevention; training camp, roster balance/fill, weekly tools, weather editing/generation, cap cleanup, depth-chart reordering, playbook matching, injury/IR fixes, jersey/commentary/presentation fixes, draft-order recalculation and coach wardrobe shuffling.
- Settings, theme/window preferences, help/tutorial behavior, update checks/download/install, and any additional conditional tools found during runtime audit.

## Architecture proposal

Use a Cargo workspace with focused libraries for domain types, M27 formats, application services, persistence/jobs and MMC interchange, plus a thin Tauri host. Avoid a crate per screen. React/TypeScript owns presentation, local form state, navigation and accessibility; Rust owns authoritative game data, validation, calculations, transformations and writes. Use typed DTOs and structured errors at the IPC boundary. Keep filesystem paths and game bytes out of browser storage.

Use a Rust-accessed SQLite database for workspace metadata, historical data, settings, edit history and progression snapshots with migrations. The save file remains authoritative for game state. Source-independent domain identifiers must map explicitly to table/record references and game IDs.

The M27 draft-class, roster and franchise-save codecs are three separate modules. Porting the draft-class writer implements neither of the others, and its legacy rebuild behavior is not ported: writes patch only changed fields. Investigate the installed `madden-franchise` implementation and schemas for a native Rust reader/writer; prove container parsing, compression, table schemas, references, checksums and unknown-byte preservation before enabling edits. No Node/Electron/Express sidecar is the planned shipped backend. Report a true codec blocker instead of masking it with a JavaScript backend.

## Save integrity

Read originals without mutation, including at load: file ID, draft-pick reorganization and visual repairs are previewed and written only on commit. Each edit produces a validated change set and preview. The first commit defaults to a new `CAREER-` file and later commits update that file with rolling backups; any explicit replacement of the original must retain a restorable backup. A setting can commit after each action through the same safe path. Before committing, verify the source hash/revision, validate cross-record invariants, write a temporary sibling, flush, reopen and validate it, then use the platform-appropriate atomic commit mechanism. Account for Windows locks and platform filesystem differences. Cancellation, disk-full, conflicting external edits and process interruption must leave the source intact.

Preserve unknown fields, table relationships, record identity, untouched visual data and franchise bytes after the compressed stream. A no-op returns the original bytes; if compressed output exceeds the file's fixed capacity, refuse and write nothing. Select the schema through an explicit version map and refuse to edit an unmapped version. Reject M26/other formats with a useful message. M27 contracts, guarantees, incentives, void years and roster moves require their own verified field maps and invariants. Preserve autosaves; use copied fixtures for testing. Long-running bulk operations preview their scope and commit as coherent changes rather than partial saves.

## Responsive and reactive UI acceptance

- Adapt layout at 800×600, 1024×768, 1440×900 and 1920×1080; verify 200% zoom and high-DPI displays. These are proposed validation sizes, not additional mobile-platform scope.
- Collapsible navigation, resizable master/detail panes, useful empty/loading/error states, no inaccessible actions or clipped dialogs. Wide datasets may scroll inside tables; primary controls remain visible.
- Virtualize large player/pool/stat lists, debounce searches, page/stream data and avoid sending entire franchise tables to React on every edit.
- File parsing, compression, database scans, image work and generation run off the UI thread. Provide progress, cancellation and recoverable errors through typed Tauri channels/events; clean up listeners.
- Rust commits return a revision and affected IDs. React updates/invalidate the corresponding queries; job IDs/revisions prevent stale responses from overwriting newer edits. Preserve selection, scroll and unsaved form state across background refreshes.
- Validate fields immediately. Show pending mutations and rollback/errors honestly. Undo/redo covers supported edits without conflating persisted and unsaved state.
- Keyboard navigation, visible focus, labeled inputs, screen-reader status announcements and reduced-motion support are acceptance criteria.
- Proposed performance targets: input feedback within 100 ms; measured p95 local filter/sort within 200 ms over the existing ~32k-player pool; responsive scrolling during a background file job. Record hardware, dataset, build mode and measurements; revise targets with evidence instead of claiming performance from unit tests.

## Cross-platform and MMC boundaries

All three platforms must open copied M27 files, edit, generate and export portable outputs using user-selected paths. Windows saves-folder discovery is an optional convenience; macOS/Linux must not depend on Windows registry or drive-letter paths. Consider user-selected Wine/Proton locations only when verified; do not promise native Madden or MMC execution on macOS/Linux.

Separate three capabilities: draft-class export, franchise-save editing, and mod/asset interchange. Verify MMC's DDS portrait import and PID mapping with the supplied version, using the procedure in the discovery report. The `.fbmod` collision scan is unverified, so it stays off until tested on real MMC 1.1.0.7 mods, with a manual ID exclusion list as the fallback. Pack manifests carry the custom ID registry so classes and packs built on different machines agree. A PNG directory is not a `.fbmod` package. No unsupported one-click API or fabricated `.fbmod` writer.

Show detected MMC paths/version and capabilities. Opening the tool folder and showing the steps is the default; launching is added only if the observed `-launch` Mod Manager argument or a plain launch is verified. On macOS/Linux, allow creating the same portable asset packs for transfer to a Windows MMC installation. Direct MMC support remains a separately evidenced capability. Do not change game launchers or anti-cheat as part of application setup.

## Verification and delivery

Deliver small dependent PRs using `docs/PR-ROADMAP.md`. Each feature lands as working backend + UI + relevant proof; shared format work can precede feature slices. Retain source provenance and license notices when incorporating permitted material. Keep real game saves, installers, proprietary assets, personal paths and secrets out of public artifacts. Use generated/minimized fixtures and documented local fixture discovery.

CI runs Rust formatting, clippy and tests; frontend formatting/lint/typecheck/tests; and Tauri builds on Windows, macOS and Linux. Proposed package targets: Windows x64 installer, macOS arm64 and x64 app/DMG, Linux x64 AppImage/deb; Opus verifies runner/tool availability. Add install/open/export smoke checks on actual platform runners. A build job passing is not proof of interactive runtime behavior.

Original unminified source was not found for MyFranchise calculations and tools. Compare implementations against goldens captured from an isolated run of extracted 2.0.1 (WP00R); testing copies is authorized, and the implementer must verify isolation before execution. Use real and malformed fixtures for codecs, no-op and single-edit round trips, unknown-field preservation, relationship/contract invariants, deterministic seeded generation, database migrations, cancellation/recovery, source-conflict detection, and portraits/PID collisions. Compare the Rust draft output against the existing TypeScript oracle at semantic and binary levels where meaningful. Confirm final exported files in Madden and asset packs in MMC before calling those paths runtime-verified. Record any hardware/game-access limits explicitly.

Only create release/update artifacts with correct app identity and configured endpoints. Test unsigned packages where appropriate; production signing/notarization and updater keys are separate release dependencies, never invented. Node may be a build/test dependency, not the shipped backend runtime.

## Settled implementation decisions

- Workspace: `mt-core`, `mt-draft-format`, `mt-roster-format`, `mt-franchise-format`, `mt-franchise`, `mt-draftgen`, `mt-store`, `mt-interop`, and a thin `app/src-tauri`. React lives in `ui/src`.
- Stable lines: Tauri 2, React 19, TanStack Query 5, TanStack Virtual 3, rusqlite, flate2. Pin exact versions at scaffold.
- One session owner thread per open file. Reads return a page and a revision. Commits return the new revision and affected IDs.
- No general file read or write commands. Paths come from dialogs or saved scoped entries. Images use a scoped protocol.
- Draft export limit is 402 active, capacity 455 is displayed, and 403 to 455 stay blocked until an in-game test proves them.
- Real fixtures stay outside the repository and are referenced by `MADDENTOOL_FIXTURES`. Repository fixtures are synthetic.
- Working name: MaddenTool. Use original branding; do not claim affiliation with MyFranchise/MMC/EA.

## Audit revisions

1. Added the M27 roster format as a third codec.
2. Replaced the legacy draft rebuild with patch-only writes, because it changes every active visual block on a no-op.
3. Added deviations D-1 to D-6 from MyFranchise (report section 7.1): no writes at load, session then commit, the 2.0.2 Injured Reserve filter, job progress instead of a blocking overlay, a resizable window, and clearer run-once wording.
4. Added golden capture (WP00R), an explicit schema version map, and capacity and trailing-byte rules for franchise files.
5. Marked the legacy franchise tools as off by default and unproven on M27. They stay in scope and must be proved again.
6. Marked `.fbmod` collision detection as unverified and made the custom ID registry portable.
