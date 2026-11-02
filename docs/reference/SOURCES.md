# Evidence ledger — 2026-11-01

## Local inputs

### Existing draft generator

- Root: `%USERPROFILE%\Documents\Projects\Madden26DraftClass\draft-class-generator`
- Remote: `https://github.com/AMattHT/Madden-Draft-Generator.git`
- Observed branch: `release-1.5.2`; HEAD `04b24393cc58880b9986c65c9c2e7725728442c0`; package version 1.5.2.
- Working tree contains existing edits and untracked probe/agent files. Preserve them. The working tree is the user's current reference; record any behavior that differs from HEAD.
- The parent folder's old M26 name is misleading: the README and implemented services include M27 behavior.
- Current architecture: Electron, React, Node/Express, SQLite; `madden-franchise` dependency range `^4.3.6`.
- The local README covers historical drafts, three ratings modes, custom classes, blind scouting, likeness, equipment, and spreadsheet export. Routes and components add roster and franchise features not exhaustively described in that README.

Read `server/src/vendor/draft-class/M27Parser.js`, `M27Writer.js`, `server/src/services/Mdc27Service.ts`, `M27Fields.ts`, `FranchiseService.ts`, `HistoricFranchiseService.ts`, `LeagueHistoryService.ts`, `PortraitPackService.ts`, and the associated tests. Read the parent `M27-PORT.md` as historical evidence; resolve conflicting notes against current code and files.

The M27 parser describes 5,876-byte records, a 5,632-byte visual JSON region, and 244 attribute bytes. Its header says 455 physical blocks; the product uses a 402-player draft board. Verify physical capacity, header count, active prospects, and filler separately rather than treating these numbers as interchangeable. Preserve unknown fields and donor bytes during writes.

### MyFranchise baseline

- Requested installer: `%USERPROFILE%\Downloads\MyFranchise-2.0.1-setup.exe` (243,904,344 bytes).
- Read-only extraction with locally installed 7-Zip yielded `%USERPROFILE%\AppData\Local\Temp\maddentool-myfranchise-2.0.1-reference\app.asar`. The installer was not run.
- Extracted package version: **2.0.1**. This is the requested parity baseline.
- Installed app: `%USERPROFILE%\AppData\Local\Programs\MyFranchise`, package version **2.0.2**. Use as a supplemental reference; record deltas instead of silently changing the baseline.
- 2.0.1 and 2.0.2 preload files have the same SHA-256: `17bde95f8788d58f4db6e0fb31e3dc6ebfd74832fa4dceb819a77bc13eed3461`. Matching preload APIs do not establish matching UI or algorithms.
- `myfranchise-2.0.1-inventory.json` records renderer hashes, IPC channels, component names, and route strings. Static presence is not proof that a feature is reachable or works.
- Package metadata reports MIT. Verify actual applicable licenses before incorporating code or data; a package label does not establish rights to every bundled image or dataset. Use original branding for the new application.

The updater references `Bowersrd/MyFranchiseRelease`. Its public repository is a release repository, not a verified source-code distribution: https://github.com/Bowersrd/MyFranchiseRelease.

### MMC

- Root: `%USERPROFILE%\Downloads\MMC_Modding_Tools_v1.1.0.7`
- Executables: `MMC_Editor_v1.1.0.7\MMCEditor.exe` and `MMC_ModManager_v1.1.0.7\MMCModManager.exe`.
- Package includes `MADDEN27SDK.dll` and M27 caches. Other profiles also exist, but the new app targets M27 only.
- `ReadMe.txt` distinguishes creating/editing mods in MMC Editor from applying `.fbmod` packages with MMC Mod Manager. No supported automation API or CLI has been verified.
- Current draft code's `PortraitPackService.ts` describes the Portrait Manager input as a directory of portrait-ID-named DDS files, with 512px output. Trace encoding and confirm an actual import before declaring compatibility.
- `PortraitModService.ts` also contains an older PNG replacement-folder workflow. Its text alone is insufficient evidence for the newer MMC portrait path.
- No launcher, anti-cheat, game installation, or real save was modified. The interoperability work concerns offline files and user-selected companion tools.

## Workspace and tooling

- Destination: `%USERPROFILE%\Documents\Projects\MaddenTool`; initially empty and not a Git repository.
- GitHub CLI is authenticated. The queried `AMattHT/MaddenTool` repository was not found; no destination remote has been created.
- Rust and Cargo 1.99.0 are installed on this Windows host. This does not prove macOS/Linux native build availability.
- The direct T3 `claudeAgent` / `claude-opus-5-5` delegation succeeded despite stale capability metadata. Opus completed work package 00. Continue using that exact provider/model.

## Upstream references

- tstack: https://github.com/t3dotgg/tstack at `1085050f3baed1ac724caa4a8166145fb2ac88af`.
- Tauri platform and frontend architecture: https://v2.tauri.app/start/.
- Tauri Rust commands, async work, and channels: https://v2.tauri.app/develop/calling-rust/.

Recheck live technical documentation when selecting dependency versions. Local binary-layout observations above are project evidence, not an official EA specification.

## Audit additions — 2026-11-01

Work package 00 verified this ledger. Corrections are in `docs/DISCOVERY-REPORT.md` section 3.

- `2.0.1` and `2.0.2` share `main.js`, `preload.js` and the metadata worker. The renderers differ only in one Injured Reserve Fix expression.
- Upstream has reached MyFranchise 2.0.4. The baseline stays 2.0.1.
- Legacy franchise routes mount only with `DRAFT_TOOL_FRANCHISE=1`.
- The user supplied three game-exported RL4 classes (`CAREERDRAFT-TEMP1`, `TEMP2`, `TEMP3`). Hashes are in report section 5.2.
- The only M27 franchise file found is a MyFranchise backup from build RL2.5.
- Coordinator verified authenticated identity via `gh api user`: `AMattHT`, ID `262685875`, matching the legacy remote owner. `amatthewsHT` in CLI status is stale; that API login returns 404.
- Probe scripts and derived maps are in `reference/audit/`. Scratch output, including a copy of a real save, is in `%TEMP%\maddentool-audit` and is not part of this repository.
