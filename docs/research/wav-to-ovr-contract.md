# wAV to overall rating: research contract

Status: **contract for DG04, not a calibrated model.** Written 2026-11-02 in DG01. It fixes what wAV may and may not mean in generation, what DG04 must measure, and which tests DG04 must pass. It gives no validated formula, because none exists yet. Any coefficient DG04 introduces must come with its fitting data and held-out error.

## 1. What the metric is

From Pro Football Reference's [glossary](https://www.pro-football-reference.com/about/glossary.htm) and the Sports Reference [AV article](https://www.sports-reference.com/blog/approximate-value/). PFR answers automated requests with HTTP 403, so these points were read through search results and the article. Recheck them in a browser before quoting them in the product.

- **AV** is "our attempt to put a single number on each player-season since 1960" (glossary). The SR article mentions 1950. This contract uses **1960**, the glossary figure, which also matches the legacy tool's observation that pre-1960 careers undercount (Alan Ameche: 4 Pro Bowls, stored wAV 3).
- **Weighted career AV (wAV)** sums 100% of the best season's AV, 95% of the second best, 90% of the third, and so on. It favours peak over longevity.
- **Career AV** is the unweighted sum. It is a different number. In the 2026-10-28 nflverse file it is blank in all 12,927 rows, so it is unavailable.
- **DrAV** is AV earned for the drafting team.
- PFR describes AV as approximate: a 16 and a 14 are not confidently different seasons.

## 2. Draft day versus hindsight

wAV is measured over a whole career, so **it was never known on draft day**. The generator offers three lenses. Hindsight `h` runs from 0 to 1.

| Lens | May use | Must not use |
|---|---|---|
| Draft-day order (`h = 0`) | Draft year, round, overall pick, position, age, combine, and population priors fitted on other players' outcomes, such as expected wAV by draft slot | The player's own wAV, games, starts, Pro Bowls, All-Pros or career span |
| Career (hindsight) | The player's own career record | Treating an active or missing career as if it were complete |
| Blend (`0 < h < 1`) | `h * career_score + (1 - h) * draft_day_score`, the legacy board score | |

A slot prior fitted on historical wAV is allowed at `h = 0`, because it describes the slot and not the player. The UI must never present a hindsight rating as draft-day knowledge.

The Launch Day lens uses EA's release-day ratings and is outside this contract.

## 3. Inputs DG01 already provides

`mt_store::wav::classify` (tested in DG01) turns a stored value into one of these, using the import's as-of date and never the system clock:

- `Missing`: the source had no value. It never becomes 0.
- `NoCompletedSeason { reported }`: no season had finished. The nflverse 2026 class shows 0 here. It is not evidence.
- `Observed { wav, completed_seasons, career: Ended | PossiblyActive | Unknown, pre_av_seasons }`.

DG04 consumes this classification instead of re-deriving it.

## 4. Comparability

Raw wAV is not comparable across positions or eras in a way that maps onto Madden overall:

- **Position:** AV accrues differently by role. Kickers, punters and long snappers accrue slowly. The legacy anchors already reflect this, for example a quarterback needs far more wAV than a long snapper for the same overall.
- **Era:** the NFL season was 12 games in 1960, 14 from 1961, 16 from 1978 and 17 from 2021. The AFL played 14-game seasons beside it from 1960 to 1969. Seasons before 1960 have no AV at all.

**Contract:** generation ranks players by wAV *percentile within a comparison group*, meaning position group × era band × career completeness. It does not read wAV against absolute anchors. Group boundaries are DG04 parameters, which DG04 must justify with data. The starting proposal is the legacy position groups (QB, RB, WR, TE, OL, EDGE, IDL, LB, CB, S, K, P, LS) and era bands that follow season length.

## 5. Mapping to Madden's scale

The Realistic lens maps a player's **rank within the generated class** onto an **empirical M27 rookie-class overall curve**, and dev traits onto the empirical trait rates. The legacy app already does this, and it is the sound part to keep. Raw wAV never becomes an overall through a formula. It only orders players.

The curve must be rebuilt and reproducible:

- Legacy `madden-calibration-m27.json` was built from `CAREERDRAFT-TEST1..3` and `CAREERDRAFT-TESTSUPERSTRONG`. A deliberately super-strong class probably lifts the curve: legacy M27 shows p50 66, p99 80, p100 86. The script that built it is not in the legacy repository.
- DG04 rebuilds the curve with the DG02 reader from game-generated RL4 classes (`TEMP1`, `TEMP2` and `TEMP3`, hashes in `DISCOVERY-REPORT.md` 5.2), recording file hashes and the build script. A strong-class curve, if wanted for the class-strength option, is fitted separately and labelled.

The Career lens is *uncapped* in the legacy app: it reads wAV through hand-seeded per-position anchors. No validation backs them (`av_rating_correlation.json` is `{}`). DG04 must do one of two things:

- (a) Justify a target distribution, for example the overalls of veterans with known wAV in an M27 roster, which needs a roster reader.
- (b) Ship the Career lens labelled as a heuristic, with its anchors stated.

The user decides which (question Q2).

## 6. Active, short and missing careers

- `NoCompletedSeason`: rate from the draft-day prior only, and mark the rating as projected.
- `PossiblyActive` with few completed seasons: do not compare the raw total with completed careers. Use production per completed season, shrunk toward the slot prior by weight `n / (n + k)`, where `n` is completed seasons. The legacy value is `k = 3`, which DG04 must fit.
- `Missing` with career signals (games, starts, accolades, span): estimate wAV from those signals. The legacy aggregate estimator reports a held-out MAE of 8.84 before and 6.17 after its low-end calibration, on 14,149 players with real wAV. Keep it as a candidate and re-measure it on DG01 data.
- `Missing` with no signal: use the slot prior.
- Any estimate is marked estimated in data and UI, never shown as a PFR value.
- `pre_av_seasons`: the observed value undercounts, so use the estimator and record why.

## 7. Legacy assessment

| Legacy element | Verdict |
|---|---|
| Rank to empirical Madden curve, dev rates by outcome rank, elite X-Factor cap (2% of class, minimum 3) | **Keep**, rebuilt on reproducible data |
| Hindsight blend of career and slot score | **Keep** (`DG-RATE-04`) |
| Slot expectation `0.85 * 100 * pick^-0.67` (code credits a Chase Stuart power law) | **Keep as the draft-day prior**, refitted on nflverse 1980+ completed careers with error reported |
| Aggregate wAV estimator with low-end calibration | **Keep as candidate**; re-validate |
| Per-position anchor tables, accolade bonus (+Pro Bowl, +2 × All-Pro, cap 8), `careerLengthBoost` (up to 1.5 ×) | **Replace or validate.** Hand-seeded, no held-out evidence |
| Realistic cap at 85 with knee at 78 | **Re-derive** from the rebuilt curve |
| `CURRENT_YEAR = new Date()` in rating code | **Remove.** Ratings drift by calendar; use the dataset as-of date |
| Treating recent `wAV = 0` as a placeholder | **Superseded** by `NoCompletedSeason` |

## 8. Acceptance tests DG04 must pass

1. **No leakage at draft day:** with `h = 0`, changing any player's own career fields leaves every rating unchanged. Property test.
2. **Missing is not zero:** a `Missing` player is never rated as a real 0, and the output carries `estimated`.
3. **Monotone within a group:** in the Career lens, with all else equal and the same group and career state, higher wAV never gives a lower overall.
4. **Curve fidelity:** generated Realistic classes match the rebuilt M27 curve and dev rates within a tolerance DG04 states before measuring, for example a KS statistic and a trait-rate difference.
5. **Position balance:** per-position share of top-32 overalls, over many seeds and years, is reported against the game classes. Large deviations are explained or fixed.
6. **Determinism:** the same database, options and seed give byte-identical output on every OS and on any date.
7. **Active careers:** a two-season starter is not ranked against fifteen-season totals.
8. **Legacy comparison:** for 1965, 1987, 2003 and 2026, rank correlation and overall differences against the TypeScript oracle are recorded per position. Deliberate changes from this contract are listed; parity is not assumed.
9. **Fitted parameters:** each one ships with its training data, held-out error and the as-of date of the data.

## 9. Open questions for the user

- **Q1** Which lens is the default: Realistic (legacy default) or Career?
- **Q2** Should the Career lens be Madden-shaped, or remain uncapped "how good he became"?
- **Q3** May DG04 calibrate veteran overalls from an M27 roster file? That needs a roster reader first.
- **Q4** Show wAV and its source to the user with ratings visible? It is always hidden when spoilers are off.
