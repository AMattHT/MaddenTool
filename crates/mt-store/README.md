# mt-store: the historical player database (DG01)

A SQLite database of real NFL draftees imported from documented datasets, with stable identity, cross-source IDs, and provenance for every value. It is the data foundation for historical draft-class generation. **It does not generate ratings, read or write Madden files, or download anything.** See the [draft-first plan](../../docs/superpowers/plans/2026-11-02-draft-first.md) for what comes next.

## Run it

Rust is installed through rustup; `rust-toolchain.toml` pins 1.99.0. From the repository root:

```sh
cargo build --release -p mt-store          # builds target/release/mt-data
cargo test -p mt-store                     # unit, integration and CLI tests

# Try it on the synthetic fixtures (fabricated rows, marked --synthetic):
mt-data import --db scratch.db --format nflverse-draft-picks \
  --file crates/mt-store/tests/fixtures/synthetic-nflverse-draft-picks-v1.csv \
  --as-of 2026-10-28 --synthetic
mt-data query --db scratch.db --year 2001 --position QB
mt-data query --db scratch.db --year 2001 --json
mt-data imports --db scratch.db
mt-data info --db scratch.db
mt-data player --db scratch.db --uid <uid>   # any uid ever issued, with every selection
mt-data conflicts --db scratch.db            # identities the data cannot settle
mt-data reconcile --db scratch.db            # repair a database written before schema v2
```

`mt-data help` lists every option. Keep databases out of the repository (`*.db` is ignored).

## Supported datasets

| `--format` | File | Coverage | Where it comes from |
|---|---|---|---|
| `nflverse-draft-picks` | `draft_picks.csv` | NFL drafts 1980 onward | The nflverse-data [`draft_picks` release](https://github.com/nflverse/nflverse-data/releases/tag/draft_picks). The repository licence is CC-BY-4.0; the release says the data is "courtesy of Pro Football Reference". |
| `legacy-player-lookup` | `ALL_PLAYER_LOOKUP.csv` | 1936 to 2026, AFL and undrafted rows included | The user's own [Madden-Draft-Generator](https://github.com/AMattHT/Madden-Draft-Generator) (`server/data/lookups`). Its columns follow PFR draft tables; it does not say when they were retrieved. |

Getting the nflverse file is a manual step for now (an in-app download with progress is a later goal):

```sh
curl -L -o draft_picks.csv https://github.com/nflverse/nflverse-data/releases/download/draft_picks/draft_picks.csv
curl -L https://github.com/nflverse/nflverse-data/releases/download/draft_picks/timestamp.json
# Use timestamp.json's last_updated date as --as-of.
mt-data import --db players.db --format nflverse-draft-picks --file draft_picks.csv \
  --as-of 2026-10-28 --source-url https://github.com/nflverse/nflverse-data/releases/download/draft_picks/draft_picks.csv
```

**Data terms.** Approximate Value is a Pro Football Reference metric. Sports Reference's [data use policy](https://www.sports-reference.com/data_use.html) welcomes reuse of facts from individual pages with credit, but prohibits building a competing database from scraped data, and says some data may not be redistributed. So:

- The tool never contacts Pro Football Reference, and nothing here scrapes it.
- Real datasets stay on the user's machine. Neither the repository nor CI downloads, stores or ships them.
- Each import stores its attribution, so a display of this data can credit nflverse and Pro Football Reference.
- A user who copies a table from an individual PFR page can only import it after a format for it exists. None is implemented yet, because no PFR export could be inspected (PFR refuses automated requests, and this project does not work around that). The legacy format is the supported user-provided path today.

Whether nflverse's CC-BY-4.0 licence covers redistributing PFR-derived values is not established. Nothing in MaddenTool redistributes them.

## What is stored

| Table | Holds |
|---|---|
| `dataset_import` | One row per import: format, label, source URL, **as-of date**, file name and SHA-256 (never the full path), `synthetic` flag, attribution, row counts, tool version, time |
| `player` | A person, with a `uid` (`mt-` + 16 hex digits) that never changes once issued. `merged_into` marks a provisional player later found to be another; its uid still resolves |
| `player_external_id` | PFR, GSIS and CFB IDs, each owned by exactly one player |
| `source_record` | One row per source row (a draft selection or undrafted entry): the mapped values plus the original row as JSON (unknown columns included), the import that created it and the import that last confirmed it |
| `player_merge` | Every merge: from, into, rule, the records and facts relied on, the records moved |
| `record_correction` | A verified correction applied to one record: field, source value, corrected value, evidence |
| `import_issue` | Every rejected or unreconciled row, with its source line number |

Migrations are append-only and tracked in `PRAGMA user_version`. `PRAGMA application_id` marks the file as MaddenTool's. The tool refuses to open a database from a newer version or an unrelated SQLite file. Every import runs in one transaction: it either lands completely or not at all.

## Rules that matter

**Missing is not zero.** A blank cell (or nflverse `NA`) is stored as NULL. `0` is stored as 0. Query output adds a `wav_evidence` classification, computed against the import's as-of date rather than today:

| `wav_evidence` | Meaning |
|---|---|
| `missing` | The source has no wAV |
| `no_completed_season` | The source gives a number, but no season since the draft had finished by the as-of date. nflverse lists the 2026 class as 0 while its first season is in progress |
| `observed:ended`, `observed:possibly_active` | A real value, including a real 0, with the career state |
| `...:pre_av_seasons` | Part of the career predates 1960, the first season with AV, so the value undercounts |

`wav` is PFR's *weighted* career AV. Unweighted career AV (`car_av`) is a separate column. It is blank in every row of the 2026-10-28 nflverse file and absent from the legacy file. The two are never substituted for each other.

**Identity.** The natural key is `season:pick` for nflverse. For the legacy file it is year, league, round, pick, normalized name and college. Re-importing a key updates its values and keeps its player. Linking rules, in order:

1. An external ID the database already knows. nflverse files Bo Jackson's 1986 and 1987 selections under one PFR ID, so they become one player.
2. The same draft year, league and overall pick in a different source, with a compatible surname.
3. Otherwise a new player.

Conflicts and mismatches are recorded as issues, never forced. Rows missing from a newer import are kept, and their `as_of` shows the last import that had them. Rows repeated within one file are rejected after their first occurrence.

**One person, many selections.** A person can be drafted more than once: by the AFL and the NFL in one year (1960 to 1966), or again in a later year. After every import, a reconciliation pass looks across all records and merges provisional players only on evidence. A name alone is never enough.

| Rule | Merges when |
|---|---|
| `draft_slot_surname` | Two sources hold the same year, league and overall pick with compatible surnames, one record each. This fixes a re-drafted player imported from the legacy file first ([#2](https://github.com/AMattHT/MaddenTool/issues/2)) |
| `dual_draft_name_college` | One source holds an AFL and an NFL selection in one 1960-66 year with the same name and college, and careers that do not conflict |
| `dual_draft_name_variant_college` | As above, with variant given names: nicknames (Bob/Robert), one-letter spellings of five-letter-plus names (Emery/Emory), or a dropped middle name (Billy Joe/Billy) |
| `dual_draft_career_footprint` | As above, with unrelated given names but the same distinctive career on both rows |

Each side must have exactly one candidate. A merge is refused if the two players hold different IDs in one namespace, or if one person would get two selections in one league and year from one source. Everything else is listed by `mt-data conflicts`, not merged: different colleges, conflicting careers, ambiguous partners, initials ("C.B."), unrelated given names. The older player survives. The absorbed uid stays as an alias, so `mt-data player --uid` resolves any uid ever issued. A database written before schema v2 is migrated on open; `mt-data reconcile` repairs it in place.

**Verified corrections.** `data/corrections.json` fixes individual source rows that are demonstrably wrong. Each entry names one row by format and natural key, the values it expects the source to hold, the replacement, and cited evidence. Only `college` and `position` can be corrected. If the source row no longer matches, the correction is reported stale and not applied. The raw row is never edited, and `record_correction` keeps the original value. The first entry is Bob Brown's 1964 AFL selection: the legacy file gives it his namesake's college and position ([#4](https://github.com/AMattHT/MaddenTool/issues/4)).

**Precedence.** A query returns each person once per draft year. Values come from the representative selection: nflverse before legacy, drafted before undrafted, then NFL before AFL, then earliest round and pick. `selections` lists every selection that year, with its own source, league, slot, wAV and corrections. `sources` lists the contributing formats.

**Positions** are the source's own labels (for example, nflverse files 1,932 of its 12,927 picks as `DB`, not `CB` or `S`). The position filter is an exact, case-insensitive match. Mapping to Madden positions is part of generation.

## Known limits

- On the real legacy file (2026-09-15), reconciliation merges 118 AFL/NFL pairs and leaves 25 unsettled: 13 unconfirmed name aliases, 6 career conflicts and 6 college conflicts. Each needs evidence, either a verified correction or a source with IDs, before it can merge. `mt-data conflicts` lists them.
- Same-source re-drafts in different years (for example 1967 to 1979 legacy-only players) are not merged. A single-year class is unaffected; multi-year classes are a generation concern (DG04a).
- Uids depend on import order. Query results do not: on the real data, both orders give identical output for every year apart from uids.
- Legacy `isHOF` is stored as the file states it. The legacy app found that it is stamped by name and can mark a namesake, so it is not verified.
- Undrafted players exist only in the legacy file. The nflverse `players` table could add their careers; that is a later import.
- No unique ID links legacy rows to people outside the 1980 onward draft slots, and the legacy file's Madden asset IDs are known to collide between namesakes. They are kept in the raw JSON only.
