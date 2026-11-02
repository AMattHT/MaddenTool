# Synthetic fixtures. Not real data.

Every row in these files was written by hand for tests. The players, teams (`AAA`, `BBB`...), colleges, IDs (`SYN...`, `Synt..00`) and numbers are invented. They copy only the **column layout** of the real formats so the importers can be tested without committing third-party data.

| File | Layout copied from | What it exercises |
|---|---|---|
| `synthetic-nflverse-draft-picks-v1.csv` | nflverse-data `draft_picks.csv` header, 2026-10-28 | missing vs zero `w_av`, one person drafted twice under one PFR ID, a current-class placeholder `0`, non-ASCII names, a bad season, a fractional value, a duplicate draft slot |
| `synthetic-nflverse-draft-picks-v2.csv` | same | re-import: one value changed, one blank filled, one row gone, one row added |
| `synthetic-legacy-player-lookup.csv` | Madden-Draft-Generator `ALL_PLAYER_LOOKUP.csv` header, with its UTF-8 byte-order mark | slot link to nflverse, slot name mismatch, undrafted, unordered 1960 AFL round, Hall of Fame marker, pre-1960 career, post-1967 `AFL` label, unknown round |

Always import them with `--synthetic`, which marks every value they produce in the database.
