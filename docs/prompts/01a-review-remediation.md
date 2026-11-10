# WP01a review remediation: trustworthy foundation gates

Execute with Claude Opus 5.5 and runtime effort `high`. Work on the existing
`feat/cross-platform-foundation` branch and PR
<https://github.com/AMattHT/MaddenTool/pull/1>. Do the implementation yourself;
do not start children or create another checkout. The approved design and stack
remain unchanged. This package strengthens WP01a verification, not WP01b features.

Read applicable instructions, `OPUS-MASTER.md`, the approved design, the foundation
plan, `docs/DELIVERY.md`, and the relevant code. Preserve unrelated edits. The
coordinator created this prompt as an intentional change; include it in the PR.
Use the pinned tstack workflow when updating the PR. Do not merge, enable
auto-merge, watch the PR, poll CI, or change the TypeScript toolchain.

## Starting evidence

Expected starting head: `76ce7666fd1fa39b1010d7c86f3b561cb17f52bb`.
The independent review examined `ea67c8691ea05421733e3f2b739c88ee9231bbe4`.
The origin correction in `76ce766` already selects Tauri's platform-specific
local origin and requires the exact unknown-command rejection in the Rust test.
Review that small diff and preserve it. The stale high-reasoning ledger line was
also corrected there. These are findings 1 and 12 below, not new implementation.
Native CI on the final head remains necessary.

The reviewer found no defect in the app's own startup, timeout, Retry, or stale
result behavior. Its native probes used an existing release binary rather than a
fresh build of the reviewed commit. Distinguish code evidence, synthetic tests,
and actual native observations in the report.

The coordinator independently reproduced finding 4 and the save-prefix/personal
path gaps in finding 8 through the real `evaluate()` function with the existing
tracked-file manifest. All eight probes were incorrectly accepted. The smoke and
test-source weaknesses also match the current code. For remaining mechanisms,
reproduce enough to establish a failing regression before fixing them; do not
claim the reviewer's probes as your own observations.

## Required changes

1. **Previously fixed origin failure (review 1).** Keep the platform-origin and
   exact-error correction. Do not weaken ACLs, skip tests, or revert it. Report
   native results only for the head they actually ran against.

2. **Windows target attachment (review 2, high).** The page-only target lookup in
   `scripts/native-smoke.mjs` accepts `about:blank`. The reviewer observed four
   early attachments where DOM reads worked but Fetch injection did not fire;
   twelve runs waiting past the blank page all injected correctly. Wait for the
   intended local app page, bound the wait, and record the selected URL in failure
   diagnostics. Cover the initial blank target followed by the application target.
   The historical timeout lacked URL diagnostics: describe this as a reproduced
   failure mechanism, not a conclusively identified cause of that earlier run.

3. **ACL assertions (review 3).** The real smoke currently treats any rejection
   as proof of ACL denial. Supply valid arguments to the registered core commands
   being probed, based on the pinned Tauri source, and require the actual ACL
   denial. Keep unknown-command registration rejection distinct from ACL proof.
   Prove that an argument-validation error cannot count as a successful boundary
   check. The Rust capability test must account for every applicable capability
   file, not just `default.json`; ensure another grant cannot silently evade it.
   Check the actual configured capability loading path, including any inline
   configuration. Add a meaningful regression for an additional capability.
   Keep the shipped permissions and CSP narrow; temporary negative probes must
   be reverted and verified. Mock-context IPC remains registration/origin proof,
   not real capability-enforcement evidence.

4. **Provenance catch-alls (review 4).** Current blanket rules for UI, Rust host,
   docs, and reference subtrees classify new material automatically. Constrain
   source rules to reviewed paths/types and explicitly list media/data resources.
   Unknown files under `docs/reference/` must fail even if their extension looks
   like source. Review icons and derived metadata entries as well as the three
   named catch-alls. Require explicit provenance for new assets/data. Add
   regressions showing these synthetic paths fail without a specific approved
   record: `ui/public/madden-27-logo.png`,
   `app/src-tauri/resources/tuning.json`, and
   `docs/reference/myfranchise-extract/index-CqbtHSxo.js`.
   Verify all intentionally tracked files are accounted for. Explain what this
   check can establish; it cannot determine licensing from file contents. Keep
   XP-015's status and evidence honest. If requirements remain unmet, downgrade
   it rather than declaring success. Dependency release notices remain WP11.

5. **Bounded smoke operations and cleanup (review 5).** `until()` currently
   cannot bound a hung probe. CDP has neither per-call timeouts nor rejection of
   pending calls when its socket closes; Linux requests lack abort signals.
   Bound connection, transport, and probe waits as needed. Reject pending CDP
   work on close/error, clear timers/listeners, handle late responses safely, and
   clean up the application/socket/driver/session on both success and failure.
   Demonstrate bounded failure for socket loss and a stalled request without
   waiting for the CI job limit. Avoid a broad harness rewrite beyond what these
   failures require.

6. **Console evidence (review 6).** Include `Runtime.consoleAPICalled` with type
   `error` and `Runtime.exceptionThrown`, alongside existing Log/CSP checks.
   Verify console errors, uncaught exceptions, and unhandled rejections cause a
   failure. Preserve useful diagnostics. Don't suppress unexpected events to
   make the smoke green. Separate deliberate negative probes from the normal
   acceptance run; amend the previous unsupported console-error claim.

7. **Cargo lock enforcement (review 7).** Enforce the committed lockfile for
   clippy, tests, and release builds, including the CI Tauri build. Verify the
   pinned CLI's argument forwarding and document working root commands. Prove
   actual dependency drift is rejected without updating the lockfile. Do not
   upgrade dependencies as part of this package.

8. **Provenance patterns (review 8).** Detect personal absolute paths on Windows
   case-insensitively and in macOS/Linux home layouts. Reject `ROSTER-` and
   `PROFILE-` save names below the size threshold, alongside existing career
   patterns, and add corresponding ignore rules. Use synthetic examples and
   avoid self-matching the scanner or putting real personal paths in Git. Test
   both rejected cases and intentional portable placeholders.

9. **Fetch teardown race (review 9).** Event callbacks currently discard promises
   from `Fetch.continueRequest` and `Fetch.fulfillRequest`. A callback can race
   `Fetch.disable`, become an unhandled rejection, and bypass cleanup. Coordinate
   handler detachment, in-flight work, and disabling so failures are handled and
   the app is cleaned up. Do not simply swallow all errors. Reproduce the race in
   a deterministic test and verify the intended failure/cleanup semantics.

10. **Meaningful stale-result tests (review 10).** Replace or narrow the unmount
    assertion that only checks the already-removed DOM. Add direct cases for a
    timeout followed by a late settlement and Retry while the old call remains
    outstanding, including rejection where relevant. Demonstrate that removing
    the stale-result guard breaks the relevant tests, then restore it. Preserve
    existing useful overlap tests. Change production behavior only if evidence
    reveals a real defect.

11. **Linux prerequisites (review 11).** Include the compiler/linker prerequisite
    (`build-essential` for the documented Ubuntu/Debian setup). Keep documented
    and CI prerequisites consistent. Do not describe a clean-machine test unless
    one was actually run.

12. **Ledger accuracy (review 12 and related findings).** Preserve the corrected
    high-effort record. Update `DELIVERY.md` with each disposition, tests actually
    run, final platform evidence, and unresolved gates. Previous failed/cancelled
    runs are historical evidence, not final-head results.

## Validation and delivery

Use focused regression checks while implementing, then run `npm run check` and a
fresh locked Windows release build. Run the normal native smoke against that
rebuilt binary and sufficient targeted negative probes to prove the changed
guards. Do not run endless success loops. Inspect the diff for unrelated edits,
source-data contamination, temporary mutations, and leftover app processes.
Run parity verification if the ledger changes.

Commit and push the coherent remediation to this PR. Link the PR through T3 as
the next call after pushing, as required by the host. No repeated watchers or CI
polling: take one current status snapshot and return; the coordinator owns the
existing watch. Keep the PR open and unmerged. Update its description only if
needed for the final scope, using a body file and the pinned tstack guidance.

Report finding-by-finding dispositions with concise evidence and the final SHA.
Label unverified items explicitly. All four native jobs must pass on the final
head before trusting cross-platform build evidence. Linux real DOM/IPC facts
remain a required CI gate. macOS process-only smoke does not prove rendering or
IPC; keep that gate open without asking the user to accept it away. High-DPI,
installer signing, and full accessibility keep their existing later-package
scope. The ts-rust evaluation is separate and must not alter this PR's compiler.

Final line: `PR <full URL>, head <SHA>, checks and reviews <state>.`

## Additional CI evidence: Windows on 76ce766

The coordinator inspected completed Windows job `125115572953` in run
`41395383140`, for head `76ce7666fd1fa39b1010d7c86f3b561cb17f52bb`:
<https://github.com/AMattHT/MaddenTool/actions/runs/41395383140/job/125115572953>.
All code gates and the unsigned NSIS build passed. The native smoke failed with:

```text
Native smoke failed: Timed out waiting for the WebView2 debug target: fetch failed
```

This occurs before page selection and is not evidence of the reviewed
`about:blank` attachment race. The log captures neither child output nor its exit
state, so the startup cause remains unestablished. The runner was Windows Server
2025 image `20261021.278.1`, using Node `24.22.1`. Its published software inventory
lists Edge but does not explicitly list WebView2 Runtime; absence from that list
alone does not prove the runtime is missing.

Investigate this within the existing smoke remediation. Check actual runtime
availability and runner launch conditions; add bounded startup diagnostics that
distinguish spawn errors, early exit, missing runtime, and an unreachable debug
endpoint. Record useful child/transport diagnostics without exposing unrelated
environment variables or personal data. If the runtime prerequisite is absent,
make CI provision/verify it through an official supported path. Do not merely
extend the timeout, skip the smoke, or claim a missing-runtime cause without
evidence. Keep the final Windows CI gate pending until a new-head run proves it.
