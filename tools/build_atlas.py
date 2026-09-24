#!/usr/bin/env python3
"""Build Emberdeep's sprite atlas from the CC0 Dungeon Crawl Stone Soup tiles.

Usage:
    python3 tools/build_atlas.py /path/to/crawl-tiles/releases/Nov-2015 [--preview sheet.png]

Writes (relative to the repository root):
    assets/atlas.png      every sprite as a 32x32 cell, 16 columns, row-major in enum order
    src/gfx/sprites.rs    the generated `Sprite` enum that indexes the atlas

With --preview it also writes a labelled contact sheet (each sprite at 2x on mid-grey).

Source art is the CC0 1.0 tile export of Dungeon Crawl Stone Soup
(https://github.com/crawl/tiles). Only files from the given release directory are
read, and any file named in TILES_UNDER_UNKNOWN_LICENSE.md (found next to the
`releases` folder) is refused. Some sprites are composited, recoloured or drawn
from primitives here; the recipe for every sprite is in SPRITES below, in enum order.

Needs Python 3 with Pillow and numpy. The output is deterministic.
"""

import argparse
import re
import sys
from pathlib import Path

import numpy as np
from PIL import Image, ImageDraw, ImageFilter, ImageFont

CELL = 32
COLUMNS = 16
REPO = Path(__file__).resolve().parent.parent

# --------------------------------------------------------------------------
# Tile loading (with the licence guard)
# --------------------------------------------------------------------------

RELEASE: Path = Path()
BLACKLIST: set[str] = set()
USED: set[str] = set()


def load_blacklist(release: Path) -> set[str]:
    """Every *.png file name mentioned in TILES_UNDER_UNKNOWN_LICENSE.md."""
    for cand in (release.parent.parent / "TILES_UNDER_UNKNOWN_LICENSE.md",
                 release.parent / "TILES_UNDER_UNKNOWN_LICENSE.md",
                 release / "TILES_UNDER_UNKNOWN_LICENSE.md"):
        if cand.is_file():
            text = cand.read_text(encoding="utf-8", errors="replace")
            return {Path(m).name for m in re.findall(r"[\w./+-]+\.png", text)}
    print("warning: TILES_UNDER_UNKNOWN_LICENSE.md not found; cannot double-check sources",
          file=sys.stderr)
    return set()


def T(rel: str) -> Image.Image:
    """Load one source tile (path relative to the release dir) as RGBA."""
    path = (RELEASE / rel).resolve()
    if RELEASE.resolve() not in path.parents:
        raise SystemExit(f"refusing to read outside the release dir: {rel}")
    if path.name in BLACKLIST:
        raise SystemExit(f"refusing to use {rel}: listed in TILES_UNDER_UNKNOWN_LICENSE.md")
    if not path.is_file():
        raise SystemExit(f"missing source tile: {rel}")
    USED.add(rel)
    return Image.open(path).convert("RGBA")


def blank() -> Image.Image:
    return Image.new("RGBA", (CELL, CELL), (0, 0, 0, 0))


# --------------------------------------------------------------------------
# Image helpers
# --------------------------------------------------------------------------

def arr(im):
    return np.asarray(im, dtype=np.float64).copy()


def img(a):
    return Image.fromarray(np.clip(np.rint(a), 0, 255).astype(np.uint8), "RGBA")


def L(*layers) -> Image.Image:
    """Alpha-composite layers bottom-to-top onto a transparent 32x32 cell.
    A layer is an image or a (image, dx, dy) tuple."""
    out = blank()
    for layer in layers:
        if layer is None:
            continue
        dx = dy = 0
        if isinstance(layer, tuple):
            layer, dx, dy = layer
        tmp = blank()
        tmp.paste(layer, (dx, dy))
        out = Image.alpha_composite(out, tmp)
    return out


DOLL_ROOTS = ("mon/", "item/", "dngn/", "effect/", "gui/", "misc/", "UNUSED/", "player/")


def D(*parts) -> Image.Image:
    """Paper-doll composite; bare parts are relative to player/ (DCSS doll layers)."""
    ims = []
    for p in parts:
        if isinstance(p, Image.Image):
            ims.append(p)
        else:
            ims.append(T(p if p.startswith(DOLL_ROOTS) else "player/" + p + ".png"))
    return L(*ims)


def rgb_to_hsv(rgb):
    r, g, b = rgb[..., 0] / 255.0, rgb[..., 1] / 255.0, rgb[..., 2] / 255.0
    mx = np.maximum(np.maximum(r, g), b)
    mn = np.minimum(np.minimum(r, g), b)
    d = mx - mn
    h = np.zeros_like(mx)
    nz = d > 1e-9
    rm = nz & (mx == r)
    gm = nz & (mx == g) & ~rm
    bm = nz & ~rm & ~gm
    h[rm] = ((g[rm] - b[rm]) / d[rm]) % 6
    h[gm] = (b[gm] - r[gm]) / d[gm] + 2
    h[bm] = (r[bm] - g[bm]) / d[bm] + 4
    h = h * 60.0
    s = np.where(mx > 1e-9, d / np.maximum(mx, 1e-9), 0)
    return h, s, mx


def hsv_to_rgb(h, s, v):
    h = (h % 360.0) / 60.0
    i = np.floor(h).astype(int) % 6
    f = h - np.floor(h)
    p = v * (1 - s)
    q = v * (1 - s * f)
    t = v * (1 - s * (1 - f))
    r = np.choose(i, [v, q, p, p, t, v])
    g = np.choose(i, [t, v, v, q, p, p])
    b = np.choose(i, [p, p, t, v, v, q])
    return np.stack([r, g, b], axis=-1) * 255.0


def region_mask(shape, box):
    m = np.zeros(shape[:2], dtype=bool)
    if box is None:
        m[:] = True
    else:
        x0, y0, x1, y1 = box
        m[y0:y1, x0:x1] = True
    return m


def hsv(im, dh=0.0, s=1.0, v=1.0, hue=None, where=None, box=None, sat_add=0.0):
    """Adjust hue/saturation/value. `hue` sets an absolute hue; `where(h, s, v)`
    and `box` restrict which pixels change."""
    a = arr(im)
    h, sa, va = rgb_to_hsv(a[..., :3])
    m = region_mask(a.shape, box) & (a[..., 3] > 0)
    if where is not None:
        m &= where(h, sa, va)
    nh = (h + dh) if hue is None else np.full_like(h, hue)
    ns = np.clip(sa * s + sat_add, 0, 1)
    nv = np.clip(va * v, 0, 1)
    rgb = hsv_to_rgb(nh, ns, nv)
    a[..., :3] = np.where(m[..., None], rgb, a[..., :3])
    return img(a)


def hue_in(lo, hi, min_s=0.0, min_v=0.0):
    """Mask factory: hue in [lo, hi] (degrees, wraps) with saturation/value floors."""
    def f(h, s, v):
        hm = (h >= lo) & (h <= hi) if lo <= hi else (h >= lo) | (h <= hi)
        return hm & (s >= min_s) & (v >= min_v)
    return f


def lum(a):
    return (0.299 * a[..., 0] + 0.587 * a[..., 1] + 0.114 * a[..., 2]) / 255.0


def gmap(im, stops, amount=1.0, where=None, box=None, norm=False):
    """Gradient-map luminance onto a colour ramp [(pos, (r,g,b)), ...]."""
    a = arr(im)
    y = lum(a)
    m = region_mask(a.shape, box) & (a[..., 3] > 0)
    if where is not None:
        h, s, v = rgb_to_hsv(a[..., :3])
        m &= where(h, s, v)
    if norm and m.any():
        lo, hi = np.percentile(y[m], 2), np.percentile(y[m], 98)
        y = np.clip((y - lo) / max(hi - lo, 1e-6), 0, 1)
    xs = [p for p, _ in stops]
    out = np.stack([np.interp(y, xs, [c[i] for _, c in stops]) for i in range(3)], axis=-1)
    mix = a[..., :3] * (1 - amount) + out * amount
    a[..., :3] = np.where(m[..., None], mix, a[..., :3])
    return img(a)


def tint(im, rgb, amount=0.5):
    """Multiply-style colourise that keeps shading: blend toward lum*colour."""
    a = arr(im)
    y = lum(a)[..., None]
    col = np.array(rgb, dtype=np.float64)[None, None, :]
    target = np.clip(y * col * 1.6, 0, 255)
    a[..., :3] = a[..., :3] * (1 - amount) + target * amount
    return img(a)


def bright(im, f=1.0, add=0.0):
    a = arr(im)
    a[..., :3] = a[..., :3] * f + add
    return img(a)


def alpha_mul(im, f):
    a = arr(im)
    a[..., 3] *= f
    return img(a)


def opaque(im, bg=(0, 0, 0)):
    """Flatten onto a solid colour (floors and walls must be fully opaque)."""
    base = Image.new("RGBA", im.size, bg + (255,))
    return Image.alpha_composite(base, im)


def clear_below(im, alpha_min):
    a = arr(im)
    a[..., 3] = np.where(a[..., 3] < alpha_min, 0, a[..., 3])
    return img(a)


def keep(im, where=None, box=None):
    """Make every pixel NOT selected transparent."""
    a = arr(im)
    m = region_mask(a.shape, box)
    if where is not None:
        h, s, v = rgb_to_hsv(a[..., :3])
        m &= where(h, s, v)
    a[..., 3] = np.where(m, a[..., 3], 0)
    return img(a)


def dilate(mask, r=1):
    out = mask.copy()
    for dy in range(-r, r + 1):
        for dx in range(-r, r + 1):
            sh = np.zeros_like(mask)
            ys = slice(max(dy, 0), mask.shape[0] + min(dy, 0))
            yd = slice(max(-dy, 0), mask.shape[0] + min(-dy, 0))
            xs = slice(max(dx, 0), mask.shape[1] + min(dx, 0))
            xd = slice(max(-dx, 0), mask.shape[1] + min(-dx, 0))
            sh[ys, xs] = mask[yd, xd]
            out |= sh
    return out


def drop_shadow_pixels(im):
    """Remove the flat black drop shadow DCSS paints to the lower right of items:
    black pixels that are not part of the 1px outline around coloured pixels."""
    a = arr(im)
    black = (a[..., :3].max(axis=-1) < 8) & (a[..., 3] > 0)
    colour = (a[..., 3] > 0) & ~black
    shadow = black & (~dilate(colour, 1) | (a[..., 3] < 255))
    a[..., 3] = np.where(shadow, 0, a[..., 3])
    return img(a)


def bbox_crop(im):
    bb = im.getchannel("A").getbbox()
    return im.crop(bb) if bb else im


def resize(im, w, h, smooth=True, alpha_cut=None):
    """Resize with premultiplied alpha; optionally threshold alpha for crisp pixels."""
    if not smooth:
        return im.resize((w, h), Image.NEAREST)
    out = im.convert("RGBa").resize((w, h), Image.LANCZOS).convert("RGBA")
    if alpha_cut is not None:
        a = arr(out)
        a[..., 3] = np.where(a[..., 3] >= alpha_cut, 255, 0)
        out = img(a)
    return out


def scale(im, f, smooth=True, alpha_cut=96):
    w, h = max(1, round(im.width * f)), max(1, round(im.height * f))
    if not smooth and f == int(f):
        return im.resize((w, h), Image.NEAREST)
    return resize(im, w, h, smooth=smooth, alpha_cut=alpha_cut)


def place(im, cx=16, cy=16, anchor="center"):
    """Place an image on a blank cell centred at (cx, cy) or bottom-centred at cy."""
    x = cx - im.width // 2
    y = cy - im.height // 2 if anchor == "center" else cy - im.height
    return L((im, x, y))


def icon(rel, factor=2, cx=16, cy=16):
    """Crop a small glyph (brand/status icons live in a tile corner) and blow it
    up with nearest-neighbour so it stays crisp, centred in the cell."""
    g = bbox_crop(T(rel))
    g = g.resize((g.width * factor, g.height * factor), Image.NEAREST)
    return place(g, cx, cy)


def fit(im, anchor="bottom"):
    """Force any source to a 32x32 cell (crop to content, shrink if needed)."""
    if im.size == (CELL, CELL):
        return im
    g = bbox_crop(im)
    if g.width > CELL or g.height > CELL:
        f = min(CELL / g.width, CELL / g.height)
        g = resize(g, max(1, int(g.width * f)), max(1, int(g.height * f)), alpha_cut=96)
    return place(g, 16, 32 if anchor == "bottom" else 16, anchor)


def blur_alpha(im, radius):
    return im.getchannel("A").filter(ImageFilter.GaussianBlur(radius))


def glow(im, rgb, radius=2.0, strength=1.0, inner=False):
    """Soft coloured halo behind (or over, if inner) the sprite's silhouette."""
    a = np.asarray(blur_alpha(im, radius), dtype=np.float64) * strength
    g = np.zeros((im.height, im.width, 4))
    g[..., :3] = rgb
    g[..., 3] = np.clip(a, 0, 255)
    halo = img(g)
    return Image.alpha_composite(im, halo) if inner else Image.alpha_composite(halo, im)


def radial(rgb, cx, cy, r, peak=255, power=1.5):
    """A radial light blob as its own layer."""
    yy, xx = np.mgrid[0:CELL, 0:CELL]
    d = np.sqrt((xx + 0.5 - cx) ** 2 + (yy + 0.5 - cy) ** 2) / r
    al = np.clip(1 - d, 0, 1) ** power * peak
    g = np.zeros((CELL, CELL, 4))
    g[..., :3] = rgb
    g[..., 3] = al
    return img(g)


def outline(im, rgb=(0, 0, 0), alpha=255):
    """1px outline around the opaque silhouette (drawn beneath)."""
    a = np.asarray(im.getchannel("A")) > 0
    grown = a.copy()
    grown[1:, :] |= a[:-1, :]
    grown[:-1, :] |= a[1:, :]
    grown[:, 1:] |= a[:, :-1]
    grown[:, :-1] |= a[:, 1:]
    ring = grown & ~a
    g = np.zeros((im.height, im.width, 4))
    g[..., :3] = rgb
    g[..., 3] = np.where(ring, alpha, 0)
    return Image.alpha_composite(img(g), im)


def flip(im):
    return im.transpose(Image.FLIP_LEFT_RIGHT)


def rng(seed):
    return np.random.default_rng(seed)


# --------------------------------------------------------------------------
# Sprites that need composition or drawing
# --------------------------------------------------------------------------

def warm(im, amount=0.25):
    return tint(im, (150, 118, 80), amount)


def wood_floor():
    # Planks from the door, turned sideways and tiled into 4 staggered rows.
    door = T("dngn/doors/closed_door.png")
    plank = door.crop((5, 6, 27, 30)).rotate(90, expand=True)   # 24 x 22
    out = blank()
    rows = [0, 8, 16, 24]
    offs = [0, 11, 5, 16]
    for y, o in zip(rows, offs):
        band = plank.crop((0, 2, plank.width, 10))
        for x in range(-24, 40, plank.width):
            out.paste(band, (x + o, y))
    out = hsv(out, s=0.85, v=1.05)
    d = ImageDraw.Draw(out)
    for y in rows:
        d.line([(0, y), (31, y)], fill=(38, 22, 14, 255))
    return opaque(out)


def cobble(rel):
    """Warm, light cobbles for the village paths (DCSS grey cobbles re-graded)."""
    return opaque(gmap(T(rel), [(0.0, (44, 36, 28)), (0.45, (112, 95, 76)),
                                (1.0, (184, 166, 138))], norm=True))


def timber_wall():
    """Half-timbered village wall: DCSS stone textures re-graded to plaster and
    footing, with oak beams in the palette of the DCSS door wood."""
    out = gmap(T("dngn/wall/stone_gray0.png"),
               [(0.0, (120, 100, 76)), (0.5, (186, 168, 134)), (1.0, (226, 212, 180))], norm=True)
    d = ImageDraw.Draw(out)
    dark, mid, hi = (46, 28, 16, 255), (84, 54, 30, 255), (122, 84, 48, 255)
    d.rectangle([0, 0, 31, 2], fill=mid)
    d.line([(0, 0), (31, 0)], fill=hi)
    d.line([(0, 2), (31, 2)], fill=dark)
    d.line([(3, 16), (28, 3)], fill=mid, width=3)
    d.line([(3, 15), (28, 2)], fill=hi)
    for x0, x1 in ((0, 2), (29, 31)):
        d.rectangle([x0, 0, x1, 31], fill=mid)
        d.line([(x0, 0), (x0, 31)], fill=hi)
        d.line([(x1, 0), (x1, 31)], fill=dark)
    footing = tint(T("dngn/wall/brick_gray0.png"), (150, 130, 110), 0.4).crop((0, 0, 32, 13))
    out.paste(footing, (0, 19))
    d.rectangle([0, 17, 31, 18], fill=mid)
    d.line([(0, 17), (31, 17)], fill=hi)
    d.line([(0, 19), (31, 19)], fill=dark)
    return opaque(out)


def tree(rel, canopy_hue=None, canopy_s=1.0, canopy_v=1.0):
    t = T(rel)
    if canopy_hue is not None:
        t = hsv(t, hue=canopy_hue, s=canopy_s, v=canopy_v,
                where=hue_in(35, 75, min_s=0.25), box=(0, 0, 32, 20))
        # trunk: olive -> bark brown
        t = hsv(t, hue=28, s=0.9, v=0.85, where=hue_in(35, 75), box=(0, 17, 32, 32))
    return t


def lamppost(lit):
    """Street lantern: the DCSS lamp (lit / inert) on an iron post."""
    lamp = T("item/misc/misc_lamp.png" if lit else "item/misc/misc_lamp_inert.png")
    lamp = scale(bbox_crop(drop_shadow_pixels(lamp)), 0.68)
    post = blank()
    d = ImageDraw.Draw(post)
    ink, iron, hi = (10, 10, 12, 255), (34, 32, 38, 255), (96, 92, 104, 255)
    top = lamp.height - 3
    d.rectangle([14, top, 17, 29], fill=ink)          # pole
    d.rectangle([15, top, 16, 29], fill=iron)
    d.line([(16, top), (16, 29)], fill=hi)
    d.rectangle([11, 28, 20, 31], fill=ink)           # foot
    d.rectangle([12, 29, 19, 30], fill=iron)
    d.line([(12, 29), (19, 29)], fill=hi)
    out = L(post, (lamp, 16 - lamp.width // 2, 0))
    if lit:
        cy = lamp.height * 0.55
        out = L(radial((255, 180, 80), 16, cy, 15, peak=140), out,
                radial((255, 235, 160), 16, cy, 5, peak=120))
    else:
        out = tint(hsv(out, s=0.2, v=0.65), (110, 130, 165), 0.3)
    return out


SIGN_BRACKET_H = 10


def shop_sign(item_rel, factor=0.72, dy=0):
    """Hanging shop sign: the iron bracket and chains of the DCSS tavern sign
    with the shop's ware hanging from it."""
    tav = T("UNUSED/gui/tavern.png")
    bracket = tav.crop((0, 0, 32, SIGN_BRACKET_H))
    ware = drop_shadow_pixels(bbox_crop(T(item_rel)))
    ware = outline(scale(ware, factor), (20, 14, 10))
    return L(bracket, (ware, 16 - ware.width // 2, SIGN_BRACKET_H - 1 + dy))


def remove_stone_frame(im):
    """Keep the wooden leaves of a DCSS door, drop the grey stone jamb."""
    a = arr(im)
    h, s, v = rgb_to_hsv(a[..., :3])
    core = s > 0.22                     # wood / runes / iron bands are coloured
    keepm = core | ((v < 0.2) & dilate(core, 1))   # plus their dark outline
    a[..., 3] = np.where(keepm, a[..., 3], 0)
    return img(a)


def sealed_gate():
    """DCSS runed door without its stone jamb; the runes glow."""
    door = remove_stone_frame(T("dngn/doors/runed_door.png"))
    shape = np.asarray(remove_stone_frame(T("dngn/doors/closed_door.png")).getchannel("A")) > 0
    a = arr(door)
    a[..., 3] = np.where(shape, a[..., 3], 0)          # same silhouette as the plain door
    door = img(a)
    runes = keep(door, where=hue_in(180, 250, min_s=0.35))
    return L(door, glow(runes, (90, 190, 255), radius=1.6, strength=1.2))


def chest(open_):
    """Treasure chest drawn from primitives and textured with DCSS door wood;
    the open one shows a DCSS gold pile inside."""
    wood = T("dngn/doors/closed_door.png").crop((6, 6, 26, 30)).resize((24, 24), Image.NEAREST)
    ink = (26, 16, 10, 255)
    gold, gold_hi, gold_lo = (230, 180, 60, 255), (255, 232, 130, 255), (150, 100, 30, 255)
    iron, iron_hi = (70, 66, 72, 255), (140, 136, 146, 255)
    out = blank()
    out.paste(wood.crop((0, 4, 22, 16)), (5, 17))            # front panel
    if open_:
        lid = hsv(wood.crop((0, 12, 22, 20)), v=0.7)          # lid swung back
        out.paste(lid, (5, 4))
    else:
        out.paste(wood.crop((0, 12, 22, 20)), (5, 9))
    d = ImageDraw.Draw(out)
    if open_:
        d.rectangle([4, 3, 27, 12], outline=ink)
        d.line([(5, 11), (26, 11)], fill=iron_hi)
        d.rectangle([5, 12, 26, 16], fill=(30, 16, 8, 255))   # dark inside
        coins = scale(bbox_crop(T("item/gold/16.png")), 0.55)
        coins = coins.crop((0, 0, coins.width, min(coins.height, 9)))
        out = L(out, (coins, 16 - coins.width // 2, 17 - coins.height))
        d = ImageDraw.Draw(out)
        d.line([(4, 12), (4, 16)], fill=ink)
        d.line([(27, 12), (27, 16)], fill=ink)
        d.line([(5, 17), (26, 17)], fill=iron_hi)
        d.rectangle([4, 17, 27, 28], outline=ink)
        top = 17
    else:
        d.rounded_rectangle([4, 8, 27, 17], radius=3, outline=ink)
        d.line([(5, 16), (26, 16)], fill=iron)
        d.line([(5, 17), (26, 17)], fill=iron_hi)
        d.rectangle([4, 17, 27, 28], outline=ink)
        top = 9
    for x in (8, 22):                                          # iron bands
        d.rectangle([x, top, x + 1, 27], fill=iron)
        d.line([(x, top), (x, 27)], fill=iron_hi)
    ly = 16 if not open_ else 18                               # lock plate
    d.rectangle([14, ly, 17, ly + 5], fill=gold, outline=ink)
    d.point([(15, ly + 1), (16, ly + 1)], fill=gold_hi)
    d.point([(15, ly + 3)], fill=ink)
    d.line([(15, ly + 4), (16, ly + 4)], fill=gold_lo)
    sh = blank()
    ImageDraw.Draw(sh).ellipse([3, 26, 29, 31], fill=(0, 0, 0, 90))
    return L(sh, out)


def bookshelf():
    """Opaque wall-height bookshelf: DCSS carved-wood frame + book spines coloured
    from the DCSS book tiles."""
    frame = T("dngn/wall/relief_brown0.png")
    frame = hsv(frame, v=0.75)
    out = frame.copy()
    d = ImageDraw.Draw(out)
    names = ["red", "dark_blue", "dark_green", "tan", "purple", "light_brown",
             "dark_brown", "turquoise", "magenta", "cloth", "light_gray", "yellow"]
    cols = []
    for n in names:
        b = arr(T(f"item/book/{n}.png"))
        m = b[..., 3] > 200
        cols.append(tuple(int(c) for c in np.median(b[m][:, :3], axis=0)))
    r = rng(7)
    shelves = [(3, 11), (13, 21), (23, 30)]
    for (y0, y1) in shelves:
        d.rectangle([2, y0, 29, y1], fill=(20, 12, 8, 255))
        x = 3
        while x < 29:
            w = int(r.integers(2, 4))
            top = y0 + int(r.integers(0, 3))
            c = cols[int(r.integers(0, len(cols)))]
            dark = tuple(max(0, int(v * 0.6)) for v in c)
            x1 = min(x + w - 1, 28)
            d.rectangle([x, top, x1, y1], fill=c + (255,))
            d.line([(x1, top), (x1, y1)], fill=dark + (255,))
            d.point([(x, top + 2)], fill=tuple(min(255, int(v * 1.4) + 30) for v in c) + (255,))
            x = x1 + 1 + (1 if r.random() < 0.15 else 0)
        d.line([(2, y1 + 1), (29, y1 + 1)], fill=(120, 80, 44, 255))
        d.line([(2, y1 + 2), (29, y1 + 2)], fill=(52, 30, 16, 255))
    return opaque(out)


def wall_torch():
    """DCSS hand-held torch mounted in a drawn iron sconce, with warm light."""
    torch = bbox_crop(T("player/hand2/misc/torch.png"))
    s = blank()
    d = ImageDraw.Draw(s)
    ink, iron, hi = (10, 10, 12, 255), (44, 40, 46, 255), (116, 110, 120, 255)
    d.rectangle([12, 21, 19, 25], fill=ink)           # wall plate
    d.rectangle([13, 22, 18, 24], fill=iron)
    d.line([(13, 22), (18, 22)], fill=hi)
    d.rectangle([13, 15, 18, 18], fill=ink)           # cup
    d.rectangle([14, 16, 17, 17], fill=iron)
    d.line([(15, 18), (15, 21)], fill=ink, width=2)
    out = L((torch, 16 - torch.width // 2, 2), s)
    return L(radial((255, 170, 60), 16, 7, 13, peak=150), out,
             radial((255, 230, 150), 16, 6, 4, peak=110))


def bones():
    """Scattered bones: DCSS skull (enlarged) + a few long bones drawn to match."""
    out = blank()
    d = ImageDraw.Draw(out)
    bone, shade, ink = (226, 218, 196, 255), (160, 150, 128, 255), (30, 26, 20, 255)

    def long_bone(x0, y0, x1, y1):
        d.line([(x0, y0), (x1, y1)], fill=ink, width=4)
        for (x, y) in ((x0, y0), (x1, y1)):
            d.ellipse([x - 2, y - 2, x + 2, y + 2], fill=ink)
        d.line([(x0, y0), (x1, y1)], fill=bone, width=2)
        for (x, y) in ((x0, y0), (x1, y1)):
            d.ellipse([x - 1, y - 1, x + 1, y + 1], fill=bone)
        d.point([(x1, y1 + 1)], fill=shade)

    long_bone(4, 26, 15, 21)
    long_bone(18, 27, 28, 20)
    long_bone(7, 17, 12, 24)
    skull = bbox_crop(T("player/hand1/misc/skull.png"))
    skull = skull.resize((skull.width * 2, skull.height * 2), Image.NEAREST)
    out = L(out, (outline(skull, (30, 26, 20)), 15, 9))
    return out


def rubble():
    out = blank()
    rocks = [("item/weapon/ranged/rock.png", 0.75, 4, 12),
             ("effect/rock0.png", 1.0, 17, 14),
             ("item/weapon/ranged/stone.png", 1.0, 11, 20),
             ("effect/stone0.png", 1.0, 22, 22),
             ("effect/stone0.png", 0.8, 3, 23)]
    for rel, f, x, y in rocks:
        r = bbox_crop(drop_shadow_pixels(T(rel)))
        if f != 1.0:
            r = scale(r, f)
        r = hsv(r, s=0.35, v=0.95)
        out = L(out, (r, x, y))
    return out


def memory_shard():
    shard = bbox_crop(drop_shadow_pixels(T("item/misc/runes/rune_cocytus.png")))
    shard = gmap(shard, [(0.0, (30, 10, 60)), (0.35, (110, 50, 200)),
                         (0.7, (200, 140, 255)), (1.0, (255, 245, 255))], norm=True)
    shard = scale(shard, 0.8)
    out = place(shard, 16, 17)
    out = glow(out, (190, 120, 255), radius=2.2, strength=1.6)
    sp = blank()
    d = ImageDraw.Draw(sp)
    for (x, y) in ((6, 7), (26, 9), (25, 25)):
        d.point([(x, y)], fill=(255, 255, 255, 255))
        d.point([(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)], fill=(220, 180, 255, 200))
    return L(radial((150, 80, 255), 16, 17, 15, peak=90), out, sp)


def waystone():
    ob = T("mon/statues/obelisk.png")
    ob = drop_shadow_pixels(ob)
    ob = gmap(ob, [(0.0, (10, 30, 50)), (0.4, (40, 120, 150)),
                   (0.75, (120, 220, 230)), (1.0, (230, 255, 255))])
    d = ImageDraw.Draw(ob)
    # rune glyphs down the face
    for y in (10, 15, 20):
        d.line([(15, y), (17, y)], fill=(255, 255, 255, 255))
        d.point([(16, y + 1)], fill=(255, 255, 255, 255))
    ob = glow(ob, (80, 230, 255), radius=2.5, strength=1.5)
    return L(radial((60, 200, 255), 16, 20, 15, peak=80), ob)


def ilsa():
    return D("cloak/white", "base/human_f", "boots/mesh_white", "legs/leg_armour01",
             "body/half_plate2", "gloves/gauntlet_blue", "hand1/long_sword_slant",
             "hand2/shield_knight_gray", "hair/fem_white")


def ilsa_pale():
    """Ilsa taken by the Pale: the same doll re-graded to ice, with a frost crown."""
    fig = gmap(ilsa(), [(0.0, (16, 24, 48)), (0.3, (70, 110, 170)),
                        (0.65, (170, 215, 245)), (1.0, (250, 255, 255))])
    crown = gmap(T("player/head/crown_gold2.png"),
                 [(0.0, (10, 20, 60)), (0.5, (60, 170, 230)), (1.0, (220, 255, 255))], norm=True)
    fig = L(fig, (crown, 0, -1))
    return glow(fig, (150, 220, 255), radius=1.6, strength=1.1)


def aurelian():
    f = T("mon/demons/shadow_fiend.png")
    # red eyes -> icy white-blue; skull and claws -> frost; body -> deep indigo
    f = hsv(f, hue=195, s=0.9, v=1.3, where=hue_in(340, 20, min_s=0.4))
    f = gmap(f, [(0.0, (6, 4, 18)), (0.25, (28, 20, 60)), (0.55, (80, 110, 170)),
                 (0.8, (170, 220, 250)), (1.0, (240, 255, 255))],
             where=lambda h, s, v: s < 0.5)
    return glow(f, (120, 200, 255), radius=1.8, strength=0.9)


def aurelian_true():
    """The healer's spirit: the Elder-style doll made translucent and pale."""
    fig = D("base/human_m", "body/robe_white2", "hand1/staff_plain", white_beard(), "hair/fem_white")
    fig = gmap(fig, [(0.0, (40, 70, 110)), (0.4, (130, 180, 220)),
                     (0.75, (215, 240, 255)), (1.0, (255, 255, 255))])
    fig = alpha_mul(fig, 0.88)
    fig = glow(fig, (190, 230, 255), radius=2.2, strength=1.5)
    return L(radial((200, 235, 255), 16, 16, 16, peak=70), fig)


RAT_EYE = (5, 19)


def gristlemaw():
    """The Rat King: the DCSS rat re-furred, with a bristling mane, fangs, burning
    eyes and a stolen crown."""
    r = T("mon/undead/zombies/zombie_rat.png")
    a = arr(r)
    a[RAT_EYE[1], RAT_EYE[0], :3] = (255, 60, 40)
    r = img(a)
    r = hsv(r, hue=0, s=0.6, v=0.45, where=hue_in(290, 345, min_s=0.3))       # rot -> scars
    r = gmap(r, [(0.0, (16, 12, 10)), (0.35, (84, 70, 62)), (0.7, (150, 134, 120)),
                 (1.0, (220, 205, 190))], where=lambda h, s_, v: s_ < 0.45, norm=True)
    d = ImageDraw.Draw(r)
    for x in (15, 18, 21, 24):                                              # mane
        top = 7 if x < 20 else 6
        d.line([(x, top - 3), (x, top)], fill=(20, 14, 12, 255))
        d.point([(x, top - 3)], fill=(150, 134, 120, 255))
        d.point([(x + 1, top - 1)], fill=(20, 14, 12, 255))
    d.point([(5, 23), (7, 23)], fill=(250, 240, 200, 255))                   # fangs
    eye = blank()
    eye.putpixel(RAT_EYE, (255, 40, 30, 255))
    r = L(glow(eye, (255, 30, 20), radius=1.2, strength=2.0), r, eye)
    crown = bbox_crop(T("player/head/crown_gold1.png"))
    return L(r, (outline(crown, (20, 12, 0)), 2, 9))


def mother_of_spores():
    m = T("mon/fungi_plants/deathcap.png")
    m = hsv(m, hue=285, s=1.0, v=0.95, where=hue_in(50, 110, min_s=0.2), box=(0, 0, 32, 20))
    m = hsv(m, hue=170, s=0.9, v=1.25, where=hue_in(60, 110, min_s=0.2), box=(0, 0, 32, 20))
    m = hsv(m, hue=285, s=0.5, v=0.85, where=hue_in(40, 110), box=(0, 18, 32, 32))
    m = glow(m, (120, 255, 220), radius=1.5, strength=0.7)
    sp = blank()
    d = ImageDraw.Draw(sp)
    for (x, y) in ((3, 6), (28, 4), (29, 15), (2, 17), (25, 27)):
        d.point([(x, y)], fill=(170, 255, 230, 255))
        d.point([(x + 1, y + 1)], fill=(120, 200, 190, 170))
    return L(m, sp)


def mushroom(cap_ramp=None, stalk=(232, 222, 180)):
    """A plain mushroom: the cap of the DCSS wandering mushroom on a drawn stalk
    (the original walks on legs)."""
    cap = bbox_crop(T("mon/fungi_plants/wandering_mushroom.png").crop((0, 0, 32, 15)))
    if cap_ramp is not None:
        cap = gmap(cap, cap_ramp, where=hue_in(0, 60, min_s=0.3), norm=True)
    w, h = cap.size
    sw, sh = max(4, w // 4), max(4, int(h * 0.75))
    out = Image.new("RGBA", (w, h + sh - 2), (0, 0, 0, 0))
    d = ImageDraw.Draw(out)
    x0 = w // 2 - sw // 2
    d.rectangle([x0 - 1, h - 3, x0 + sw, h + sh - 3], fill=(20, 16, 10, 255))
    d.rectangle([x0, h - 3, x0 + sw - 1, h + sh - 4], fill=stalk + (255,))
    d.line([(x0 + sw - 1, h - 3), (x0 + sw - 1, h + sh - 4)],
           fill=tuple(int(c * 0.7) for c in stalk) + (255,))
    out.alpha_composite(cap, (0, 0))
    return out


CAP_PURPLE = [(0.0, (40, 8, 60)), (0.5, (170, 60, 220)), (1.0, (245, 190, 255))]
CAP_TEAL = [(0.0, (0, 40, 50)), (0.5, (40, 190, 190)), (1.0, (205, 255, 240))]


def fungus_decor():
    out = blank()
    specs = [(CAP_TEAL, 0.50, 7, 25), (CAP_PURPLE, 0.46, 26, 22),
             (CAP_PURPLE, 0.80, 14, 31), (CAP_TEAL, 0.60, 25, 31)]
    for ramp, f, cx, by in specs:
        m = scale(mushroom(ramp, stalk=(196, 212, 222)), f)
        out = L(out, (m, cx - m.width // 2, by - m.height))
    return glow(out, (120, 255, 230), radius=1.8, strength=1.3)


def haunted_tome():
    wings = bbox_crop(T("mon/panlord/demon_wings_medium.png"))
    wings = gmap(wings, [(0.0, (20, 14, 30)), (0.5, (120, 110, 150)), (1.0, (235, 230, 250))])
    book = bbox_crop(drop_shadow_pixels(T("item/book/book_of_the_dead.png")))
    fig = L((wings, 16 - wings.width // 2, 4), (book, 16 - book.width // 2, 10))
    return glow(fig, (160, 120, 255), radius=1.5, strength=0.9)


def wren():
    """The Lamplighter: doll with leather armour, short sword and a lit lantern."""
    fig = D("base/human_f", "boots/middle_brown", "legs/pants_darkgreen", "body/leather_heavy",
            "hand1/short_sword", "hand2/misc/lantern", "hair/aragorn")
    return L(radial((255, 200, 90), 25, 24, 7, peak=120), fig)


def smuggler():
    return D("base/human_m", "boots/short_brown2", "legs/pants_black", "body/leather_jacket",
             "hand1/dagger", "hair/brown1", "head/bandana_ybrown")


def cutthroat():
    return D("cloak/black", "base/human_m", "boots/middle_brown", "legs/pants_black",
             "body/leather_stud", "gloves/glove_black", "hand1/sword_thief", "head/hood_black2")


def drowned_scholar():
    fig = D("base/human_m", "legs/pants_short_gray", "body/robe_brown3",
            "hand2/misc/book_cyan_dim", "hair/brown2")
    fig = gmap(fig, [(0.0, (8, 20, 18)), (0.35, (40, 80, 70)), (0.7, (110, 160, 140)),
                     (1.0, (200, 230, 210))], amount=0.75)
    d = ImageDraw.Draw(fig)
    for (x, y) in ((11, 20), (20, 24), (14, 28)):
        d.point([(x, y)], fill=(120, 200, 230, 255))
    return fig


def ink_slime():
    j = T("mon/amorphous/azure_jelly.png")
    j = gmap(j, [(0.0, (4, 4, 10)), (0.4, (20, 16, 48)), (0.75, (70, 50, 130)),
                 (1.0, (190, 170, 255))], norm=True)
    return glow(j, (110, 90, 200), radius=1.2, strength=0.9)   # rim so it reads on dark floors


def wisp():
    """A will-o-wisp: the DCSS silver star as a green-white mote of light."""
    s = bbox_crop(T("UNUSED/monsters/silver_star.png"))
    s = scale(gmap(s, [(0.0, (10, 50, 40)), (0.4, (50, 190, 140)), (0.8, (190, 255, 220)),
                       (1.0, (255, 255, 245))], norm=True), 0.62)
    return L(radial((90, 255, 190), 16, 16, 15, peak=120), place(s, 16, 16),
             radial((255, 255, 230), 16, 16, 5, peak=255, power=1.0))


def hollow_one():
    n = T("mon/undead/necrophage.png")
    n = gmap(n, [(0.0, (20, 14, 26)), (0.3, (120, 110, 130)), (0.65, (210, 204, 214)),
                 (1.0, (250, 248, 245))], where=lambda h, s, v: s < 0.45, norm=True)
    return hsv(n, hue=285, s=0.9, where=hue_in(60, 170, min_s=0.2))   # fungal rags


def ember_golem():
    g = T("mon/nonliving/iron_golem.png")
    g = gmap(g, [(0.0, (10, 6, 6)), (0.35, (60, 30, 22)), (0.6, (160, 70, 30)),
                 (0.85, (255, 150, 50)), (1.0, (255, 230, 150))])
    return glow(g, (255, 110, 30), radius=1.4, strength=0.8)


def ash_thrall():
    """A charred dwarf: DCSS deep dwarf re-graded to ash with ember highlights."""
    t = gmap(T("mon/deep_dwarf.png"),
             [(0.0, (8, 6, 6)), (0.45, (62, 58, 56)), (0.8, (128, 122, 116)),
              (0.9, (220, 110, 40)), (1.0, (255, 210, 120))], norm=True)
    return glow(t, (255, 90, 20), radius=1.2, strength=0.6)


def winter_wolf():
    w = T("mon/animals/wolf.png")
    return gmap(w, [(0.0, (14, 20, 34)), (0.3, (90, 110, 140)), (0.6, (190, 210, 230)),
                    (1.0, (255, 255, 255))], where=lambda h, s, v: s < 0.5)


def pale_shade():
    s = T("mon/undead/shadow.png")
    s = gmap(s, [(0.0, (10, 12, 26)), (0.4, (70, 90, 130)), (0.8, (190, 215, 240)),
                 (1.0, (255, 255, 255))], norm=True)
    return alpha_mul(glow(s, (180, 220, 255), radius=1.5, strength=0.8), 0.95)


def sewer_rat():
    r = T("mon/undead/zombies/zombie_rat.png")
    a = arr(r)
    a[RAT_EYE[1], RAT_EYE[0], :3] = (220, 40, 30)
    r = hsv(img(a), hue=24, s=0.4, v=0.6, where=hue_in(290, 345, min_s=0.3))
    return gmap(r, [(0.0, (20, 12, 8)), (0.35, (100, 74, 54)), (0.7, (160, 128, 100)),
                    (1.0, (230, 205, 180))], where=lambda h, s, v: s < 0.45, norm=True)


def white_beard():
    """The DCSS doll beard, greyed to white."""
    return gmap(T("player/beard/pj.png"),
                [(0.0, (60, 60, 64)), (0.5, (190, 190, 196)), (1.0, (250, 250, 250))], norm=True)


def elder():
    return D("base/human_m", "boots/middle_gray", "body/robe_white", "hand1/great_staff",
             white_beard(), "hair/fem_white")


def priestess():
    robe = gmap(T("player/body/robe_red_gold.png"),         # red -> white, gold trim kept
                [(0.0, (120, 114, 104)), (0.5, (232, 229, 220)), (1.0, (255, 255, 252))],
                where=hue_in(330, 25, min_s=0.35), norm=True)
    return D("base/human_f", "boots/middle_gold", robe, "hand1/sceptre", "hair/fem_yellow")


def item_feather():
    f = bbox_crop(drop_shadow_pixels(T("item/misc/misc_fan.png")))
    f = gmap(f, [(0.0, (40, 6, 0)), (0.4, (200, 50, 10)), (0.75, (255, 150, 40)),
                 (1.0, (255, 240, 170))], where=lambda h, s, v: s < 0.3)
    return glow(place(f, 16, 16), (255, 120, 30), radius=1.5, strength=0.8)


def flask(rel, overlay_rel, ov_scale=1.0, dx=0, dy=0):
    p = T(rel)
    ov = bbox_crop(T(overlay_rel))
    if ov_scale != 1.0:
        ov = ov.resize((round(ov.width * ov_scale), round(ov.height * ov_scale)), Image.NEAREST)
    return L(p, (ov, 20 + dx, 3 + dy))


def item_key():
    out = blank()
    d = ImageDraw.Draw(out)
    ink, gold, hi, lo = (40, 24, 6, 255), (220, 170, 50, 255), (255, 235, 140, 255), (150, 100, 24, 255)
    # bow (ring)
    d.ellipse([4, 4, 14, 14], fill=ink)
    d.ellipse([5, 5, 13, 13], fill=gold)
    d.ellipse([7, 7, 11, 11], fill=ink)
    d.ellipse([8, 8, 10, 10], fill=(0, 0, 0, 0))
    d.point([(7, 6), (6, 7)], fill=hi)
    # shaft (diagonal)
    d.line([(12, 12), (26, 26)], fill=ink, width=5)
    d.line([(12, 12), (26, 26)], fill=gold, width=3)
    d.line([(12, 11), (26, 25)], fill=hi, width=1)
    d.line([(13, 14), (26, 27)], fill=lo, width=1)
    # bit (teeth)
    for (x, y) in ((21, 24), (24, 27)):
        d.polygon([(x, y), (x - 4, y + 4), (x - 2, y + 6), (x + 2, y + 2)], fill=ink)
        d.polygon([(x, y + 1), (x - 3, y + 4), (x - 2, y + 5), (x + 1, y + 2)], fill=gold)
    return out


def item_seed():
    out = blank()
    d = ImageDraw.Draw(out)
    ink = (30, 18, 8, 255)
    seeds = [(9, 18), (17, 21), (13, 13), (21, 14)]
    for x, y in seeds:
        d.ellipse([x - 3, y - 4, x + 3, y + 4], fill=ink)
        d.ellipse([x - 2, y - 3, x + 2, y + 3], fill=(150, 98, 44, 255))
        d.line([(x - 1, y - 2), (x - 1, y + 1)], fill=(206, 150, 84, 255))
        d.point([(x + 1, y + 2)], fill=(96, 58, 24, 255))
    # sprout from the top seed
    d.line([(13, 9), (13, 5)], fill=(40, 110, 30, 255), width=1)
    d.ellipse([13, 2, 18, 6], fill=(80, 170, 50, 255), outline=(20, 60, 16, 255))
    d.ellipse([8, 3, 13, 6], fill=(110, 200, 70, 255), outline=(20, 60, 16, 255))
    return out


def item_mushroom():
    return place(mushroom(), 16, 17)


def hollows_floor(rel, specks=None):
    f = gmap(T(rel), [(0.0, (10, 4, 16)), (0.5, (60, 30, 80)), (1.0, (150, 100, 190))], norm=True)
    if specks:
        m = gmap(T(specks), [(0.0, (20, 90, 90)), (1.0, (150, 255, 230))])
        f = L(f, glow(m, (80, 230, 210), radius=1.0, strength=0.8))
    return opaque(f)


PALE_FLOOR = [(0.0, (58, 72, 98)), (0.5, (128, 148, 174)), (1.0, (196, 212, 230))]


def spell(rel):
    return T("gui/spells/" + rel + ".png")


def skill_light():
    s = spell("ice/ozocubus_refrigeration")
    return gmap(s, [(0.0, (0, 0, 0)), (0.25, (60, 40, 10)), (0.55, (230, 170, 50)),
                    (0.8, (255, 235, 150)), (1.0, (255, 255, 240))],
                where=lambda h, s_, v: (h > 180) & (h < 260) & (s_ > 0.2) | (v > 0.8))


def status_stun():
    return hsv(icon("item/weapon/brands/i-distortion.png"), hue=52, s=1.0, v=1.1,
               where=hue_in(260, 340, min_s=0.2))


def status_blind():
    ic = icon("misc/icons/blind.png", factor=3)
    d = ImageDraw.Draw(ic)
    d.line([(7, 24), (25, 8)], fill=(0, 0, 0, 255), width=4)
    d.line([(7, 24), (25, 8)], fill=(230, 230, 230, 255), width=2)
    return ic


def status_sunder():
    ic = icon("item/armour/brands/i-protection.png")
    d = ImageDraw.Draw(ic)
    pts = [(16, 5), (13, 11), (18, 15), (13, 21), (16, 27)]
    d.line(pts, fill=(0, 0, 0, 255), width=3)
    d.line(pts, fill=(255, 80, 40, 255), width=1)
    return ic


def fx_slash():
    """A white crescent slash (no suitable DCSS tile)."""
    yy, xx = np.mgrid[0:CELL, 0:CELL].astype(np.float64) + 0.5
    cx, cy, r = 4.0, 30.0, 25.0
    d = np.hypot(xx - cx, yy - cy)
    ang = np.degrees(np.arctan2(cy - yy, xx - cx))      # 0 = right, 90 = up
    t = np.clip((ang - 5) / 80.0, 0, 1)                  # along the arc
    width = 1.0 + 3.2 * np.sin(np.pi * t)                # thick in the middle
    inside = (np.abs(d - r) <= width) & (ang > 5) & (ang < 85)
    edge = (np.abs(d - r) <= width + 1.0) & (ang > 3) & (ang < 87) & ~inside
    g = np.zeros((CELL, CELL, 4))
    core = np.abs(d - r) <= width * 0.45
    g[inside] = (205, 230, 255, 235)
    g[inside & core] = (255, 255, 255, 255)
    g[edge] = (120, 170, 255, 110)
    return img(g)


def fx_shock():
    b = T("effect/bolt01.png")
    z = T("effect/zap1.png")
    return L(b, z, (flip(z), 0, 0))


def fx_light():
    halo = alpha_mul(T("player/halo/halo_player.png"), 0.7)
    star = T("effect/searing_ray5.png")
    star = gmap(star, [(0.0, (120, 70, 0)), (0.5, (255, 200, 60)), (1.0, (255, 255, 235))])
    sp = clear_below(T("UNUSED/other/gold_sparkles1.png"), 40)
    return L(halo, star, sp)


def fx_shadow():
    c = T("effect/cloud_neg2.png")
    return hsv(c, hue=275, s=0.9, v=1.0, where=hue_in(150, 210, min_s=0.2))


def fx_heal():
    gold = clear_below(T("UNUSED/other/gold_sparkles1.png"), 40)
    green = hsv(flip(gold), hue=120, s=0.9, v=1.0)
    spark = bbox_crop(T("player/hand1/misc/spark.png"))
    spark_g = hsv(spark, hue=110, s=0.8)
    return L(radial((120, 255, 120), 16, 16, 14, peak=70), green, (gold, 2, -2),
             (spark, 10, 9), (spark_g, 18, 16))


def fx_buff():
    out = blank()
    d = ImageDraw.Draw(out)
    for i, y in enumerate((22, 14, 6)):
        c = [(255, 170, 40, 255), (255, 205, 80, 255), (255, 240, 150, 255)][i]
        pts = [(8, y + 8), (16, y), (24, y + 8)]
        d.line(pts, fill=(60, 30, 0, 255), width=5)
        d.line(pts, fill=c, width=3)
    spark = bbox_crop(T("player/hand1/misc/spark.png"))
    return L(radial((255, 200, 80), 16, 18, 15, peak=60), out, (spark, 1, 4), (spark, 22, 20))


def fx_explosion():
    fire = T("effect/cloud_forest_fire.png")
    star = scale(bbox_crop(T("effect/searing_ray5.png")), 1.25, alpha_cut=None)
    return L(fire, place(star, 16, 16))


def bolt(rel, size=28):
    """A projectile glyph enlarged to about `size` px (soft glow edges kept)."""
    g = bbox_crop(T(rel))
    f = size / max(g.width, g.height)
    return place(scale(g, f, alpha_cut=None), 16, 16)


# --------------------------------------------------------------------------
# The manifest: (enum variant, recipe) in enum order.
# A recipe is a source path (copied as-is) or a function returning a 32x32 RGBA image.
# --------------------------------------------------------------------------

SPRITES = [
    # --- Terrain: town (Hollowmere) ---
    ("TownGrassA", lambda: warm(T("dngn/floor/grass/grass0.png"), 0.15)),
    ("TownGrassB", lambda: warm(T("dngn/floor/grass/grass1.png"), 0.15)),
    ("TownGrassC", lambda: warm(T("dngn/floor/grass/grass_flowers_yellow1.png"), 0.15)),
    ("TownPathA", lambda: cobble("dngn/floor/cobble_blood1.png")),
    ("TownPathB", lambda: cobble("dngn/floor/cobble_blood3.png")),
    ("TownWall", timber_wall),
    ("WoodFloor", wood_floor),
    ("TownWater", "dngn/water/deep_water.png"),
    ("TreeA", lambda: tree("dngn/trees/tree1_yellow.png", canopy_hue=100, canopy_s=0.9, canopy_v=0.8)),
    ("TreeB", lambda: tree("dngn/trees/tree2_lightred.png")),
    ("Fountain", "dngn/blue_fountain.png"),
    ("Statue", lambda: gmap(T("dngn/vaults/golden_statue_1.png"),
                            [(0.0, (14, 14, 18)), (0.4, (88, 90, 96)), (0.75, (160, 162, 166)),
                             (1.0, (230, 232, 228))])),
    ("LamppostLit", lambda: lamppost(True)),
    ("LamppostOut", lambda: lamppost(False)),
    ("TempleAltar", "dngn/altars/shining_one.png"),
    ("ShopWeapon", lambda: shop_sign("item/weapon/long_sword1.png")),
    ("ShopArmour", lambda: shop_sign("item/armour/scale_mail1.png", 0.8)),
    ("ShopPotion", lambda: shop_sign("item/potion/ruby.png", 0.9)),
    ("ShopGeneral", lambda: shop_sign("item/gold/10.png", 0.9, 3)),
    ("ShopInn", "UNUSED/gui/tavern.png"),
    ("VaultGate", "dngn/gateways/enter_crypt.png"),

    # --- Terrain: dungeon biomes ---
    ("UndercroftFloorA", lambda: opaque(bright(T("dngn/floor/pebble_brown0.png"), 1.15))),
    ("UndercroftFloorB", lambda: opaque(bright(T("dngn/floor/pebble_brown2.png"), 1.15))),
    ("UndercroftFloorC", lambda: opaque(bright(T("dngn/floor/pebble_brown5.png"), 1.15))),
    ("UndercroftWallA", lambda: opaque(hsv(T("dngn/wall/brick_brown0.png"), s=0.6, v=0.68))),
    ("UndercroftWallB", lambda: opaque(bright(tint(T("dngn/wall/brick_gray1.png"), (150, 116, 86), 0.6), 0.85))),
    ("ArchiveFloorA", lambda: opaque(T("dngn/floor/marble_floor1.png"))),
    ("ArchiveFloorB", lambda: opaque(T("dngn/floor/marble_floor3.png"))),
    ("ArchiveFloorC", lambda: opaque(T("dngn/floor/marble_floor5.png"))),
    ("ArchiveWallA", lambda: opaque(bright(tint(T("dngn/wall/stone2_gray0.png"), (100, 124, 170), 0.55), 0.72))),
    ("ArchiveWallB", lambda: opaque(bright(tint(T("dngn/wall/marble_wall1.png"), (100, 124, 170), 0.5), 0.72))),
    ("ShallowWater", "dngn/water/shallow_water.png"),
    ("Bookshelf", bookshelf),
    ("HollowsFloorA", lambda: hollows_floor("dngn/floor/floor_nerves0.png")),
    ("HollowsFloorB", lambda: hollows_floor("dngn/floor/floor_nerves3.png")),
    ("HollowsFloorC", lambda: hollows_floor("dngn/floor/floor_nerves5.png", specks="misc/mold_glowing2.png")),
    ("HollowsWallA", lambda: opaque(gmap(T("dngn/wall/pebble_red0.png"),
                                         [(0.0, (8, 2, 14)), (0.4, (60, 28, 84)), (0.8, (130, 80, 160)), (1.0, (200, 160, 220))], norm=True))),
    ("HollowsWallB", lambda: opaque(gmap(T("dngn/wall/pebble_red2.png"),
                                         [(0.0, (2, 10, 14)), (0.4, (24, 64, 76)), (0.8, (70, 140, 150)), (1.0, (160, 220, 210))], norm=True))),
    ("FungusDecor", fungus_decor),
    ("ForgeFloorA", lambda: opaque(T("dngn/floor/volcanic_floor0.png"))),
    ("ForgeFloorB", lambda: opaque(T("dngn/floor/volcanic_floor2.png"))),
    ("ForgeFloorC", lambda: opaque(T("dngn/floor/volcanic_floor5.png"))),
    ("ForgeWallA", lambda: opaque(T("dngn/wall/volcanic_wall0.png"))),
    ("ForgeWallB", lambda: opaque(T("dngn/wall/volcanic_wall3.png"))),
    ("Lava", "UNUSED/features/lava.png"),
    ("PaleFloorA", lambda: opaque(gmap(T("dngn/floor/white_marble0.png"), PALE_FLOOR, norm=True))),
    ("PaleFloorB", lambda: opaque(gmap(T("dngn/floor/white_marble3.png"), PALE_FLOOR, norm=True))),
    ("PaleFloorC", lambda: opaque(gmap(T("dngn/floor/ice1.png"), PALE_FLOOR, norm=True))),
    ("PaleWallA", lambda: opaque(gmap(T("dngn/wall/marble_wall2.png"),
                                      [(0.0, (70, 90, 120)), (0.5, (175, 200, 222)), (1.0, (248, 252, 255))], norm=True))),
    ("PaleWallB", lambda: opaque(gmap(T("dngn/wall/crystal_wall_white.png"),
                                      [(0.0, (20, 30, 50)), (0.4, (120, 150, 185)), (0.8, (215, 235, 250)), (1.0, (255, 255, 255))]))),
    ("IceCrystal", "mon/statues/block_of_ice.png"),

    # --- Common features ---
    ("DoorClosed", lambda: remove_stone_frame(T("dngn/doors/closed_door.png"))),
    ("DoorOpen", lambda: remove_stone_frame(T("dngn/doors/open_door.png"))),
    ("StairsDown", lambda: opaque(gmap(T("UNUSED/features/rock_stairs_down.png"),
                                       [(0.0, (10, 8, 8)), (0.4, (70, 62, 56)), (0.8, (150, 140, 128)), (1.0, (210, 200, 186))], norm=True))),
    ("Waystone", waystone),
    ("ChestClosed", lambda: chest(False)),
    ("ChestOpen", lambda: chest(True)),
    ("MemoryShard", memory_shard),
    ("NoteScroll", "item/misc/runes/rune_spider.png"),
    ("WallTorch", wall_torch),
    ("Brazier", "dngn/altars/makhleb_flame1.png"),
    ("Bones", bones),
    ("Rubble", rubble),
    ("SealedGate", sealed_gate),

    # --- Party ---
    ("Wren", wren),
    ("Brannoc", "mon/dwarf.png"),
    ("Maelis", lambda: D("base/elf_f", "boots/middle_purple", "body/robe_purple",
                         "hand1/staff_mage", "hair/arwen")),
    ("Pip", "mon/spriggan/spriggan_druid.png"),

    # --- Story characters ---
    ("Ilsa", ilsa),
    ("IlsaPale", ilsa_pale),
    ("Aurelian", aurelian),
    ("AurelianTrue", aurelian_true),
    ("Curator", "mon/undead/ancient_lich.png"),
    ("Gristlemaw", gristlemaw),
    ("MotherOfSpores", mother_of_spores),
    ("IronWarden", "mon/unique/iron_giant.png"),
    ("VexHarlan", "mon/unique/maurice.png"),

    # --- Town NPCs ---
    ("Elder", elder),
    ("Innkeeper", lambda: D("base/human_f", "boots/short_brown2", "legs/skirt_red",
                            "body/shirt_white1", "hand1/misc/bottle", "hair/fem_red")),
    ("Smith", lambda: D("base/dwarf_f", "boots/middle_brown", "legs/pants_black",
                        "body/leather_heavy", "gloves/glove_brown", "UNUSED/worn/hammer_one.png",
                        "hair/pigtail_red")),
    ("Apothecary", lambda: D("base/human_m", "boots/middle_green", "body/robe_green",
                             "hand1/misc/bottle", "hair/brown1")),
    ("Priestess", priestess),
    ("Captain", lambda: D("cloak/blue", "base/human_m", "boots/mesh_black", "legs/leg_armour02",
                          "body/plate_and_cloth2", "hand1/long_sword_slant", "hand2/shield_knight_blue",
                          "head/helm_plume")),
    ("OldWoman", "mon/unique/josephine.png"),
    ("Child", "mon/halfling.png"),
    ("VillagerA", "mon/human.png"),
    ("VillagerB", lambda: D("base/human_f", "boots/short_brown2", "legs/skirt_blue",
                            "body/shirt_white2", "hair/pigtails_yellow")),
    ("GuardNpc", lambda: D("base/human_m", "boots/mesh_black", "legs/leg_armour01",
                           "body/green_chain", "hand1/spear", "hand2/shield_knight_blue", "head/iron_red")),
    ("Cat", "player/felids/cat9.png"),

    # --- Enemies: Act I Undercroft ---
    ("SewerRat", sewer_rat),
    ("CaveBat", "mon/animals/bat.png"),
    ("GiantCentipede", "UNUSED/monsters/giant_centipede.png"),
    ("Smuggler", smuggler),
    ("Cutthroat", cutthroat),
    ("Ghoul", "mon/undead/ghoul.png"),
    ("KoboldScavenger", "mon/kobold.png"),
    # --- Act II Drowned Archive ---
    ("DrownedScholar", drowned_scholar),
    ("InkSlime", ink_slime),
    ("HauntedTome", haunted_tome),
    ("ArchiveSentinel", "mon/undead/skeletal_warrior.png"),
    ("Wisp", wisp),
    ("MireEel", "mon/aquatic/electric_eel.png"),
    ("Phantom", "mon/undead/phantom.png"),
    # --- Act III Mycelium Hollows ---
    ("Sporeling", "mon/fungi_plants/wandering_mushroom.png"),
    ("CaveSpider", "mon/animals/wolf_spider.png"),
    ("Weaver", "mon/animals/orb_spider.png"),
    ("GiantSlug", "mon/animals/elephant_slug.png"),
    ("HollowOne", hollow_one),
    ("Toadstool", lambda: gmap(T("player/transform/mushroom_form.png"),
                               [(0.0, (50, 4, 4)), (0.5, (200, 30, 24)), (0.8, (240, 110, 90)), (1.0, (255, 240, 230))],
                               where=hue_in(50, 120, min_s=0.3), box=(0, 0, 32, 18), norm=True)),
    ("RotWalker", "mon/fungi_plants/treant.png"),
    # --- Act IV Ember Forge ---
    ("FireImp", "mon/demons/crimson_imp.png"),
    ("Salamander", "mon/salamander.png"),
    ("EmberGolem", ember_golem),
    ("AshThrall", ash_thrall),
    ("CinderHound", "mon/animals/hell_hound.png"),
    ("FireElemental", "mon/nonliving/fire_elemental.png"),
    ("Efreet", "mon/demons/efreet.png"),
    # --- Act V Pale Reach ---
    ("FrostWraith", "mon/undead/shadow_wraith.png"),
    ("IceBeast", "mon/animals/ice_beast.png"),
    ("PaleShade", pale_shade),
    ("WinterWolf", winter_wolf),
    ("PaleKnight", "mon/death_knight.png"),
    ("FrostGiant", "mon/frost_giant.png"),
    ("SoulEater", "mon/demons/soul_eater.png"),
    ("RimeDrake", "mon/dragons/ice_dragon.png"),

    # --- Item icons ---
    ("ItemPotionRed", "item/potion/ruby.png"),
    ("ItemPotionBlue", "item/potion/brilliant_blue.png"),
    ("ItemPotionGold", "item/potion/golden.png"),
    ("ItemPotionGreen", "item/potion/bubbly.png"),
    ("ItemPotionPurple", "item/potion/magenta.png"),
    ("ItemFeather", item_feather),
    ("ItemFlaskFire", lambda: flask("item/potion/orange.png", "player/hand1/misc/fire_red.png")),
    ("ItemFlaskFrost", lambda: flask("item/potion/sky_blue.png", "effect/frost0.png", 1.0, -3, -2)),
    ("ItemSmoke", "item/potion/cloudy.png"),
    ("ItemScroll", "item/scroll/scroll-blue.png"),
    ("ItemSword", "item/weapon/long_sword1.png"),
    ("ItemSwordFine", "item/weapon/long_sword3.png"),
    ("ItemAxe", "UNUSED/weapons/war_axe4.png"),
    ("ItemHammer", "UNUSED/weapons/hammer1.png"),
    ("ItemStaff", "item/staff/staff01.png"),
    ("ItemWand", "item/wand/gem_glass.png"),
    ("ItemDagger", "item/weapon/dagger.png"),
    ("ItemLightArmor", "item/armour/leather_armour1.png"),
    ("ItemHeavyArmor", "item/armour/plate1.png"),
    ("ItemRobe", "item/armour/robe_ego2.png"),
    ("ItemShield", "item/armour/shields/large_shield1.png"),
    ("ItemRing", "item/ring/ruby.png"),
    ("ItemAmulet", "item/amulet/celtic_blue.png"),
    ("ItemGold", "item/gold/10.png"),
    ("ItemKey", item_key),
    ("ItemOre", "item/misc/runes/rune_gehenna.png"),
    ("ItemBook", "item/book/leather.png"),
    ("ItemSeed", item_seed),
    ("ItemTag", "item/misc/runes/generic.png"),
    ("ItemPage", lambda: bright(flip(T("item/misc/runes/rune_spider.png")), 1.08, 6)),
    ("ItemRelic", "item/misc/runes/rune_tomb.png"),
    ("ItemMushroom", item_mushroom),

    # --- Skill icons ---
    ("SkillSlash", lambda: spell("enchantment/sure_blade")),
    ("SkillFire", lambda: spell("fire/fireball")),
    ("SkillFrost", lambda: spell("ice/freeze")),
    ("SkillShock", lambda: spell("air/lightning_bolt")),
    ("SkillLight", skill_light),
    ("SkillShadow", lambda: spell("summoning/summon_shadow_creatures")),
    ("SkillNature", lambda: spell("summoning/summon_forest")),
    ("SkillHeal", lambda: spell("necromancy/regeneration")),
    ("SkillBuff", lambda: spell("enchantment/haste")),
    ("SkillDebuff", lambda: spell("enchantment/cause_fear")),
    ("SkillShield", lambda: spell("ice/condensation_shield")),
    ("SkillRevive", lambda: spell("necromancy/borgnjors_revivification")),
    ("SkillEarth", lambda: spell("earth/lees_rapid_deconstruction")),
    ("SkillArcane", lambda: spell("conjuration/iskenderuns_mystic_blast")),
    ("SkillUltimate", lambda: spell("fire/fire_storm")),

    # --- Status icons ---
    ("StatusPoison", lambda: icon("item/weapon/brands/i-venom.png")),
    ("StatusBurn", lambda: icon("item/weapon/brands/i-flaming.png")),
    ("StatusFrozen", lambda: icon("item/weapon/brands/i-freezing.png")),
    ("StatusStun", status_stun),
    ("StatusBlind", status_blind),
    ("StatusWeak", lambda: icon("item/weapon/brands/i-sickness.png")),
    ("StatusSunder", status_sunder),
    ("StatusHaste", lambda: icon("item/armour/brands/i-running.png")),
    ("StatusRegen", lambda: icon("item/amulet/i-regeneration.png")),
    ("StatusShield", lambda: icon("item/armour/brands/i-spirit.png")),
    ("StatusTaunt", lambda: icon("item/weapon/brands/i-frenzy.png")),
    ("StatusSlow", lambda: icon("item/weapon/brands/i-slowing.png")),

    # --- Battle effects ---
    ("FxSlash", fx_slash),
    ("FxFire", "effect/cloud_fire2.png"),
    ("FxFrost", "effect/cloud_cold2.png"),
    ("FxShock", fx_shock),
    ("FxLight", fx_light),
    ("FxShadow", fx_shadow),
    ("FxPoison", "effect/cloud_poison2.png"),
    ("FxHeal", fx_heal),
    ("FxBuff", fx_buff),
    ("FxExplosion", fx_explosion),
    ("FxBoltFire", lambda: bolt("effect/flame0.png", 26)),
    ("FxBoltFrost", lambda: bolt("effect/frost0.png", 28)),
    ("FxBoltShock", lambda: bolt("effect/zap1.png", 26)),
    ("FxBoltArcane", lambda: bolt("effect/magic_dart0.png", 26)),
    ("FxCloud", "effect/cloud_grey_smoke.png"),
]


# --------------------------------------------------------------------------
# Output
# --------------------------------------------------------------------------

def render(recipe) -> Image.Image:
    im = T(recipe) if isinstance(recipe, str) else recipe()
    im = fit(im)
    assert im.size == (CELL, CELL) and im.mode == "RGBA"
    return im


def write_rust(names, rows, path: Path):
    lines = [
        "//! @generated by tools/build_atlas.py from the CC0 Dungeon Crawl Stone Soup tiles. Do not edit by hand.",
        "",
        "/// One 32x32 cell of `assets/atlas.png`.",
        "#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]",
        "pub enum Sprite {",
    ]
    lines += [f"    {n}," for n in names]
    lines += [
        "}",
        "",
        "pub const CELL: u32 = 32;",
        "pub const ATLAS_COLUMNS: u32 = 16;",
        f"pub const ATLAS_ROWS: u32 = {rows};",
        'pub static ATLAS_PNG: &[u8] = include_bytes!("../../assets/atlas.png");',
        "",
        "impl Sprite {",
        f"    pub const ALL: [Sprite; {len(names)}] = [",
    ]
    lines += [f"        Sprite::{n}," for n in names]
    lines += [
        "    ];",
        "",
        "    /// (column, row) of this sprite's cell in the atlas.",
        "    pub fn cell(self) -> (u32, u32) {",
        "        let i = self as u32;",
        "        (i % ATLAS_COLUMNS, i / ATLAS_COLUMNS)",
        "    }",
        "}",
        "",
    ]
    path.write_text("\n".join(lines), encoding="utf-8")


def contact_sheet(names, cells, path: Path):
    scale_, cols = 2, 12
    cw, ch = 104, 32 * scale_ + 22
    rows = (len(cells) + cols - 1) // cols
    sheet = Image.new("RGBA", (cols * cw, rows * ch), (128, 128, 128, 255))
    try:
        font = ImageFont.truetype("DejaVuSans.ttf", 10)
    except OSError:
        font = ImageFont.load_default()
    d = ImageDraw.Draw(sheet)
    for i, (n, im) in enumerate(zip(names, cells)):
        x, y = (i % cols) * cw, (i // cols) * ch
        sheet.alpha_composite(im.resize((32 * scale_, 32 * scale_), Image.NEAREST),
                              (x + (cw - 32 * scale_) // 2, y + 3))
        tw = d.textlength(n, font=font)
        d.text((x + (cw - tw) / 2, y + 32 * scale_ + 6), n, fill=(255, 255, 255, 255), font=font)
    sheet.convert("RGB").save(path)


def main():
    global RELEASE, BLACKLIST
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("release", type=Path, help="crawl-tiles/releases/Nov-2015 directory")
    ap.add_argument("--preview", type=Path, help="also write a labelled contact sheet here")
    args = ap.parse_args()
    RELEASE = args.release.resolve()
    if not (RELEASE / "dngn").is_dir():
        raise SystemExit(f"{RELEASE} does not look like a crawl-tiles release directory")
    BLACKLIST = load_blacklist(RELEASE)

    names = [n for n, _ in SPRITES]
    assert len(names) == len(set(names)), "duplicate sprite name"
    cells = [render(r) for _, r in SPRITES]

    rows = (len(cells) + COLUMNS - 1) // COLUMNS
    atlas = Image.new("RGBA", (COLUMNS * CELL, rows * CELL), (0, 0, 0, 0))
    for i, im in enumerate(cells):
        atlas.paste(im, ((i % COLUMNS) * CELL, (i // COLUMNS) * CELL))

    (REPO / "assets").mkdir(exist_ok=True)
    (REPO / "src" / "gfx").mkdir(parents=True, exist_ok=True)
    atlas.save(REPO / "assets" / "atlas.png", optimize=True)
    write_rust(names, rows, REPO / "src" / "gfx" / "sprites.rs")
    if args.preview:
        contact_sheet(names, cells, args.preview)
    print(f"{len(cells)} sprites, {len(USED)} source tiles -> assets/atlas.png "
          f"({atlas.width}x{atlas.height}), src/gfx/sprites.rs")


if __name__ == "__main__":
    main()
