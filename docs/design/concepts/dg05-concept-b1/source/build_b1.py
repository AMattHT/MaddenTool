"""Assemble the DG05-CONCEPT-B1 static frames (design work, not app code).

Direction A (Scouting Desk, selected by the user) in a light appearance, at
1024x768 in light and dark, at 200% zoom, plus one keyboard-focus state.
Every frame is a fixed-size static HTML/CSS layout screenshot by headless
Chrome. Nothing here is MaddenTool application code, and nothing here is
interactive: the focus ring is drawn as a state, not produced by a keypress.

    python -I build_b1.py <legacy dev-icons dir> <work dir> <identity-rows.json> [chrome.exe]

Inputs:
  * the five original dev-trait icons from AMattHT/Madden-Draft-Generator at
    9a4ac69f, checked against the brief's SHA-256 values and inlined byte for
    byte (never redrawn or recoloured);
  * identity-rows.json, the identity-only excerpt written by
    ../../dg05-concept-a/source/extract_rows.py from a local mt-data import.
    It is real identity data and stays out of Git. Stat fields are rejected.

Outputs in <work dir>: one .html and one .png per frame, the focus-detail
composite, and checks.json (measured rows, clipping, DOM contrast, token
contrast, icon/backing-chip contrast, parity with concept A's sample values).
"""
import base64, hashlib, html, io, json, os, re, subprocess, sys

sys.dont_write_bytecode = True  # the parity check imports concept A's script; leave no __pycache__ behind

ICON_DIR, OUT, ROWS = sys.argv[1], sys.argv[2], sys.argv[3]
CHROME = sys.argv[4] if len(sys.argv) > 4 else r"C:\Program Files\Google\Chrome\Application\chrome.exe"
HERE = os.path.dirname(os.path.abspath(__file__))

# Displayed tier -> legacy file (ui.tsx DEV_ART_NAME / DEV_LABELS), with brief hashes.
ICONS = {
    "Normal": ("slow.png", "bbd4fd7a9492b74859cc88aff4c32704af88b2c7d70a2a193fcdadabcb910a87"),
    "Star": ("normal.png", "d66cddd97512c5ec224c56a7f44f5db879c7719479e0f33a60a0c7f8a0bd5d34"),
    "Superstar": ("quick.png", "3f0cab1c56ac6e777f68491547476a92f28cb17e772e1e289499141faecec428"),
    "X-Factor": ("superstar.png", "c484dc2c92981295046962fb225621e18496a232451f9acfa8cef72afd3ad038"),
    "Unscouted": ("hidden.png", "07abd50c1bb48d2b925d41617ad5d1f0d2b47b1d51134e72c3248631eeb15b2c"),
}
ICON_BYTES, URI = {}, {}
for _tier, (_f, _sha) in ICONS.items():
    _b = open(os.path.join(ICON_DIR, _f), "rb").read()
    assert hashlib.sha256(_b).hexdigest() == _sha, f"{_f}: hash mismatch"
    ICON_BYTES[_tier] = _b
    URI[_tier] = "data:image/png;base64," + base64.b64encode(_b).decode()

D = json.load(open(ROWS, encoding="utf-8"))
E = html.escape

# ---------- sample inputs: identical to concept A (parity checked below) ----------
STAT_KEYS = {"wav", "wav_evidence", "career", "career_to"}
for _r in D["board"] + D["review"] + [D["namesake"]]:
    assert not STAT_KEYS & _r.keys(), f"identity-only rows expected; got stats for {_r['name']}"

SAMPLE_MISSING_AT = {7, 24}
SAMPLE_ZERO_AT = {19}
SAMPLE_CAREER = {"seasons": 10, "pb": 3, "ap1": 1, "st": 8}


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


def tier(ovr):
    return "X-Factor" if ovr >= 86 else "Superstar" if ovr >= 80 else "Star" if ovr >= 72 else "Normal"


def pct_of(ovr):
    return min(99, max(3, round((ovr - 50) * 2.6)))


def illus(r):
    """Illustrative OVR / tier / percentile from the sample inputs only (same as A)."""
    if r["wav"] is None:
        ovr, est = 60 + max(0, 4 - r["round"]) * 2, True
    elif r["wav"] == 0:
        ovr, est = 52, False
    else:
        ovr, est = min(90, round(50 + 40 * (r["wav"] / 80) ** 0.7)), False
    return ovr, tier(ovr), est, pct_of(ovr)


def evidence(r, short=False):
    w = r["wav_evidence"]
    if w["status"] == "missing":
        return ("Missing", "missing")
    if w["wav"] == 0:
        return ("Obs 0 · ended" if short else "Observed 0 · ended", "zero")
    return ("Obs · ended" if short else "Observed · ended", "obs")


def sel_text(r):
    return " · ".join(f'{s["league"]} r{s["round"]} p{s["pick"]}' for s in r["selections"])


def corrected(r):
    return any(s["corrections"] for s in r["selections"])


BOB = next(r for r in D["board"] if r["name"] == "Bob Brown")
NAMESAKE = D["namesake"]
REVIEW = D["review"]
ACTIVE = D["persons_in_class"]
AS_OF = BOB["as_of"]
SHA = "b06f3e26…"

# ---------------------------------------------------------------- tokens
# Light: B's ledger palette (paper, ink, ledger green), with faint text raised
# to pass 4.5:1. Dark: A's palette, with faint text raised the same way.
# Focus is one blue family in both themes so it never reads as selection
# (selection uses the theme accent: green in light, gold in dark).
LIGHT = dict(bg="#F7F5F0", chrome="#EFECE5", panel="#FFFFFF", thead="#FBFAF7", field="#FFFFFF", btn="#FFFFFF",
             line="#D9D5CC", line2="#8C8578", rowline="#ECE9E2", fg="#1B2433", fg2="#3A4558", muted="#5D6778",
             faint="#646B77", accent="#2F6B4F", onaccent="#FFFFFF", selbg="#E3EEE7", focus="#1D5FD1",
             warn="#8A5A0B", warnline="#D9B77A", warnbg="#FBF3E2", info="#1F5F8B", infoline="#A9C7DE",
             danger="#A3261D", dangerline="#E1A49E", groupbg="#F1EEE7", reviewbg="#FBEDEB",
             sil="#B5B9C0", silbg="#E4E6EA", chip="transparent")
DARK = dict(bg="#121417", chrome="#0E1013", panel="#171A1E", thead="#15181C", field="#0F1114", btn="#1E2227",
            line="#2A2F36", line2="#68707A", rowline="#1E2227", fg="#E8EAED", fg2="#C2C8D0", muted="#99A2AD",
            faint="#8A939E", accent="#E8A93A", onaccent="#17130B", selbg="#2A2416", focus="#7DB8FF",
            warn="#E9B45C", warnline="#6B5328", warnbg="#241D10", info="#7DB9E8", infoline="#2F4D66",
            danger="#F08A80", dangerline="#6E3732", groupbg="#1A1D21", reviewbg="#1F1716",
            sil="#59616B", silbg="#262B31", chip="#A7AFB9")


def root_css(t):
    return ":root{" + ";".join(f"--{k}:{v}" for k, v in t.items()) + "}"


# ---------------------------------------------------------------- contrast maths
def _lin(c):
    c /= 255
    return c / 12.92 if c <= 0.04045 else ((c + 0.055) / 1.055) ** 2.4


def lum(hexc):
    h = hexc.lstrip("#")
    r, g, b = (int(h[i:i + 2], 16) for i in (0, 2, 4))
    return 0.2126 * _lin(r) + 0.7152 * _lin(g) + 0.0722 * _lin(b)


def ratio(a, b):
    la, lb = sorted((lum(a), lum(b)), reverse=True)
    return round((la + 0.05) / (lb + 0.05), 2)


# Pairs the frames actually use; text needs 4.5:1, non-text indicators 3:1.
TEXT_PAIRS = [("fg", "bg"), ("fg", "selbg"), ("fg", "reviewbg"), ("fg", "panel"), ("fg2", "bg"), ("fg2", "selbg"),
              ("fg2", "groupbg"), ("fg2", "chrome"), ("muted", "bg"), ("muted", "selbg"), ("muted", "panel"),
              ("muted", "reviewbg"), ("faint", "bg"), ("faint", "panel"), ("faint", "selbg"), ("faint", "groupbg"),
              ("faint", "chrome"), ("faint", "reviewbg"), ("warn", "bg"), ("warn", "warnbg"), ("warn", "reviewbg"),
              ("warn", "panel"), ("info", "bg"), ("info", "selbg"), ("info", "panel"), ("danger", "reviewbg"),
              ("onaccent", "accent")]
UI_PAIRS = [("focus", "bg", "focus ring on a row"), ("focus", "selbg", "focus ring on the selected row"),
            ("focus", "panel", "focus ring in the inspector"), ("focus", "chrome", "focus ring on the rail"),
            ("accent", "selbg", "selection bar on its tint"), ("accent", "bg", "selection bar edge"),
            ("line2", "bg", "control outline"), ("line2", "panel", "control outline in the inspector")]


def token_contrast(t):
    text = [{"fg": a, "bg": b, "ratio": ratio(t[a], t[b]), "pass_4_5": ratio(t[a], t[b]) >= 4.5} for a, b in TEXT_PAIRS]
    ui = [{"fg": a, "bg": b, "use": u, "ratio": ratio(t[a], t[b]), "pass_3": ratio(t[a], t[b]) >= 3} for a, b, u in UI_PAIRS]
    return {"text": text, "non_text": ui}


def icon_contrast(t):
    """Share of each icon's opaque pixels (alpha > 200) that reach 3:1 against
    the bare row background and against the backing chip. A coarse measure of
    how much of the original artwork separates from what it sits on; the text
    label is the real fallback. 'transparent' chip = the row background."""
    from PIL import Image
    out = {}
    for tier_, b in ICON_BYTES.items():
        raw = Image.open(io.BytesIO(b)).convert("RGBA").tobytes()
        lums = [0.2126 * _lin(raw[i]) + 0.7152 * _lin(raw[i + 1]) + 0.0722 * _lin(raw[i + 2])
                for i in range(0, len(raw), 4) if raw[i + 3] > 200]

        def share(hexc):
            lb = lum(hexc)
            return round(sum(1 for l in lums if (max(l, lb) + 0.05) / (min(l, lb) + 0.05) >= 3) / len(lums), 2)
        chip = t["bg"] if t["chip"] == "transparent" else t["chip"]
        out[tier_] = {"share_3to1_vs_row_bg": share(t["bg"]), "share_3to1_vs_backing": share(chip)}
    return out


# ---------------------------------------------------------------- shared CSS
BASE_CSS = """
*{box-sizing:border-box;margin:0;padding:0}
html,body{width:{W}px;height:{H}px;overflow:hidden}
body{background:var(--bg);color:var(--fg);font:12.5px/1.3 "Segoe UI Variable Text","Segoe UI",system-ui,sans-serif;font-variant-numeric:tabular-nums}
.app{display:flex;flex-direction:column;width:{W}px;height:{H}px}
.title{height:32px;display:flex;align-items:center;gap:10px;padding:0 0 0 12px;background:var(--chrome);border-bottom:1px solid var(--line);flex:none}
.title .mark{width:16px;height:16px;border-radius:3px;border:2px solid var(--accent)}
.title .name{font-weight:600}
.title .sp{flex:1}
.title .wc{width:46px;height:32px;display:grid;place-items:center;color:var(--muted);font-size:13px}
.chip{display:inline-flex;align-items:center;gap:5px;height:22px;padding:0 8px;border:1px solid var(--line2);border-radius:4px;color:var(--fg2);font-size:12px;white-space:nowrap}
.chip b{color:var(--fg);font-weight:600}
.chip.warn{border-color:var(--warnline);color:var(--warn);background:var(--warnbg)}
.btn{height:28px;padding:0 12px;border-radius:5px;border:1px solid var(--line2);background:var(--btn);color:var(--fg);display:inline-flex;align-items:center;gap:6px;white-space:nowrap;flex:none}
.btn.primary{background:var(--accent);border-color:var(--accent);color:var(--onaccent);font-weight:600}
.field{height:28px;border:1px solid var(--line2);border-radius:5px;background:var(--field);display:flex;align-items:center;gap:6px;padding:0 9px;color:var(--muted);white-space:nowrap;flex:none}
.caret::after{content:"▾";margin-left:6px;color:var(--muted);font-size:10px}
table{border-collapse:collapse;width:100%;table-layout:fixed}
th{height:26px;text-align:left;font-weight:600;color:var(--fg2);font-size:11.5px;border-bottom:1px solid var(--line);padding:0 6px;white-space:nowrap;overflow:hidden;text-overflow:ellipsis;background:var(--thead)}
th .s{color:var(--muted);font-weight:400;margin-left:3px}
td{height:21px;padding:0 6px;white-space:nowrap;overflow:hidden;text-overflow:ellipsis;border-bottom:1px solid var(--rowline)}
td.num,th.num{text-align:right}
td.player{font-weight:500}
.devw{display:inline-flex;align-items:center;gap:5px;vertical-align:middle;height:20px}
.devchip{display:inline-grid;place-items:center;border-radius:4px;background:var(--chip);flex:none}
.devchip img{display:block}
.devl{color:var(--fg)}
.ev.obs,.ev.zero{color:var(--fg2)} .ev.missing{color:var(--warn)}
.flag{display:inline-flex;align-items:center;gap:3px;height:17px;padding:0 5px;border-radius:3px;font-size:11px;border:1px solid}
.flag.corr{color:var(--info);border-color:var(--infoline)}
.flag.id{color:var(--danger);border-color:var(--dangerline)}
.est{color:var(--warn);font-size:11px;margin-left:3px}
.muted{color:var(--muted)} .faint{color:var(--faint)}
tr.group td{height:22px;background:var(--groupbg);color:var(--fg2);font-weight:600;font-size:11.5px}
tr.review td{background:var(--reviewbg)}
/* selection: tint + 4px accent bar + bold name (aria-selected) */
tr.sel td{background:var(--selbg)}
tr.sel td:first-child{box-shadow:inset 4px 0 0 var(--accent)}
tr.sel td.player{font-weight:700}
/* keyboard focus: 2px ring in the focus colour, drawn around the whole row or control */
tr.kf{outline:2px solid var(--focus);outline-offset:-2px}
.kf:not(tr){outline:2px solid var(--focus);outline-offset:2px}
h3{font-size:11.5px;letter-spacing:.04em;text-transform:uppercase;color:var(--muted);font-weight:600;margin:0 0 6px}
h3 .x{text-transform:none;letter-spacing:0;font-weight:400;color:var(--muted)}
.kv{display:grid;grid-template-columns:118px 1fr;row-gap:3px;column-gap:8px;font-size:12px}
.kv dt{color:var(--muted)} .kv dd{color:var(--fg)}
.note{font-size:11.5px;color:var(--muted);line-height:1.35}
.banner{border:1px solid var(--warnline);background:var(--warnbg);color:var(--warn);border-radius:5px;padding:6px 8px;font-size:11.5px;line-height:1.35}
.sil{width:56px;height:56px;flex:none}
.ctx{height:40px;display:flex;align-items:center;gap:8px;padding:0 14px;border-bottom:1px solid var(--line);flex:none;background:var(--bg)}
.ctx .cls{font-size:16px;font-weight:600;margin-right:6px;white-space:nowrap}
.tools{height:42px;display:flex;align-items:center;gap:8px;padding:0 14px;border-bottom:1px solid var(--line);flex:none}
.sp{flex:1}
.sw{display:inline-flex;align-items:center;gap:7px;color:var(--fg2);white-space:nowrap;flex:none}
.sw i{width:30px;height:16px;border-radius:9px;background:var(--accent);position:relative;display:inline-block}
.sw i::after{content:"";position:absolute;right:2px;top:2px;width:12px;height:12px;border-radius:50%;background:var(--panel)}
.main{flex:1;display:flex;min-height:0}
.rail{width:48px;flex:none;border-right:1px solid var(--line);background:var(--chrome);display:flex;flex-direction:column;align-items:center;padding-top:8px;gap:6px}
.rail span{width:34px;height:34px;border-radius:6px;display:grid;place-items:center;color:var(--muted);font-size:15px}
.rail span.on{background:var(--selbg);color:var(--fg);box-shadow:inset 3px 0 0 var(--accent)}
.grid{flex:1;min-width:0;overflow:hidden;background:var(--bg)}
.insp{width:{IW}px;flex:none;border-left:1px solid var(--line);background:var(--panel);padding:12px 14px;display:flex;flex-direction:column;gap:11px;overflow:hidden}
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
.disc{display:flex;align-items:center;justify-content:space-between;border-top:1px solid var(--line);padding-top:8px}
.disc h3{margin:0}
.disc .chev{color:var(--muted);font-size:12px}
.tabs{display:flex;border-bottom:1px solid var(--line);gap:2px}
.tabs span{padding:5px 8px;font-size:12px;color:var(--fg2);border-bottom:3px solid transparent;white-space:nowrap}
.tabs span.on{color:var(--fg);font-weight:600;border-bottom-color:var(--accent)}
.status{height:26px;flex:none;display:flex;align-items:center;border-top:1px solid var(--line);background:var(--chrome);font-size:12px;color:var(--fg2)}
.status span{padding:0 12px;border-right:1px solid var(--line);height:100%;display:flex;align-items:center;gap:5px;white-space:nowrap}
.status b{color:var(--fg);font-weight:600}
.status .sp{border:0}
"""

MEASURE_JS = r"""
(function(){
const R=e=>e.getBoundingClientRect();
const grid=document.querySelector('.grid'), gb=R(grid);
const rows={board:0,review:0,group:0,partial:0}, visible_board=[];
grid.querySelectorAll('tbody tr').forEach(tr=>{const r=R(tr); if(r.top>=gb.bottom-0.5) return;
  if(r.bottom>gb.bottom+0.5){rows.partial++;return}
  if(tr.classList.contains('group'))rows.group++; else if(tr.classList.contains('review'))rows.review++;
  else {rows.board++; visible_board.push(tr.cells[0].textContent.trim())}});
const heads=[...grid.querySelectorAll('thead th')].map(t=>t.textContent.trim());
const trunc=[];
grid.querySelectorAll('thead th').forEach(th=>{if(th.scrollWidth>th.clientWidth)trunc.push({where:'header',text:th.textContent.trim()})});
grid.querySelectorAll('tbody tr').forEach(tr=>{if(R(tr).bottom>gb.bottom+0.5)return;
  [...tr.cells].forEach((td,i)=>{if(td.scrollWidth>td.clientWidth)trunc.push({where:tr.classList.contains('group')?'group row':heads[i],text:td.textContent.trim()})})});
const truncBelow=[];
grid.querySelectorAll('tbody tr').forEach(tr=>{if(R(tr).bottom<=gb.bottom+0.5)return;
  [...tr.cells].forEach((td,i)=>{if(td.scrollWidth>td.clientWidth)truncBelow.push({where:tr.classList.contains('group')?'group row':heads[i],text:td.textContent.trim()})})});
const bars=[];
document.querySelectorAll('.title,.ctx,.tools,.status').forEach(bar=>{const b=R(bar);
  bar.querySelectorAll(':scope > *').forEach(el=>{const r=R(el); if(!r.width)return;
    if(r.right>b.right+0.5||r.left<b.left-0.5)bars.push({bar:bar.className,text:el.textContent.trim(),issue:'outside bar'});
    if(el.scrollWidth>el.clientWidth)bars.push({bar:bar.className,text:el.textContent.trim(),issue:'text overflow'})})});
const insp=document.querySelector('.insp'), ib=R(insp), pb=parseFloat(getComputedStyle(insp).paddingBottom);
const sections=[...insp.children].map(el=>{const r=R(el); const h=el.querySelector('h3,.nm,.tabs');
  return {section:(h?h.textContent:el.textContent).trim().replace(/\s+/g,' ').slice(0,48), bottom:Math.round(r.bottom), fully_visible:r.bottom<=ib.bottom+0.5, inside_padding:r.bottom<=ib.bottom-pb+0.5}});
const inspTrunc=[]; insp.querySelectorAll('*').forEach(el=>{const cs=getComputedStyle(el);
  if(cs.textOverflow==='ellipsis'&&el.scrollWidth>el.clientWidth)inspTrunc.push(el.textContent.trim().slice(0,60))});
function rgba(s){const m=s.match(/[\d.]+/g).map(Number);return {r:m[0],g:m[1],b:m[2],a:m.length>3?m[3]:1}}
function lin(c){c/=255;return c<=0.04045?c/12.92:Math.pow((c+0.055)/1.055,2.4)}
function L(c){return 0.2126*lin(c.r)+0.7152*lin(c.g)+0.0722*lin(c.b)}
function bgOf(el){const stack=[];for(let n=el;n&&n.nodeType===1;n=n.parentElement){const c=rgba(getComputedStyle(n).backgroundColor);if(c.a>0){stack.push(c);if(c.a>=1)break}}
  let out={r:255,g:255,b:255};for(let i=stack.length-1;i>=0;i--){const c=stack[i];out={r:c.r*c.a+out.r*(1-c.a),g:c.g*c.a+out.g*(1-c.a),b:c.b*c.a+out.b*(1-c.a)}}return out}
const app=R(document.querySelector('.app')); const pairs={}; let minFont=99, minFontText='';
document.querySelectorAll('.app *').forEach(el=>{
  if(![...el.childNodes].some(n=>n.nodeType===3&&n.textContent.trim()))return;
  const r=R(el); if(!r.width||!r.height||r.top>=app.bottom||r.left>=app.right)return;
  const g=el.closest('.grid'); if(g&&r.top>=gb.bottom)return;
  const i2=el.closest('.insp'); if(i2&&r.top>=ib.bottom)return;
  const cs=getComputedStyle(el); const fg=rgba(cs.color), bg=bgOf(el);
  const a=L(fg),b=L(bg); const cr=(Math.max(a,b)+0.05)/(Math.min(a,b)+0.05);
  const fs=parseFloat(cs.fontSize), bold=parseInt(cs.fontWeight)>=700;
  if(fs<minFont){minFont=fs;minFontText=el.textContent.trim().slice(0,30)}
  const large=fs>=24||(bold&&fs>=18.66); const need=large?3:4.5;
  const key=cs.color+' on '+`rgb(${Math.round(bg.r)}, ${Math.round(bg.g)}, ${Math.round(bg.b)})`;
  const p=pairs[key]||(pairs[key]={pair:key,ratio:Math.round(cr*100)/100,needed:need,count:0,sample:el.textContent.trim().slice(0,40)});
  p.count++; if(need<p.needed)p.needed=need;});
const contrast=Object.values(pairs).sort((x,y)=>x.ratio-y.ratio);
const out={css_viewport:[document.documentElement.clientWidth>0?parseInt(getComputedStyle(document.body).width):0,parseInt(getComputedStyle(document.body).height)],
  visible_rows:rows, visible_board_numbers:visible_board, truncated_cells:trunc, truncated_cells_in_rows_below_fold:truncBelow, bar_clipping:bars,
  inspector_sections:sections, inspector_ellipsis:inspTrunc, smallest_font_px:minFont, smallest_font_sample:minFontText,
  dom_text_contrast:{pairs:contrast.length, min_ratio:contrast.length?contrast[0].ratio:null, failures:contrast.filter(p=>p.ratio<p.needed), lowest:contrast.slice(0,6)}};
const pre=document.createElement('pre');pre.id='__measure';pre.textContent=JSON.stringify(out);document.documentElement.appendChild(pre);
})();
"""


def page(title, w, h, theme, iw, css, body, measure=False):
    base = BASE_CSS.replace("{W}", str(w)).replace("{H}", str(h)).replace("{IW}", str(iw))
    js = f"<script>{MEASURE_JS}</script>" if measure else ""
    return (f'<!doctype html><html lang="en"><head><meta charset="utf-8"><title>{E(title)}</title>'
            f'<style>{root_css(theme)}{base}{css}</style></head><body>{body}{js}</body></html>')


# ---------------------------------------------------------------- components
def dev_cell(tier_, label=True, size=18):
    alt = "Hidden — turn on Spoilers to reveal" if tier_ == "Unscouted" else f"{tier_} development"
    c = size + 2
    img = (f'<span class="devchip" style="width:{c}px;height:{c}px"><img src="{URI[tier_]}" width="{size}" height="{size}" '
           f'alt="{E(alt)}" title="{E(alt)}"></span>')
    return f'<span class="devw">{img}' + (f'<span class="devl">{tier_}</span>' if label else "") + "</span>"


SILHOUETTE = ('<svg viewBox="0 0 64 64" aria-hidden="true"><rect width="64" height="64" rx="6" fill="var(--silbg)"/>'
              '<circle cx="32" cy="25" r="11" fill="var(--sil)"/><path d="M12 60c2-13 10-19 20-19s18 6 20 19z" fill="var(--sil)"/></svg>')


def titlebar():
    return ('<div class="title"><span class="mark"></span><span class="name">MaddenTool</span>'
            '<span class="sp"></span><span class="wc">—</span><span class="wc">☐</span><span class="wc">✕</span></div>')


def portrait(r):
    return f'<span class="muted">ID {E(r["photo_id"])}</span>' if r["photo_id"] else '<span class="muted">Generic</span>'


def flags(r, compact):
    out = []
    if corrected(r):
        out.append('<span class="flag corr" title="A verified correction changed this row" aria-label="Corrected">✎</span>' if compact else
                   '<span class="flag corr" title="A verified correction changed this row">✎ Corrected</span>')
    if r.get("_review"):
        out.append('<span class="flag id" title="Identity unsettled; not merged" aria-label="Unsettled identity">⚠</span>' if compact else
                   '<span class="flag id" title="Identity unsettled; not merged">⚠ Unsettled ID</span>')
    return "".join(out)


# Column sets. Each tuple: header, width (0 = remainder), css class, cell key.
COLS = {
    "full": [("#", 34, "", "#"), ("Selections", 164, "", "sel"), ("Player", 126, "", "player"), ("Pos", 38, "", "pos"),
             ("College", 110, "", "college"), ("OVR ◇", 58, "num", "ovr"), ("Dev ◇", 112, "", "dev"),
             ("wAV ◇", 50, "num", "wav"), ("Evidence ◇", 126, "", "ev"), ("Portrait", 72, "", "portrait"),
             ("Flags", 0, "", "flags")],
    # 1024x768 and the 960x540 CSS zoom viewport: College and Portrait move to the
    # inspector and the Columns menu; Flags become glyph chips with names.
    "mid": [("#", 34, "", "#"), ("Selections", 156, "", "sel"), ("Player", 120, "", "player"), ("Pos", 38, "", "pos"),
            ("OVR ◇", 50, "num", "ovr"), ("Dev ◇", 94, "", "dev"), ("wAV ◇", 54, "num", "wav"),
            ("Evidence ◇", 0, "", "ev"), ("Flags", 42, "", "flags")],
    # 200% zoom also drops '#': Selections carries the pick and the sort order.
    "zoom": [("Selections", 160, "", "sel"), ("Player", 120, "", "player"), ("Pos", 38, "", "pos"),
             ("OVR ◇", 50, "num", "ovr"), ("Dev ◇", 96, "", "dev"), ("wAV ◇", 54, "num", "wav"),
             ("Evidence ◇", 0, "", "ev"), ("Flags", 42, "", "flags")],
}


def board_table(layout, focus_row=None):
    """focus_row: None -> focus is on the selected row; int -> that board row has focus."""
    compact = layout != "full"
    cols = COLS[layout]
    head = "".join(f'<th class="{c}" style="{f"width:{w}px" if w else ""}" scope="col">{E(n)}'
                   + ('<span class="s" aria-label="sorted ascending">▾</span>' if i == 0 else "") + "</th>"
                   for i, (n, w, c, _) in enumerate(cols))
    review = [dict(r, _review=True) for r in REVIEW[:2]]

    def cells(r, idx):
        ovr, dev, est, _ = illus(r)
        evt, evc = evidence(r, short=compact)
        wav = "—" if r["wav"] is None else str(r["wav"])
        wav_aria = "Sample wAV: missing" if r["wav"] is None else f"Sample wAV {wav}"
        ovr_html = f'{ovr}<span class="est" title="Estimated: wAV missing">est.</span>' if est else str(ovr)
        c = {"#": f'<td class="muted">{idx}</td>' if idx else '<td class="muted">–</td>',
             "sel": f'<td title="{E(sel_text(r))}">{E(sel_text(r))}</td>',
             "player": f'<td class="player">{E(r["name"])}</td>',
             "pos": f'<td>{E(r["position"])}</td>',
             "college": f'<td class="muted">{E(r["college"])}</td>',
             "ovr": f'<td class="num" style="font-weight:600">{ovr_html}</td>',
             "dev": f"<td>{dev_cell(dev)}</td>",
             "wav": f'<td class="num" aria-label="{wav_aria}">{wav}</td>',
             "ev": f'<td><span class="ev {evc}">{E(evt)}</span></td>',
             "portrait": f"<td>{portrait(r)}</td>",
             "flags": f"<td>{flags(r, compact)}</td>"}
        return "".join(c[k] for *_, k in cols)

    n = len(cols)
    if layout == "full":
        body = [f'<tr class="group review"><td colspan="{n}">⚠ Needs review · 2 unsettled identities in this class — '
                'shown as separate people, not merged <span class="muted">(+1 more pair: Bob Young × 2)</span></td></tr>']
    else:
        body = [f'<tr class="group review"><td colspan="{n}">⚠ Needs review · 2 unsettled identities, kept separate '
                '<span class="muted">(+ Bob Young × 2)</span></td></tr>']
    body += [f'<tr class="review">{cells(r, None)}</tr>' for r in review]
    body.append(f'<tr class="group"><td colspan="{n}">Board · round, then NFL before AFL, then pick '
                '<span class="muted">(provisional)</span></td></tr>')
    for i, r in enumerate(D["board"], 1):
        cls = []
        attrs = ""
        if r["name"] == "Bob Brown":
            cls.append("sel")
            attrs += ' aria-selected="true"'
            if focus_row is None:
                cls.append("kf")
        if focus_row == i:
            cls.append("kf")
        if cls:
            attrs = f' class="{" ".join(cls)}"' + attrs
        body.append(f"<tr{attrs}>{cells(r, i)}</tr>")
    return (f'<table role="grid" aria-label="1964 class board" aria-rowcount="{len(D["board"]) + 2}">'
            f'<thead><tr>{head}</tr></thead><tbody>{"".join(body)}</tbody></table>')


def insp_head(compact):
    ovr, dev, _, _ = illus(BOB)
    sub = f'{E(BOB["position"])} · {E(BOB["college"])} · 1964 class'
    sub2 = f'<div class="sub muted">Portrait ID {E(BOB["photo_id"])}</div>' if compact else ""
    return (f'<div class="ihead"><div class="sil">{SILHOUETTE}</div><div><div class="nm">{E(BOB["name"])}</div>'
            f'<div class="sub">{sub}</div>{sub2}'
            f'<div class="big"><span class="o">{ovr}</span><span class="muted" style="font-size:11px">OVR ◇</span>'
            f'{dev_cell(dev, size=24)}</div></div></div>')


def insp_selections(narrow=False):
    nfl, afl = BOB["selections"]
    ml = 38 if narrow else 48
    return (f'<div class="sec"><h3>Selections · one person</h3>'
            f'<div class="selrow"><span class="lg">NFL</span><span>Round 1, pick 2 · {E(nfl["position"])} · {E(nfl["college"])}'
            f'{"" if narrow else " <span class=muted>· legacy lookup</span>"}</span></div>'
            f'<div class="selrow"><span class="lg">AFL</span><span>Round 1, pick 4 · {E(afl["position"])} · {E(afl["college"])} '
            f'<span class="flag corr">✎ Corrected</span></span></div>'
            f'<div class="corrbox" style="margin-left:{ml}px">Source row said <s>Ark-Pine Bluff, DT</s> (a namesake\'s). '
            f'4 citations · correction <code>1964-afl-r1p4-bob-brown</code> · <span class="link">View evidence</span></div>'
            f'<div class="note" style="margin:4px 0 0 {ml}px">Merged by DG01 rule: same name and college, after the verified correction</div>'
            f'<div class="note" style="margin-top:6px">Different person: Robert Brown · NFL r13 p169 · '
            f'{E(NAMESAKE["college"])} {E(NAMESAKE["position"])} · own row</div></div>')


def insp_career():
    car = SAMPLE_CAREER
    return (f'<div class="sec"><h3>Career record <span class="x">· ◇ sample, not this player\'s record · hindsight</span></h3>'
            f'<dl class="kv"><dt>Seasons</dt><dd>{car["seasons"]} · career ended <span class="muted">◇ sample</span></dd>'
            f'<dt>Pro Bowls · All-Pro</dt><dd>{car["pb"]} · {car["ap1"]} <span class="muted">◇ sample</span></dd>'
            f'<dt>Years as starter</dt><dd>{car["st"]} <span class="muted">◇ sample</span></dd></dl></div>')


def insp_why(short_banner=False):
    _, _, _, pct = illus(BOB)
    evt, _ = evidence(BOB)
    banner = ("◇ Sample inputs; OVR, dev tier and percentile are illustrative. Ratings model (DG04b) not built."
              if short_banner else
              "◇ Concept: wAV, evidence and career numbers are sample inputs, not this player's record. OVR, dev tier "
              "and percentile are illustrative. The ratings model (DG04b) is not built.")
    return (f'<div class="sec"><h3>Why this rating — inputs only</h3>'
            f'<dl class="kv"><dt>Lens</dt><dd>Career <span class="muted">(example setting)</span></dd>'
            f'<dt>Hindsight</dt><dd>1.00 · career outcome</dd>'
            f'<dt>wAV</dt><dd>{BOB["wav"]} <span class="muted">◇ sample input</span></dd>'
            f'<dt>Evidence</dt><dd>{E(evt)} <span class="muted">◇ sample</span></dd>'
            f'<dt>Comparison group</dt><dd>OL · 1960s era band <span class="muted">(proposed)</span></dd>'
            f'<dt>Group percentile</dt><dd>{pct}th <span class="muted">◇ illustrative</span></dd>'
            f'<dt>Estimated</dt><dd>No</dd></dl>'
            f'<div class="banner" style="margin-top:7px">{banner}</div></div>')


def insp_prov():
    return (f'<div class="sec"><h3>Provenance</h3><dl class="kv"><dt>Identity, selections</dt><dd>Legacy ALL_PLAYER_LOOKUP.csv</dd>'
            f'<dt>Import as-of</dt><dd>{AS_OF} · sha256 {SHA}</dd>'
            '<dt>Stats shown</dt><dd>◇ Sample inputs (concept)</dd>'
            '<dt>Attribution</dt><dd>Pro Football Reference draft tables; retrieval date not recorded</dd></dl></div>')


def disclosure(title, hint):
    return (f'<div class="disc" role="button" aria-expanded="false"><h3>{title} <span class="x">{hint}</span></h3>'
            '<span class="chev" aria-hidden="true">▸</span></div>')


def inspector(layout):
    if layout == "full":
        return insp_head(False) + insp_selections() + insp_career() + insp_why() + insp_prov()
    if layout == "mid":
        # Height is the constraint at 768: Selections and the rating explanation stay
        # open; the sample career record and provenance collapse to one line each.
        return (insp_head(True) + insp_selections(narrow=True) + insp_why(short_banner=True)
                + disclosure("Career record", "· ◇ sample · hindsight") + disclosure("Provenance", "· legacy lookup · 2026-09-15"))
    # zoom: same docked inspector; sections become tabs so one fits the 540px height.
    tabs = ('<div class="tabs" role="tablist" aria-label="Player details">'
            '<span class="on" role="tab" aria-selected="true">Selections</span><span role="tab">Rating ◇</span>'
            '<span role="tab">Career ◇</span><span role="tab">Source</span></div>')
    return insp_head(False) + tabs + insp_selections(narrow=True).replace('<div class="sec"><h3>Selections · one person</h3>',
                                                                       '<div role="tabpanel" aria-label="Selections">')


RAIL = ('<div class="rail" role="navigation" aria-label="Views"><span class="on" title="Class board" aria-current="page">▤</span>'
        '<span title="Data and sources">⛁</span><span title="Class Studio">✚</span><span title="Settings">⚙</span></div>')


def frame(theme, w, h, layout, focus_row=None, measure=False):
    t = LIGHT if theme == "light" else DARK
    sw = '<span class="sw" role="switch" aria-checked="true"><i></i>Spoilers on</span>'
    if layout == "full":
        iw, extra = 392, ""
        ctx = ('<div class="ctx"><span class="cls">1964 class</span><span class="chip">AFL + NFL</span>'
               '<span class="chip">Lens <b>Career</b> · example</span><span class="chip">Hindsight <b>1.00</b></span>'
               '<span class="chip">Seed <b>7F3A-21C9</b> · example</span>'
               f'<span class="chip">Offline · data as-of <b>{AS_OF}</b></span><span class="sp"></span>'
               '<span class="chip warn">◇ Concept · sample stats · illustrative ratings</span></div>')
        tools = ('<div class="tools"><span class="field" style="width:220px">⌕ Search 1964 class</span>'
                 '<span class="field caret">Position</span><span class="field caret">League: AFL + NFL</span>'
                 '<span class="field caret">Evidence</span><span class="field caret">Sort: Selection</span>'
                 f'<span class="btn">▦ Columns</span><span class="sp"></span>{sw}'
                 '<span class="btn">Generate…</span><span class="btn primary">Validate &amp; export…</span></div>')
        status = (f'<div class="status"><span>Active <b>{ACTIVE}</b></span>'
                  '<span>Export limit <b>402</b> <span class="muted">· in-game import of 402 not yet verified</span></span>'
                  '<span>File capacity <b>455</b></span><span>Overflow <b>0</b></span>'
                  '<span>Identities: <b>2</b> unsettled</span><span class="sp"></span>'
                  '<span style="border-left:1px solid var(--line)">Validation not run · concept</span></div>')
    elif layout == "mid":
        iw = 296
        extra = """.rail{width:44px}.rail span{width:32px;height:32px}
.insp{padding:11px 12px;gap:9px}.ihead .nm{font-size:16px}.kv{grid-template-columns:104px 1fr}
.ctx{gap:6px;padding:0 12px}.ctx .cls{font-size:15px;margin-right:2px}.tools{gap:7px;padding:0 12px}"""
        ctx = ('<div class="ctx"><span class="cls">1964 class</span><span class="chip">AFL + NFL</span>'
               '<span class="chip">Lens <b>Career</b> · example</span><span class="chip">Hindsight <b>1.00</b></span>'
               '<span class="chip">Seed <b>7F3A-21C9</b> · example</span>'
               f'<span class="chip">Offline · as-of <b>{AS_OF}</b></span><span class="sp"></span>'
               '<span class="chip warn">◇ Sample stats</span></div>')
        tools = ('<div class="tools"><span class="field" style="width:190px">⌕ Search 1964 class</span>'
                 '<span class="btn caret">Filters · 0</span><span class="field caret">Sort: Selection</span>'
                 f'<span class="btn">▦ Columns · 2 hidden</span><span class="sp"></span>{sw}'
                 '<span class="btn">Generate…</span><span class="btn primary">Validate &amp; export…</span></div>')
        status = (f'<div class="status"><span>Active <b>{ACTIVE}</b></span>'
                  '<span>Export limit <b>402</b> <span class="muted">· not yet verified in game</span></span>'
                  '<span>File capacity <b>455</b></span><span>Overflow <b>0</b></span>'
                  '<span><b>2</b> unsettled IDs</span><span class="sp"></span>'
                  '<span style="border-left:1px solid var(--line)">Validation not run</span></div>')
    else:  # zoom: 960x540 CSS viewport
        iw = 268
        extra = """.rail{width:40px;gap:4px;padding-top:6px}.rail span{width:30px;height:30px;font-size:14px}
.ctx{height:36px;gap:6px;padding:0 10px}.ctx .cls{font-size:15px;margin-right:2px}
.tools{height:40px;gap:6px;padding:0 10px}.btn{padding:0 10px}.field{padding:0 8px}
.insp{padding:10px 12px;gap:8px}.ihead .nm{font-size:16px}.big{margin-top:2px}.big .o{font-size:20px}
.title{height:30px}.title .wc{height:30px;width:40px}"""
        ctx = ('<div class="ctx"><span class="cls">1964 class</span><span class="chip">AFL + NFL</span>'
               '<span class="chip">Career · h <b>1.00</b> · example</span><span class="chip">Seed <b>7F3A-21C9</b></span>'
               f'<span class="chip">Offline · <b>{AS_OF}</b></span><span class="sp"></span>'
               '<span class="chip warn">◇ Sample stats</span></div>')
        tools = ('<div class="tools"><span class="field" style="width:150px">⌕ Search</span>'
                 '<span class="btn caret">Filters · 0</span><span class="field caret">Sort: Selection</span>'
                 f'<span class="btn">▦ Columns</span><span class="sp"></span>{sw}'
                 '<span class="btn">Generate…</span><span class="btn primary">Validate &amp; export…</span></div>')
        status = (f'<div class="status"><span>Active <b>{ACTIVE}</b></span>'
                  '<span>Export limit <b>402</b> <span class="muted">· unverified in game</span></span>'
                  '<span>Capacity <b>455</b></span><span>Overflow <b>0</b></span>'
                  '<span><b>2</b> unsettled IDs</span></div>')
    body = (f'<div class="app">{titlebar()}{ctx}{tools}<div class="main">{RAIL}'
            f'<div class="grid">{board_table(layout, focus_row)}</div>'
            f'<aside class="insp" aria-label="Player inspector: Bob Brown">{inspector(layout)}</aside></div>{status}</div>')
    return page(f"MaddenTool board concept A ({theme}, {layout})", w, h, t, iw, extra, body, measure)


# ---------------------------------------------------------------- rendering
def chrome(args):
    subprocess.run([CHROME, "--headless=new", "--disable-gpu", "--hide-scrollbars", "--no-first-run",
                    "--disable-extensions", *args], check=True, capture_output=True, timeout=120)


def file_url(p):
    return "file:///" + os.path.abspath(p).replace("\\", "/")


def shoot(name, w, h, dsf, render):
    hp = os.path.join(OUT, name + ".html")
    open(hp, "w", encoding="utf-8", newline="\n").write(render(False))
    png = os.path.join(OUT, name + ".png")
    chrome([f"--force-device-scale-factor={dsf}", f"--window-size={w},{h}", f"--screenshot={png}", file_url(hp)])
    mp = os.path.join(OUT, name + ".measure.html")
    open(mp, "w", encoding="utf-8", newline="\n").write(render(True))
    dom = subprocess.run([CHROME, "--headless=new", "--disable-gpu", "--hide-scrollbars", "--no-first-run",
                          "--disable-extensions", f"--force-device-scale-factor={dsf}", f"--window-size={w},{h}",
                          "--dump-dom", file_url(mp)], check=True, capture_output=True, timeout=120).stdout.decode("utf-8")
    m = re.search(r'<pre id="__measure">(.*?)</pre>', dom, re.S)
    assert m, f"{name}: no measurement"
    os.remove(mp)
    return png, json.loads(html.unescape(m.group(1)))


FRAMES = [
    # name, theme, css w, css h, device scale, layout, focus row
    ("A-light-1440x900", "light", 1440, 900, 1, "full", None),
    ("A-light-focus-1440x900", "light", 1440, 900, 1, "full", 5),
    ("A-light-1024x768", "light", 1024, 768, 1, "mid", None),
    ("A-dark-1024x768", "dark", 1024, 768, 1, "mid", None),
    ("A-light-zoom200-960x540css", "light", 960, 540, 2, "zoom", None),
]


def focus_detail(paths):
    """Composite of focus/selection crops, in colour and in true luminance
    greyscale (PIL 'L'), to show the ring survives without hue."""
    from PIL import Image
    crops = []
    light = Image.open(paths["A-light-focus-1440x900"]).convert("RGB").crop((48, 222, 48 + 640, 222 + 120))
    dark = Image.open(paths["A-dark-1024x768"]).convert("RGB").crop((44, 222, 44 + 640, 222 + 120))
    for lab, im in [("Light · focus on row 5, selection on row 2", light), ("Dark · focus and selection on row 2", dark)]:
        for mode in ("colour", "greyscale"):
            j = im if mode == "colour" else im.convert("L").convert("RGB")
            buf = io.BytesIO()
            j.resize((j.width * 2, j.height * 2), Image.NEAREST).save(buf, "PNG", optimize=False)
            crops.append((f"{lab} · {mode}", "data:image/png;base64," + base64.b64encode(buf.getvalue()).decode()))
    cells = "".join(f'<figure><figcaption>{E(t)}</figcaption><img src="{u}" width="1280" height="240" alt="{E(t)}"></figure>'
                    for t, u in crops)
    doc = ('<!doctype html><html><head><meta charset="utf-8"><style>*{margin:0;box-sizing:border-box}'
           'html,body{width:2600px;height:650px;overflow:hidden;background:#FFFFFF;color:#1B2433;'
           'font:600 20px "Segoe UI",system-ui,sans-serif}'
           '.sample-note{height:50px;padding:0 12px;display:flex;align-items:center;font-size:24px}'
           'main{display:grid;grid-template-columns:1280px 1280px;gap:16px 24px;padding:12px 12px}'
           'figcaption{height:30px}img{display:block;border:1px solid #B9B3A7}</style></head>'
           f'<body><main>{cells}</main>'
           '<p class="sample-note">Sample inputs: wAV and evidence. OVR and dev tiers are illustrative. '
           'Identity and draft facts are real.</p></body></html>')
    hp = os.path.join(OUT, "A-focus-detail.html")
    open(hp, "w", encoding="utf-8", newline="\n").write(doc)
    png = os.path.join(OUT, "A-focus-detail.png")
    chrome(["--force-device-scale-factor=1", "--window-size=2600,650", f"--screenshot={png}", file_url(hp)])
    return png


def parity_with_concept_a():
    """Concept A's own script must give the same sample and illustrative values."""
    import importlib.util
    a_path = os.path.normpath(os.path.join(HERE, "..", "..", "dg05-concept-a", "source", "build_frames.py"))
    saved = sys.argv
    sys.argv = [a_path, ICON_DIR, OUT, ROWS]
    try:
        spec = importlib.util.spec_from_file_location("concept_a_frames", a_path)
        a = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(a)
    finally:
        sys.argv = saved
    mism = [r["name"] for ra, r in zip(a.D["board"], D["board"])
            if (ra["wav"], a.illus(ra)) != (r["wav"], illus(r))]
    return {"board_rows_compared": len(D["board"]), "mismatches": mism, "career_sample_equal": a.SAMPLE_CAREER == SAMPLE_CAREER}


if __name__ == "__main__":
    os.makedirs(OUT, exist_ok=True)
    checks = {"inputs": {"identity_rows_sha256": hashlib.sha256(open(ROWS, "rb").read()).hexdigest(),
                         "icons_verified": {t: f for t, (f, _) in ICONS.items()}},
              "parity_with_concept_a": parity_with_concept_a(), "frames": {},
              "token_contrast": {"light": token_contrast(LIGHT), "dark": token_contrast(DARK)},
              "icon_contrast": {"light": icon_contrast(LIGHT), "dark": icon_contrast(DARK)}}
    pngs = {}
    for name, theme, w, h, dsf, layout, fr in FRAMES:
        png, meas = shoot(name, w, h, dsf, lambda m, a=(theme, w, h, layout, fr): frame(*a[:4], focus_row=a[4], measure=m))
        from PIL import Image
        meas.update({"theme": theme, "css_viewport": [w, h], "device_scale_factor": dsf,
                     "png_size": list(Image.open(png).size), "png_sha256": hashlib.sha256(open(png, "rb").read()).hexdigest()})
        checks["frames"][name] = meas
        pngs[name] = png
        vr = meas["visible_rows"]
        print(f"{name}: png {meas['png_size']} rows board={vr['board']} review={vr['review']} group={vr['group']} "
              f"partial={vr['partial']} trunc={len(meas['truncated_cells'])} bars={len(meas['bar_clipping'])} "
              f"contrast_min={meas['dom_text_contrast']['min_ratio']} fails={len(meas['dom_text_contrast']['failures'])}")
    fd = focus_detail(pngs)
    checks["focus_detail_png_sha256"] = hashlib.sha256(open(fd, "rb").read()).hexdigest()
    # Public-boundary scan of every frame document that was rendered.
    banned = re.compile(r"[A-Za-z]:\\|/Users/|/home/|AppData|@[\w.-]+\.(?:com|org|net)\b|career_to|\"uid\"", re.I)
    checks["public_scan"] = {n: bool(banned.search(open(os.path.join(OUT, n + ".html"), encoding="utf-8").read()))
                             for n, *_ in FRAMES}
    assert not any(checks["public_scan"].values()), checks["public_scan"]
    json.dump(checks, open(os.path.join(OUT, "checks.json"), "w", encoding="utf-8", newline="\n"), indent=1, ensure_ascii=False)
    print("parity", checks["parity_with_concept_a"])
