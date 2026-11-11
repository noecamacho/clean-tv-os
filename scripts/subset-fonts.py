#!/usr/bin/env python3
"""Regenerates the font assets embedded in lume.

Downloads pinned upstream sources, subsets them and writes:
  crates/lume-core/assets/fonts/PlusJakartaSans-<weight>.ttf   (Latin-1 + Latin Extended-A)
  crates/lume-core/assets/fonts/lucide-subset.ttf              (only the icons lume uses, drawn from
                                                               the Lucide SVGs with LUCIDE_STROKE)
  crates/lume-core/assets/fonts/OFL.txt, LUCIDE-LICENSE.txt
  crates/lume-core/src/icons.rs                                (icon code points)

Requirements: python3 and `pip install fonttools`.
Usage: scripts/subset-fonts.py [--icons-only]
"""

import json
import math
import pathlib
import re
import sys
import urllib.request

from fontTools import subset
from fontTools.fontBuilder import FontBuilder
from fontTools.pens.basePen import BasePen
from fontTools.pens.ttGlyphPen import TTGlyphPen
from fontTools.svgLib.path import parse_path
from fontTools.ttLib import TTFont, newTable
from fontTools.ttLib.tables._k_e_r_n import KernTable_format_0

PLUS_JAKARTA_COMMIT = "18d1cd2f7ea10481919d2f05c1f7064b7307fc26"
PLUS_JAKARTA_URL = (
    "https://raw.githubusercontent.com/tokotype/PlusJakartaSans/"
    + PLUS_JAKARTA_COMMIT
    + "/{path}"
)
LUCIDE_VERSION = "1.59.0"
LUCIDE_URL = "https://unpkg.com/lucide-static@" + LUCIDE_VERSION + "/{path}"
# Stroke of the icons on Lucide's 24 grid. Lucide draws with 2; 1.75 gives the 36 px top bar glyph
# the stem of the tab text (Plus Jakarta Sans 700 at 22), so icons and text read as one family.
LUCIDE_STROKE = 1.75
# Font units per Lucide grid unit (the 24 grid is the em, as in Lucide's own font).
ICON_UNITS = 50

# Weights the theme uses (see themes/lume/theme.css).
WEIGHTS = {"Medium": 500, "Bold": 700, "ExtraBold": 800}

# Latin-1 + Latin Extended-A, plus the punctuation the UI strings use.
TEXT_UNICODES = (
    list(range(0x0020, 0x007F))
    + list(range(0x00A0, 0x0100))
    + list(range(0x0100, 0x0180))
    + [0x2013, 0x2014, 0x2018, 0x2019, 0x201C, 0x201D, 0x2022, 0x2026, 0x2039, 0x203A, 0x20AC]
)

# Lucide icon names used by the UI, mapped to Rust constant names.
ICONS = {
    "settings": "SETTINGS",
    "log-in": "INPUTS",
    "chevron-up": "CHEVRON_UP",
    "chevron-down": "CHEVRON_DOWN",
    "chevron-left": "CHEVRON_LEFT",
    "chevron-right": "CHEVRON_RIGHT",
    "trash-2": "TRASH",
    "x": "CLOSE",
    "check": "CHECK",
    "plus": "PLUS",
    "play": "PLAY",
    "eye": "EYE",
    "eye-off": "EYE_OFF",
    "sliders-horizontal": "CONTROL_CENTER",
    "wifi": "WIFI",
    "bluetooth": "BLUETOOTH",
    "contrast": "PICTURE",
    "volume-2": "SOUND",
    "moon": "SLEEP",
}

# Kerning pairs smaller than this many font units (of 1000 per em) are invisible
# at TV sizes and are dropped to keep the legacy `kern` table small.
MIN_KERN_UNITS = 10
# Kerning is kept for Basic Latin only; accented letters fall back to no kerning.
KERN_UNICODES = range(0x0021, 0x007F)

ROOT = pathlib.Path(__file__).resolve().parent.parent
FONTS = ROOT / "crates/lume-core/assets/fonts"


def download(url: str) -> bytes:
    with urllib.request.urlopen(url) as response:
        return response.read()


def gpos_kern_pairs(font: TTFont, glyphs: list[str]) -> dict[tuple[str, str], int]:
    """Expands the GPOS `kern` feature (pair adjustment) into plain glyph pairs."""
    gpos = font["GPOS"].table
    lookup_indices = set()
    for record in gpos.FeatureList.FeatureRecord:
        if record.FeatureTag == "kern":
            lookup_indices.update(record.Feature.LookupListIndex)

    wanted = set(glyphs)
    pairs: dict[tuple[str, str], int] = {}

    def advance(value) -> int:
        return getattr(value, "XAdvance", 0) or 0 if value is not None else 0

    for index in sorted(lookup_indices):
        for subtable in gpos.LookupList.Lookup[index].SubTable:
            if subtable.LookupType == 9:  # extension
                subtable = subtable.ExtSubTable
            if subtable.LookupType != 2:  # pair adjustment
                continue
            first_glyphs = [g for g in subtable.Coverage.glyphs if g in wanted]
            if subtable.Format == 1:
                for glyph, pair_set in zip(subtable.Coverage.glyphs, subtable.PairSet):
                    if glyph not in wanted:
                        continue
                    for record in pair_set.PairValueRecord:
                        if record.SecondGlyph in wanted:
                            pairs.setdefault((glyph, record.SecondGlyph), advance(record.Value1))
            else:
                classes1 = subtable.ClassDef1.classDefs
                classes2 = subtable.ClassDef2.classDefs
                for first in first_glyphs:
                    row = subtable.Class1Record[classes1.get(first, 0)].Class2Record
                    for second in glyphs:
                        value = advance(row[classes2.get(second, 0)].Value1)
                        pairs.setdefault((first, second), value)
    return {pair: v for pair, v in pairs.items() if abs(v) >= MIN_KERN_UNITS}


def replace_gpos_with_kern_table(font: TTFont) -> None:
    """fontdue only reads the legacy `kern` table, so convert GPOS kerning into one."""
    cmap = font.getBestCmap()
    glyphs = [cmap[c] for c in KERN_UNICODES if c in cmap]
    pairs = gpos_kern_pairs(font, glyphs)
    if len(pairs) > 10920:
        raise SystemExit(f"{len(pairs)} kerning pairs do not fit one kern subtable")
    kern = newTable("kern")
    kern.version = 0
    subtable = KernTable_format_0()
    subtable.coverage = 1
    subtable.format = 0
    subtable.kernTable = pairs
    kern.kernTables = [subtable]
    font["kern"] = kern
    del font["GPOS"]


def subset_font(data: bytes, unicodes: list[int], out: pathlib.Path, kern: bool) -> None:
    options = subset.Options()
    options.layout_features = ["kern"] if kern else []
    options.hinting = False
    options.name_IDs = [0, 1, 2, 3, 4, 5, 6, 13, 14]
    options.notdef_outline = True
    options.drop_tables += ["GSUB", "GDEF", "DSIG", "gasp", "cvt ", "fpgm", "prep"]
    font = TTFont(__import__("io").BytesIO(data))
    subsetter = subset.Subsetter(options)
    subsetter.populate(unicodes=unicodes)
    subsetter.subset(font)
    if kern:
        replace_gpos_with_kern_table(font)
    font.save(out)
    print(f"{out.relative_to(ROOT)}: {out.stat().st_size} bytes")


class _Flattener(BasePen):
    """Collects an SVG path as polylines; curves become short segments whose inner points are smooth."""

    def __init__(self):
        super().__init__(None)
        self.lines = []  # [(points, closed)], each point (x, y, corner)

    def _moveTo(self, pt):
        self.lines.append(([(pt[0], pt[1], True)], False))

    def _lineTo(self, pt):
        self.lines[-1][0].append((pt[0], pt[1], True))

    def _curveToOne(self, p1, p2, p3):
        x0, y0, _ = self.lines[-1][0][-1]
        steps = 16
        for i in range(1, steps + 1):
            t = i / steps
            u = 1 - t
            x = u**3 * x0 + 3 * u * u * t * p1[0] + 3 * u * t * t * p2[0] + t**3 * p3[0]
            y = u**3 * y0 + 3 * u * u * t * p1[1] + 3 * u * t * t * p2[1] + t**3 * p3[1]
            self.lines[-1][0].append((x, y, i == steps))

    def _qCurveToOne(self, p1, p2):
        x0, y0, _ = self.lines[-1][0][-1]
        c1 = (x0 + 2 / 3 * (p1[0] - x0), y0 + 2 / 3 * (p1[1] - y0))
        c2 = (p2[0] + 2 / 3 * (p1[0] - p2[0]), p2[1] + 2 / 3 * (p1[1] - p2[1]))
        self._curveToOne(c1, c2, p2)

    def _closePath(self):
        points, _ = self.lines[-1]
        self.lines[-1] = (points, True)

    def _endPath(self):
        pass


def _svg_polylines(svg: str):
    """The polylines of every <path> and <circle> of a Lucide icon, in grid units (y down)."""
    pen = _Flattener()
    for d in re.findall(r'<path[^>]*\sd="([^"]+)"', svg):
        parse_path(d, pen)
    for attrs in re.findall(r"<circle([^>]*)>", svg):
        get = lambda name: float(re.search(name + r'="([-\d.]+)"', attrs).group(1))
        cx, cy, r = get("cx"), get("cy"), get("r")
        points = [(cx + r * math.cos(a), cy + r * math.sin(a), False) for a in
                  (2 * math.pi * i / 48 for i in range(48))]
        pen.lines.append((points, True))
    if not pen.lines:
        raise SystemExit("icon has no path or circle")
    return pen.lines


def _stroke_polygons(lines, half: float):
    """Polygons that cover the stroke: a rectangle per segment, a disc on each corner and end
    (Lucide's round caps and joins) and a small quad on each smooth point of a curve."""
    polygons = []
    disc = [(half * math.cos(a), half * math.sin(a)) for a in (2 * math.pi * i / 16 for i in range(16))]
    for points, closed in lines:
        pts = list(points)
        if closed and (pts[0][0], pts[0][1]) != (pts[-1][0], pts[-1][1]):
            pts.append(pts[0])
        segments = []
        for (x0, y0, _), (x1, y1, _) in zip(pts, pts[1:]):
            length = math.hypot(x1 - x0, y1 - y0)
            if length < 1e-6:
                continue
            nx, ny = -(y1 - y0) / length * half, (x1 - x0) / length * half
            segments.append((nx, ny))
            polygons.append([(x0 + nx, y0 + ny), (x1 + nx, y1 + ny), (x1 - nx, y1 - ny), (x0 - nx, y0 - ny)])
        for index, (x, y, corner) in enumerate(pts):
            if corner or index == 0 or index == len(pts) - 1 or index - 1 >= len(segments) or index >= len(segments):
                polygons.append([(x + dx, y + dy) for dx, dy in disc])
            else:
                (ax, ay), (bx, by) = segments[index - 1], segments[index]
                polygons.append([(x + ax, y + ay), (x + bx, y + by), (x - ax, y - ay), (x - bx, y - by)])
    return polygons


def _area(polygon) -> float:
    return sum(x0 * y1 - x1 * y0 for (x0, y0), (x1, y1) in zip(polygon, polygon[1:] + polygon[:1])) / 2


def build_icon_font(svgs: dict[int, str], stroke: float, out: pathlib.Path) -> None:
    """Draws each Lucide SVG with the given stroke into a font, at the given code points. All
    contours wind the same way, so where they overlap the rasterizer just covers the pixel."""
    em = 24 * ICON_UNITS
    names = [".notdef"] + [f"icon{code:04x}" for code in svgs]
    glyphs = {".notdef": TTGlyphPen(None).glyph()}
    metrics = {".notdef": (em, 0)}
    for code, svg in svgs.items():
        pen = TTGlyphPen(None)
        for polygon in _stroke_polygons(_svg_polylines(svg), stroke / 2):
            # Font space: y up, the 24 grid filling the em above the baseline.
            points = [(round(x * ICON_UNITS), round((24 - y) * ICON_UNITS)) for x, y in polygon]
            if _area(points) < 0:  # counterclockwise outside, like every other contour
                points.reverse()
            if abs(_area(points)) < 1:
                continue
            pen.moveTo(points[0])
            for point in points[1:]:
                pen.lineTo(point)
            pen.closePath()
        glyph = pen.glyph()
        name = f"icon{code:04x}"
        glyphs[name] = glyph
        glyph.recalcBounds(None)
        metrics[name] = (em, getattr(glyph, "xMin", 0))
    fb = FontBuilder(em, isTTF=True)
    fb.setupGlyphOrder(names)
    fb.setupCharacterMap({code: f"icon{code:04x}" for code in svgs})
    fb.setupGlyf(glyphs)
    fb.setupHorizontalMetrics(metrics)
    fb.setupHorizontalHeader(ascent=em, descent=0)
    fb.setupNameTable({"familyName": "lume icons", "styleName": "Regular"})
    fb.setupOS2(sTypoAscender=em, sTypoDescender=0, usWinAscent=em, usWinDescent=0)
    fb.setupPost()
    fb.save(out)
    print(f"{out.relative_to(ROOT)}: {out.stat().st_size} bytes (stroke {stroke})")


def main() -> None:
    FONTS.mkdir(parents=True, exist_ok=True)

    for name in [] if "--icons-only" in sys.argv else WEIGHTS:
        data = download(PLUS_JAKARTA_URL.format(path=f"fonts/ttf/PlusJakartaSans-{name}.ttf"))
        subset_font(data, TEXT_UNICODES, FONTS / f"PlusJakartaSans-{name}.ttf", kern=True)
    if "--icons-only" not in sys.argv:
        (FONTS / "OFL.txt").write_bytes(download(PLUS_JAKARTA_URL.format(path="OFL.txt")))

    info = json.loads(download(LUCIDE_URL.format(path="font/info.json")))
    codepoints = {}
    for icon, const in ICONS.items():
        codepoints[const] = int(info[icon]["encodedCode"].lstrip("\\"), 16)
    build_icon_font(
        {
            codepoints[const]: download(LUCIDE_URL.format(path=f"icons/{icon}.svg")).decode()
            for icon, const in ICONS.items()
        },
        LUCIDE_STROKE,
        FONTS / "lucide-subset.ttf",
    )
    (FONTS / "LUCIDE-LICENSE.txt").write_bytes(download(LUCIDE_URL.format(path="LICENSE")))

    lines = [
        "//! Icon code points in `assets/fonts/lucide-subset.ttf`.",
        "//!",
        f"//! @generated by scripts/subset-fonts.py from Lucide {LUCIDE_VERSION}. Do not edit.",
        "",
    ]
    for const, code in codepoints.items():
        lines.append(f"pub const {const}: char = '\\u{{{code:x}}}';")
    (ROOT / "crates/lume-core/src/icons.rs").write_text("\n".join(lines) + "\n")
    print("crates/lume-core/src/icons.rs written")


if __name__ == "__main__":
    main()
