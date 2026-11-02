# MaddenTool

A desktop franchise and draft companion for **Madden 27 only**, built with a Rust backend and a Tauri + React/TypeScript interface for Windows, macOS and Linux.

**Current state:** foundation only. The app opens a native window and shows version and platform facts from the Rust backend. It cannot open game files yet. The release goal is full MyFranchise 2.0.1 and draft-tool parity, tracked in [`docs/FEATURE-PARITY.csv`](docs/FEATURE-PARITY.csv) and [`docs/PR-ROADMAP.md`](docs/PR-ROADMAP.md). Results and gaps are in [`docs/DELIVERY.md`](docs/DELIVERY.md).

MaddenTool is independent and is not affiliated with EA, MyFranchise or MMC.

## Requirements

- Node 24 (see `.nvmrc`). npm is the package manager.
- Rust through rustup. `rust-toolchain.toml` pins the version and it installs automatically.
- Platform requirements for [Tauri 2](https://v2.tauri.app/start/prerequisites/):
  - **Windows:** Microsoft C++ Build Tools and WebView2 (included with Windows 11).
  - **macOS:** Xcode Command Line Tools.
  - **Linux (Debian/Ubuntu):**
    ```sh
    sudo apt-get install build-essential libwebkit2gtk-4.1-dev libxdo-dev libssl-dev \
      libayatana-appindicator3-dev librsvg2-dev patchelf file
    ```

## Commands

Run from the repository root, after `npm ci`.

| Command | Does |
|---|---|
| `npm run dev` | Runs the native app with hot reload |
| `npm run build` | Builds the UI, the release binary and unsigned installers, refusing a stale `Cargo.lock` |
| `npm run tauri -- build --bundles nsis -- --locked` | The same for chosen bundles. Options before the second `--` go to Tauri and `--locked` after it goes to Cargo, so `npm run build -- --bundles nsis` does not work |
| `npm run check` | Every gate below |
| `npm run format:check` / `lint` / `typecheck` / `test` | Frontend and script gates |
| `npm run provenance:check` | Fails on unlisted files, game data or personal paths ([policy](docs/provenance/README.md)) |
| `npm run check:rust` | `build:ui`, then rustfmt, then clippy and `cargo test` with `--locked` |
| `npm run smoke:native` | Launches the release binary and checks the window, after `npm run build` |

Rust compiles the built UI into the app, so `ui/dist` must exist before running `cargo` directly. Run `npm run build:ui` first.

`smoke:native` differs by platform. Windows drives the real WebView2 window, including IPC, capability denial, console errors, layouts and Retry. It needs the WebView2 Runtime, which Microsoft Edge does not provide; `scripts/ensure-webview2.ps1` is how CI checks for it and installs it only if absent. Linux reads the window through `tauri-driver` under `xvfb` and needs `webkit2gtk-driver` and `cargo install tauri-driver --version 2.1.0 --locked`. macOS only proves the process stays running, because WKWebView has no WebDriver.

## Layout

| Path | Holds |
|---|---|
| `app/src-tauri` | Thin Tauri host, `get_app_info`, capabilities and CSP |
| `ui` | React startup view and the typed IPC wrapper |
| `scripts` | Provenance check and native smoke |
| `docs` | Design, parity ledger, roadmap, delivery ledger and audit evidence |

Later crates (`mt-core`, the format crates, `mt-store` and others) join the Cargo workspace in the package that first needs them. The application version lives only in the root `Cargo.toml`.

## Data rules

Real saves, draft classes, extracted packages, proprietary art and secrets never enter Git. Real fixtures live elsewhere and are found through `MADDENTOOL_FIXTURES`; repository fixtures are synthetic.
