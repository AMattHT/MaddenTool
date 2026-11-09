"""Fit a raw Higgsfield output into a 1440x900 frame with a caption strip.

The image itself is only resized (Lanczos, aspect kept, 2592x1536 -> 1440x853);
nothing inside it is edited, repainted or composited. A 47px strip below it
states that it is raw model output and not authoritative.

    python -I label_raw.py <in.png> <out.png> "<caption>"
"""
import sys
from PIL import Image, ImageDraw, ImageFont

src, dst, caption = sys.argv[1:4]
im = Image.open(src).convert("RGB")
w = 1440
h = round(im.height * w / im.width)
im = im.resize((w, h), Image.LANCZOS)
out = Image.new("RGB", (1440, 900), (32, 26, 10))
out.paste(im, (0, 0))
d = ImageDraw.Draw(out)
try:
    font = ImageFont.truetype("segoeuib.ttf", 15)
except OSError:
    font = ImageFont.load_default()
d.text((14, h + 14), caption, fill=(242, 196, 107), font=font)
out.save(dst, optimize=True)
print(dst, im.size, "strip", 900 - h)
