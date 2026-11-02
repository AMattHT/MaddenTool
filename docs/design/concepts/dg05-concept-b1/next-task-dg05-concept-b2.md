# Next task: DG05-CONCEPT-B2, board states (prompt draft)

Act as the design sub-agent for MaddenTool design goal **DG05-CONCEPT-B2**. Work in your prepared isolated checkout only. Base it on current `origin/main`, which must include DG05-CONCEPT-B1, and record the actual base.

**Outcome.** Static design frames of direction A, the Scouting Desk (a table with a docked player inspector, selected by the user), in the board states that B1 left out. This is one bounded design package, not the app. Keep A's structure and B1's tokens, column priority, and focus and selection treatment. Do not reopen A vs B, the architecture, or the open preferences (default lens, Career cap, 389 vs 402, seed visibility, window arrangement, product name).

**Deliverables.** All at 1440×900 in light unless noted. Add one dark frame wherever the state uses colour.

1. **Empty filter result.** A search or filter that matches nobody.
   - Say why, show the active filters, and offer a one-step clear.
   - The three counts still show the class, not the filter result.
2. **Overflow panel.**
   - The players dropped beyond the 402 export limit, and why each one was dropped.
   - Keep active, the 402 export limit (in-game import still unverified) and the 455 capacity as three separate numbers.
   - Use an illustrative class whose size is labelled; do not claim a real class size you have not queried.
3. **2026 class with "No completed season".**
   - The evidence state "No completed season" is distinct from Missing and from Observed 0.
   - The rating is shown as estimated, and nothing says "in progress".
   - Spoilers-on and spoilers-off variants.
4. **Column review flow.**
   - The column chooser: the B1 priority, hidden columns, reset, and the long-College truncation found in B1.
   - At 1024×768 as well.
5. **Identity review flow.**
   - Resolving an unsettled identity from the "Needs review" group: Jack Adams / Jake Adams (Virginia Tech) and Bob Young × 2.
   - Show the evidence and the choices: keep separate, merge, or defer. Never merge silently.
   - Make clear that the frames do not perform a merge.
6. A comparison HTML, standalone PNGs, reproducible scripts (extend `dg05-concept-b1/source/build_b1.py` rather than fork it), a decision record, and a local inline page outside Git for `html_preview` then `html_render`.

**Data and publication rules (unchanged from B1).**

- Real data is limited to identity, draft selections, colleges and the corrections.
- wAV, evidence and career numbers are labelled ◇ sample inputs; ratings are illustrative. Use no real stat-derived values anywhere public.
- For 2026, query the identity data locally. Do not invent prospects presented as real; label any filler as sample.
- Spoilers off means absent, including from tooltips, accessible names, sorts and filters.
- Use the original five dev icons, verified against the brief's hashes and never redrawn, with their text labels.
- No account state, no private paths, and no new paid image generation unless a genuinely new style asset is needed.

**Visual proof.**

- Inspect each frame at its native size.
- Reuse B1's in-page measurement for visible rows, clipping and DOM contrast.
- Check the determinism of the rebuild.
- Label the keyboard behaviour as a state concept unless it is actually exercised.

**Out of scope.**

- Profile and editor, data and sources, setup, generation, Class Studio, open, validate and export.
- Any app code, codec, ratings or data change; PR #1; PR #9.

**Handoff.**

- Commit as AMattHT (author and committer, `262685875+AMattHT@users.noreply.github.com`).
- Do not push, open a PR or merge before a fresh GPT-6.1 Sol High review.
- Report the checkout, branch, base and head, the checks actually run, the paths, the assumptions and the gaps.
