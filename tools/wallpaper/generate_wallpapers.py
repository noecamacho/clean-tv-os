"""Generates the Lume wallpapers (1920x1080) from the masters in tools/wallpaper/masters/.

Run from the repository root:  uv run --with pillow --with numpy tools/wallpaper/generate_wallpapers.py [--calibrate]
(by default the masters are only reduced, as the author chose; --calibrate darkens the top and the dock
so the UI text keeps 4.5:1 of contrast, which the contrast tests need)

Each master is a lossless 4:4:4 image of at least 1920x1080 (the catalog uses 3840x2160). The pipeline
keeps floating point until the very last step:

1. contrast calibration at the master's size (top bar and dock need 4.5:1 against their text);
2. reduction to 1920x1080 by averaging (area filter; 3840x2160 is exactly 2:1);
3. triangular dither of +-1 level and quantization to 8 bits, seeded by the wallpaper id;
4. lossless QOI encoding (decodes about 2x faster than deflate or PNG; see docs/wallpaper.md).

A master smaller than 1920x1080, or one that is a smaller image enlarged (a "fake Full HD"), is refused.
Also writes the 128x72 thumbnail atlas for the home screen editor and an empty star list per wallpaper
(the stars are part of the masters).
"""
import os
import sys
import zlib

import numpy as np
from PIL import Image

WIDTH, HEIGHT = 1920, 1080

# Catalog order (the order of `WallpaperId::ALL`). The default is the first.
CATALOG = ["cinematic", "beach_sunset", "beach_dawn", "aurora", "mountain_stars", "mirror_lake", "misty_forest", "glass_facade", "synthwave_grid"]

# Fake Full HD detector: the mean absolute luminance error (0-255) between the 1920x1080 image and the
# same image reduced to 1280x720 and enlarged back. An enlarged 720p image has no detail above that
# scale, so it comes back almost unchanged. Calibrated in docs/wallpaper.md.
MIN_DETAIL = 0.19

# The top bar's muted text needs 4.5:1, which caps the brightest pixel behind it at about this
# relative luminance (with a little margin).
TOP_MAX_LUMINANCE = 0.050

# Targets under the limits checked below (top < 40, bottom < 25), with margin.
TOP_TARGET, BOTTOM_TARGET = 34.0, 21.0


def smooth(a, b, x):
    t = np.clip((x - a) / (b - a), 0.0, 1.0)
    return t * t * (3 - 2 * t)


def luma(px):
    return 0.2126 * px[..., 0] + 0.7152 * px[..., 1] + 0.0722 * px[..., 2]


def relative_luminance(px):
    c = px / 255.0
    lin = np.where(c <= 0.04045, c / 12.92, ((c + 0.055) / 1.055) ** 2.4)
    return 0.2126 * lin[..., 0] + 0.7152 * lin[..., 1] + 0.0722 * lin[..., 2]


def band_luma(px, y0, y1):
    """Average luminance of the rows y0..y1 given in a 360-row scale."""
    k = px.shape[0] / 360
    return float(luma(px[int(y0 * k):int(y1 * k)]).mean())


def detail(px):
    """The fake Full HD measure of a 1920x1080 float image (see MIN_DETAIL)."""
    y = Image.fromarray(np.clip(luma(px), 0, 255).round().astype(np.uint8))
    back = y.resize((1280, 720), Image.Resampling.LANCZOS).resize((WIDTH, HEIGHT), Image.Resampling.LANCZOS)
    return float(np.abs(np.asarray(y, dtype=np.float32) - np.asarray(back, dtype=np.float32)).mean())


def reduce_area(px):
    """Reduces to 1920x1080 by averaging whole blocks (the master is an integer multiple)."""
    h, w = px.shape[:2]
    if (w, h) == (WIDTH, HEIGHT):
        return px
    if w % WIDTH or h % HEIGHT or w // WIDTH != h // HEIGHT:
        raise SystemExit(f"master {w}x{h} is not an integer multiple of {WIDTH}x{HEIGHT} with 16:9 proportions")
    f = w // WIDTH
    return px.reshape(HEIGHT, f, WIDTH, f, 3).mean(axis=(1, 3))


def calibrate(px):
    """Darkens the top rows and the dock area at the master's size so the UI text keeps its contrast."""
    h = px.shape[0]
    k = h / 360
    rows = np.arange(h, dtype=np.float32)
    top_l = band_luma(px, 0, 30)
    bot_l = band_luma(px, 300, 360)

    fade = np.ones(h, dtype=np.float32)
    # Gentle dock protection in the bottom rows (260..360).
    factor = 0.78 if bot_l >= 18.0 else 0.88
    fade *= 1.0 - (1.0 - factor) * smooth(0.0, 1.0, (rows - 260 * k) / (100.0 * k))
    # Subtle top protection if the top is bright.
    if top_l > 35.0:
        top = rows < 35 * k
        fade[top] *= 0.75 + 0.25 * smooth(0.0, 1.0, rows[top] / (35.0 * k))
    # Cap the brightest pixel behind the top bar: the average is not enough, a single bright pixel
    # (a star) fails. Each row gets the factor its own peak needs, and the factor curve is made smooth
    # without ever exceeding it (a minimum filter, then a box blur of the same width).
    band = 90 * h // 720
    w = 8 * h // 720
    rows_peak = relative_luminance(px[:band] * fade[:band, None, None]).max(axis=1)
    needed = np.minimum(1.0, (TOP_MAX_LUMINANCE / np.maximum(rows_peak, 1e-9)) ** (1 / 2.2))
    needed = np.concatenate([needed, np.ones(2 * w, dtype=np.float32)])
    padded = np.pad(needed, (w, w), mode="edge")
    low = np.lib.stride_tricks.sliding_window_view(padded, 2 * w + 1).min(axis=1)
    padded = np.pad(low, (w, w), mode="edge")
    smooth_needed = np.lib.stride_tricks.sliding_window_view(padded, 2 * w + 1).mean(axis=1)
    fade[:band + 2 * w] *= smooth_needed
    # Bring the band averages under the targets (a bright sky or a bright ground), fading out over
    # the rows next to the band so no edge shows.
    for y0, y1, reach, limit, down in ((0, 30, 35, TOP_TARGET, True), (300, 360, 340, BOTTOM_TARGET, False)):
        for _ in range(12):
            avg = float(luma(px[int(y0 * k):int(y1 * k)] * fade[int(y0 * k):int(y1 * k), None, None]).mean())
            if avg <= limit:
                break
            s = limit / avg
            if down:
                zone = rows < reach * k
                fade[zone] *= s + (1.0 - s) * smooth(0.0, 1.0, rows[zone] / (reach * k))
            else:
                zone = rows >= reach * k - 40 * k
                fade[zone] *= 1.0 - (1.0 - s) * smooth(0.0, 1.0, (rows[zone] - (reach * k - 40 * k)) / (40 * k))
    return px * fade[:, None, None]


def quantize(px, seed):
    """Triangular dither of +-1 level, then 8 bits."""
    rng = np.random.default_rng(zlib.crc32(seed.encode()))
    noise = rng.random(px.shape, dtype=np.float32) - rng.random(px.shape, dtype=np.float32)
    return np.clip(np.rint(px + noise), 0, 255).astype(np.uint8)


def load_master(path):
    im = Image.open(path)
    if im.mode not in ("RGB", "RGB;16", "I;16"):
        im = im.convert("RGB")
    w, h = im.size
    if w < WIDTH or h < HEIGHT:
        raise SystemExit(f"{path}: master {w}x{h} is smaller than {WIDTH}x{HEIGHT}")
    return np.asarray(im.convert("RGB"), dtype=np.float32)


def process(name, masters_dir, raw=False):
    if name in ("cinematic", "beach_sunset", "beach_dawn"):
        filenames = {
            "cinematic": "beach-cove-source.png",
            "beach_sunset": "beach-sunset-source.png",
            "beach_dawn": "beach-dawn-source.png",
        }
        source = os.path.join(os.path.dirname(masters_dir), "generated", filenames[name])
        image = Image.open(source).convert("RGB").resize((WIDTH, HEIGHT), Image.Resampling.LANCZOS)
        return np.asarray(image)
    px = load_master(os.path.join(masters_dir, f"{name}.png"))
    reduced = reduce_area(px)
    score = detail(reduced)
    if score < MIN_DETAIL:
        raise SystemExit(f"{name}: fake Full HD (detail {score:.3f} < {MIN_DETAIL})")
    # `raw` (the default): the master as it is, without the top and dock darkening.
    out = quantize(reduce_area(px if raw else calibrate(px)), name)

    top, bottom, mid = band_luma(out.astype(np.float32), 0, 30), band_luma(out.astype(np.float32), 300, 360), band_luma(out.astype(np.float32), 120, 240)
    if not raw:
        assert top < 40.0, f"{name}: top luma {top} >= 40"
        assert bottom < 25.0, f"{name}: bottom luma {bottom} >= 25"
        assert mid > bottom, f"{name}: mid luma {mid} <= bottom {bottom}"
    print(f"{name}: detail {score:.3f}, luma top {top:.1f} mid {mid:.1f} bottom {bottom:.1f}")
    return out


def qoi_encode(rgb, width, height):
    """Encodes 8-bit RGB as QOI (lossless; https://qoiformat.org). Deterministic."""
    data = rgb.tobytes()
    out = bytearray(b"qoif" + width.to_bytes(4, "big") + height.to_bytes(4, "big") + bytes([3, 0]))
    index = [(0, 0, 0, 0)] * 64
    pr, pg, pb = 0, 0, 0
    run = 0
    last = len(data) - 3
    for i in range(0, len(data), 3):
        r, g, b = data[i], data[i + 1], data[i + 2]
        if r == pr and g == pg and b == pb:
            run += 1
            if run == 62 or i == last:
                out.append(0xC0 | (run - 1))
                run = 0
            continue
        if run:
            out.append(0xC0 | (run - 1))
            run = 0
        h = (r * 3 + g * 5 + b * 7 + 255 * 11) % 64
        if index[h] == (r, g, b, 255):
            out.append(h)
        else:
            index[h] = (r, g, b, 255)
            dr, dg, db = (r - pr + 128) & 255, (g - pg + 128) & 255, (b - pb + 128) & 255
            dr, dg, db = dr - 128, dg - 128, db - 128
            drg, dbg = dr - dg, db - dg
            if -2 <= dr <= 1 and -2 <= dg <= 1 and -2 <= db <= 1:
                out.append(0x40 | (dr + 2) << 4 | (dg + 2) << 2 | (db + 2))
            elif -32 <= dg <= 31 and -8 <= drg <= 7 and -8 <= dbg <= 7:
                out.append(0x80 | (dg + 32))
                out.append((drg + 8) << 4 | (dbg + 8))
            else:
                out += bytes([0xFE, r, g, b])
        pr, pg, pb = r, g, b
    out += b"\x00" * 7 + b"\x01"
    return bytes(out)


def build_thumbnails(images, assets_dir):
    atlas = Image.new("RGBA", (640, 144), (0, 0, 0, 0))
    for i, name in enumerate(CATALOG):
        thumb = Image.fromarray(images[name]).convert("RGBA").resize((128, 72), Image.Resampling.LANCZOS)
        atlas.paste(thumb, ((i % 5) * 128, (i // 5) * 72))
    with open(os.path.join(assets_dir, "thumbnails-640x144.rgba"), "wb") as f:
        f.write(atlas.tobytes())


def main():
    root = os.path.dirname(os.path.abspath(__file__))
    raw = "--calibrate" not in sys.argv
    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    assets_dir = args[0] if args else os.path.join(root, "../../crates/lume-core/assets/wallpaper")
    masters_dir = os.path.join(root, "masters")
    os.makedirs(assets_dir, exist_ok=True)

    images = {}
    for name in CATALOG:
        images[name] = process(name, masters_dir, raw)
        with open(os.path.join(assets_dir, f"{name}-{WIDTH}x{HEIGHT}.qoi"), "wb") as f:
            f.write(qoi_encode(images[name], WIDTH, HEIGHT))
        with open(os.path.join(assets_dir, f"{name}-stars.txt"), "w") as f:
            f.write(f"# 0 stars for {name}: they are part of the master\n")
    build_thumbnails(images, assets_dir)
    print(f"Generated {len(CATALOG)} wallpapers and the thumbnail atlas into {assets_dir}")


if __name__ == "__main__":
    main()
