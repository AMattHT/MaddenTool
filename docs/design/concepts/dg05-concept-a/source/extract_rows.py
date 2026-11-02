"""Derive the identity-only concept-row excerpt from a local mt-data query of 1964.

Inputs (never committed): the mt-data JSON query output and the legacy lookup CSV
(for PhotoID only). Output: concept-rows.json with identity and draft-selection
fields only. wAV, its evidence, and career span or honours are deliberately not
copied: the public concept frames use labelled sample inputs for those instead
(see build_frames.py), so no PFR-derived career statistic reaches them.
"""
import csv, json, sys

query_path, csv_path, out_path = sys.argv[1:4]
people = json.load(open(query_path, encoding="utf-8"))
photo = {}
for r in csv.DictReader(open(csv_path, encoding="utf-8-sig")):
    if r["Draft Class"] == "1964":
        photo[(r["First Name"] + " " + r["Last Name"], r["League"], r["Pick"])] = r["PhotoID"]


def row(p):
    sels = []
    for s in sorted(p["selections"], key=lambda s: (s["league"] != "NFL", s["draft_round"] or 99)):
        sels.append({"league": s["league"], "round": s["draft_round"], "pick": s["draft_pick"],
                     "position": s["position"], "college": s["college"],
                     "corrections": s["corrections"]})
    pid = ""
    for s in p["selections"]:
        pid = pid or photo.get((s["name"], s["league"], str(s["draft_pick"])), "")
    return {"name": p["name"], "position": p["position"], "college": p["college"],
            "league": p["league"], "round": p["draft_round"], "pick": p["draft_pick"],
            "as_of": p["as_of"], "selections": sels, "photo_id": pid}


drafted = [p for p in people if p["draft_round"] is not None]
order = sorted(drafted, key=lambda p: (p["draft_round"], p["league"] != "NFL", p["draft_pick"]))
board = [row(p) for p in order[:44]]
by_name = {p["name"]: p for p in people}
review = [row(by_name[n]) for n in ("Jack Adams", "Jake Adams")]
review += [row(p) for p in people if p["name"] == "Bob Young"]
json.dump({"query": "mt-data query --year 1964 --json", "persons_in_class": len(people),
           "board": board, "review": review, "namesake": row(by_name["Robert Brown"])},
          open(out_path, "w", encoding="utf-8"), indent=1)
print(out_path, "board", len(board), "review", len(review))
