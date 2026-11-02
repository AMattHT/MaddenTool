# Provenance policy (XP-015)

A private repository is not permission to redistribute. Every tracked file needs a recorded origin and right to ship, and `manifest.json` is that record.

Run `npm run provenance:check`. CI runs it on every platform. It fails when:

- a tracked file matches no manifest entry, or an entry matches no tracked file
- a file looks like a real save, game data, a reference binary, an archive, a local environment file or key material, even if it is listed
- a text file contains a personal absolute path
- a file exceeds 1 MiB without an explicit `allowLarge`
- third-party material does not name its source

## Entry fields

| Field | Meaning |
|---|---|
| `path` | Glob. The first matching entry applies, so specific entries come first. |
| `origin` | `original`, `generated`, `derived-metadata` or `third-party`. |
| `source` | Required for `third-party`: where it came from, pinned where possible. |
| `license` | Licence or terms that give the right to use it. `UNLICENSED` means this project's own material, with no licence granted. |
| `ships` | Whether it can reach users in an installed application. |
| `notes`, `allowLarge` | Optional. |

## Adding material

No schemas, art or tuning data from MyFranchise, MMC, the legacy draft tool or EA are imported in WP01a. Before adding any, verify the actual terms for that specific material, because a package's licence label does not establish rights to everything bundled with it. Real fixtures stay outside Git and are reached through `MADDENTOOL_FIXTURES`; repository fixtures are synthetic.

## Known limits

This check covers tracked files only. It does not yet produce third-party notices for npm and Cargo dependencies or show them in the app: the About screen is `DG-SHELL-04` (WP01b) and the release gate is WP11.
