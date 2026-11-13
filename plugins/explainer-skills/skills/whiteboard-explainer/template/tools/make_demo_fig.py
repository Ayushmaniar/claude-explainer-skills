"""Draws the demo's "paper figure" (figs/guesses.png): a plain bar chart of the worst-case number of guesses
binary search needs, floor(log2 n) + 1, for lists of 10 .. 1,000,000 entries. In a real video, figs/ holds the
paper's own figures instead (cropped with PIL, see prep_figs.py).

    python -I tools/make_demo_fig.py      (needs Pillow; then python -I tools/prep_figs.py)

Prints each bar's rectangle in figure pixels, for Card::mark / ring / zoom in src/s_figure.rs.
"""
import math
import pathlib

from PIL import Image, ImageDraw, ImageFont

ROOT = pathlib.Path(__file__).resolve().parent.parent
W, H = 1600, 1000
X0, X1, BASE, TOP, YMAX = 200, 1540, 840, 200, 22  # plot area in figure px
SIZES = [10, 100, 1_000, 10_000, 100_000, 1_000_000]
NAMES = ["10", "100", "1k", "10k", "100k", "1M"]
BAR_W = 130


def guesses(n: int) -> int:
    return math.floor(math.log2(n)) + 1


def font(name: str, size: int):
    return ImageFont.truetype(str(ROOT / "media" / name), size)


def y_of(v: float) -> float:
    return BASE - (BASE - TOP) * v / YMAX


im = Image.new("RGB", (W, H), "white")
d = ImageDraw.Draw(im)
d.text((X0 - 120, 60), "Figure 1.  Worst-case guesses of binary search", font=font("Inter-SemiBold.ttf", 44), fill="#111111")
for v in range(0, YMAX + 1, 5):
    y = y_of(v)
    d.line([(X0, y), (X1, y)], fill="#e3e3e3" if v else "#222222", width=2 if v else 3)
    d.text((X0 - 24, y), str(v), font=font("Inter-Regular.ttf", 30), fill="#444444", anchor="rm")
d.line([(X0, TOP - 20), (X0, BASE)], fill="#222222", width=3)
slot = (X1 - X0) / len(SIZES)
bars = []
for i, (n, label) in enumerate(zip(SIZES, NAMES)):
    g = guesses(n)
    cx = X0 + slot * (i + 0.5)
    x0, x1, y0 = cx - BAR_W / 2, cx + BAR_W / 2, y_of(g)
    d.rectangle([x0, y0, x1, BASE], fill="#4c72b0", outline="#2b4a80", width=3)
    d.text((cx, y0 - 14), str(g), font=font("Inter-Bold.ttf", 36), fill="#111111", anchor="ms")
    d.text((cx, BASE + 46), label, font=font("Inter-Medium.ttf", 34), fill="#222222", anchor="ms")
    bars.append((label, n, g, round(x0), round(y0), round(x1), BASE))
d.text(((X0 + X1) / 2, BASE + 110), "list size (entries)", font=font("Inter-Regular.ttf", 32), fill="#444444", anchor="ms")
ylab = Image.new("RGBA", (420, 50), (255, 255, 255, 0))
ImageDraw.Draw(ylab).text((210, 25), "guesses (worst case)", font=font("Inter-Regular.ttf", 32), fill="#444444", anchor="mm")
ylab = ylab.rotate(90, expand=True)
im.paste(ylab, (40, int((TOP + BASE) / 2 - 210)), ylab)
d.text((X1, BASE + 110), "guesses = floor(log2 n) + 1", font=font("JetBrainsMono-Regular.ttf", 26), fill="#666666", anchor="rs")
out = ROOT / "figs" / "guesses.png"
out.parent.mkdir(exist_ok=True)
im.save(out, optimize=True)
print(out.name, im.size)
for b in bars:
    print("  bar %-4s n=%-8d guesses=%-2d rect=(%d, %d, %d, %d)" % b)
