"""Assemble the DG05-CONCEPT-A static frames (design work, not app code).

Builds standalone HTML frames from concept-rows.json and the five original
dev-trait icons, then a headless Chromium screenshots each frame at its exact
size. Nothing here is MaddenTool application code.

    python -I build_frames.py <legacy dev-icons dir> <out dir> <concept-rows.json> [chrome.exe]

The icons are read from the legacy Madden-Draft-Generator checkout and checked
against the SHA-256 values in docs/design/draft-generator-higgsfield-brief.md
before use. They are embedded byte-for-byte as data URIs, never redrawn.
concept-rows.json comes from extract_rows.py over a local mt-data import; it is
real data and stays out of Git.
"""
import base64, hashlib, html, json, os, subprocess, sys

ICON_DIR, OUT, ROWS = sys.argv[1], sys.argv[2], sys.argv[3]
CHROME = sys.argv[4] if len(sys.argv) > 4 else r"C:\Program Files\Google\Chrome\Application\chrome.exe"

# Displayed tier -> legacy file (ui.tsx DEV_ART_NAME / DEV_LABELS), with brief hashes.
ICONS = {
    "Normal": ("slow.png", "bbd4fd7a9492b74859cc88aff4c32704af88b2c7d70a2a193fcdadabcb910a87"),
    "Star": ("normal.png", "d66cddd97512c5ec224c56a7f44f5db879c7719479e0f33a60a0c7f8a0bd5d34"),
    "Superstar": ("quick.png", "3f0cab1c56ac6e777f68491547476a92f28cb17e772e1e289499141faecec428"),
    "X-Factor": ("superstar.png", "c484dc2c92981295046962fb225621e18496a232451f9acfa8cef72afd3ad038"),
    "Unscouted": ("hidden.png", "07abd50c1bb48d2b925d41617ad5d1f0d2b47b1d51134e72c3248631eeb15b2c"),
}
URI = {}
for tier, (fname, sha) in ICONS.items():
    data = open(os.path.join(ICON_DIR, fname), "rb").read()
    got = hashlib.sha256(data).hexdigest()
    assert got == sha, f"{fname}: {got} != {sha}"
    URI[tier] = "data:image/png;base64," + base64.b64encode(data).decode()

D = json.load(open(ROWS, encoding="utf-8"))
E = html.escape


# ---------- sample inputs (public rendering path) ----------
# The rows carry identity and draft selections only. wAV, its evidence state and
# career numbers shown in the frames are these explicit sample inputs, chosen by
# board position and independent of any player's real record. Every frame labels
# them "◇ sample".
STAT_KEYS = {"wav", "wav_evidence", "career", "career_to"}
for _r in D["board"] + D["review"] + [D["namesake"]]:
    assert not STAT_KEYS & _r.keys(), f"identity-only rows expected; got stats for {_r['name']}"

SAMPLE_MISSING_AT = {7, 24}   # sample "Missing" state (shown as —, never 0)
SAMPLE_ZERO_AT = {19}         # sample "Observed 0" state
SAMPLE_CAREER = {"seasons": 10, "pb": 3, "ap1": 1, "st": 8}  # inspector sample career record


def sample_wav(idx):
    if idx is None or idx in SAMPLE_MISSING_AT:
        return None
    if idx in SAMPLE_ZERO_AT:
        return 0
    return max(4, 74 - 2 * idx + (idx * 29) % 23 - 11)


for _i, _r in enumerate(D["board"], 1):
    _w = sample_wav(_i)
    _r["wav"] = _w
    _r["wav_evidence"] = {"status": "missing"} if _w is None else {"status": "observed", "wav": _w, "career": "ended"}
for _r in D["review"]:
    _r["wav"], _r["wav_evidence"] = None, {"status": "missing"}


# ---------- illustrative values (layout only; not generator output) ----------
def illus(r):
    """Hand-shaped illustrative OVR / dev tier / percentile, computed only from
    the sample inputs above. Not a rating model; DG04b has not been built."""
    if r["wav"] is None:
        ovr, est = 60 + max(0, 4 - r["round"]) * 2, True
    elif r["wav"] == 0:
        ovr, est = 52, False
    else:
        ovr, est = min(90, round(50 + 40 * (r["wav"] / 80) ** 0.7)), False
    return ovr, tier(ovr), est, pct_of(ovr)


def tier(ovr):
    return "X-Factor" if ovr >= 86 else "Superstar" if ovr >= 80 else "Star" if ovr >= 72 else "Normal"


def pct_of(ovr):
    return min(99, max(3, round((ovr - 50) * 2.6)))


def illus_blend(r, idx, h=0.35):
    """Direction B's example settings (Realistic, hindsight 0.35): an illustrative
    blend of a slot-shaped draft-day value and the career value, so the two
    frames do not show identical numbers under different settings. Layout only."""
    career, _, est, _ = illus(r)
    draft_day = round(82 - 0.5 * idx)
    ovr = draft_day if est else round(h * career + (1 - h) * draft_day)
    return ovr, tier(ovr), est, pct_of(ovr)


def evidence(r, short=False):
    w = r["wav_evidence"]
    if w["status"] == "missing":
        return ("Missing", "missing")
    if w["status"] == "no_completed_season":
        return ("No season" if short else "No completed season", "nocs")
    career = w.get("career")
    tail = {"ended": "ended", "possibly_active": "possibly active"}.get(career, "unknown")
    if w["wav"] == 0:
        return ("Obs 0 · " + tail if short else "Observed 0 · " + tail, "zero")
    return ("Obs · " + tail if short else "Observed · " + tail, "obs")


def sel_text(r):
    return " · ".join(f'{s["league"]} r{s["round"]} p{s["pick"]}' for s in r["selections"])


def corrected(r):
    return any(s["corrections"] for s in r["selections"])


def dev_cell(tier, label=True, size=18):
    alt = "Hidden — turn on Spoilers to reveal" if tier == "Unscouted" else f"{tier} development"
    img = f'<img class="dev" src="{URI[tier]}" width="{size}" height="{size}" alt="{E(alt)}" title="{E(alt)}">'
    return img + (f'<span class="devl">{tier}</span>' if label else "")


SILHOUETTE = ('<svg viewBox="0 0 64 64" aria-hidden="true"><rect width="64" height="64" rx="6" fill="var(--sil-bg)"/>'
              '<circle cx="32" cy="25" r="11" fill="var(--sil)"/><path d="M12 60c2-13 10-19 20-19s18 6 20 19z" fill="var(--sil)"/></svg>')

BASE_CSS = """
*{box-sizing:border-box;margin:0;padding:0}
html,body{width:%(w)dpx;height:%(h)dpx;overflow:hidden}
body{background:var(--bg);color:var(--fg);font:12.5px/1.3 "Segoe UI Variable Text","Segoe UI",system-ui,sans-serif;font-variant-numeric:tabular-nums}
.app{display:flex;flex-direction:column;width:%(w)dpx;height:%(h)dpx}
.title{height:32px;display:flex;align-items:center;gap:10px;padding:0 0 0 12px;background:var(--chrome);border-bottom:1px solid var(--line);flex:none}
.title .mark{width:16px;height:16px;border-radius:3px;border:2px solid var(--accent)}
.title .name{font-weight:600;font-size:12.5px}
.title .sp{flex:1}
.title .wc{width:46px;height:32px;display:grid;place-items:center;color:var(--muted);font-size:13px}
.chip{display:inline-flex;align-items:center;gap:5px;height:22px;padding:0 8px;border:1px solid var(--line2);border-radius:4px;color:var(--fg2);font-size:12px;white-space:nowrap}
.chip b{color:var(--fg);font-weight:600}
.chip.warn{border-color:var(--warnline);color:var(--warn);background:var(--warnbg)}
.btn{height:28px;padding:0 12px;border-radius:5px;border:1px solid var(--line2);background:var(--btn);color:var(--fg);display:inline-flex;align-items:center;gap:6px;font-size:12.5px;white-space:nowrap}
.btn.primary{background:var(--accent);border-color:var(--accent);color:var(--onaccent);font-weight:600}
.btn.dis{color:var(--faint);border-style:dashed}
.field{height:28px;border:1px solid var(--line2);border-radius:5px;background:var(--field);display:flex;align-items:center;gap:6px;padding:0 9px;color:var(--muted);font-size:12.5px;white-space:nowrap}
.caret::after{content:"▾";margin-left:6px;color:var(--muted);font-size:10px}
table{border-collapse:collapse;width:100%%;table-layout:fixed}
th{height:26px;text-align:left;font-weight:600;color:var(--fg2);font-size:11.5px;border-bottom:1px solid var(--line);padding:0 6px;white-space:nowrap;background:var(--thead)}
th .s{color:var(--faint);font-weight:400;margin-left:3px}
td{height:21px;padding:0 6px;white-space:nowrap;overflow:hidden;text-overflow:ellipsis;border-bottom:1px solid var(--rowline)}
td.num,th.num{text-align:right}
td.mono{font-variant-numeric:tabular-nums}
.dev{vertical-align:-4px;margin-right:5px}
.devl{color:var(--fg)}
.ev{font-size:12px}
.ev.obs{color:var(--fg2)} .ev.zero{color:var(--fg2)} .ev.missing{color:var(--warn)} .ev.nocs{color:var(--info)}
.flag{display:inline-flex;align-items:center;gap:3px;height:17px;padding:0 5px;border-radius:3px;font-size:11px;border:1px solid}
.flag.corr{color:var(--info);border-color:var(--infoline)}
.flag.id{color:var(--danger);border-color:var(--dangerline)}
.flag.est{color:var(--warn);border-color:var(--warnline)}
.est{color:var(--warn);font-size:11px;margin-left:3px}
.muted{color:var(--muted)} .faint{color:var(--faint)}
tr.sel td{background:var(--selbg)}
tr.sel td:first-child{box-shadow:inset 3px 0 0 var(--accent)}
tr.sel{outline:2px solid var(--focus);outline-offset:-2px}
tr.group td{height:22px;background:var(--groupbg);color:var(--fg2);font-weight:600;font-size:11.5px}
tr.review td{background:var(--reviewbg)}
h3{font-size:11.5px;letter-spacing:.04em;text-transform:uppercase;color:var(--muted);font-weight:600;margin:0 0 6px}
.kv{display:grid;grid-template-columns:118px 1fr;row-gap:3px;column-gap:8px;font-size:12px}
.kv dt{color:var(--muted)} .kv dd{color:var(--fg)}
.note{font-size:11.5px;color:var(--muted);line-height:1.35}
.banner{border:1px solid var(--warnline);background:var(--warnbg);color:var(--warn);border-radius:5px;padding:6px 8px;font-size:11.5px;line-height:1.35}
.sil{width:56px;height:56px;flex:none}
"""

DARK = """:root{--bg:#121417;--chrome:#0E1013;--panel:#171A1E;--thead:#15181C;--field:#0F1114;--btn:#1E2227;--line:#2A2F36;--line2:#363C44;--rowline:#1E2227;
--fg:#E8EAED;--fg2:#C2C8D0;--muted:#99A2AD;--faint:#6E7782;--accent:#E8A93A;--onaccent:#17130B;--focus:#F2C46B;--selbg:#2A2416;
--warn:#E9B45C;--warnline:#6B5328;--warnbg:#241D10;--info:#7DB9E8;--infoline:#2F4D66;--danger:#F08A80;--dangerline:#6E3732;
--groupbg:#1A1D21;--reviewbg:#1F1716;--sil:#59616B;--sil-bg:#262B31}"""

LIGHT = """:root{--bg:#F7F5F0;--chrome:#EFECE5;--panel:#FFFFFF;--thead:#FBFAF7;--field:#FFFFFF;--btn:#FFFFFF;--line:#D9D5CC;--line2:#C9C4B9;--rowline:#ECE9E2;
--fg:#1B2433;--fg2:#3A4558;--muted:#5D6778;--faint:#8A909B;--accent:#2F6B4F;--onaccent:#FFFFFF;--focus:#2F6B4F;--selbg:#E7F0EA;
--warn:#8A5A0B;--warnline:#D9B77A;--warnbg:#FBF3E2;--info:#1F5F8B;--infoline:#A9C7DE;--danger:#A3261D;--dangerline:#E1A49E;
--groupbg:#F1EEE7;--reviewbg:#FBEDEB;--sil:#B5B9C0;--sil-bg:#E4E6EA}"""


def page(title, w, h, theme, css, body):
    return (f'<!doctype html><html lang="en"><head><meta charset="utf-8"><title>{E(title)}</title>'
            f'<style>{theme}{BASE_CSS % {"w": w, "h": h}}{css}</style></head><body>{body}</body></html>')


def titlebar(extra=""):
    return ('<div class="title"><span class="mark"></span><span class="name">MaddenTool</span>'
            f'{extra}<span class="sp"></span><span class="wc">—</span><span class="wc">☐</span><span class="wc">✕</span></div>')


BOB = next(r for r in D["board"] if r["name"] == "Bob Brown")
CRAIN = next(r for r in D["board"] if r["name"] == "Pat Crain")
NAMESAKE = D["namesake"]
REVIEW = D["review"]
ACTIVE = D["persons_in_class"]
SHA = "b06f3e26…"
AS_OF = BOB["as_of"]


SAMPLE_HEAD = {"wAV": "wAV ◇", "Evidence": "Evidence ◇"}  # sample-input columns


def flags(r, spoilers=True):
    out = []
    if corrected(r):
        out.append('<span class="flag corr" title="A verified correction changed this row">✎ Corrected</span>')
    if r.get("_review"):
        out.append('<span class="flag id" title="Identity unsettled; not merged">⚠ Unsettled ID</span>')
    return "".join(out)


def portrait(r):
    return f'<span class="muted">ID {E(r["photo_id"])}</span>' if r["photo_id"] else '<span class="faint">Generic</span>'


# ---------------------------------------------------------------- Direction A
A_CSS = """
.ctx{height:40px;display:flex;align-items:center;gap:8px;padding:0 14px;border-bottom:1px solid var(--line);flex:none;background:var(--bg)}
.ctx .cls{font-size:16px;font-weight:600;margin-right:6px}
.tools{height:42px;display:flex;align-items:center;gap:8px;padding:0 14px;border-bottom:1px solid var(--line);flex:none}
.tools .sp{flex:1}
.sw{display:inline-flex;align-items:center;gap:7px;color:var(--fg2)}
.sw i{width:30px;height:16px;border-radius:9px;background:var(--accent);position:relative;display:inline-block}
.sw i::after{content:"";position:absolute;right:2px;top:2px;width:12px;height:12px;border-radius:50%%;background:#fff}
.sw.off i{background:var(--line2)} .sw.off i::after{right:auto;left:2px}
.main{flex:1;display:flex;min-height:0}
.rail{width:48px;flex:none;border-right:1px solid var(--line);background:var(--chrome);display:flex;flex-direction:column;align-items:center;padding-top:8px;gap:6px}
.rail span{width:34px;height:34px;border-radius:6px;display:grid;place-items:center;color:var(--muted);font-size:15px}
.rail span.on{background:var(--selbg);color:var(--accent);box-shadow:inset 2px 0 0 var(--accent)}
.grid{flex:1;min-width:0;overflow:hidden;background:var(--bg)}
.insp{width:%(iw)dpx;flex:none;border-left:1px solid var(--line);background:var(--panel);padding:12px 14px;display:flex;flex-direction:column;gap:11px;overflow:hidden}
.ihead{display:flex;gap:11px;align-items:center}
.ihead .nm{font-size:17px;font-weight:600}
.ihead .sub{color:var(--fg2);font-size:12px;margin-top:2px}
.big{display:flex;align-items:center;gap:10px;margin-top:4px}
.big .o{font-size:22px;font-weight:700}
.sec{border-top:1px solid var(--line);padding-top:9px}
.selrow{display:grid;grid-template-columns:40px 1fr;gap:2px 8px;font-size:12px;margin-bottom:5px}
.selrow .lg{font-weight:600;color:var(--fg2)}
.corrbox{margin:3px 0 0 48px;border-left:2px solid var(--infoline);padding:3px 0 3px 8px;font-size:11.5px;color:var(--fg2);line-height:1.35}
.corrbox s{color:var(--muted)}
.link{color:var(--info);text-decoration:underline;text-underline-offset:2px}
.status{height:26px;flex:none;display:flex;align-items:center;border-top:1px solid var(--line);background:var(--chrome);font-size:12px;color:var(--fg2)}
.status span{padding:0 12px;border-right:1px solid var(--line);height:100%%;display:flex;align-items:center;gap:5px;white-space:nowrap}
.status b{color:var(--fg);font-weight:600}
.status .sp{flex:1;border:0}
.hiddenpanel{display:flex;gap:10px;align-items:center;border:1px dashed var(--line2);border-radius:6px;padding:8px 10px;color:var(--fg2);font-size:12px}
"""


def a_table(spoilers, rows_n, compact=False):
    board = [dict(r) for r in D["board"]]
    review = [dict(r, _review=True) for r in REVIEW[:2]]
    if compact:  # 800x600: priority columns only
        cols = [("#", 26, ""), ("Player", 116, ""), ("Pos", 34, ""), ("OVR", 52, "num"), ("Dev ", 96, ""),
                ("wAV", 38, "num"), ("Evidence", 0, "")]
    elif spoilers:
        cols = [("#", 34, ""), ("Selections", 164, ""), ("Player", 126, ""), ("Pos", 38, ""), ("College", 110, ""),
                ("OVR ◇", 58, "num"), ("Dev ◇", 104, ""), ("wAV", 44, "num"), ("Evidence", 132, ""),
                ("Portrait", 72, ""), ("Flags", 0, "")]
    else:
        cols = [("#", 34, ""), ("Selections", 170, ""), ("Player", 150, ""), ("Pos", 46, ""), ("College", 140, ""),
                ("Scouting", 150, ""), ("Portrait", 82, ""), ("Flags", 0, "")]
    head = "".join(
        f'<th class="{c}" style="{f"width:{w}px" if w else ""}">{E(SAMPLE_HEAD.get(n, n))}'
        + ('<span class="s">▾</span>' if n in ("#",) else "") + "</th>" for n, w, c in cols)

    def cells(r, idx):
        ovr, dev, est, _ = illus(r)
        evt, evc = evidence(r, short=compact)
        wav = "—" if r["wav"] is None else str(r["wav"])
        wav_aria = "Sample wAV: missing" if r["wav"] is None else f"Sample wAV {wav}"
        c = {"#": f'<td class="mono muted">{idx}</td>' if idx else '<td class="faint">–</td>',
             "Selections": f'<td>{E(sel_text(r))}</td>',
             "Player": f'<td style="font-weight:500">{E(r["name"])}</td>',
             "Pos": f'<td>{E(r["position"])}</td>',
             "College": f'<td class="muted">{E(r["college"])}</td>',
             "Portrait": f"<td>{portrait(r)}</td>",
             "Flags": f"<td>{flags(r)}</td>",
             "Scouting": f'<td>{dev_cell("Unscouted")}</td>' if not compact else ""}
        if spoilers:
            ovr_html = f'{ovr}<span class="est" title="Estimated: wAV missing">est.</span>' if est else str(ovr)
            c.update({"OVR ◇": f'<td class="num" style="font-weight:600">{ovr_html}</td>',
                      "OVR": f'<td class="num" style="font-weight:600">{ovr_html}</td>',
                      "Dev ◇": f"<td>{dev_cell(dev)}</td>",
                      "Dev ": f"<td>{dev_cell(dev)}</td>",
                      "wAV": f'<td class="num" aria-label="{wav_aria}">{wav}</td>',
                      "Evidence": f'<td><span class="ev {evc}">{E(evt)}</span></td>'})
        return "".join(c[n] for n, _, _ in cols)

    if compact:
        body = [f'<tr class="group review"><td colspan="{len(cols)}">⚠ 2 unsettled identities · kept separate</td></tr>']
    else:
        body = [f'<tr class="group review"><td colspan="{len(cols)}">⚠ Needs review · 2 unsettled identities in this class — '
                f'shown as separate people, not merged <span class="faint">(+1 more pair: Bob Young × 2)</span></td></tr>']
    for r in review:
        body.append(f'<tr class="review">{cells(r, None)}</tr>')
    if compact:
        body.append(f'<tr class="group"><td colspan="{len(cols)}">Board · round, NFL then AFL, pick <span class="faint">(provisional)</span></td></tr>')
    else:
        body.append(f'<tr class="group"><td colspan="{len(cols)}">Board · order: round, then NFL before AFL, then pick '
                    f'<span class="faint">(provisional; AFL/NFL interleave is DG04a)</span></td></tr>')
    shown = 0
    for i, r in enumerate(board, 1):
        if shown >= rows_n:
            break
        cls = ' class="sel" aria-selected="true"' if r is not None and r["name"] == "Bob Brown" else ""
        body.append(f"<tr{cls}>{cells(r, i)}</tr>")
        shown += 1
    return f'<table><thead><tr>{head}</tr></thead><tbody>{"".join(body)}</tbody></table>', shown


def a_inspector(spoilers, compact=False):
    r = BOB
    ovr, dev, est, pct = illus(r)
    car = SAMPLE_CAREER
    nfl, afl = r["selections"]
    head = (f'<div class="ihead"><div class="sil">{SILHOUETTE}</div><div><div class="nm">{E(r["name"])}</div>'
            f'<div class="sub">{E(r["position"])} · {E(r["college"])} · 1964 class</div>'
            + (f'<div class="big"><span class="o">{ovr}</span><span class="faint" style="font-size:11px">OVR ◇</span>'
               f'{dev_cell(dev, size=26)}</div>' if spoilers else
               f'<div class="big">{dev_cell("Unscouted", label=False, size=26)}<span class="muted" style="font-size:12px">Unscouted — ratings hidden</span></div>')
            + "</div></div>")
    corr_ev = ("4 citations · correction <code>1964-afl-r1p4-bob-brown</code> · <span class='link'>View evidence</span>"
               if spoilers else "Evidence text mentions career outcomes — <b>hidden while spoilers are off</b>")
    if compact:
        corr_ev = "<span class='link'>View evidence</span>"
    merge = ('<div class="note" style="margin:4px 0 0 48px">Merged by DG01 rule: same name and college, after the verified correction</div>'
             if not compact else "")
    sels = (f'<div class="sec"><h3>Selections · one person</h3>'
            f'<div class="selrow"><span class="lg">NFL</span><span>Round 1, pick 2 · {E(nfl["position"])} · {E(nfl["college"])} <span class="faint">· legacy lookup</span></span></div>'
            f'<div class="selrow"><span class="lg">AFL</span><span>Round 1, pick 4 · {E(afl["position"])} · {E(afl["college"])} '
            f'<span class="flag corr">✎ Corrected</span></span></div>'
            f'<div class="corrbox">Source row said <s>Ark-Pine Bluff, DT</s> (a namesake\'s). {corr_ev}</div>{merge}'
            f'<div class="note" style="margin-top:6px">Different person: Robert Brown · NFL r13 p169 · {E(NAMESAKE["college"])} {E(NAMESAKE["position"])}'
            " · own row</div></div>")
    if spoilers:
        career = (f'<div class="sec"><h3>Career record <span class="faint" style="text-transform:none;letter-spacing:0">· ◇ sample, not this player\'s record · hindsight</span></h3>'
                  f'<dl class="kv"><dt>Seasons</dt><dd>{car["seasons"]} · career ended <span class="faint">◇ sample</span></dd>'
                  f'<dt>Pro Bowls · All-Pro</dt><dd>{car["pb"]} · {car["ap1"]} <span class="faint">◇ sample</span></dd>'
                  f'<dt>Years as starter</dt><dd>{car["st"]} <span class="faint">◇ sample</span></dd></dl></div>')
        evt, _ = evidence(r)
        why = (f'<div class="sec"><h3>Why this rating — inputs only</h3>'
               f'<dl class="kv"><dt>Lens</dt><dd>Career <span class="faint">(example setting)</span></dd>'
               f'<dt>Hindsight</dt><dd>1.00 · career outcome</dd>'
               f'<dt>wAV</dt><dd>{r["wav"]} <span class="faint">◇ sample input</span></dd>'
               f'<dt>Evidence</dt><dd>{E(evt)} <span class="faint">◇ sample</span></dd>'
               f'<dt>Comparison group</dt><dd>OL · 1960s era band <span class="faint">(proposed; DG04)</span></dd>'
               f'<dt>Group percentile</dt><dd>{pct}th <span class="faint">◇ illustrative</span></dd>'
               f'<dt>Estimated</dt><dd>No</dd></dl>'
               + ('<div class="banner" style="margin-top:6px">◇ Sample stats and illustrative ratings; ratings model (DG04b) not built.</div></div>' if compact else
                  '<div class="banner" style="margin-top:7px">◇ Concept: wAV, evidence and career numbers are sample inputs, not this player\'s record. OVR, dev tier and percentile are illustrative. The ratings model (DG04b) is not built.</div></div>'))
    else:
        career = ('<div class="sec"><h3>Career record</h3><div class="hiddenpanel">'
                  f'{dev_cell("Unscouted", label=False, size=22)}<span>Hidden — turn on Spoilers to reveal. '
                  'Career span, honours and wAV are outcomes, not draft-day knowledge.</span></div></div>')
        why = ('<div class="sec"><h3>Why this rating</h3><div class="hiddenpanel">'
               '<span>Hidden while spoilers are off. Nothing about this player\'s rating is sent to this view.</span></div></div>')
    prov = (f'<div class="sec"><h3>Provenance</h3><dl class="kv"><dt>Identity, selections</dt><dd>Legacy ALL_PLAYER_LOOKUP.csv</dd>'
            f'<dt>Import as-of</dt><dd>{AS_OF} · sha256 {SHA}</dd>'
            + ('<dt>Stats shown</dt><dd>◇ Sample inputs (concept)</dd>' if spoilers else "")
            + '<dt>Attribution</dt><dd>Pro Football Reference draft tables; retrieval date not recorded</dd></dl></div>')
    if compact:
        return head + sels + (why if spoilers else career) + prov
    return head + sels + career + why + prov


def frame_a(spoilers=True):
    w, h = 1440, 900
    tbl, n = a_table(spoilers, rows_n=32)
    ctx = ('<div class="ctx"><span class="cls">1964 class</span>'
           '<span class="chip">AFL + NFL</span>'
           '<span class="chip">Lens <b>Career</b> · example</span>'
           '<span class="chip">Hindsight <b>1.00</b></span>'
           '<span class="chip">Seed <b>7F3A-21C9</b> · example</span>'
           f'<span class="chip">Offline · data as-of <b>{AS_OF}</b></span>'
           '<span class="sp" style="flex:1"></span>'
           + ('<span class="chip warn">◇ Concept · sample stats · illustrative ratings</span>' if spoilers else
              '<span class="chip warn">Concept · spoilers off</span>') + "</div>")
    sort = ('<span class="field caret">Sort: Selection</span>' if not spoilers else
            '<span class="field caret">Sort: Selection</span>')
    tools = ('<div class="tools"><span class="field" style="width:220px">⌕ Search 1964 class</span>'
             '<span class="field caret">Position</span><span class="field caret">League: AFL + NFL</span>'
             + ('<span class="field caret">Evidence</span>' if spoilers else
                '<span class="field faint" title="Hidden while spoilers are off">Evidence filter off</span>')
             + sort + '<span class="btn">▦ Columns</span><span class="sp"></span>'
             + (f'<span class="sw"><i></i>Spoilers on</span>' if spoilers else '<span class="sw off"><i></i>Spoilers off</span>')
             + '<span class="btn">Generate…</span><span class="btn primary">Validate &amp; export…</span></div>')
    status = (f'<div class="status"><span>Active <b>{ACTIVE}</b></span>'
              '<span>Export limit <b>402</b> <span class="faint">· in-game import of 402 not yet verified</span></span>'
              '<span>File capacity <b>455</b></span><span>Overflow <b>0</b></span>'
              '<span>Identities: <b>2</b> unsettled</span><span class="sp"></span>'
              '<span style="border-left:1px solid var(--line)">Validation not run · concept</span></div>')
    rail = '<div class="rail"><span class="on" title="Class board">▤</span><span title="Data and sources">⛁</span><span title="Class Studio">✚</span><span title="Settings">⚙</span></div>'
    body = (f'<div class="app">{titlebar()}{ctx}{tools}<div class="main">{rail}<div class="grid">{tbl}</div>'
            f'<div class="insp" aria-label="Player inspector">{a_inspector(spoilers)}</div></div>{status}</div>')
    css = A_CSS % {"iw": 392}
    return page("MaddenTool board concept A", w, h, DARK, css, body), n


def frame_a800():
    w, h = 800, 600
    tbl, n = a_table(True, rows_n=24, compact=True)
    css = A_CSS % {"iw": 300} + """
.insp{padding:10px 11px;gap:8px}.ihead .nm{font-size:15px}.kv{grid-template-columns:96px 1fr;font-size:11.5px}
.selrow{grid-template-columns:30px 1fr;font-size:11.5px}.corrbox{margin-left:38px}
.ctx{height:34px;padding:0 10px;gap:6px}.ctx .cls{font-size:14px}.tools{height:38px;padding:0 10px;gap:6px}
.rail{width:40px}.rail span{width:30px;height:30px}
"""
    ctx = ('<div class="ctx"><span class="cls">1964 class</span><span class="chip">Career · h 1.00</span>'
           '<span class="chip">Seed 7F3A…</span><span class="chip">Offline</span>'
           '<span style="flex:1"></span><span class="chip warn">◇ Sample stats</span></div>')
    tools = ('<div class="tools"><span class="field" style="flex:1">⌕ Search</span><span class="btn">Filters · 0</span>'
             '<span class="btn">▦</span><span class="sw"><i></i>Spoilers</span>'
             '<span class="btn primary">Export…</span></div>')
    status = (f'<div class="status"><span>Active <b>{ACTIVE}</b></span><span>Limit <b>402</b> <span class="faint">unverified in-game</span></span>'
              '<span>Capacity <b>455</b></span><span>2 unsettled</span></div>')
    rail = '<div class="rail"><span class="on">▤</span><span>⛁</span><span>✚</span><span>⚙</span></div>'
    body = (f'<div class="app">{titlebar()}{ctx}{tools}<div class="main">{rail}<div class="grid">{tbl}</div>'
            f'<div class="insp" aria-label="Player inspector">{a_inspector(True, compact=True)}</div></div>{status}</div>')
    return page("MaddenTool board concept A 800", w, h, DARK, css, body), n


# ---------------------------------------------------------------- Direction B
B_CSS = """
.main{flex:1;display:flex;min-height:0}
.recipe{width:262px;flex:none;border-right:1px solid var(--line);padding:14px 16px;display:flex;flex-direction:column;gap:12px;background:var(--bg);overflow:hidden}
.recipe h2{font-size:15px;font-weight:650}
.grp label{display:block;font-size:11.5px;color:var(--muted);font-weight:600;margin-bottom:4px}
.seg{display:flex;border:1px solid var(--line2);border-radius:5px;overflow:hidden}
.seg span{flex:1;text-align:center;height:26px;line-height:26px;font-size:12px;border-right:1px solid var(--line2);color:var(--fg2);background:var(--panel)}
.seg span:last-child{border-right:0}.seg span.on{background:var(--accent);color:#fff;font-weight:600}
.slider{position:relative;height:16px;margin:4px 2px 0}.slider::before{content:"";position:absolute;left:0;right:0;top:7px;height:3px;background:var(--line);border-radius:2px}
.slider i{position:absolute;left:0;top:7px;height:3px;background:var(--accent);border-radius:2px}
.slider b{position:absolute;top:1px;width:14px;height:14px;border-radius:50%;background:#fff;border:2px solid var(--accent)}
.ends{display:flex;justify-content:space-between;font-size:11px;color:var(--muted)}
.meter{margin-bottom:7px}.meter .l{display:flex;justify-content:space-between;font-size:12px}
.meter .bar{height:5px;background:var(--groupbg);border:1px solid var(--line);border-radius:3px;margin-top:3px;position:relative}
.meter .bar i{position:absolute;left:0;top:0;bottom:0;background:var(--accent);border-radius:2px}
.right{flex:1;min-width:0;display:flex;flex-direction:column}
.tools{height:44px;display:flex;align-items:center;gap:8px;padding:0 14px;border-bottom:1px solid var(--line);flex:none}
.tools .sp{flex:1}
.grid{flex:1;overflow:hidden;background:var(--panel)}
tr.band td{height:auto;padding:0;white-space:normal;background:var(--panel);border-bottom:2px solid var(--accent)}
.bandin{display:grid;grid-template-columns:1.05fr 1.15fr 1fr;gap:0;border-left:3px solid var(--accent);background:#FAFCFA}
.bandin>div{padding:9px 14px;border-right:1px solid var(--rowline)}
.bandin .kv{grid-template-columns:108px 1fr;row-gap:2px;font-size:11.8px}
.bandin h3{margin-bottom:5px}
tr.sel{outline:none}
tr.sel td{border-bottom:0}
.sw{display:flex;align-items:center;justify-content:space-between;font-size:12.5px}
.sw i{width:30px;height:16px;border-radius:9px;background:var(--accent);position:relative;display:inline-block}
.sw i::after{content:"";position:absolute;right:2px;top:2px;width:12px;height:12px;border-radius:50%;background:#fff}
"""


def frame_b():
    w, h = 1440, 900
    board = [dict(r) for r in D["board"]]
    cols = [("#", 32, ""), ("Selections", 164, ""), ("Player", 128, ""), ("Pos", 38, ""), ("College", 108, ""),
            ("Archetype ◇", 108, ""), ("OVR ◇", 58, "num"), ("Dev ◇", 102, ""), ("wAV", 42, "num"),
            ("Evidence", 128, ""), ("Portrait", 64, ""), ("Flags", 0, "")]
    arche = {"QB": "Field General", "HB": "Power Back", "FB": "Blocking", "SE": "Deep Threat", "FL": "Route Runner",
             "WR": "Route Runner", "TE": "Possession", "LT": "Pass Protector", "T": "Power", "C": "Agile",
             "LG": "Power", "DT": "Run Stopper", "LE": "Power Rusher", "DE": "Power Rusher", "MLB": "Field General",
             "CB": "Zone", "S": "Zone", "K": "Accurate"}
    head = "".join(f'<th class="{c}" style="{f"width:{w_}px" if w_ else ""}">{E(SAMPLE_HEAD.get(n, n))}</th>' for n, w_, c in cols)

    def row(r, i, sel=False):
        ovr, dev, est, _ = illus_blend(r, i)
        evt, evc = evidence(r)
        wav = "—" if r["wav"] is None else str(r["wav"])
        ovr_html = f'{ovr}<span class="est">est.</span>' if est else str(ovr)
        cl = ' class="sel" aria-expanded="true"' if sel else ""
        return (f'<tr{cl}><td class="muted">{i}</td><td>{E(sel_text(r))}</td><td style="font-weight:500">{E(r["name"])}</td>'
                f'<td>{E(r["position"])}</td><td class="muted">{E(r["college"])}</td>'
                f'<td class="muted">{E(arche.get(r["position"], "—"))}</td><td class="num" style="font-weight:600">{ovr_html}</td>'
                f'<td>{dev_cell(dev)}</td><td class="num">{wav}</td><td><span class="ev {evc}">{E(evt)}</span></td>'
                f'<td>{portrait(r)}</td><td>{flags(r)}</td></tr>')

    crain_idx = next(i for i, r in enumerate(board, 1) if r["name"] == "Pat Crain")
    ovr, dev, est, pct = illus_blend(CRAIN, crain_idx)
    nfl, afl = CRAIN["selections"]
    band = (f'<tr class="band"><td colspan="{len(cols)}"><div class="bandin">'
            f'<div><h3>Identity &amp; selections</h3><dl class="kv"><dt>Person</dt><dd>{E(CRAIN["name"])} · {E(CRAIN["college"])}</dd>'
            f'<dt>NFL</dt><dd>Round {nfl["round"]}, pick {nfl["pick"]} · {E(nfl["position"])}</dd>'
            f'<dt>AFL</dt><dd>Round {afl["round"]}, pick {afl["pick"]} · {E(afl["position"])}</dd>'
            f'<dt>Merged by</dt><dd>DG01 rule: same name and college</dd>'
            f'<dt>Career record</dt><dd>Hidden in this concept; stats here are ◇ sample</dd></dl></div>'
            f'<div><h3>Why this rating — inputs only</h3><dl class="kv"><dt>Lens</dt><dd>Realistic <span class="faint">(example)</span></dd>'
            f'<dt>Hindsight</dt><dd>0.35 · blend of draft-day and career</dd>'
            f'<dt>wAV</dt><dd><b style="color:var(--warn)">Missing</b> <span class="faint">◇ sample state</span> — shown as —, never 0</dd>'
            f'<dt>Evidence</dt><dd>Missing · estimate path set by DG04</dd>'
            f'<dt>Group percentile</dt><dd>{pct}th <span class="faint">◇ illustrative</span></dd>'
            f'<dt>Estimated</dt><dd><span class="flag est">Yes · estimated</span></dd></dl></div>'
            f'<div><h3>Provenance &amp; corrections</h3><dl class="kv"><dt>Selections from</dt><dd>Legacy lookup, 2 rows</dd>'
            f'<dt>Stats shown</dt><dd>◇ Sample inputs (concept)</dd>'
            f'<dt>Import as-of</dt><dd>{AS_OF}</dd><dt>File</dt><dd>sha256 {SHA}</dd>'
            f'<dt>Corrections</dt><dd>None on these rows</dd>'
            f'<dt>Attribution</dt><dd>PFR draft tables via legacy file</dd></dl></div>'
            f'</div></td></tr>')
    body = [f'<tr class="group review"><td colspan="{len(cols)}">⚠ Needs review · 2 unsettled identities — '
            'Jack Adams (AFL r12 p93) / Jake Adams (NFL r13 p177), Virginia Tech · Bob Young ×2 — kept separate</td></tr>']
    i = 0
    cur = None
    rows = 0
    for r in board:
        if r["round"] != cur:
            cur = r["round"]
            body.append(f'<tr class="group"><td colspan="{len(cols)}">Round {cur}</td></tr>')
        i += 1
        sel = r["name"] == "Pat Crain"
        body.append(row(r, i, sel))
        rows += 1
        if sel:
            body.append(band)
        if rows >= 31:
            break
    tbl = f'<table><thead><tr>{head}</tr></thead><tbody>{"".join(body)}</tbody></table>'
    recipe = (
        '<div class="recipe"><h2>Class recipe</h2>'
        '<div class="grp"><label>Source</label><div class="field caret" style="color:var(--fg)">Historical · 1964 · AFL + NFL</div>'
        '<div class="note" style="margin-top:3px">AFL and NFL selections merged per person</div></div>'
        '<div class="grp"><label>Lens <span class="faint" style="font-weight:400">(example; default undecided)</span></label>'
        '<div class="seg"><span class="on">Realistic</span><span>Career</span><span>Launch Day</span></div></div>'
        '<div class="grp"><label>Hindsight 0.35</label><div class="slider"><i style="width:35%"></i><b style="left:calc(35% - 7px)"></b></div>'
        '<div class="ends"><span>Draft day</span><span>Career</span></div></div>'
        '<div class="grp"><label>Class strength · studs · generational #1</label><div class="field caret" style="color:var(--fg)">Auto · off · off <span class="faint">&nbsp;example</span></div></div>'
        '<div class="grp"><label>Seed</label><div style="display:flex;gap:6px"><div class="field" style="flex:1;color:var(--fg)">7F3A-21C9</div><span class="btn">↻ Reroll</span></div></div>'
        '<div class="grp sw"><span>Spoilers</span><span style="display:flex;gap:6px;align-items:center;color:var(--fg2)">On <i></i></span></div>'
        f'<div class="grp"><label>Data</label><div class="note">Legacy ALL_PLAYER_LOOKUP.csv · import as-of <b style="color:var(--fg)">{AS_OF}</b> · sha256 {SHA}<br>'
        'PFR draft tables; retrieval date not recorded · Offline</div></div>'
        '<div class="grp"><label>Counts — three separate numbers</label>'
        f'<div class="meter"><div class="l"><span>Active</span><b>{ACTIVE}</b></div></div>'
        '<div class="meter"><div class="l"><span>Export limit</span><b>402</b></div><div class="note">MaddenTool limit; in-game import not yet verified</div></div>'
        '<div class="meter"><div class="l"><span>File capacity</span><b>455</b></div><div class="note">Records in an M27 class file</div></div>'
        '</div>'
        '<div class="banner">◇ Concept: wAV and evidence are sample inputs, not players\' records. OVR, dev tier, archetype and percentile are illustrative. The ratings model (DG04b) is not built.</div>'
        '</div>')
    tools = ('<div class="tools"><span class="field" style="width:250px">⌕ Search players, colleges</span>'
             '<span class="field caret">Position</span><span class="field caret">League</span><span class="field caret">Evidence</span>'
             '<span class="field caret">Group: Round</span><span class="sp"></span><span class="btn">▦ Columns</span>'
             '<span class="btn">Generate…</span><span class="btn primary">Validate &amp; export…</span></div>')
    body_html = (f'<div class="app">{titlebar("<span class=faint>·</span><span>1964 class</span>")}<div class="main">{recipe}'
                 f'<div class="right">{tools}<div class="grid">{tbl}</div></div></div></div>')
    return page("MaddenTool board concept B", w, h, LIGHT, B_CSS, body_html), rows


def shoot(name, html_text, w, h):
    hp = os.path.join(OUT, name + ".html")
    open(hp, "w", encoding="utf-8").write(html_text)
    png = os.path.join(OUT, name + ".png")
    subprocess.run([CHROME, "--headless=new", "--disable-gpu", "--hide-scrollbars", "--force-device-scale-factor=1",
                    f"--window-size={w},{h}", f"--screenshot={png}", "file:///" + hp.replace("\\", "/")],
                   check=True, capture_output=True)
    return png


if __name__ == "__main__":
    os.makedirs(OUT, exist_ok=True)
    for name, (doc, n), w, h in [
        ("A-scouting-desk-dark-1440x900", frame_a(True), 1440, 900),
        ("A-scouting-desk-spoilers-off-1440x900", frame_a(False), 1440, 900),
        ("A-scouting-desk-dark-800x600", frame_a800(), 800, 600),
        ("B-class-ledger-light-1440x900", frame_b(), 1440, 900),
    ]:
        print(name, "board rows built:", n, "->", shoot(name, doc, w, h))
