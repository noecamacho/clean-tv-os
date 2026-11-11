#!/usr/bin/env python3
"""Measures where the ink is in a screenshot, to check spacing and alignment against the design
system ruler (openspec/specs/design-system, "Régua de layout").

Standard library only. Coordinates are pixels of the screenshot; take it at 1280x720 so they
are logical units (lume-desktop --size 1280x720 --screenshot <screen> <file.png>).

  # horizontal runs of ink in a band (gaps between items of a row)
  measure-ink.py shot.png columns --y 28 60
  # vertical extent of the ink in some x ranges (heights and centers)
  measure-ink.py shot.png rows --x logo=66:102 gear=1058:1110 --y 0 88

A pixel is ink when R+G+B is above --threshold; a column (or row) counts when it has at least
--min pixels of ink, which filters out the stars of the wallpaper.
"""

import argparse
import struct
import zlib


def load(path):
    data = open(path, "rb").read()
    if data[:8] != b"\x89PNG\r\n\x1a\n":
        raise SystemExit(f"{path}: not a PNG")
    i, idat = 8, b""
    while i < len(data):
        length, kind = struct.unpack(">I4s", data[i : i + 8])
        chunk = data[i + 8 : i + 8 + length]
        if kind == b"IHDR":
            width, height, depth, color = struct.unpack(">IIBB", chunk[:10])
            if depth != 8 or color not in (2, 6):
                raise SystemExit("only 8-bit RGB or RGBA PNGs are supported")
        elif kind == b"IDAT":
            idat += chunk
        i += 12 + length
    bpp = 3 if color == 2 else 4
    raw = zlib.decompress(idat)
    stride = width * bpp
    rows, prev, p = [], bytearray(stride), 0
    for _ in range(height):
        kind, line = raw[p], bytearray(raw[p + 1 : p + 1 + stride])
        p += 1 + stride
        for x in range(stride):
            a = line[x - bpp] if x >= bpp else 0
            b = prev[x]
            c = prev[x - bpp] if x >= bpp else 0
            if kind == 1:
                line[x] = (line[x] + a) & 255
            elif kind == 2:
                line[x] = (line[x] + b) & 255
            elif kind == 3:
                line[x] = (line[x] + (a + b) // 2) & 255
            elif kind == 4:
                pa, pb, pc = abs(b - c), abs(a - c), abs(a + b - 2 * c)
                line[x] = (line[x] + (a if pa <= pb and pa <= pc else b if pb <= pc else c)) & 255
        rows.append(line)
        prev = line
    return width, height, bpp, rows


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("png")
    ap.add_argument("mode", choices=["columns", "rows"])
    ap.add_argument("--y", nargs=2, type=int, default=[0, 88], metavar=("TOP", "BOTTOM"))
    ap.add_argument("--x", nargs="*", default=[], metavar="NAME=LEFT:RIGHT")
    ap.add_argument("--threshold", type=int, default=300)
    ap.add_argument("--min", type=int, default=5)
    ap.add_argument("--merge", type=int, default=12, help="join runs closer than this (letters of a word)")
    args = ap.parse_args()

    width, height, bpp, rows = load(args.png)
    y0, y1 = args.y[0], min(args.y[1], height)

    def ink(x, y):
        return sum(rows[y][x * bpp : x * bpp + 3]) > args.threshold

    if args.mode == "columns":
        runs = []
        for x in range(width):
            if sum(1 for y in range(y0, y1) if ink(x, y)) < args.min:
                continue
            if runs and x - runs[-1][1] <= args.merge:
                runs[-1][1] = x
            else:
                runs.append([x, x])
        runs = [r for r in runs if r[1] - r[0] >= 3]
        previous = None
        for left, right in runs:
            gap = "" if previous is None else f"  gap {left - previous - 1}"
            print(f"x {left}..{right}  width {right - left + 1}{gap}")
            previous = right
    else:
        for spec in args.x:
            name, span = spec.split("=")
            x0, x1 = (int(v) for v in span.split(":"))
            ys = [y for y in range(y0, y1) if sum(1 for x in range(x0, x1) if ink(x, y)) >= args.min // 2 + 1]
            if ys:
                print(f"{name}: y {ys[0]}..{ys[-1]}  height {ys[-1] - ys[0] + 1}  center {(ys[0] + ys[-1]) / 2}")
            else:
                print(f"{name}: no ink")


if __name__ == "__main__":
    main()
