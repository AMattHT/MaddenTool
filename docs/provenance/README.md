# Provenance policy (XP-015)

A private repository is not permission to redistribute. Every tracked file needs a recorded origin and right to ship, and `manifest.json` is that record.

Run `npm run provenance:check`. CI runs it on every platform. It fails when:

- a tracked file matches no manifest entry, or an entry matches no tracked file
- a manifest rule is a catch-all (see below)
- a file's name looks like a real save, roster, profile or draft class, or its type is game data, a reference binary, an archive, a local environment file or key material, even if it is listed
- a text file contains a personal absolute path
- a file exceeds 1 MiB without an explicit `allowLarge`
- third-party material does not name its source

## Entry fields

| Field | Meaning |
|---|---|
| `path` | Exact path or glob. The first matching entry applies, so specific entries come first. |
| `origin` | `original`, `generated`, `derived-metadata` or `third-party`. |
| `source` | Required for `third-party`: where it came from, pinned where possible. |
| `license` | Licence or terms that give the right to use it. `UNLICENSED` means this project's own material, with no licence granted. |
| `ships` | Whether it can reach users in an installed application. |
| `notes`, `allowLarge` | Optional. |

## What may use a wildcard

A wildcard may only cover a source type someone can read in a diff: `.md`, `.ts`, `.tsx`, `.css`, `.rs`, `.mjs`, `.ps1` or `.yml`. The check refuses any other wildcard rule, so media, data and configuration each need an entry naming the exact file. A new icon, JSON resource or capability therefore fails until someone records where it came from.

Nothing under `docs/reference/` may use a wildcard, whatever its extension. An extracted bundle looks like ordinary source, so an unknown file there always fails.

## Adding material

No schemas, art or tuning data from MyFranchise, MMC, the legacy draft tool or EA are imported in WP01a. Before adding any, verify the actual terms for that specific material, because a package's licence label does not establish rights to everything bundled with it. Real fixtures stay outside Git and are reached through `MADDENTOOL_FIXTURES`; repository fixtures are synthetic.

Write paths with placeholders such as `%USERPROFILE%`, `$HOME` or `<you>`. The personal path scan covers Windows paths in any case, and macOS, Linux, Git Bash and WSL home layouts. `Public`, `Default`, `All Users` and `Shared` are not treated as people.

## What this check can and cannot establish

It establishes that every tracked path has a record, that unrecorded new paths fail, and that known save, binary and personal-path patterns are absent.

It cannot determine licensing from file contents. A person writes each record and the check does not verify it. In particular:

- Pasted third-party code or data under a path a source rule already covers, such as `ui/src/tuning.ts`, passes. Only review can catch that.
- A file replaced in place at an exact path, such as a different image at an icon's path, passes. Only review can catch that too.
- Binary files are not scanned for personal paths.
- Save names are matched regardless of case, so a source file named like `roster-table.tsx` fails and needs a different name. Git may also ignore it, because `.gitignore` matches case-insensitively where `core.ignorecase` is set.

## Known limits

This check covers tracked files only. It does not yet produce third-party notices for npm and Cargo dependencies or show them in the app: the About screen is `DG-SHELL-04` (WP01b) and the release gate is WP11. `XP-015` stays `discovered` until those are met.
