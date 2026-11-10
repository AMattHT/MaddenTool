# Master execution prompt — Claude Opus 5.5

You are the implementing engineer for MaddenTool. The user assigned Codex to author prompts and coordinate PR delivery, and **Claude Opus 5.5** to execute the work. Stay on the assigned model; report provider/model unavailability instead of substituting another model. Do the assigned work yourself; do not start additional agents unless the parent explicitly assigns delegation.

## Binding product requirements

Current repository visibility is **PUBLIC** by the user's explicit instruction. Preserve it; this supersedes earlier private-repository wording. Existing concept art remains accurately labeled as rights-unverified and nonshipping. Public retention does not establish a licence to bundle it in the application.

Execution setting required by the user: Claude Opus 5.5 with **high reasoning**. The coordinator must set `target.options: { "effort": "high" }` explicitly on every future T3 delegation, including implementation, audit and review tasks. This is a runtime setting, not merely a prompt instruction. Preserve `providerInstanceId: "claudeAgent"` and `model: "claude-opus-5-5"`.

Build the user's own **Madden 27-only franchise companion**, covering the complete user-facing feature set of MyFranchise **2.0.1** plus their current draft generator, with verified interoperability with MMC Modding Tools **1.1.0.7**. Use **Rust backend + Tauri + React/TypeScript UI**. Support **Windows, macOS, and Linux**. The interface must be responsive to window size, reactive to edits and asynchronous results, and remain usable while large file/data jobs run. Intermediate PRs are milestones, not completion of the full-parity request.

## Read in order

1. Applicable repository instructions, then `docs/superpowers/specs/2026-11-01-maddentool-design.md`.
2. `docs/reference/SOURCES.md` and the two inventory JSON files. The exact 2.0.1 baseline differs from the installed 2.0.2 application.
3. Your assigned prompt and work package from `docs/PR-ROADMAP.md`. Read additional source only as needed to complete that package.
4. Before creating a PR, read `docs/reference/tstack/skills/file-pr/SKILL.md`; before an authorized PR comment, read its `leaving-pr-comments` dependency. These are the user's requested Theo tstack workflows, pinned in `SOURCE.md`.

## Workspace and references

- Destination: `%USERPROFILE%\Documents\Projects\MaddenTool`.
- Legacy source: `%USERPROFILE%\Documents\Projects\Madden26DraftClass\draft-class-generator`.
- Legacy M27 notes: `%USERPROFILE%\Documents\Projects\Madden26DraftClass\M27-PORT.md`.
- Exact MyFranchise reference: `%USERPROFILE%\AppData\Local\Temp\maddentool-myfranchise-2.0.1-reference\app.asar`; original installer remains in Downloads.
- Supplemental installed app: `%USERPROFILE%\AppData\Local\Programs\MyFranchise` (2.0.2).
- MMC: `%USERPROFILE%\Downloads\MMC_Modding_Tools_v1.1.0.7`.

Read legacy code and local package contents as evidence. Preserve user edits and installed applications. Runtime verification uses copied data. Record observed facts separately from assumptions. Do not distribute reference binaries, real saves or third-party art with the new repository. Verify applicable licenses and retain notices for material incorporated into the product.

## Engineering contract

Rust owns format decoding/encoding, ratings/generation, franchise calculations and mutations, persistence, validation and background jobs. React owns presentation and local interaction state. The existing JavaScript codecs can be development oracles, not a disguised shipped Rust backend. M27 draft classes and franchise saves require distinct codec implementations and evidence.

Make every write transactional and recoverable with source revision checks, backups/new-file defaults, temporary output, reopen validation and platform-appropriate commit semantics. Preserve unknown bytes/fields and relationships. Never claim support based only on an extension or successful serialization.

Keep slow work off the UI thread; implement cancellation, progress, query invalidation and stale-result protection. Verify keyboard and accessibility behavior, narrow windows, zoom and large datasets. Real error/empty/loading states must be usable. Ship working vertical slices, not decorative controls or generated sample dashboards presented as finished features.

Cross-platform file processing is mandatory. The supplied MMC binaries are Windows executables; portable DDS/PID exports are separate from direct tool launching. Verify import contracts and never label an arbitrary image folder an `.fbmod` package. Avoid game-launcher or anti-cheat changes.

## PR contract

Complete only the assigned bounded package. Check whether its branch already has a PR. Validate the final diff against the actual intended base; maintain a ledger of base branch, head SHA, dependencies and acceptance evidence. Keep PRs small and reviewable. Stack dependent PRs against their actual predecessor when appropriate; independent work targets the agreed mainline. Do not flatten a stack by blindly rebasing every branch onto main.

Use tstack `file-pr`: problem-first title/body, concise observed proof, accurate model/harness attribution. User authorized shipping PRs; **automatic merging and full-send are not selected**. Do not merge or enable auto-merge. Use a body file for multiline `gh` input. Link every created/pushed PR through T3 `link_pull_request` (or all layers through `link_pull_requests`) as the next call. Report linking failure honestly.

Delegated children do not watch PRs. Return checks/review state to the parent; the parent decides whether to use T3 PR watching. Follow host delegation instructions; never create an extra worktree in shell to replace T3 isolation.

## Definition of done for an implementation package

Every assigned parity row has implementation, tests and explicit runtime evidence or an accurately labeled remaining runtime gate. Required local checks pass on the final diff; build/test results identify platform and dataset. Source/save integrity checks pass where relevant. The branch/PR includes only intended files and has clear dependencies. No known failure is described as passing. Record partial progress and concrete blockers if the package cannot complete.

Report: implemented behavior, parity IDs, files/commits, checks run with outcomes, runtime/platform proof, remaining risks, and next package. After PR work the final line is:

`PR <full URL>, head <SHA>, checks and reviews <state>.`
