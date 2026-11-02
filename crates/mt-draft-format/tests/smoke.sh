#!/usr/bin/env bash
# CLI smoke test on a SYNTHETIC one-player Madden 27 class (not game data).
# Usage: tests/smoke.sh <path to mt-draft binary> <scratch directory>
# Portable across the CI runners: bash, printf, dd, head, cmp, grep.
set -euo pipefail

bin=$1
dir=$2
mkdir -p "$dir"
f="$dir/SYNTHETIC-smoke-class"

head -c 2673650 /dev/zero > "$f"
put() { printf "$2" | dd of="$f" bs=1 seek="$1" conv=notrunc 2>/dev/null; }
# Header: magic, version 1, header size 52, payload 2,673,580, size field 2,673,632,
# timestamp 2026-11-02 12:34:56, build string, constant 7, active count 1.
put 0 'FBCHUNKS\001\000\064\000\000\000\254\313\050\000\340\313\050\000'
put 22 '\352\007\013\000\002\000\014\000\042\000\070\000'
put 34 'Madden-27-RL4-9244223'
put 62 '\007\000\000\000\001\000\000\000'
# Slot 0: visual JSON at 0x46, attributes at 0x46 + 0x1600.
put 70 '{"genericHeadName":"gen_SYNTHETIC_0","loadouts":[]}'
put 5702 'Smoke'
put 5723 'Synthetic'
put 5787 '\120' # overall 80 at attribute offset 0x55

cp "$f" "$f.orig"

"$bin" inspect "$f" > "$dir/human.txt"
grep -q '1 active of 455 slots; 454 inactive, 454 of them blank' "$dir/human.txt"
grep -q 'Smoke Synthetic' "$dir/human.txt"

"$bin" inspect "$f" --json > "$dir/inspect.json"
grep -q '"schema": "mt-draft-inspect/1"' "$dir/inspect.json"
grep -q '"overall": 80' "$dir/inspect.json"

"$bin" inspect "$f" --slot 0 > /dev/null
cmp "$f" "$f.orig" # inspect never writes

head -c 1000000 "$f" > "$dir/truncated"
set +e
"$bin" inspect "$dir/truncated" 2> "$dir/err.txt"
status=$?
set -e
test "$status" = 1
grep -q 'truncated draft class' "$dir/err.txt"
echo "mt-draft smoke: ok"
