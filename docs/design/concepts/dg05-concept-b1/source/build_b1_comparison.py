"""Build the DG05-CONCEPT-B1 comparison page (all images inlined as data URIs).

    python -I build_b1_comparison.py <legacy dev-icons dir> <concept-b1 dir> [inline-out.html]

Reads assembled/*.png and assembled/checks.json (written by build_b1.py), so
every number on the page is the measured one. The optional third argument
writes a small preview-host page that references the PNGs by absolute local
path; it is machine-specific and must be written outside the repository.
"""
import base64, hashlib, html, json, os, sys

ICON_DIR, ROOT = sys.argv[1], sys.argv[2]
E = html.escape
C = json.load(open(os.path.join(ROOT, "assembled", "checks.json"), encoding="utf-8"))
F = C["frames"]

ICONS = [("Normal", "slow.png", "bbd4fd7a9492b74859cc88aff4c32704af88b2c7d70a2a193fcdadabcb910a87"),
         ("Star", "normal.png", "d66cddd97512c5ec224c56a7f44f5db879c7719479e0f33a60a0c7f8a0bd5d34"),
         ("Superstar", "quick.png", "3f0cab1c56ac6e777f68491547476a92f28cb17e772e1e289499141faecec428"),
         ("X-Factor", "superstar.png", "c484dc2c92981295046962fb225621e18496a232451f9acfa8cef72afd3ad038"),
         ("Unscouted", "hidden.png", "07abd50c1bb48d2b925d41617ad5d1f0d2b47b1d51134e72c3248631eeb15b2c")]


def uri(path):
    return "data:image/png;base64," + base64.b64encode(open(path, "rb").read()).decode()


def rows(name):
    v = F[name]["visible_rows"]
    return f'{v["board"] + v["review"]} fully visible rows ({v["board"]} board + {v["review"]} review), {v["partial"]} partly visible'


def tc(theme, fg, bg):
    return next(p["ratio"] for p in C["token_contrast"][theme]["text"] if p["fg"] == fg and p["bg"] == bg)


def ui(theme, use):
    return next(p["ratio"] for p in C["token_contrast"][theme]["non_text"] if p["use"] == use)


FIGS = [
    ("A-light-1440x900", "A · light · 1440×900",
     "Same docked-inspector structure as the selected dark frame, in B's ledger palette. Bob Brown is selected (tint, 4px green bar, bold name) and has keyboard focus (blue ring). Every column of the dark frame is kept; all four inspector sections are open. " + rows("A-light-1440x900") + "."),
    ("A-light-1024x768", "A · light · 1024×768",
     "Column priority: College and Portrait move to the inspector and the Columns menu (“2 hidden”); Flags become glyph chips with accessible names; evidence uses the short form. Position, League and Evidence filters fold into one Filters button; every action stays visible. The inspector keeps Selections and “Why this rating” open; Career record and Provenance collapse to one-line disclosures. " + rows("A-light-1024x768") + "."),
    ("A-dark-1024x768", "A · dark · 1024×768",
     "The same layout in A's dark palette. Dev icons sit on a neutral backing chip (icon bytes unchanged) because several icons are dark artwork. Faint text and control outlines are raised against concept A's dark tokens. " + rows("A-dark-1024x768") + "."),
    ("A-light-zoom200-960x540css", "A · light · 200% zoom",
     "Physical window 1920×1080 at 200% zoom, so the effective CSS viewport is 960×540; the PNG is 1920×1080 physical pixels. The layout really reflows to 960 CSS px: '#' is dropped (Selections carries the pick and sort), the inspector stays docked at 268 CSS px and turns its sections into tabs (Selections · Rating ◇ · Career ◇ · Source). Shown here at half size; open 1:1 to see physical pixels. " + rows("A-light-zoom200-960x540css") + "."),
    ("A-light-focus-1440x900", "Keyboard focus vs selection · light · 1440×900 (state concept)",
     "Focus moved to row 5 (Bill Munson) without changing the selection: row 2 keeps tint, bar and bold name, and the inspector still shows Bob Brown. Drawn as a static state; no key was pressed."),
    ("A-focus-detail", "Focus detail · colour and greyscale (crops enlarged 2×)",
     "Crops from the focus frame (light) and the dark 1024 frame, each also converted to luminance greyscale. The ring stays visible without hue, and the selection still reads by its bar and bold name."),
]


def size_of(name):
    return (2600, 600) if name == "A-focus-detail" else tuple(F[name]["png_size"])


def fig(name, title, cap):
    w, h = size_of(name)
    dw = 960 if name.startswith("A-light-zoom") else w
    dh = round(h * dw / w)
    return (f'<figure><div class="frame"><img src="{uri(os.path.join(ROOT, "assembled", name + ".png"))}" width="{dw}" height="{dh}" '
            f'data-w="{w}" data-h="{h}" alt="{E(title)}" loading="lazy"></div><figcaption><span class="tag">Assembled · static layout, headless render</span>'
            f'<b>{E(title)}</b> <span class="dim">{w}×{h} px · <code>assembled/{name}.png</code></span><br>{E(cap)}</figcaption></figure>')


legend = []
for label, fname, sha in ICONS:
    p = os.path.join(ICON_DIR, fname)
    assert hashlib.sha256(open(p, "rb").read()).hexdigest() == sha, fname
    alt = "Hidden — turn on Spoilers to reveal" if fname == "hidden.png" else f"{label} development"
    ic = C["icon_contrast"]
    legend.append(f'<tr><td><img src="{uri(p)}" width="32" height="32" alt="{alt}"></td><td><b>{label}</b></td><td><code>{fname}</code></td>'
                  f'<td>{ic["light"][label]["share_3to1_vs_backing"]:.0%}</td><td>{ic["dark"][label]["share_3to1_vs_row_bg"]:.0%}</td>'
                  f'<td>{ic["dark"][label]["share_3to1_vs_backing"]:.0%}</td></tr>')

meas_rows = "".join(
    f'<tr><td><code>{n}</code></td><td>{f["css_viewport"][0]}×{f["css_viewport"][1]} CSS · DPR {f["device_scale_factor"]}</td>'
    f'<td>{f["png_size"][0]}×{f["png_size"][1]}</td><td>{f["visible_rows"]["board"]} + {f["visible_rows"]["review"]}</td>'
    f'<td>{len(f["truncated_cells"])}</td><td>{len(f["bar_clipping"])}</td>'
    f'<td>{sum(1 for s in f["inspector_sections"] if s["fully_visible"])}/{len(f["inspector_sections"])}</td>'
    f'<td>{f["smallest_font_px"]:g}px</td><td>{f["dom_text_contrast"]["min_ratio"]}:1 · {len(f["dom_text_contrast"]["failures"])} fail</td></tr>'
    for n, f in F.items())

contrast_rows = "".join(
    f'<tr><td>{E(lbl)}</td><td>{a}:1</td><td>{b}:1</td><td>{E(need)}</td></tr>' for lbl, a, b, need in [
        ("Body text on board", tc("light", "fg", "bg"), tc("dark", "fg", "bg"), "4.5"),
        ("Body text on selected row", tc("light", "fg", "selbg"), tc("dark", "fg", "selbg"), "4.5"),
        ("Secondary (muted) on selected row", tc("light", "muted", "selbg"), tc("dark", "muted", "selbg"), "4.5"),
        ("Faint text, lowest pair (on selected row)", tc("light", "faint", "selbg"), tc("dark", "faint", "selbg"), "4.5"),
        ("Primary button label", tc("light", "onaccent", "accent"), tc("dark", "onaccent", "accent"), "4.5"),
        ("Warning text on warning chip", tc("light", "warn", "warnbg"), tc("dark", "warn", "warnbg"), "4.5"),
        ("Focus ring vs row", ui("light", "focus ring on a row"), ui("dark", "focus ring on a row"), "3 (non-text)"),
        ("Focus ring vs selected row", ui("light", "focus ring on the selected row"), ui("dark", "focus ring on the selected row"), "3 (non-text)"),
        ("Selection bar vs its tint", ui("light", "selection bar on its tint"), ui("dark", "selection bar on its tint"), "3 (non-text)"),
        ("Control outline vs board", ui("light", "control outline"), ui("dark", "control outline"), "3 (non-text)"),
    ])

HTML = f"""<!doctype html><html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">
<title>Scouting Desk Variants</title>
<style>
:root{{--bg:#F6F4EF;--surface:#FFFFFF;--fg:#1B2433;--muted:#5D6778;--line:#DAD5CB;--accent:#2F6B4F;--warn:#8A5A0B;--warnbg:#FBF3E2;--asm:#1F5F8B;--asmbg:#E6F0F8}}
@media (prefers-color-scheme:dark){{:root:not([data-theme="light"]){{--bg:#111316;--surface:#191C20;--fg:#E8EAED;--muted:#A0A8B3;--line:#2E333A;--accent:#7FC3A0;--warn:#E9B45C;--warnbg:#2A2113;--asm:#8CC3EC;--asmbg:#14283A}}}}
:root[data-theme="dark"]{{--bg:#111316;--surface:#191C20;--fg:#E8EAED;--muted:#A0A8B3;--line:#2E333A;--accent:#7FC3A0;--warn:#E9B45C;--warnbg:#2A2113;--asm:#8CC3EC;--asmbg:#14283A}}
*{{box-sizing:border-box}}
body{{margin:0;background:var(--bg);color:var(--fg);font:15px/1.55 "Segoe UI Variable Text","Segoe UI",system-ui,-apple-system,sans-serif}}
main{{max-width:1500px;margin:0 auto;padding:28px 16px 64px}}
h1{{font-size:26px;margin:0 0 4px}} h2{{font-size:19px;margin:40px 0 12px;padding-top:12px;border-top:1px solid var(--line)}}
p{{margin:0 0 10px;max-width:90ch}} .dim{{color:var(--muted)}} code{{font:12.5px Consolas,ui-monospace,monospace}}
.callout{{border:1px solid var(--warn);background:var(--warnbg);border-radius:8px;padding:10px 14px;margin:14px 0;max-width:110ch}}
figure{{margin:0 0 30px}}
.frame{{overflow-x:auto;border:1px solid var(--line);border-radius:8px;background:var(--surface)}}
.frame img{{display:block;max-width:none}}
.fit .frame img{{max-width:100%;height:auto}}
figcaption{{margin-top:8px;max-width:125ch}}
.tag{{display:inline-block;font-size:12px;font-weight:600;border-radius:4px;padding:1px 7px;margin-right:8px;color:var(--asm);background:var(--asmbg)}}
.tw{{overflow-x:auto}}
table.t{{border-collapse:collapse;background:var(--surface);font-size:14px;min-width:100%}}
.t th,.t td{{border:1px solid var(--line);padding:6px 9px;text-align:left;vertical-align:top}}
.t th{{font-weight:600}}
ol li,ul li{{margin:3px 0}}
label.fitto{{font-size:13px;color:var(--muted);display:inline-flex;gap:6px;align-items:center;margin-bottom:10px}}
</style></head><body><main>
<h1>Scouting Desk: light, compact and zoomed</h1>
<p class="dim">DG05-CONCEPT-B1 extends the selected direction A (a table with a docked player inspector). These are <b>static design frames</b>, not the app: nothing here generates, imports or exports, and the focus states are drawn, not produced by keyboard input.</p>
<div class="callout"><b>Data in the frames.</b> Names, draft selections, colleges, source positions and the Bob Brown correction are real identity facts from a local import. <b>wAV, evidence states and career numbers are ◇ sample inputs</b>, not any player's record; OVR, dev tier and percentile are <b>illustrative</b> values computed from those samples, identical to concept A's. Lens, hindsight and seed are example settings, not chosen defaults.</div>

<h2>Frames</h2>
<label class="fitto"><input type="checkbox" id="fit" checked> Fit frames to page width (uncheck for 1:1)</label>
<div id="frames" class="fit">{''.join(fig(*x) for x in FIGS)}</div>

<h2>Column and inspector priority</h2>
<div class="tw"><table class="t"><tr><th>Width</th><th>Board columns</th><th>Moved out (where)</th><th>Inspector</th><th>Toolbar</th></tr>
<tr><td>1440×900</td><td># · Selections · Player · Pos · College · OVR ◇ · Dev ◇ · wAV ◇ · Evidence ◇ · Portrait · Flags</td><td>—</td><td>392px, all sections open</td><td>Search, Position, League, Evidence, Sort, Columns, Spoilers, Generate…, Validate &amp; export…</td></tr>
<tr><td>1024×768</td><td># · Selections · Player · Pos · OVR ◇ · Dev ◇ · wAV ◇ · Evidence ◇ (short) · Flags (glyph)</td><td>College, Portrait → inspector header and Columns menu</td><td>296px; Selections and Why open; Career record, Provenance collapsed</td><td>Search, Filters (Position, League, Evidence), Sort, Columns · 2 hidden, Spoilers, Generate…, Validate &amp; export…</td></tr>
<tr><td>960×540 CSS (200%)</td><td>Selections · Player · Pos · OVR ◇ · Dev ◇ · wAV ◇ · Evidence ◇ (short) · Flags (glyph)</td><td>'#' (pick is in Selections); College, Portrait as at 1024</td><td>268 CSS px, still docked; sections become tabs: Selections · Rating ◇ · Career ◇ · Source</td><td>As at 1024; Validation status moves out of the status bar into Validate &amp; export</td></tr></table></div>
<p class="dim" style="margin-top:8px">Never dropped: Player, Pos, OVR, Dev icon <i>and</i> label, wAV, evidence text, the three separate counts, the Spoilers switch and Validate &amp; export. With spoilers off, the compact layouts follow concept A's rule: OVR, Dev, wAV and Evidence columns are not rendered (one Scouting column with <code>hidden.png</code>), rating sorts and the evidence filter disappear, and the Rating and Career tabs are absent rather than disabled.</p>

<h2>Keyboard focus (state concept)</h2>
<p><b>Selection</b> = row tint + 4px accent bar on the left edge + bold player name, and the inspector shows that player (<code>aria-selected</code>). <b>Focus</b> = a 2px ring in a blue focus colour around the whole row or control. The two use different shapes (edge bar vs enclosing ring), so neither depends on hue; the greyscale crops above show both survive without colour. Focus colour is the same blue family in both themes, separate from the theme accent.</p>
<p><b>Proposed tab sequence</b> (one stop per region; arrows move inside a region; F6 cycles regions):</p>
<ol><li>Skip link “Skip to board” (visible on focus)</li>
<li>View rail (one stop; ↑/↓ between views)</li>
<li>Class context chips (one stop; ←/→ between chips; Enter opens a chip's popover)</li>
<li>Toolbar: Search → Filters → Sort → Columns → Spoilers → Generate… → Validate &amp; export…</li>
<li>Board grid (one stop, roving focus): ↑/↓ moves focus <i>and</i> selection; Ctrl+↑/↓ moves focus only; Space selects the focused row; Home/End, PgUp/PgDn; Enter moves into the inspector</li>
<li>Inspector: tabs (←/→ at 200% zoom) or disclosure headers, then links such as “View evidence”; Esc returns focus to the selected row</li>
<li>Status bar links (Overflow, unsettled IDs); counts themselves are read-only text</li></ol>
<p class="dim">Not tested: nothing was exercised with a keyboard or a screen reader. The frames carry roles (<code>grid</code>, <code>switch</code>, <code>tablist</code>, <code>aria-selected</code>) as annotations only.</p>

<h2>Measured on the rendered frames</h2>
<p class="dim">From <code>assembled/checks.json</code>, written by <code>build_b1.py</code> from the frame DOM inside headless Chrome. Rows count only rows whose full height is inside the board; clipped cells use <code>scrollWidth &gt; clientWidth</code> on every visible cell and header; contrast walks every visible text element against its painted background.</p>
<div class="tw"><table class="t"><tr><th>Frame</th><th>Viewport</th><th>PNG</th><th>Rows (board + review)</th><th>Clipped cells</th><th>Bar overflow</th><th>Inspector sections whole</th><th>Smallest text</th><th>Lowest text contrast</th></tr>{meas_rows}</table></div>

<h2>Contrast by token (light · dark)</h2>
<div class="tw"><table class="t"><tr><th>Pair</th><th>Light</th><th>Dark</th><th>Target</th></tr>{contrast_rows}</table></div>

<h2>Original dev-trait icons</h2>
<p>Bytes from <code>AMattHT/Madden-Draft-Generator</code> <code>server/data/dev-icons/</code> at <code>9a4ac69</code>, hashes checked at build time; mapping from <code>ui.tsx</code>, not file names. EA artwork used at the user's request; never redrawn or recoloured. Every icon keeps its text label and <code>alt</code>. The share column is the fraction of each icon's opaque pixels that reach 3:1 against what it sits on: in dark, a neutral backing chip lifts it from 14–34% to 62–85%. Light needs no chip.</p>
<div class="tw"><table class="t"><tr><th></th><th>Tier</th><th>File</th><th>Light, bare</th><th>Dark, bare</th><th>Dark, on chip</th></tr>{''.join(legend)}</table></div>

<h2>Still open</h2>
<p>Default lens, Career cap, 389 vs 402, seed visibility, window arrangement and product name remain open; the frames use labelled example settings. Board states (empty filter, overflow, a 2026 class with no completed season, column chooser, identity review) are DG05-CONCEPT-B2. Full record: <code>DECISIONS.md</code>.</p>
</main>
<script>
(function(){{var c=document.getElementById('fit');if(!c)return;
function apply(){{document.getElementById('frames').classList.toggle('fit',c.checked)}}
try{{var s=localStorage.getItem('dg05b1-fit');if(s!==null)c.checked=s==='1'}}catch(e){{}}
apply();c.addEventListener('change',function(){{apply();try{{localStorage.setItem('dg05b1-fit',c.checked?'1':'0')}}catch(e){{}}}});}})();
</script>
</body></html>"""
out = os.path.join(ROOT, "comparison.html")
open(out, "w", encoding="utf-8", newline="\n").write(HTML)
print(out, os.path.getsize(out))

if len(sys.argv) > 3:
    inline_out = os.path.abspath(sys.argv[3])
    repo = os.path.abspath(os.path.join(ROOT, "..", "..", "..", ".."))
    assert not inline_out.startswith(repo + os.sep), "write the inline presentation outside the repository"
    P = lambda n: os.path.abspath(os.path.join(ROOT, "assembled", n + ".png"))
    body = "".join(
        f'<figure><img src="{P(n)}" alt="{E(t)}" style="{"max-width:640px" if n.endswith("1024x768") else ""}">'
        f'<figcaption><b>{E(t)}.</b> {E(c)}</figcaption></figure>' for n, t, c in [
            ("A-light-1440x900", "A · light · 1440×900", "Docked inspector kept; Bob Brown selected (tint, bar, bold) and focused (blue ring). " + rows("A-light-1440x900") + "."),
            ("A-light-1024x768", "A · light · 1024×768", "College and Portrait move to the inspector; filters fold into one button; Career record and Provenance collapse. " + rows("A-light-1024x768") + "."),
            ("A-dark-1024x768", "A · dark · 1024×768", "Same layout; dev icons on a neutral backing chip. " + rows("A-dark-1024x768") + "."),
            ("A-light-zoom200-960x540css", "A · 200% zoom", "Physical 1920×1080 at 200% = 960×540 CSS. Real reflow: '#' dropped, inspector docked at 268 CSS px with tabs. " + rows("A-light-zoom200-960x540css") + "."),
            ("A-light-focus-1440x900", "Focus vs selection (state concept)", "Focus ring on row 5; selection stays on Bob Brown."),
            ("A-focus-detail", "Focus detail", "Colour and luminance greyscale crops, enlarged 2×."),
        ])
    INLINE = f"""<!doctype html><html lang="en"><head><meta charset="utf-8"><title>Scouting Desk variants</title>
<style>
body{{margin:0}}
.w{{display:flex;flex-direction:column;gap:14px}}
.note{{border:1px solid var(--border);border-radius:var(--radius);padding:10px 14px;font-size:13px;line-height:1.5;color:var(--muted-foreground)}}
.note b{{color:var(--foreground)}}
figure{{margin:0}}
figure img{{display:block;width:100%;height:auto;border:1px solid var(--border);border-radius:6px}}
figcaption{{font-size:12.5px;line-height:1.45;margin-top:6px;color:var(--muted-foreground)}}
figcaption b{{color:var(--foreground)}}
</style></head><body><div class="w">
<div class="note"><b>DG05-CONCEPT-B1 · static design frames, not the app.</b> Direction A (selected) in light, at 1024×768 in light and dark, at 200% zoom, plus a keyboard-focus state concept. Stats are ◇ sample inputs and ratings are illustrative; identity facts are real. Contrast: all text pairs ≥ 4.5:1 in both themes; focus ring ≥ {min(ui('light','focus ring on the selected row'), ui('dark','focus ring on the selected row'))}:1 against the selected row.</div>
{body}
</div></body></html>"""
    open(inline_out, "w", encoding="utf-8", newline="\n").write(INLINE)
    print(inline_out, len(INLINE))
