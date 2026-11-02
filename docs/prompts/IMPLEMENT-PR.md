# Per-PR implementation prompt

The coordinator fills in the assignment fields before delegation. Read `docs/prompts/OPUS-MASTER.md`, the reviewed design, and the named work package. This is an implementation assignment for Claude Opus 5.5, not a request for another broad plan.

## Assignment fields

- Work package and parity IDs: supplied by coordinator.
- Destination repository and permitted branch: supplied by coordinator after repository setup.
- Base ref and predecessor PR: supplied by coordinator; verify both before editing.
- Accepted design revision and any accepted deviations: supplied by coordinator.
- Local/runtime acceptance cases: the package's acceptance criteria plus affected parity rows.

Implement the smallest complete vertical slice satisfying this assignment. Preserve unassigned work and reference applications. Use Rust for authoritative backend logic, typed Tauri boundaries and React for the interface. Run relevant tests and checks on the final diff, including actual UI/native-path proof for changed user-facing behavior. Verify malformed inputs, failure recovery and save invariants where applicable.

Update the parity ledger with actual evidence and the delivery ledger with base/head/dependencies. Read the pinned tstack file-pr skill and create or update the PR without merging. Link the PR through T3 immediately after creating or pushing it. Report CI/review status once without waiting loops or child-owned watches. Return blockers precisely; do not mark an incomplete feature complete.

If repository/base fields are missing, report that prerequisite to the coordinator instead of pushing to the old draft-generator repository or inventing a remote. Do not create or migrate repository identity as an incidental feature change.

Final line: `PR <full URL>, head <SHA>, checks and reviews <state>.`
