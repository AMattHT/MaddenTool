#!/usr/bin/env python3
"""Independent check that FEATURE-PARITY.csv accounts for every inventoried symbol."""
import csv, json, os, sys
H = os.path.dirname(os.path.abspath(__file__)); D = os.path.dirname(os.path.dirname(H))
rows = list(csv.DictReader(open(os.path.join(D, "FEATURE-PARITY.csv"), encoding="utf-8-sig")))
mf = json.load(open(os.path.join(H, "..", "myfranchise-2.0.1-inventory.json"), encoding="utf8"))
dg = json.load(open(os.path.join(H, "..", "draft-tool-inventory.json"), encoding="utf8"))
sy = [r for r in rows if r["row_type"] == "symbol"]; fe = {r["id"]: r for r in rows if r["row_type"] == "feature"}
def have(src, kind): return {r["symbol"] for r in sy if r["source"] == src and r["symbol_kind"] == kind}
M, G = "MyFranchise", "Draft generator"
checks = [
 ("MF components", set(mf["vue_components"]), have(M, "component")), ("MF routes", set(mf["route_paths"]), have(M, "route")),
 ("MF IPC", set(mf["ipc_channels"]), have(M, "ipc")),
 ("Draft routes", {f"{r['method']} {r['path']}" for r in dg["routes"]}, have(G, "route")),
 ("Draft components", set(dg["react_components"]), have(G, "component")), ("Draft services", set(dg["services"]), have(G, "service")),
]
bad = 0
for n, want, got in checks:
    miss = want - got; print(f"{n}: inventory {len(want)}, mapped {len(want & got)}, missing {len(miss)}"); bad += len(miss)
for r in sy:
    if r["classification"] == "feature" and not r["maps_to"]: print("no target", r["id"]); bad += 1
    for t in filter(None, r["maps_to"].split(";")):
        if t not in fe: print("bad target", r["id"], t); bad += 1
for i, r in fe.items():
    for t in filter(None, r["dependencies"].split(";")):
        if t not in fe: print("bad dep", i, t); bad += 1
    if r["status"] == "runtime-verified": print("unexpected runtime-verified", i); bad += 1
print("features", len(fe), "symbols", len(sy), "statuses", {s: sum(1 for r in fe.values() if r["status"] == s) for s in {r["status"] for r in fe.values()}})
print("unresolved", [r["symbol"] for r in sy if r["classification"] == "unresolved"]); print("dead", [r["symbol"] for r in sy if r["classification"] == "dead"])
sys.exit(1 if bad else 0)
