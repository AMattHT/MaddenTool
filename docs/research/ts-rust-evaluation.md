# Tooling evaluation: ts-rust (`tsc-rs` 0.8.0)

Date: 2026-11-01. Evaluated at MaddenTool commit `ea67c86`. No package manifest, lockfile, tsconfig or CI file was changed.

## Decision

**Defer. Do not add `typecheck:rust` now. Keep `tsc --noEmit` (TypeScript 6.0.3) as the required gate.** Do not enable `--fast`.

- **Compatibility was not the blocker.** `tsc-rs` 0.8.0 matched its pinned upstream TS7 byte for byte in 32 of 32 cases on Windows x64.
- **The benefit is too small for the cost.** It saves about 0.4 s against a check that already takes 0.64 s, or 1.4 s through `npm run typecheck`. In exchange we would add a third-party native binary, a postinstall script and a second compiler generation that can disagree with the required gate.
- **It does not fill the role that would justify a second checker.** First-party TS7 is the better forward-compatibility preview, and was faster here on wall clock.

Anyone who wants a TS7-generation check today can run this from the repository root, with no repository change. Tested: exit 0 on the project, exit 2 on an invalid fixture, manifests unchanged.

```sh
npx --yes tsc-rs@0.8.0 --noEmit -p ui/tsconfig.json
```

## Can it build or TypeScript-test MaddenTool?

| Role | Answer | Basis |
| --- | --- | --- |
| Build the app | No | Bundling is `vite build` and native builds are Cargo/Tauri. `tsc-rs` is a type checker and JS emitter, not a TS-to-Rust tool. The only compiler step in our build is `tsc --noEmit`. |
| Run tests | No | Its scope is compiler, checker and language server, with no test runner. Vitest and `node --test` stay. |
| Type check | Yes, technically | Results below. |
| Replace the `typescript` package | No | `typescript-eslint@8.72.1` declares the peer range `>=4.8.4 <6.1.0`, which excludes the `@tsc-rs/typescript` alias at TS7. |

## Versions compared

Published 0.8.0 and current `main` have different upstream pins. Only the 0.8.0 pin was used, and no `main` conformance figure applies to these results.

| | Version | Upstream | Evidence |
| --- | --- | --- | --- |
| Required gate | `typescript@6.0.3` | gitHead `050880ce59e3` | `npm view` |
| Tool under test | `tsc-rs@0.8.0` (npm `latest` on the test date) | Pin `fed0bf24149f`, 2026-10-08, TS 7.1.0-dev. Tool gitHead `4c032be6dbef` | [v0.8.0 README](https://github.com/pingdotgg/ts-rust/blob/v0.8.0/README.md), [UPSTREAM.md](https://github.com/pingdotgg/ts-rust/blob/v0.8.0/UPSTREAM.md) |
| Oracle | `typescript@7.1.0-dev.20261008.1` | gitHead `fed0bf24149f`, the same commit, verified with `npm view` | [upstream commit](https://github.com/microsoft/TypeScript/commit/fed0bf24149fb1ed36039212648bafdafc1ea10e) |
| Not used | ts-rust `main`, 882 commits past the tag | Pin `3e9236361fa9`, TS 7.1.0-beta | [main README](https://github.com/pingdotgg/ts-rust/blob/main/README.md) |

`tsc-rs --version` prints `Version 7.1.0-dev`, not 0.8.0, as its README documents. The tool version has to be read from `package.json`.

## Method

Compilers were installed outside the repository with install scripts disabled:

```sh
npm ci                                  # in the worktree; manifests hashed unchanged
npm install --ignore-scripts --save-exact tsc-rs@0.8.0                       # disposable dir
npm install --ignore-scripts --save-exact typescript@7.1.0-dev.20261008.1    # disposable dir
```

All runs used `--noEmit`, with `cwd=ui`. Corpus runs added `--pretty false -p <case>/tsconfig.json`, and each case config extended the real `ui/tsconfig.json`. Fixtures lived in a temporary `ui/tsrs-eval-tmp/` directory, since removed.

## Results

### Real project: `ui/tsconfig.json`

| Command | Exit | Diagnostics |
| --- | --- | --- |
| `npx tsc --noEmit` | 0 | 0 |
| `npm run typecheck` (root) | 0 | 0 |
| `tsc-rs --noEmit` | 0 | 0 |
| TS7 oracle `tsc --noEmit` | 0 | 0 |

A clean run is weak evidence alone, so these were also confirmed:

- All three compilers loaded the same 258 program file paths, including our 7 sources. Paths under each compiler's own `lib` directory were normalized first. `tsc-rs` and the oracle hashed the same on the three `lib` files sampled; TS6 ships different content for two of them.
- `--showConfig` output from `tsc-rs` and the oracle was identical.
- `tsc-rs` and the oracle reported the same Files, Lines, Identifiers, Symbols, Types and Instantiations counts.

### Deliberately valid and invalid corpus: 32 cases

| Group | Cases | `tsc-rs` diagnostics |
| --- | --- | --- |
| Valid (IPC DTOs, React 19 props and hooks, narrowing, Vitest and jest-dom types, correct `@ts-expect-error`) | 5 | 0 |
| IPC DTO errors, including cross-file DTO drift | 2 | 19 |
| React props and TSX | 1 | 14 |
| Nullability and strict checks | 1 | 13 |
| `noUncheckedIndexedAccess` | 1 | 9 |
| `exactOptionalPropertyTypes` | 1 | 7 |
| `noUnused*` and `noFallthroughCasesInSwitch` | 1 | 11 |
| `@ts-expect-error` misuse | 1 | 5 |
| Module flags, syntax, and error-text elaboration | 3 | 19 |
| Port hazards: UTF-16 columns with emoji, CRLF, BOM, spaces in paths, Windows casing | 2 | 9 |
| Mutated copies of our real `ui/src` | 2 | 4 |
| Config errors | 7 | 9 |
| TS6-to-TS7 gap probes | 4 | 5 |
| All invalid groups in one program, run separately | 1 | 101 |

| Comparison | Exit code | Full output bytes |
| --- | --- | --- |
| `tsc-rs` vs TS7 oracle | 32 of 32 equal | **32 of 32 equal** |
| `tsc-rs` vs `tsc` 6.0.3 | 30 of 32 equal | 20 of 32 equal |

Other checks, all matching the oracle:

- **Exit codes:** 0 when clean, 2 on diagnostics (also under `--noEmit`), 1 on a missing project path. The native exe, Node launcher, `.bin` shim, `.cmd` shim from PowerShell and `npx` all propagated exit code 2.
- **Columns:** UTF-16 based. The fixture's expected column 41 was reported; code points would give 38 and UTF-8 bytes 47.
- **Determinism:** 10 runs of the 101-diagnostic program gave one output hash, the same for `tsc-rs` and the oracle.
- **`--checkers 8`:** same bytes as the default run and as the oracle at `--checkers 8`.
- **`--fast`:** same bytes as the default run on one run.
- **Incremental edit sequence:** clean, no change, DTO changed, rerun, fixed gave exits 0, 0, 2, 2, 0 on both. Errors did not vanish on the rerun. We do not use `incremental`; this was checked because 0.8.0 changed that path.
- **Formats:** default and `--pretty true` output matched.

**Corrections to expectations.** Expected `line:code` markers were written by hand before any compiler ran. Ten were then changed or moved, across 7 cases. Eight were my mistakes, with all three compilers agreeing against the marker; for example, `@ts-expect-errors` is a prefix match and object-rest siblings are exempt from unused checks. Two exposed a real TS6-to-TS7 difference and moved to the gap cases. After that, all markers matched on all three compilers.

### TS6 differences are version gaps, not port bugs

All 12 cases that differ from TS6 also differ in the same way on the oracle, so none is attributable to the port. Two change the **exit code**:

| Case | `tsc` 6.0.3 | TS7 oracle and `tsc-rs` |
| --- | --- | --- |
| `@ts-expect-error` above a multi-line `import { ... }` with an unused specifier | exit 0 | exit 2: TS2578 and TS6133. TS7 reports on the specifier line, not the declaration line, so the directive no longer covers it. |
| `baseUrl` with `ignoreDeprecations: "6.0"` | exit 0 | exit 2: TS5102, option removed |

The other differences do not change pass or fail:

- Unused diagnostics change code, column or count. `const [c, d]` gives two TS6133 on TS6 and one TS6198 on TS7. `import type` gives TS6133 on TS6 and TS6196 on TS7.
- Union and property order in messages changes, and overload error elaboration differs.
- Removed options report TS5102 or TS5108 where TS6 reported TS5101 or TS5107.
- `config-bad-json` gains one extra TS1136.

**Consequence:** a second gate on this generation can fail while the required gate passes, and that failure would describe TS7 rather than a defect in our code. Our code has no exposure today: `git grep` finds no `@ts-expect-error`, `@ts-ignore`, `baseUrl` or `ignoreDeprecations` in the source or tracked config.

### No emit, and existing commands

- **`--noEmit`:** SHA-256 of every file excluding `node_modules` and `.git` (82 files, all tracked) was identical before and after the baseline runs. No `.js`, `.map`, `.d.ts` or `.tsbuildinfo` appeared after the corpus. Omitting the flag also emitted nothing, because `ui/tsconfig.json` sets `noEmit`.
- **Existing gates, rerun after removing fixtures, all exit 0:**
  - `npm run typecheck`
  - `npm test` (17 Vitest tests and 19 script tests)
  - `npm run build:ui`
  - `npm run lint`
  - `npm run format:check`
  - `npm run provenance:check`
- **Not rerun:** `check:rust` and `tauri build`, since no Rust or Tauri input changed.

### Timing

Windows 11 Pro 10.0.26200, AMD Ryzen 7 9800X3D (8 logical processors exposed), 31.1 GiB RAM, Samsung 970 EVO Plus, Node 24.16.0, npm 11.13.0. Defender real-time protection reported off. The machine was **not idle** (24 to 32% CPU sampled). Times are round-robin wall clock over the real project, with one warmup.

Launcher layers, median of 15:

| Command | Median | Min to max |
| --- | ---: | ---: |
| Node startup only | 64 ms | 61 to 66 |
| `npx --version` only | 363 ms | 352 to 379 |
| `tsc` 6.0.3 via `node lib/tsc.js` | 666 ms | 644 to 829 |
| `npx tsc --noEmit` | 1408 ms | 1390 to 1567 |
| `npm run typecheck` (root) | 1418 ms | 1400 to 1523 |
| TS7 oracle, native exe | 130 ms | 124 to 147 |
| `tsc-rs`, native exe | 219 ms | 216 to 403 |
| `tsc-rs`, Node launcher | 293 ms | 284 to 556 |
| `tsc-rs`, `.bin` `.cmd` shim | 338 ms | 333 to 389 |
| `npx tsc-rs` | 1030 ms | 1006 to 1070 |
| `tsc-rs --fast`, native exe | 218 ms | 213 to 236 |

Process cost vs compiler time, median of 21:

| | `--version` wall | Check wall | Self-reported total | Self-reported check |
| --- | ---: | ---: | ---: | ---: |
| `tsc` 6.0.3 | 86 ms | 642 ms | 520 ms | 150 ms |
| TS7 oracle | 74 ms | 124 ms | 48 ms | 15 ms |
| `tsc-rs` | 173 ms | 211 ms | 37 ms | 10 ms |

Reading these numbers:

- `tsc-rs` beats TS6 by about 430 ms directly, but **took longer than first-party TS7 on wall clock here**. Almost all of that gap appears before any checking, in the `--version` run. The cause was not investigated.
- `--fast` gave no gain, as its README predicts for small projects.
- This is one 7-source program on one machine. These numbers do not confirm or refute upstream's benchmarks, and upstream's speedups should not be assumed to apply here. Larger projects may scale differently.

## Constraints and risks

- **Platform coverage:** npm publishes Windows, Linux and macOS packages for x64 and arm64, which covers all four CI runners by metadata. **Only Windows x64 was executed.** The POSIX postinstall rewrites the launcher, and that path was not exercised.
- **Not evaluated:** emit, `tsc -b`, the language server, the JS API, and the WASM build.
- **Corpus limits:** the corpus is targeted, not exhaustive, and our program is small. The result shows good compatibility on our patterns, not general equivalence.
- **Upstream maturity, from the [v0.8.0 README](https://github.com/pingdotgg/ts-rust/blob/v0.8.0/README.md):** "This is an early release", "I've never read a line of this code", "Be warned, I have no idea if this will actually work". It is labelled experimental. Releases 0.2.0 to 0.8.0 shipped in 12 days.
- **Supply chain:** `npm audit signatures` verified registry signatures and attestations for both installed packages. The win32-x64 `tsc.exe` has **no Authenticode signature**, while the Microsoft oracle's `tsc.exe` is validly signed. The package is about 52 MB unpacked per platform and has a `postinstall` that `--ignore-scripts` skipped here. Reading it found it only rewrites the launcher on POSIX and does nothing on Windows.
- **Moving target:** the pin is a 7.1 dev snapshot, not stable TS (`latest` is 7.0.2, which was not tested). Each `tsc-rs` bump changes the upstream it mirrors and needs its matching oracle.
- **Cosmetic:** `--extendedDiagnostics` reports `Memory used: 0K` from `tsc-rs` on Windows; the oracle reports 61290K.

## Revisit when

1. The required typecheck reaches about 10 s on the slowest CI runner (a judgment threshold; about 0.6 s of compiler time locally today). Compare first-party TS7 first.
2. `typescript-eslint` accepts TS 7 and a migration is planned. Fix the two exit-code gaps above as part of that work.

## Integration plan, if a trigger fires

Use a small, separate PR:

1. Add `tsc-rs` at an exact version to `ui/package.json` and record the matching oracle version next to it. Keep `typescript` untouched and do not use the `@tsc-rs/typescript` alias.
2. Add `"typecheck:rust": "tsc-rs --noEmit"` in `ui` and a root forwarder. Keep it **out of** `check` and `build`.
3. Before relying on it, run the invalid-case corpus on all four CI runners, including the POSIX launcher path. Any CI job must be non-required (`continue-on-error: true`).
4. Triage any disagreement with `tsc` by rerunning on the matching TS7 oracle. Same result means a TS7 version gap: decide on the code or config. Different result means a port bug: report upstream, and do not change code to satisfy it.
5. Do not enable `--fast`, `--checkers` or `GOPORT_*` switches. `--fast` output depends on core count where upstream's does.
6. Making it the required gate is a separate decision, needing sustained multi-platform agreement and a real time saving.

## Primary sources

- Repository and tag: [pingdotgg/ts-rust](https://github.com/pingdotgg/ts-rust), [release v0.8.0](https://github.com/pingdotgg/ts-rust/releases/tag/v0.8.0)
- Tagged docs: [README](https://github.com/pingdotgg/ts-rust/blob/v0.8.0/README.md), [UPSTREAM.md](https://github.com/pingdotgg/ts-rust/blob/v0.8.0/UPSTREAM.md), [UPSTREAM.json](https://github.com/pingdotgg/ts-rust/blob/v0.8.0/UPSTREAM.json)
- npm: [tsc-rs 0.8.0](https://www.npmjs.com/package/tsc-rs/v/0.8.0), [@tsc-rs/win32-x64 0.8.0](https://www.npmjs.com/package/@tsc-rs/win32-x64/v/0.8.0), [typescript 7.1.0-dev.20261008.1](https://www.npmjs.com/package/typescript/v/7.1.0-dev.20261008.1)
- Package contents: `lib/install.js`, `lib/tsc.js` and `lib/getExePath.js` inside the `tsc-rs@0.8.0` tarball
- Peer range: `npm view typescript-eslint@8.72.1 peerDependencies`
