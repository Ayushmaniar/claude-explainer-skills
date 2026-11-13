"""Downloads a Codex pet to be the video's host and slices it into sprites/pet_<row>_<col>.png (192x208 cells,
with a light outline so it reads on the black board).

    python -I tools/get_pet.py <pet-id>         e.g. rubber-duck-2-0, review-owl, terminal-ghost, bug-hunter
    python -I tools/get_pet.py --list           the pets in the repo right now
    python -I tools/get_pet.py --sheet my_pet.webp      slice a spritesheet you already have (8 x 9 cells of 192x208)

Source: https://github.com/gennadi-kuzmin/awesome-codex-pets (pets/<pet-id>/spritesheet.webp). The template does not
ship any pet artwork: that repo licenses its code and documentation under MIT, and asks that artwork is used only
with clear rights, so check the pet's licence before you publish a video with it. The download is kept in
assets/<pet-id>/ with a SOURCE.md note. Pick a bright pet (dark ones vanish on the black board) and a different
one per video. Without sprites the video renders fine, just without a host. Needs Pillow (with WebP support).
"""
import datetime
import json
import pathlib
import sys
import urllib.request

from PIL import Image, ImageFilter

ROOT = pathlib.Path(__file__).resolve().parent.parent
REPO = "gennadi-kuzmin/awesome-codex-pets"
CW, CH, COLS, ROWS, R = 192, 208, 8, 9, 4


def fetch(url: str) -> bytes:
    req = urllib.request.Request(url, headers={"User-Agent": "whiteboard-explainer-template"})
    with urllib.request.urlopen(req, timeout=60) as r:
        return r.read()


def slice_sheet(sheet: pathlib.Path) -> int:
    im = Image.open(sheet).convert("RGBA")
    if im.size != (COLS * CW, ROWS * CH):
        raise SystemExit(f"{sheet}: expected {COLS * CW}x{ROWS * CH} (8 x 9 cells of 192x208), got {im.size}")
    out = ROOT / "sprites"
    out.mkdir(exist_ok=True)
    for old in out.glob("pet_*.png"):
        old.unlink()
    n = 0
    for r in range(ROWS):
        for c in range(COLS):
            cell = im.crop((c * CW, r * CH, (c + 1) * CW, (r + 1) * CH))
            a = cell.getchannel("A")
            if a.getbbox() is None:
                continue
            # Outline: the alpha grown by R px, filled with a warm off-white, under the sprite.
            grown = a.point(lambda v: 255 if v > 40 else 0).filter(ImageFilter.MaxFilter(2 * R + 1))
            grown = grown.filter(ImageFilter.GaussianBlur(0.8))
            halo = Image.new("RGBA", cell.size, (236, 228, 210, 0))
            halo.putalpha(grown.point(lambda v: int(v * 0.92)))
            halo.alpha_composite(cell)
            halo.save(out / f"pet_{r}_{c}.png")
            n += 1
    return n


def main() -> None:
    if len(sys.argv) < 2 or sys.argv[1] in ("-h", "--help"):
        raise SystemExit(__doc__)
    if sys.argv[1] == "--list":
        items = json.loads(fetch(f"https://api.github.com/repos/{REPO}/contents/pets"))
        print("\n".join(i["name"] for i in items if i["type"] == "dir"))
        return
    if sys.argv[1] == "--sheet":
        n = slice_sheet(pathlib.Path(sys.argv[2]))
        print(f"{n} frames -> sprites/")
        return
    pet = sys.argv[1]
    dest = ROOT / "assets" / pet
    dest.mkdir(parents=True, exist_ok=True)
    base = f"https://raw.githubusercontent.com/{REPO}/main/pets/{pet}"
    try:
        (dest / "spritesheet.webp").write_bytes(fetch(f"{base}/spritesheet.webp"))
    except Exception as e:
        raise SystemExit(f"could not download pet {pet!r} ({e}); see --list")
    (dest / "SOURCE.md").write_text(
        f"# {pet}\n\n- From https://github.com/{REPO} (pets/{pet}), downloaded {datetime.date.today()}.\n"
        "- The repo's licence: MIT for code and documentation; check the pet artwork's rights before publishing.\n"
        "- Sliced by tools/get_pet.py into sprites/pet_<row>_<col>.png.\n", encoding="utf-8")
    n = slice_sheet(dest / "spritesheet.webp")
    print(f"{pet}: {n} frames -> sprites/ (source kept in assets/{pet}/)")


if __name__ == "__main__":
    main()
