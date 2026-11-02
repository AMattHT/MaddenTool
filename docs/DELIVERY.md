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
