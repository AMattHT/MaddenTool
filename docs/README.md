# MaddenTool build brief

This packet directs Claude Opus 5.5 to build a Madden 27 franchise companion with the combined features of MyFranchise 2.0.1 and the user's existing draft generator. Codex prepares prompts and coordinates delivery; Opus implements the application.

## Start here

1. Read [the product design](superpowers/specs/2026-11-01-maddentool-design.md).
2. Give Opus [the master prompt](prompts/OPUS-MASTER.md), then [the discovery prompt](prompts/00-discovery.md).
3. After the design is reviewed, execute the individually bounded work packages in [the PR roadmap](PR-ROADMAP.md), using [the implementation prompt](prompts/IMPLEMENT-PR.md) for each package.

The selected architecture is a Rust backend with Tauri and a responsive React/TypeScript interface. Windows, macOS, and Linux are required. Madden 27 is the only game target. An internal milestone is not the requested full-parity release.

## Evidence and workflow

- [Source notes](reference/SOURCES.md) distinguish observed facts from unverified behavior.
- [Audit tools](reference/audit/) regenerate and verify the ledger.
- [ts-rust evaluation](research/ts-rust-evaluation.md) records the compiler comparison and the decision to retain the existing required TypeScript check.
- [MyFranchise inventory](reference/myfranchise-2.0.1-inventory.json): 104 IPC channels, 365 component symbols, 51 route strings extracted from the exact 2.0.1 installer. These counts are discovery aids, not a claim of 365 independent features.
- [Draft tool inventory](reference/draft-tool-inventory.json): 65 route declarations plus React component and service names.
- [tstack reference](reference/tstack/SOURCE.md): selected upstream skills pinned to a commit with the license retained. Use `file-pr` for shipping PRs; use `babysit-pr` only when that work is assigned. Full-send is not invoked.

## Current status

Work package 00 is complete. Read [the discovery report](DISCOVERY-REPORT.md), then [the parity ledger](FEATURE-PARITY.csv). The user approved the reviewed design and instructed implementation to begin. Operational questions are resolved in report section 11. WP01a is the first implementation package; see [the execution plan](superpowers/plans/2026-11-01-foundation.md) and [delivery ledger](DELIVERY.md). WP01a's foundation is under review: see the delivery ledger for its PR and evidence. The legacy project and installed applications were inspected without modifying them.
