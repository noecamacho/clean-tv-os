#!/usr/bin/env python3
"""Generates the tiny synthetic fonts used by the font fallback tests.

Each font draws every covered character as a solid box, so tests can tell which font
supplied a glyph. Requirements: python3 and `pip install fonttools`.
Usage: scripts/make-test-fonts.py
"""

import pathlib

from fontTools.fontBuilder import FontBuilder
from fontTools.pens.ttGlyphPen import TTGlyphPen

OUT = pathlib.Path(__file__).resolve().parent.parent / "crates/lume-core/tests/fonts"

# file name -> (family, covered characters, box height in font units)
FONTS = {
    "test-cyrillic.ttf": ("LumeTestCyrillic", "ЯБВГДабвгд", 600),
    "test-cjk.ttf": ("LumeTestCJK", "日本語中文한국어", 800),
}


def box(height: int):
    pen = TTGlyphPen(None)
    pen.moveTo((50, 0))
    pen.lineTo((50, height))
    pen.lineTo((550, height))
    pen.lineTo((550, 0))
    pen.closePath()
    return pen.glyph()


def build(path: pathlib.Path, family: str, characters: str, height: int) -> None:
    glyph_names = [".notdef", "space"] + [f"uni{ord(c):04X}" for c in characters]
    builder = FontBuilder(1000, isTTF=True)
    builder.setupGlyphOrder(glyph_names)
    builder.setupCharacterMap({0x20: "space", **{ord(c): f"uni{ord(c):04X}" for c in characters}})
    glyphs = {".notdef": box(height), "space": TTGlyphPen(None).glyph()}
    glyphs.update({f"uni{ord(c):04X}": box(height) for c in characters})
    builder.setupGlyf(glyphs)
    builder.setupHorizontalMetrics({name: (600, 50) for name in glyph_names})
    builder.setupHorizontalHeader(ascent=900, descent=-200)
    builder.setupNameTable({"familyName": family, "styleName": "Regular"})
    builder.setupOS2(sTypoAscender=900, sTypoDescender=-200, usWinAscent=900, usWinDescent=200)
    builder.setupPost()
    builder.save(path)
    print(f"{path.name}: {path.stat().st_size} bytes")


if __name__ == "__main__":
    OUT.mkdir(parents=True, exist_ok=True)
    for name, (family, characters, height) in FONTS.items():
        build(OUT / name, family, characters, height)
