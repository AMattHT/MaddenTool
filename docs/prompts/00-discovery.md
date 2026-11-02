# Work package 00 — parity audit and implementation design

Read `docs/prompts/OPUS-MASTER.md`. This assignment is a research/design audit, before product scaffolding or implementation. The user chose Rust + Tauri/React, Madden 27 only, full combined feature parity, and Windows/macOS/Linux. Do not re-ask those choices.

## Tasks

1. Verify the evidence ledger against the local references. Inspect the exact MyFranchise 2.0.1 package and distinguish the installed 2.0.2 version. Use the generated symbol inventories as an index, not a complete requirements specification. Follow routes, menus, dialogs, settings and conditional tools. Trace current draft routes/services/components beyond its README.
2. Create `docs/FEATURE-PARITY.csv`. Account for every MyFranchise route, component symbol and IPC channel, plus each draft route and feature-bearing component/service, either in a feature row or an explicit shared/dead/unresolved-symbol mapping. Group technical symbols behind user-visible capabilities, but separate actions with different outcomes. Every feature gets provenance and a verifiable acceptance case. Identify runtime-unverified behavior honestly; never infer a working feature solely from a symbol.
3. Audit M27 format feasibility: draft codec, franchise container/schema, reference integrity, compression, contracts and unknown fields. Establish available local fixtures, their provenance and privacy, and proposed differential tests. Read existing M27 tests/verification scripts. Resolve the 402 active-player versus 455 physical-block distinction from evidence. Inventory asset/data licensing before proposing redistribution.
4. Audit MMC integration. Trace DDS encoding, portrait-ID selection and collision handling in the draft tool; inspect the supplied MMC docs and plugin evidence. Specify an actual validation procedure for portrait import and mod application. Mark direct APIs/CLIs unverified unless observed. Define macOS/Linux export workflows independently of Windows MMC launching.
5. Review the proposed design as a builder who must implement without access to the author. Identify only unanswered questions whose answer changes scope/architecture. Settle ordinary implementation details yourself; document reasoning. Explicitly cover responsive/reactive UI behavior, background job lifecycle, save safety and all three platform build/runtime gates.
6. Write `docs/DISCOVERY-REPORT.md`, propose evidence-backed edits to the design, and refine `docs/PR-ROADMAP.md`. Include concrete Rust/Tauri boundaries, feature dependencies, testing strategy and the first useful end-to-end slice. Preserve the complete product scope. Mark the design ready for user review if no major ambiguity remains.

## Output boundary

This work package writes documentation and audit artifacts only in MaddenTool. Do not scaffold product code, install product dependencies, create a remote repository, edit legacy source, overwrite game saves, or launch the installer. No PR is required for this initial audit in the currently uninitialized destination. Do not spawn children. Return the completed file paths and only consequential open questions to the parent.

Completion: all discovery inputs are accounted for, parity rows are actionable and traced, format/MMC uncertainties are explicit, the roadmap covers the union of both products, and the concrete design is ready for review. If runtime inspection cannot be completed, identify exactly which rows lack runtime proof rather than claiming an exhaustive behavior audit.
