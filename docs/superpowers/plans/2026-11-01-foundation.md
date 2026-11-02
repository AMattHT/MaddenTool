# Cross-platform Foundation Implementation Plan

> **For agentic workers:** Use superpowers:executing-plans to implement this plan task-by-task. The user explicitly chose Claude Opus 5.5 as implementer and approved beginning implementation. Do not introduce another approval gate for this scoped plan or spawn additional agents.

**Goal:** Establish a private repository and a working Rust-backed Tauri/React desktop foundation with native-platform CI and a reviewable first PR.

**Architecture:** Rust owns application information and authoritative operations through a thin Tauri host. React renders actual command results and recoverable loading/error states. This slice establishes a functioning boundary; game codecs and the full job system follow in later packages.

**Tech Stack:** Stable Tauri 2, Rust, React/TypeScript, Vite; exact compatible versions pinned by the implementer after checking primary documentation.

**Spec:** `docs/superpowers/specs/2026-11-01-maddentool-design.md`, approved by the user. Execution details: `docs/prompts/01a-foundation.md`.

## Global constraints

- Madden 27 only. Full combined MyFranchise/draft-tool parity remains the eventual release requirement.
- Rust backend + Tauri + React/TypeScript UI; Windows, macOS arm64/x64 and Linux.
- Destination repository: private `AMattHT/MaddenTool`; verify authenticated ownership and existing state before creation.
- Minimal baseline on main; product work on `feat/cross-platform-foundation`; open a PR without merging.
- Reference binaries, raw bundled datasets/art, real saves and secrets stay outside Git.
- Rust is the shipped backend. No Node/Electron/Express runtime sidecar.
- Preserve user/reference files. Only create modules needed by this package.
- Platform builds, native smoke results and runtime proof are reported separately.

## Review focus

1. A command failing must produce a recoverable startup error, never a permanent spinner (Task 1).
2. An unmounted view must not apply a late command result (Task 1).
3. 800×600 windows and 200% zoom must retain usable startup controls (Task 1).
4. A fresh platform checkout must find the UI and Tauri assets without Windows-specific paths (Task 2).
5. Repository initialization must preserve existing work and exclude real/reference data (Task 3 preflight, before any commit).

## File map

- Root `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`: pinned Rust workspace and tools.
- Root `package.json` and one package-manager lockfile: reproducible orchestration; prefer npm already available unless evidence warrants another tool.
- `app/src-tauri/{Cargo.toml,build.rs,tauri.conf.json}`: native host and packaging configuration.
- `app/src-tauri/src/{main.rs,lib.rs,app_info.rs}`: entrypoint, command registration and authoritative metadata.
- `app/src-tauri/capabilities/default.json`: narrow main-window capabilities.
- `ui/{package.json,index.html,vite.config.ts,tsconfig.json}`: React build and tests.
- `ui/src/{main.tsx,App.tsx,App.test.tsx,styles.css}`: startup view and behavior tests.
- `ui/src/api/{app-info.ts,app-info.test.ts}`: typed IPC wrapper and contract validation.
- `.github/workflows/ci.yml`: platform checks/builds.
- `README.md`, `docs/DELIVERY.md`, `docs/provenance/README.md`: run commands, observed delivery evidence and provenance policy.

Adjust filenames only for actual framework requirements and record the change. Do not create all future proposed crates as empty placeholders.

## Task 1: A real Rust-backed startup view

**Interfaces**

- Rust `get_app_info() -> AppInfo`, serialized using camelCase.
- `AppInfo { app_name: String, app_version: String, game: String, platform: String, arch: String }`.
- Values: `app_name = "MaddenTool"`, `game = "Madden 27"`; version from package metadata, platform/arch from the actual target.
- TypeScript `getAppInfo(): Promise<AppInfo>` invokes `get_app_info` and validates the returned contract. Native invocation failures reject to the view, not synthetic success data.

- [ ] Perform Task 3's repository/ignore preflight before writing or committing product files. Establish the metadata baseline and feature branch.
- [ ] Set up the minimal workspace, UI and host using current stable tooling; pin dependencies and a single authoritative app version source. Check Tauri's frontendDist/devUrl paths against the proposed layout.
- [ ] Add meaningful view/contract tests before the corresponding behavior: `startup_displays_native_information` asserts MaddenTool/Madden 27 and returned version/platform; `failed_command_can_retry` rejects once then resolves and asserts an error followed by real metadata; `unmounted_view_ignores_late_result` checks a deferred result does not resurrect stale UI. Contract tests reject a mismatched shape instead of silently rendering undefined fields.
- [ ] Run the new tests and confirm failure is caused by the missing intended behavior. Implement the command and wrapper, then the responsive startup state machine. Provide keyboard-operable Retry and readable loading/error states. Avoid implementing game features in this screen.
- [ ] Use fluid spacing and layout. Check 800×600, 1024×768 and a large window, plus 200% zoom. Record visual/native proof with supported tools; mocked browser IPC alone does not prove native behavior. Do not use T3 preview tools unless separately requested by the user.
- [ ] Run frontend typecheck/tests and Rust checks. Launch the actual Tauri application locally and verify the native command result is visible. Record invocation method, process cleanup and evidence limits.
- [ ] Commit the working native/UI boundary on the feature branch.

## Task 2: Reproducible native-platform builds

**Interfaces**

- Root commands `npm run dev`, `npm run build`, `npm run typecheck`, `npm run lint`, `npm test`, and `npm run format:check` orchestrate the documented packages.
- Rust gates: `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`.
- CI consumes the committed lockfiles and uses native Windows, macOS and Linux runners. Resolve actual available runner labels/architectures rather than assuming labels from audit notes remain valid.

- [ ] Define reproducible root scripts and document required OS dependencies. Keep package configuration aligned with the workspace paths.
- [ ] Configure narrowly scoped Tauri capabilities/CSP and platform build metadata. Do not request broad filesystem/shell permissions for a metadata-only screen.
- [ ] Add CI jobs for Rust/frontend gates and Tauri builds. Cover both requested macOS architectures with supported runners or a documented supported cross-build strategy. Report any unavailable runner as a blocker, not successful coverage.
- [ ] Prove a fresh checkout build path using lockfiles, without relying on reference projects or Downloads. Keep signing/notarization secrets out of this foundation PR; unsigned CI verification is explicitly labeled.
- [ ] Run the local gate commands and record exact results. CI workflow creation is not CI completion; report checks from the pushed final head.
- [ ] Commit the reproducible build/CI slice.

## Task 3: Provenance, repository and first PR

The preflight steps run before Task 1; completion steps run after Task 2.

- [ ] Inspect Git/workspace state and verify `gh api user`. If a conflicting repository or unrelated changes exist, preserve them and report the precise conflict.
- [ ] Inspect every candidate baseline file. Create ignore rules for fixtures, extracted apps, archives, caches, logs, local paths/configuration and build outputs. Keep required tstack license/provenance. Stage explicit paths and inspect `git diff --cached --stat` plus the actual diff; do not blindly add the entire directory.
- [ ] Initialize the metadata baseline and private remote as described in `01a-foundation.md`. Confirm remote visibility is PRIVATE. Branch before adding product code.
- [ ] Document provenance without treating private visibility as permission to redistribute third-party material. No imported schemas/art/tuning datasets are needed for this slice.
- [ ] Update `docs/DELIVERY.md` with baseline/design hash, branch/base/head, checks, native smoke results and pending platform results. Update feature rows only where supported by evidence.
- [ ] Review the final base-to-head diff and run only checks affected by any last edits. Read the pinned tstack file-pr skill and create an open PR against main with accurate model attribution and verification.
- [ ] Immediately link the PR through T3 after creation/push. Do not merge or watch from a child. Return the PR URL, exact head SHA, check/review state and evidence to the parent.

## Scope review

This plan covers WP01a. WP01b owns the full responsive shell, job progress/cancellation and real file selection. WP02 owns draft parsing and byte-preserving edits. All remaining franchise, roster, draft-generator, MMC and release work remains in the approved roadmap. A successful foundation PR does not satisfy the full product request.
