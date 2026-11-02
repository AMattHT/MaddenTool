# WP01a execution prompt: repository and cross-platform foundation

Execute only after the coordinator records acceptance of the reviewed design. Read `OPUS-MASTER.md`, the design, `DISCOVERY-REPORT.md` section 11 and `PR-ROADMAP.md` WP01a. Implement with Claude Opus 5.5; do not delegate further.

## Fixed decisions

- Destination: `%USERPROFILE%\Documents\Projects\MaddenTool`.
- Repository: private `AMattHT/MaddenTool`; GitHub API verified the authenticated owner as `AMattHT`, ID `262685875`. Verify current identity and repository availability again before creation. Preserve any existing remote or unrelated content and report a conflict instead of overwriting it.
- Rust authoritative backend; stable Tauri 2; React/TypeScript UI. Windows, macOS arm64/x64 and Linux desktop targets. Madden 27 only, with the entire parity ledger remaining in scope.
- Full MyFranchise/draft parity is the eventual release gate. This package is the foundation, not that release.

## Implement

1. Inspect the destination and applicable instructions. Record the accepted design revision in `docs/DELIVERY.md`. Establish Git with `main` and a minimal baseline containing reviewed project documentation and ignore rules. Local fixture paths, archives, third-party bundles, raw reference extracts, build caches, user saves and secrets stay outside commits. Audit the existing documentation/probe artifacts before adding them.
2. Create/verify the private remote and push the minimal baseline. Put all product changes on `feat/cross-platform-foundation` based on that baseline. Do not push product changes directly to main.
3. Establish the Rust workspace and Tauri/React build boundary. Create only crates needed by this slice; document later crate boundaries rather than filling empty modules with placeholders. Include a functioning native window, a typed backend health/version command and a simple truthful startup view. Verify the real native command path; this is not a fixture-only web mockup.
4. Establish formatting, linting, typechecking, meaningful smoke tests, lockfiles and CI builds for Windows, macOS and Linux using available native runners. Include schema/provenance metadata scaffolding for XP-015 without copying reference data. Explain missing platform runtime proof accurately.
5. Keep app identity independent of MyFranchise. Configure narrow Tauri capabilities and a CSP appropriate to local content. The shipped backend must not depend on Node, Electron or Express. Keep React layouts fluid from the start; full UI/job-shell acceptance remains WP01b.
6. Run relevant Rust/frontend checks and a local native launch/command smoke. Inspect the intended diff. Update `docs/DELIVERY.md` with commands, results, platform coverage and remaining gates. Add the package's implemented evidence to the parity ledger only where earned.
7. Read the pinned tstack `file-pr` skill. Create a concise open PR against `main`, with problem/behavior, observed verification and model/harness attribution. Immediately link it using T3. Leave it unmerged and return its current checks/reviews state; the parent owns monitoring.

## Acceptance

A fresh checkout has documented build commands, dependency lockfiles, a real Rust-backed Tauri/React window and native-platform CI definitions. Available checks pass on the final head. Unavailable platform checks are explicit, not labeled passing. Private remote, branch/base and PR identity are correct; no real data/reference binaries enter Git. Working copy and installed reference tools remain unchanged.

No game parsing or franchise editing is required in this package. Report the next WP01b dependency, not a completed MaddenTool product. Final line: `PR <full URL>, head <SHA>, checks and reviews <state>.`
