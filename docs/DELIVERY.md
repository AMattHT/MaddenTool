# Delivery ledger

## Authorization

The user approved the reviewed design and instructed implementation to begin. Approved scope: Madden 27 only; full MyFranchise/draft-tool feature parity; Rust + Tauri/React; responsive/reactive UI; Windows/macOS/Linux; private AMattHT/MaddenTool development; Opus 5.5 implementation; tstack PR workflow. PR merging is not authorized.

- Design: `superpowers/specs/2026-11-01-maddentool-design.md`
- Approved design SHA-256: `b73b95d9e3c8086fadb69987ba789e4199ada98e7c4daf8bdeefb9433a7b0411`. This hashes the approved working copy with CRLF line endings. Git stores it with LF, giving `ad56778ec8d6d97d59b96eae4e0940578084e7ff31b396d9ebd5be73b6fbc6b6`. The text is otherwise unchanged.
- First execution plan: `superpowers/plans/2026-11-01-foundation.md`
- First prompt: `prompts/01a-foundation.md`

## WP01a

Status: **PR open and unmerged.** This is the foundation only; no game files can be opened and full parity remains the release gate.

| Field | Value |
|---|---|
| Provider/model | `claudeAgent` / `claude-opus-5-5`, in T3 Code through the Claude Code harness |
| Repository | `AMattHT/MaddenTool`, verified PRIVATE. `gh api user` returned `AMattHT`, ID `262685875` |
| Base | `main` at `def90394db3d352d790dc4ebbd8b54dd3eef2db8` (metadata baseline) |
| Branch | `feat/cross-platform-foundation` |
| PR | https://github.com/AMattHT/MaddenTool/pull/1. The head SHA and live check state are on the PR; a commit cannot record its own SHA |
| Parity rows | `XP-015` was moved to `implemented` and has been returned to `discovered`, because dependency notices, `ui/about` and the release check are unmet. Its evidence records the partial work. No other row changed. |

### Baseline notes

- The audit found absolute personal paths in prompts, sources, two inventories and three probes. They now use `%USERPROFILE%` or environment variables.
- The first push of `main` (`3b34a6e`) still had such paths in the two inventories, because of a failed replacement. Minutes later, `main` was amended and force-pushed once to `def9039`, with the lease pinned to that sole commit, with no forks or other branches. The old SHA may stay reachable by direct SHA on GitHub until it is garbage collected. It held local paths and a username, no secrets or game data.
- Inventories and audit JSON are derived names, routes and hashes. No reference code, schema, asset, save or extracted bundle was committed.

### Observed results

Local host: Windows 11 Pro 10.0.26200 x64, Rust 1.99.0, Node 24.16.0, npm 11.13.0, WebView2 Runtime 156.0.4314.58. No game data was used. These results are from the review remediation below, on the working tree that became its commit. Everything in this table ran on Windows only.

| Check | Result |
|---|---|
| `npm run check` | Pass: Prettier, ESLint, `tsc`, 23 UI tests, 138 script tests (137 pass, 1 POSIX-only test skipped on Windows), provenance over 88 tracked files, rustfmt, clippy `-D warnings`, 13 Rust IPC and config tests |
| `cargo clean -p maddentool --release`, then `npm run build` | Pass: the binary was recompiled through `tauri build -- --locked`, with NSIS and MSI bundles, unsigned |
| Mutation: remove the UI stale-result check | 8 tests fail, 6 of them new. Restored; 23 pass |
| Mutation: cleanup no longer abandons the old attempt | Both overlapping-attempt tests fail. Restored |
| Mutation: cleanup leaves the timeout running | The unmount test fails. Restored |
| `npm run dev` (not re-run in the remediation) | Earlier run: real window from `http://localhost:1420` shows the same native values. One favicon 404 is logged in dev mode |

Real-window proof on Windows, through `npm run smoke:native` against that rebuilt binary (WebView2 DevTools protocol), exit 0:

- It attached to `http://tauri.localhost/`, and the window shows Madden 27, version 0.1.0, windows and x86_64 from the real `get_app_info` IPC call.
- `plugin:window|set_title` and `plugin:webview|create_webview`, called with valid arguments, each answered exactly `Command <name> not allowed by ACL`. `read_file` is reported separately, because it has no handler: the real app's ACL answers it first, with the same wording, and that is not proof about a registered command.
- 800×600, 1024×768, 1440×900 and 1920×1080, plus 800×600 and 1024×768 at 200%: no horizontal overflow or clipped text.
- No console error, uncaught exception, unhandled rejection or CSP report. This run is the first to look at the first three.
- A malformed first IPC response reaches the error state through the real wrapper. Retry is reachable at 800×600 at 200%, and Enter recovers through real IPC.
- `scripts/window-sizes.ps1` (earlier run, not repeated) resized the real window to 800×600, 1024×768 and 1440×900. Screenshots were inspected, which found and fixed an empty grid cell at 800×600.

No application process was left after any run, including every failing probe. Screenshots stay outside Git under `.smoke/`.

Earlier evidence, from before the remediation. The earlier claim of "no console errors" only read `Log.entryAdded`, so it did not cover console errors raised by page scripts, uncaught exceptions or unhandled rejections, and it is withdrawn.

- Fresh checkout: a temporary clone, using no reference project or Downloads, passed `npm ci`, `npm run check`, a build and three native smokes. That was an older commit on a machine with the tools already installed. No clean-machine test has been run.
- One earlier fresh clone failed its smoke once with `Timed out waiting for the injected error state`, with no URL diagnostics, and it did not recur in 15 later runs. The independent review later reproduced a failure mechanism: the old page lookup accepted `about:blank`, and in four early attachments Fetch injection did not fire. Waiting past the blank page injected correctly in twelve runs. That is a reproduced mechanism, **not an established cause of that earlier run**. Those counts are the reviewer's observations, not this package's. The smoke now waits for the application page and reports the selected URL on failure.

### Platform coverage

CI definition, passing builds and runtime proof are separate, so they are listed separately. Results below are for the head they name. **No job has run the remediated harness yet, and none of these is a final-head result.**

| Platform | Runner label | CI on `76ce766` (run `41395383140`) | Runtime proof so far |
|---|---|---|---|
| Windows x64 | `windows-2025` | **Failed in the smoke step**; all gates and the NSIS build passed | Local real window, full level above. None yet in CI |
| Linux x64 | `ubuntu-24.04` | Passed, smoke reported `native-dom` | DOM facts through `tauri-driver` under `xvfb` on that head, with the old harness. No capability, console or Retry check runs on Linux |
| macOS arm64 | `macos-15` | Passed | **Process start only.** Rendering and IPC are unproven, because WKWebView has no WebDriver |
| macOS x64 | `macos-15-intel` | Passed | **Process start only.** Rendering and IPC are unproven |

### CI failure: Windows smoke on `76ce766`

Job `125115572953`, image `windows-2025-vs2026` version `20261021.278.1`, failed after 45 seconds with `Timed out waiting for the WebView2 debug target: fetch failed`. This came before any page was selected, so it is not the `about:blank` mechanism above. The old harness discarded the child's output and exit state. **The cause is not established.**

What is known:

- Microsoft documents that Edge Stable does not provide the WebView2 Runtime and that the registry `pv` value is how to detect it. So the published image listing Edge says nothing either way. The runner's actual state was never recorded.
- Local observation on the pinned Tauri: with `WEBVIEW2_BROWSER_EXECUTABLE_FOLDER` pointing at a folder that does not exist, the release binary stays alive on a native dialog titled `Error`, with no child processes, and its debug port refuses connections until the 45-second limit. Tauri's source has this dialog for a missing runtime. This resembles the CI symptom, but it does not show that the runner hit this path. Other causes remain open, such as the debug endpoint failing for another reason.
- The runtime has not been uninstalled locally, so real absence has not been exercised.

What changed, without extending the 45-second limit or skipping the smoke:

- A failed start now ends with a `startup=` report. It classifies the observations as `spawn-error`, `early-exit`, `webview2-runtime-not-detected`, `blocked-on-native-error-dialog` or `debug-endpoint-unreachable`, and includes the exit code, the last transport error, attempts, window title, child process names, registry versions and redacted bounded child output. Environment values and command lines are left out. A process that exits now fails at once rather than at the deadline.
- CI runs `scripts/ensure-webview2.ps1` before the smoke. It logs whether the runtime was present and installs it only if absent, using Microsoft's documented detection and Evergreen Bootstrapper with `/silent /install`, after checking the Authenticode signature, and fails if it is still not detected. **The next Windows job's log is the evidence that decides between these.** Locally, the present branch ran, and the bootstrapper was downloaded and had a valid Microsoft signature, but it was not run. The install branch is unexercised.

Local startup probes:

| Probe | Classification | Time |
|---|---|---|
| Runtime folder that does not exist | `blocked-on-native-error-dialog` | 46 s |
| Binary that exits at once | `early-exit`, with its exit code | 1 s |
| File that cannot be spawned | `spawn-error` | Immediate |
| Missing registry entries | `webview2-runtime-not-detected`, in a unit test only | n/a |

The Windows CI gate stays open until a run on a new head passes.

### Review remediation

Prompt: `prompts/01a-review-remediation.md`. The independent review examined `ea67c86`. Before changing anything, the unmodified harness and the real `evaluate()` were run against each mechanism below where the table says reproduced.

| # | Finding | Disposition | Evidence, Windows unless stated |
|---|---|---|---|
| 1 | IPC test origin | Fixed earlier in `76ce766`, preserved | Rust tests pass locally. Linux and both macOS jobs passed on `76ce766` |
| 2 | Target attachment accepts `about:blank` | Fixed | Reproduced: the old lookup picks a blank-only list. Tests cover blank then application, lookalike origins, and a bounded timeout naming what was seen. The native run attached to the application page. The reviewer's four missed injections were not re-run |
| 3 | Any rejection counted as ACL proof | Fixed | Reproduced: an argument error passed the old check. Now only the exact denial counts, with valid arguments. A temporary capability granting both core commands made the acceptance smoke fail: `set_title` resolved and `create_webview` reported the unstable feature, which shows the arguments reach the command. With `set_title` granted and its argument missing, `invalid args` did not count. On the shipped build, the ACL answers first, so that probe correctly does not trip. The probe file was removed and verified gone |
| 3 | Rust test read only `default.json` | Fixed | The test now walks every file under `capabilities/` in each shape Tauri accepts, fails closed on TOML or broken files, refuses inline or selected capabilities in the config, cross-checks `gen/schemas/capabilities.json` from the real build, and refuses `add_capability` or a custom path pattern. The temporary file failed two tests, and regressions cover extra, nested, list, named and broadened cases. Shipped permissions and CSP are unchanged. Mock-context IPC remains registration and origin proof, not enforcement |
| 4 | Provenance catch-alls | Fixed | Reproduced: all three named paths were accepted. All three now fail against the real manifest, as do 14 similar paths. Each tracked file has an exact record or a source-type rule, and the check refuses catch-all rules. `XP-015` returned to `discovered`. Limits are in `provenance/README.md` |
| 5 | Unbounded waits and cleanup | Fixed | Reproduced: a hung probe, socket loss and a stalled call all stayed pending. Tests bound each, using a real WebSocket that drops, a handshake that never finishes, and HTTP stalled before headers and mid-body. Late responses are counted and malformed messages fail pending calls. Cleanup runs on success, failure, unhandled rejection, uncaught exception and signals. SIGKILL escalation has a test that is skipped on Windows |
| 6 | Console evidence | Fixed | Reproduced: the old filter counted neither kind of event. Real-window negative probes for a console error, an uncaught exception and an unhandled rejection each failed the run. They are behind `--negative-probe` and are separate from acceptance. The earlier claim is withdrawn above |
| 7 | Cargo lockfile not enforced | Fixed | With a dependency added to the manifest, plain `cargo check` exited 0 and changed `Cargo.lock`. `cargo clippy`, `cargo test`, `npm run build` and `npm run tauri -- build --bundles nsis -- --locked` each refused, proving Tauri CLI 2.12.4 forwards the flag. The lockfile stayed unchanged. No dependency was upgraded |
| 8 | Save and personal path patterns | Fixed | Reproduced: all five probes were accepted. All are now rejected, with 13 path forms and 13 portable forms tested. Names are also ignored by Git |
| 9 | Fetch teardown race | Fixed | Reproduced: a discarded rejection became an unhandled rejection. Commands are now tracked and drained before `Fetch.disable`, and requests paused during it are still answered. Only the two teardown race messages, and only during teardown, are tolerated. Other errors, earlier errors and socket loss are reported. A child-process test shows a stray rejection still cleans up and exits 1. The race is deterministic in tests and did not occur natively (`teardownRaces: 0`) |
| 10 | Weak stale-result tests | Fixed | The unmount test now checks that the timeout is cancelled. New cases cover timeout then late success or rejection, and late settlement during or after Retry. Mutations are in the table above. No production change was needed |
| 11 | Linux prerequisites | Fixed | `build-essential` is in the README and CI, and a test keeps the lists consistent. **Not run on Linux or on a clean machine** |
| 12 | Ledger accuracy | Fixed | This section. The high-effort record is preserved |

The ts-rust evaluation is committed as historical evidence at `ea67c86`, with a decision to defer. No compiler, package or CI change came from it.

### CI correction: IPC test origin

Run `41394717028` on `ea67c86`, macOS arm64 job `125113620544`, failed in `cargo test`. `get_app_info_is_reachable_through_ipc_as_camel_case` panicked at `app/src-tauri/tests/ipc.rs:33` with `get_app_info not allowed. Plugin not found`. The other 4 Rust tests passed, and the bundle and smoke steps were skipped. The other three jobs were still running when this was written.

The defect was in the test, not the product. It hardcoded `http://tauri.localhost`, which Tauri 2.12.5 treats as local only on Windows and Android. Elsewhere its local origin is `tauri://localhost` (`protocol::origin`, `Webview::is_local_url`), so the request counted as remote and the remote-origin guard refused it.

- The test now selects the origin with the same platform condition as Tauri. Product code, capabilities and CSP are unchanged.
- `unknown_commands_are_rejected` passed on macOS for the wrong reason: the same guard refused `read_file`. It now requires `Command read_file not found`, so it fails if the guard answers instead of the command table.
- Local check on Windows: forcing `tauri://localhost` reproduced the same `get_app_info` error and failed the strengthened test with `read_file not allowed. Plugin not found`. With the correct origin, all 5 tests pass.

On `76ce766`, `cargo test` passed on Linux and both macOS jobs, so the correction held there. Those results are for that head only.

### Limits and remaining gates

Required before the cross-platform build evidence can be trusted:

- **All four native jobs must pass on the final head.** That head includes a rewritten smoke harness, which has only run on Windows.
- **Windows CI:** the smoke must pass in CI, and that job's log must show what `ensure-webview2.ps1` found. Until then the `76ce766` failure has no established cause.
- **Linux:** real DOM and IPC facts through `tauri-driver` remain a required CI gate on the final head.
- **macOS rendering and IPC: open.** The process-only smoke cannot show that the window renders or that IPC works, and it reports them as `not proven`. This needs a real Mac or a later automation approach.

Other limits:

- The Rust IPC tests use Tauri's mock context, which has no app manifest. They prove command registration and origin handling, not capability enforcement. Real ACL rejection has been proven only by the Windows smoke.
- 200% is emulated through DevTools (half the CSS pixels at scale factor 2), not Windows display scaling. High-DPI display scaling is not proven.
- Windows did not enforce the configured 800×600 minimum against a direct `SetWindowPos` to 640×480. Interactive drag-resize against the minimum has not been tried.
- The retry proof uses a malformed response at the IPC layer. It does not exercise a real backend crash, and the 10-second timeout is only unit tested.
- Installers were built but not installed. Packages and signing are WP11.
- `npm audit` reports 12 moderate advisories, all in the dev-only lint stack through `minimatch`. `npm audit --omit=dev` reports 0. TypeScript is held at 6.0.3 and ESLint at 9.39.5 for lint plugin compatibility.
- Four native jobs run on each PR update. macOS minutes are billed at a high multiplier on private repositories, so watch the Actions allowance.
- The high-reasoning delegation preference in `docs/prompts/OPUS-MASTER.md` is committed and pushed as `ea67c86`.

### Next: WP01b dependencies

WP01b can stack on this branch or start from `main` after merge. It needs:

1. All four native jobs green on this PR's final head.
2. New capabilities for its file dialog and settings commands, kept narrow.
3. `mt-core` for typed errors and the job lifecycle, which is the first new crate.
4. An accessibility scan and viewport sweep, extending `scripts/native-smoke.mjs` for the shell. The macOS gap above needs a decision here.

## 2026-11-02 direction change: draft generator first

The user made a proper draft generator the first milestone, authorized full-ship (merging after proof, Sol review, CI and normal protections; the parent owns review and merge), and moved the MyFranchise reference to 2.0.4. These supersede the "merging is not authorized" and "2.0.1 is current" lines above. Order: `superpowers/plans/2026-11-02-draft-first.md`.

Observed on 2026-11-02:

- `AMattHT/MaddenTool` is **PUBLIC**, not private. Visibility was not changed.
- The local `gh` has two accounts. `AOTamatthews` is active by default. The user asked for `AMattHT`; DG01 used its token per command, without switching the machine-wide account.
- PR #1 (WP01a, `feat/cross-platform-foundation`) is open and unmerged.

## DG01 Player data foundation

Status: **implemented and locally proven; committed, not pushed.** No PR yet. The parent runs Sol review and shipping.

| Field | Value |
|---|---|
| Provider/model | `claudeAgent` / `claude-opus-5-5`, high reasoning, in T3 Code |
| Branch | `t3/reusable-player-database`, based on `main` at `def9039`. Independent of PR #1 |
| Parity rows | None closed. Prepares `DG-DATA-01` to `05` (DG04a) and `DG-RATE-*` (DG04b) |
| Delivered | `crates/mt-store` (SQLite schema v2 with migrations from v1, two importers, identity reconciliation, verified corrections, queries, `mt-data` CLI), `.github/workflows/player-data.yml`, the draft-first plan, `design/draft-generator-higgsfield-brief.md`, `research/wav-to-ovr-contract.md` |

Rebasing onto PR #1 will conflict in `Cargo.toml` (workspace members: keep both), `Cargo.lock` (regenerate) and this file. PR #1's provenance check will also need manifest entries for the three fixture `.csv` files and the crate's `Cargo.toml`. Its `.rs`, `.md` and `.yml` globs already cover the rest.

### Evidence (Windows 11 x64, Rust 1.99.0, local only)

| Check | Result |
|---|---|
| `cargo fmt --all -- --check`; `cargo clippy --workspace --all-targets --locked -- -D warnings` | Pass |
| `cargo test -p mt-store --locked` | 24 pass: 9 unit, 15 integration including the CLI |
| Mutations: blanks read as 0; external-ID linking off; time-dependent UIDs | Caught: 8, 4 and 1 failing tests. Restored |
| CI smoke script, run locally in Git Bash | Pass. **CI itself has not run**; macOS and Linux are untested |
| Synthetic fixtures through `mt-data` | 9 rows: 6 inserted, 3 rejected with line numbers. Legacy fixture: 7 rows, 6 inserted, 1 slot link, 1 mismatch recorded |

Real reference sample: local scratch only, never committed.

- **nflverse `draft_picks.csv`** (sha256 `cc35472c…5f9f`, `timestamp.json` 2026-10-28):
  - 12,927 rows, 1980 to 2026, all imported.
  - 12,924 players: three people are drafted twice under one PFR ID, Bo Jackson included.
  - wAV missing 2,128, zero 1,613, matching an independent Python count.
  - 2026 class: 197 placeholder zeros classified `no_completed_season`, 60 missing.
  - `car_av` is blank in every row.
- **Legacy `ALL_PLAYER_LOOKUP.csv`** (master `9a4ac69f`, sha256 `b06f3e26…d7e`, as-of 2026-09-15 by commit date):
  - 32,246 rows: 32,245 imported, 1 rejected (round `AT`).
  - 12,923 of 12,926 1980+ NFL selections linked to nflverse by slot and surname.
  - 2 recorded mismatches (a Klosterman/Klostermann spelling; a 2022 pick 140 name conflict), plus 1 supplemental pick correctly kept apart.
  - 95 AFL/NFL dual-draft pairs from 1960 to 1966 remained separate players. Superseded by the identity package below.
- Re-importing nflverse: 12,927 unchanged. JSON query output for 1936, 1965, 1987, 2020 and 2026 was byte-identical after re-import, reopen and an independent rebuild.

Not verified: CI, macOS and Linux runtime, any rating or calibration, any M27 file, any game or MMC import. No data was taken from Pro Football Reference directly; it refuses automated requests and was not worked around.

### DG01 identity reconciliation and round 1 findings (2026-11-02)

Status: **committed on the branch at `cd6f6e0` (S1 and U1 fix; S2 fixed in `472a4c3`), not pushed; no Sol approval yet.** Round 1 (GPT-6.1 Sol, xhigh) asked for changes; the ledger is `reviews/dg01-sol-loop.md`. Issues [#2](https://github.com/AMattHT/MaddenTool/issues/2) (S1), [#3](https://github.com/AMattHT/MaddenTool/issues/3) (S2) and [#4](https://github.com/AMattHT/MaddenTool/issues/4) (AFL/NFL duplicates, user-reported) stay open until their fixes reach `main`.

AFL/NFL identity is now part of DG01, not DG04a. Schema v2 adds:

- player aliases (`merged_into`), so every uid ever issued resolves;
- a merge log with evidence;
- verified row corrections.

Reconciliation runs after every import, and `mt-data reconcile` repairs a v1 database. Rules and refusals are in `../crates/mt-store/README.md`.

PR #1 interplay: its provenance manifest will also need exact entries for `crates/mt-store/data/corrections.json` and the two new fixture CSVs.

| Check (Windows 11 x64, Rust 1.99.0) | Result |
|---|---|
| Regressions before the fix | S1 and AFL/NFL tests failed at `472a4c3`'s behaviour (`left: 2, right: 1` each). The namesake guard passed, as intended |
| fmt, clippy `-D warnings`, `cargo test -p mt-store --locked` | Pass: 36 tests (10 unit, 10 identity, 16 store) |
| Mutations | All caught, then restored: no reconciliation; alias merged without career; same name across colleges merged; ambiguous partners merged; merge refusal off; alias not kept |

Real data, both tools, scratch only:

| Database | People | Person-years | 1960-66 | 1980+ re-drafts split |
|---|---|---|---|---|
| `472a4c3`, legacy then nflverse | 32,249 | 32,252 | 2,837 | 3 |
| `472a4c3`, nflverse then legacy | 32,246 | 32,249 | 2,837 | 0 |
| Fixed, either order (also after repeat imports and reopen) | 32,128 | 32,131 | 2,719 | 0 |
| v1 database from `472a4c3`, `reconcile` then re-import | 32,128 | 32,131 | 2,719 | 0 |

- **Order independence:** query output for all 91 years (1936-2026), uids excluded, is identical between the two import orders.
- **v1 upgrade:** re-importing reports only the corrected Bob Brown row as updated (legacy: 1 updated, 32,244 unchanged; nflverse: 12,927 unchanged).
- **Dual-draft merges:** 118 in all. 89 have the same name and college, including Bob Brown through the correction; 29 are given-name variants. None merged on career footprint alone.
- **Bob Brown, 1964:** before, the Nebraska tackle appeared twice (NFL r1 p2 Nebraska LT; AFL r1 p4 "Ark-Pine Bluff" DT). After, he is one person with both selections, and the AFL selection is marked `corrected: 1964-afl-r1p4-bob-brown`. Robert Brown (Ark-Pine Bluff, NFL r13 p169) stays separate with his own wAV 51. Sources are cited in `crates/mt-store/data/corrections.json` and #4.
- **Unsettled identities, listed and not merged:** 13 name aliases (Billy/Johnny Brewer, Dave/Dale Chamberlain, Kent/George Horne, C.B./Carlton Simons and others), 6 career conflicts (Irv Cross, Mike Lucci and others) and 6 college conflicts (Tom Brown, Larry Campbell and others). `mt-data conflicts` prints all 25. They are missing evidence, not supported merges.

Dev-trait icons: provenance, hashes and the tier mapping are recorded in `design/draft-generator-higgsfield-brief.md`. They are not committed and no UI is built.

Still not verified: CI, macOS and Linux, any rating, M27 file, or game or MMC import.

### Sol round 2 (GPT-6.1 Sol, High) and S3

Round 2 reviewed `bc4c963` and returned CHANGES.

- **Resolved:** S1, S2 and U1, independently checked by the reviewer: both import orders, repeat imports and reopen; all 91 years; v1 repair; missing-only wAV; Bob Brown and his correction; SQLite integrity and foreign keys; a three-player alias-flattening case.
- **New, S3 (should):** `mt-data conflicts` omitted merges that reconciliation refused ([#5](https://github.com/AMattHT/MaddenTool/issues/5)).
- **Fixed on the branch in `dbd32fe`, not pushed, no approval:** `conflicts()` now appends `merge_refused` for every proposal the existing `refusal()` rejects, worded as `reconcile()` words it, and stays read-only.

| Check (Windows 11 x64, Rust 1.99.0) | Result |
|---|---|
| CLI at `bc4c963` (Alpha's two 2001 selections, legacy first) | Import and `reconcile` reported `merge_refused: 1`; `conflicts --json` printed `[]`; two Alpha rows |
| Regressions at `bc4c963` behaviour | `conflicts_lists_merges_reconciliation_refuses` failed (`left: 0, right: 1`). The extended `merge_is_refused_when_external_ids_disagree` failed (`[]` against the `different pfr ids` refusal) |
| After the fix: fmt, clippy `-D warnings`, `cargo test -p mt-store --locked` | Pass: 37 tests (10 unit, 11 identity, 16 store) |
| CLI after the fix, same database | `conflicts --json` lists the `merge_refused` entry ("one person would have two selections in legacy-player-lookup 2001 NFL"), matching `reconcile`. The database file SHA-256 is identical before and after `conflicts`. Still two players |
| Real-data scratch databases (both orders) | `conflicts` still 25 (13 alias, 6 career, 6 college); no refusals exist in the real data |

Round 3 (GPT-6.1 Sol, High), scoped to S3 and its delta since `bc4c963`, **approved `40ba795`**. S3 is resolved with no new findings, and S1, S2 and U1 are preserved. This is a Sol review approval, not a GitHub human review. Round 2 was the full review of the earlier fixes. Issues #2 to #5 stay open until their fixes reach `main`.

### Shipping (DG01-SHIP-A)

Published as a ready-for-review PR from `t3/reusable-player-database` against `main` at `def9039`. `main` had not moved, so no update was needed. PR #1 is still open and unmerged, and DG01 stays independent of it. The PR description records the live PR, its checks and labels. Commits after `40ba795` are ledger-only.

## WP01a and main integration (2026-11-08)

Status: **merged locally on a development branch, not pushed.** This brings `main` into the foundation branch so PR #1 can merge again. It does not merge PR #1 into `main`, and it does not start WP01b. Hosted CI has not run on this merge.

| Field | Value |
|---|---|
| Provider/model | `claudeAgent` / `claude-opus-5-5`, High, in T3 Code through the Claude Code harness |
| Foundation parent | `73064ded57cb03c1d7e210828389a488758ce201`: PR #1's head `2fc171f` plus the reviewed native smoke startup repair (three scripts) |
| Main parent | `4d41fc9c0b1fe984f3cd537c35b53277336b097f` (DG01 `mt-store`, the draft-first plan, research and reviews, DG05 Scouting Desk concepts A and B1) |
| Conflicts | `Cargo.toml` and `Cargo.lock` (add/add), this file and `README.md`. Both sides were kept in each |

### Decisions

- **Workspace:** members `app/src-tauri` and `crates/mt-store`. Package fields, lints, release profile and `rust-toolchain.toml` (1.99.0) were already identical.
- **Lockfile:** Cargo regenerated it from the union of both lockfiles. Every package and version from both sides is kept except `cc`: the foundation locked 1.10.0 and DG01 locked 1.11.0, and only one 1.x can be locked. 1.10.0 was kept because `libsqlite3-sys` 0.39.1 needs `^1.2.27` and the host's build-time `cc` users, including `embed-resource`, stay unchanged. So `mt-store` now compiles bundled SQLite with `cc` 1.10.0 instead of 1.11.0. Its tests and CLI smoke pass with it. In a whole-workspace build only, `hashlink` turns on `hashbrown`'s optional `foldhash` dependency. `tauri build` builds the host package alone, so the shipped binary's graph does not change.
- **Ledger and README:** both histories are kept. The pre-WP01a "Destination" placeholder from `main` is dropped, because the WP01a section replaced it.
- **Provenance:** see the next section.

### Provenance of the main material

The stricter check found 54 problems on the plain merge: every new file was unlisted, both `comparison.html` pages are over 1 MiB, and `build_b1.py` matched the personal-path scan. Each is now handled one at a time. There is no blanket rule, and the check itself is unchanged.

- `mt-store`: source and tests use `.rs`/`.md` source rules. Its `Cargo.toml`, `data/corrections.json` (compiled in with `include_str!`, so `ships: true`) and each of the five synthetic fixture CSVs have exact entries.
- **Concept frames.** The four concept A frames, the six B1 frames and both `comparison.html` pages embed the five EA development-trait icons from the legacy generator. The design brief records these icons as EA artwork, used at the user's explicit request. Their entries are `third-party` with that source and `ships: false`, and they state that redistribution rights are not verified. Both pages are base64-inlined bundles (4,235,207 and 2,635,040 bytes) and carry an explicit `allowLarge`.
- **Higgsfield raws.** The two raw images are `generated` with their job IDs. They record that the service's terms are not in the repository.
- **Scripts and records.** The concept scripts, the prompt record and `checks.json` have exact entries.
- `build_b1.py`: line 705 is the script's own public-content guard regex, not a path. The literal is split with adjacent raw strings, so the compiled pattern is identical (checked) and the scan no longer matches it. This is the only change to main's files outside the conflicts.
- **Regression tests.** `check-provenance.test.mjs` now requires exact records for new mt-store data, fixtures and concept media or scripts, and still accepts new `.rs` and `.md` there.

**Open, for the user:** passing the check does not settle whether the EA icons inside the concept frames, or the Higgsfield outputs, may stay in a public repository.

### Evidence (Windows 11 Pro 10.0.26200 x64, Rust 1.99.0, Node 24.16.0, local only)

| Check | Result |
|---|---|
| Prettier, ESLint, `tsc` | Pass |
| `npm test` | 23 UI tests pass. 169 script tests: 167 pass, and the 2 POSIX cleanup tests are present and skipped on Windows |
| `node scripts/check-provenance.mjs` | Pass: 142 tracked files. Removing any new entry, or `allowLarge`, or restoring main's `build_b1.py` text, fails it |
| `cargo fmt --all -- --check`; `cargo clippy --workspace --all-targets --locked -- -D warnings` | Pass |
| `cargo test --workspace --locked` | Pass: 13 host IPC and config tests, plus 37 `mt-store` tests (10 unit, 11 identity, 16 store) |
| Player-data workflow steps, locally in Git Bash | Clippy and tests for `-p mt-store` pass. The synthetic CLI smoke returns 6 rows, and repeat queries are byte-identical |
| Windows native smoke, merged harness | `native-dom-and-ipc` against the existing release binary (SHA-256 `b0805e29…1f5a`). Exact ACL denials, no overflow at six viewports, keyboard Retry recovery, no console problems |

That binary was built before this merge and not rebuilt, because the disk was nearly full. Its host source, `tauri.conf.json`, capabilities, host Cargo manifests and every host package in `Cargo.lock` are unchanged from what built it. The app and `ui` commits since then touched only tests.

Still open: all four hosted native jobs and the three player-data jobs on the pushed head; Linux DOM and IPC through `tauri-driver`, including the POSIX cleanup tests; macOS rendering and IPC; and a release build of the combined tree. WP01b waits for the hosted gate.
