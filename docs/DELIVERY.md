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
| Parity rows | `XP-015` moved to `implemented`. No other row changed. |

### Baseline notes

- The audit found absolute personal paths in prompts, sources, two inventories and three probes. They now use `%USERPROFILE%` or environment variables.
- The first push of `main` (`3b34a6e`) still had such paths in the two inventories, because of a failed replacement. Minutes later, `main` was amended and force-pushed once to `def9039`, with the lease pinned to that sole commit, with no forks or other branches. The old SHA may stay reachable by direct SHA on GitHub until it is garbage collected. It held local paths and a username, no secrets or game data.
- Inventories and audit JSON are derived names, routes and hashes. No reference code, schema, asset, save or extracted bundle was committed.

### Observed results

Local host: Windows 11 Pro 10.0.26200 x64, Rust 1.99.0, Node 24.16.0, npm 11.13.0. No game data was used.

| Check | Result |
|---|---|
| `npm run check` on the working branch | Pass: Prettier, ESLint, `tsc`, 17 UI tests, 19 script tests, provenance over all tracked files, rustfmt, clippy `-D warnings`, 5 Rust IPC and config tests |
| Mutation: remove the UI stale-result guard | Both overlapping-attempt tests fail, so the guard is really tested |
| `npm run build` | Pass: release binary, NSIS and MSI, unsigned |
| Provenance check with planted `CAREER-PLANTED`, `ui/app.asar`, an unlisted file | Fails with exit 1 and names each file |
| `npm run dev` | Real window from `http://localhost:1420` shows the same native values. One favicon 404 is logged in dev mode |

Real-window proof on Windows, through `npm run smoke:native` against the release binary (WebView2 DevTools protocol):

- The window shows Madden 27, version 0.1.0, windows and x86_64 from the real `get_app_info` IPC call.
- `read_file`, `plugin:window|set_title` and `plugin:webview|create_webview` are all rejected: `not allowed by ACL`.
- 800×600, 1024×768, 1440×900 and 1920×1080, plus 800×600 and 1024×768 at 200%: no horizontal overflow or clipped text. No console errors or CSP violations.
- A malformed first IPC response reaches the error state through the real wrapper. Retry is reachable at 800×600 at 200%, and Enter recovers through real IPC.
- `scripts/window-sizes.ps1` resized the real window to 800×600, 1024×768 and 1440×900. Screenshots were inspected, which found and fixed an empty grid cell at 800×600.

The application process was gone after each run. Screenshots stay outside Git under `.smoke/`.

Fresh checkout: a temporary clone, separate from this working copy and using no reference project or Downloads, passed `npm ci`, `npm run check`, `npm run build -- --bundles nsis` and three native smokes.

One earlier fresh clone built and passed its gates, but its smoke failed once with `Timed out waiting for the injected error state`. The cause was **not established**. It did not recur in 15 later runs: 6 before any change, 6 after narrowing the injection to the `get_app_info` request, and 3 in the second fresh clone. The smoke now reports IPC and page state on that timeout. Treat a recurrence in CI as a real signal, not as noise.

### Platform coverage

CI definition, passing builds and runtime proof are separate, so they are listed separately. "Defined" means the job exists in `.github/workflows/ci.yml`; it does not mean it has passed. Check the PR for results.

| Platform | Runner label | CI | Build seen passing | Runtime proof |
|---|---|---|---|---|
| Windows x64 | `windows-2025` | Defined | Locally | Local real window, full level above |
| Linux x64 | `ubuntu-24.04` | Defined | Not yet observed | None yet. CI smoke reads the DOM through `tauri-driver` under `xvfb` |
| macOS arm64 | `macos-15` | Defined | Not yet observed | None yet. CI smoke is process-start only, because WKWebView has no WebDriver |
| macOS x64 | `macos-15-intel` | Defined | Not yet observed | None yet. CI smoke is process-start only |

### Limits and remaining gates

- Linux and macOS have never run this code, and the workflow itself had not run when this was written.
- The macOS smoke cannot show that the window renders or that IPC works. That needs a real Mac or a later automation approach.
- 200% is emulated through DevTools (half the CSS pixels at scale factor 2), not Windows display scaling. High-DPI display scaling is not proven.
- Windows did not enforce the configured 800×600 minimum against a direct `SetWindowPos` to 640×480. Interactive drag-resize against the minimum has not been tried.
- The retry proof uses a malformed response at the IPC layer. It does not exercise a real backend crash, and the 10-second timeout is only unit tested.
- Installers were built but not installed. Packages and signing are WP11.
- `npm audit` reports 12 moderate advisories, all in the dev-only lint stack through `minimatch`. `npm audit --omit=dev` reports 0. TypeScript is held at 6.0.3 and ESLint at 9.39.5 for lint plugin compatibility.
- Four native jobs run on each PR update. macOS minutes are billed at a high multiplier on private repositories, so watch the Actions allowance.
- `docs/prompts/OPUS-MASTER.md` had an uncommitted coordinator edit in the shared checkout during this work. It is deliberately not part of this PR.

### Next: WP01b dependencies

WP01b can stack on this branch or start from `main` after merge. It needs:

1. Green Linux and macOS jobs on this PR.
2. New capabilities for its file dialog and settings commands, kept narrow.
3. `mt-core` for typed errors and the job lifecycle, which is the first new crate.
4. An accessibility scan and viewport sweep, extending `scripts/native-smoke.mjs` for the shell. The macOS gap above needs a decision here.
