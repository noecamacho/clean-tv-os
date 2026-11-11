"""Generates 4K (3840x2160) wallpapers for Lume TV Launcher with Option A, B, and C.
Full 4K native resolution, high-precision gamma/contrast calibration, and triangular dither.
"""

import os
import sys
import numpy as np
from PIL import Image

WIDTH = 3840
HEIGHT = 2160

PETROL = np.array([15.0, 23.0, 30.0], dtype=np.float32)
BG_DEEP = np.array([6.0, 10.0, 16.0], dtype=np.float32)
ACCENT_BLUE = np.array([77.0, 179.0, 255.0], dtype=np.float32)
CYAN = np.array([70.0, 215.0, 255.0], dtype=np.float32)
INDIGO = np.array([70.0, 90.0, 230.0], dtype=np.float32)
PURPLE = np.array([130.0, 60.0, 220.0], dtype=np.float32)
EMERALD = np.array([30.0, 180.0, 140.0], dtype=np.float32)
SILVER = np.array([180.0, 205.0, 225.0], dtype=np.float32)

def smoothstep(edge0, edge1, x):
    t = np.clip((x - edge0) / (edge1 - edge0), 0.0, 1.0)
    return t * t * (3.0 - 2.0 * t)

def apply_tv_calibration_and_dither(img_rgb, rng_seed=42):
    h, w, _ = img_rgb.shape
    y_coords = np.arange(h, dtype=np.float32).reshape((h, 1, 1))

    # Dock protection (bottom rows y >= 0.72 of height)
    dock_start = h * 0.72
    dock_fade = 1.0 - 0.28 * smoothstep(dock_start, float(h), y_coords)
    img_rgb = img_rgb * dock_fade

    # Top bar protection (top rows y <= 0.125 of height)
    top_end = h * 0.125
    top_fade = 0.72 + 0.28 * smoothstep(0.0, top_end, y_coords)
    img_rgb = img_rgb * top_fade

    # Triangular dither (+- 1 level)
    rng = np.random.default_rng(rng_seed)
    dither = rng.uniform(-0.5, 0.5, size=img_rgb.shape) + rng.uniform(-0.5, 0.5, size=img_rgb.shape)
    img_dithered = np.clip(img_rgb + dither, 0.0, 255.0).astype(np.uint8)
    return img_dithered

# -------------------------------------------------------------
# CORE 4K NATIVE RENDERERS (Aurora & The 2 User Favorites)
# -------------------------------------------------------------
def render_aurora_4k():
    y = np.linspace(0.0, 1.0, HEIGHT, dtype=np.float32).reshape((HEIGHT, 1, 1))
    x = np.linspace(0.0, 1.0, WIDTH, dtype=np.float32).reshape((1, WIDTH, 1))

    sky_v = smoothstep(0.0, 1.0, y)
    rgb = (1.0 - sky_v) * BG_DEEP.reshape((1, 1, 3)) + sky_v * PETROL.reshape((1, 1, 3))
    rgb = np.broadcast_to(rgb, (HEIGHT, WIDTH, 3)).copy()

    layers = [
        {"base": 0.50, "amp": 0.09, "f1": 1.3, "f2": 2.7, "rf1": 22.0, "rf2": 41.0, "h": 0.20, "gain": 1.05, "low": ACCENT_BLUE, "high": INDIGO},
        {"base": 0.40, "amp": 0.08, "f1": 1.7, "f2": 3.3, "rf1": 18.0, "rf2": 37.0, "h": 0.15, "gain": 0.75, "low": CYAN, "high": ACCENT_BLUE},
        {"base": 0.58, "amp": 0.06, "f1": 2.1, "f2": 4.1, "rf1": 31.0, "rf2": 53.0, "h": 0.12, "gain": 0.50, "low": EMERALD, "high": CYAN},
    ]

    top_fade = smoothstep(0.0, 0.14, y)

    for idx, L in enumerate(layers):
        wave = (np.sin(2 * np.pi * L["f1"] * x + 1.2 * idx) * 0.6 + np.sin(2 * np.pi * L["f2"] * x + 2.5 * idx) * 0.4)
        edge_y = (L["base"] + L["amp"] * wave) * HEIGHT

        rays = 0.5 + 0.5 * (np.sin(2 * np.pi * L["rf1"] * x + 0.7 * idx) * 0.6 + np.sin(2 * np.pi * L["rf2"] * x + 1.9 * idx) * 0.4)
        ray_mult = 0.45 + 0.55 * rays

        dy = edge_y - (y * HEIGHT)
        up_mask = dy >= 0.0
        inten = np.zeros((HEIGHT, WIDTH, 1), dtype=np.float32)
        inten[up_mask] = np.exp(-dy[up_mask] / (L["h"] * HEIGHT))
        inten[~up_mask] = np.maximum(0.0, 1.0 + dy[~up_mask] / 24.0)

        total_inten = inten * ray_mult * L["gain"] * top_fade
        t_col = np.clip(dy / (L["h"] * HEIGHT * 2.2), 0.0, 1.0)
        c_layer = (1.0 - t_col) * L["low"].reshape((1, 1, 3)) + t_col * L["high"].reshape((1, 1, 3))
        rgb += c_layer * total_inten

    # 4K crisp stars (840 sharp sub-pixel stars)
    rng_stars = np.random.default_rng(21)
    for _ in range(840):
        sx = rng_stars.integers(0, WIDTH)
        sy = rng_stars.integers(0, int(HEIGHT * 0.7))
        sk = rng_stars.uniform(0.3, 1.0)
        rgb[sy, sx] += ACCENT_BLUE * (sk * 0.7) + CYAN * (sk * 0.3)
        if sk > 0.8:
            if sy + 1 < HEIGHT: rgb[sy + 1, sx] += ACCENT_BLUE * (sk * 0.3)
            if sx + 1 < WIDTH: rgb[sy, sx + 1] += CYAN * (sk * 0.3)

    rgb = 255.0 * (1.0 - np.exp(-rgb / 165.0))
    return apply_tv_calibration_and_dither(rgb, rng_seed=101)

def render_abstract_prism_topo_4k():
    y = np.linspace(0.0, 1.0, HEIGHT, dtype=np.float32).reshape((HEIGHT, 1, 1))
    x = np.linspace(0.0, 1.0, WIDTH, dtype=np.float32).reshape((1, WIDTH, 1))

    f1 = np.sin(3.5 * np.pi * x) * np.cos(2.8 * np.pi * y)
    f2 = np.sin(6.0 * np.pi * (x * 0.8 + y * 0.6)) * 0.5
    f3 = np.cos(9.0 * np.pi * (x * 0.5 - y * 0.8)) * 0.25
    elevation = (f1 + f2 + f3) * 0.5 + 0.5

    stripes = np.sin(elevation * 42.0 * np.pi)
    lines = np.exp(-((stripes - 0.94) / 0.05) ** 2)

    bg = BG_DEEP.reshape((1, 1, 3)) * (0.8 + 0.4 * elevation)
    color_map = (1.0 - elevation) * ACCENT_BLUE.reshape((1, 1, 3)) + elevation * PURPLE.reshape((1, 1, 3))
    rgb = bg + color_map * lines * 1.6 + (CYAN.reshape((1, 1, 3)) * (elevation ** 2) * 0.3)
    rgb = 255.0 * (1.0 - np.exp(-rgb / 155.0))
    return apply_tv_calibration_and_dither(rgb, rng_seed=201)

def render_abstract_silk_wave_4k():
    y = np.linspace(0.0, 1.0, HEIGHT, dtype=np.float32).reshape((HEIGHT, 1, 1))
    x = np.linspace(0.0, 1.0, WIDTH, dtype=np.float32).reshape((1, WIDTH, 1))

    rgb = (BG_DEEP * 0.7).reshape((1, 1, 3)) + (PETROL * 0.3).reshape((1, 1, 3)) * y
    rgb = np.broadcast_to(rgb, (HEIGHT, WIDTH, 3)).copy()

    ribbons = [
        {"y0": 0.48, "freq": 1.6, "amp": 0.16, "thick": 0.035, "col": ACCENT_BLUE},
        {"y0": 0.54, "freq": 2.2, "amp": 0.13, "thick": 0.028, "col": CYAN},
        {"y0": 0.42, "freq": 1.2, "amp": 0.19, "thick": 0.045, "col": INDIGO},
        {"y0": 0.60, "freq": 2.8, "amp": 0.10, "thick": 0.020, "col": PURPLE},
    ]

    for r in ribbons:
        curve = r["y0"] + r["amp"] * np.sin(r["freq"] * np.pi * x + r["thick"] * 10.0)
        dist = np.abs(y - curve)
        glow = np.exp(- (dist / r["thick"]) ** 1.8)
        rgb += r["col"].reshape((1, 1, 3)) * glow * 1.4

    rgb = 255.0 * (1.0 - np.exp(-rgb / 160.0))
    return apply_tv_calibration_and_dither(rgb, rng_seed=202)

# Helper to process real photographic image to 4K calibrated
def process_photo_to_4k(im_path, seed=501):
    im = Image.open(im_path).convert("RGB")
    # Lanczos high-fidelity resize/crop to 3840x2160
    im_ratio = im.width / im.height
    target_ratio = WIDTH / HEIGHT
    if im_ratio > target_ratio:
        # crop horizontally
        new_w = int(im.height * target_ratio)
        offset = (im.width - new_w) // 2
        im = im.crop((offset, 0, offset + new_w, im.height))
    else:
        # crop vertically
        new_h = int(im.width / target_ratio)
        offset = (im.height - new_h) // 2
        im = im.crop((0, offset, im.width, offset + new_h))

    im = im.resize((WIDTH, HEIGHT), Image.Resampling.LANCZOS)
    arr = np.array(im, dtype=np.float32)

    # Tone mapping & grading towards project petrol/dark aesthetic
    arr = arr * 0.85
    return apply_tv_calibration_and_dither(arr, rng_seed=seed)

# -------------------------------------------------------------
# OPTION C: ADVANCED VOLUMETRIC / PHOTOREALISTIC PROCEDURAL
# -------------------------------------------------------------
def render_proc_photoreal_nature_lake_4k():
    y = np.linspace(0.0, 1.0, HEIGHT, dtype=np.float32).reshape((HEIGHT, 1, 1))
    x = np.linspace(0.0, 1.0, WIDTH, dtype=np.float32).reshape((1, WIDTH, 1))

    horizon = 0.50
    sky_mask = y < horizon

    # Night atmospheric Rayleigh sky
    sky = BG_DEEP.reshape((1, 1, 3)) + (PETROL * 0.75).reshape((1, 1, 3)) * (1.0 - y / horizon)
    sky = np.broadcast_to(sky, (HEIGHT, WIDTH, 3)).copy()
    # Milky way / starry band
    milky_band = np.exp(- ((x - 0.55 - (y * 0.3)) / 0.15) ** 2)
    sky += (CYAN * 0.4 + SILVER * 0.6).reshape((1, 1, 3)) * milky_band * 0.5

    # Mirror lake with Fresnel reflection
    dy = np.maximum(y - horizon, 0.001)
    water_v = dy / (1.0 - horizon)
    fresnel = 0.04 + 0.96 * (1.0 - water_v) ** 5

    # Perspective ripples
    ripple = np.sin(40.0 / (water_v + 0.05) + np.sin(x * 25.0) * 1.2) * 0.015
    refl_y = np.clip(horizon - dy * 0.85 + ripple, 0.0, horizon)
    refl_sky = BG_DEEP.reshape((1, 1, 3)) + (PETROL * 0.75).reshape((1, 1, 3)) * (1.0 - refl_y / horizon)
    refl_sky = np.broadcast_to(refl_sky, (HEIGHT, WIDTH, 3)).copy()
    refl_sky += (CYAN * 0.4 + SILVER * 0.6).reshape((1, 1, 3)) * milky_band * 0.4

    water = (BG_DEEP * 0.5).reshape((1, 1, 3)) * (1.0 - fresnel) + refl_sky * fresnel
    rgb = np.where(sky_mask, sky, water)

    # Distant mountain silhouettes at the horizon
    m1 = 0.50 - 0.08 * np.abs(np.sin(x * 5.0 * np.pi)) - 0.04 * np.sin(x * 14.0 * np.pi)
    mountain_mask = (y >= m1) & (y <= horizon)
    rgb = np.where(mountain_mask, (BG_DEEP * 0.4).reshape((1, 1, 3)), rgb)

    rgb = 255.0 * (1.0 - np.exp(-rgb / 155.0))
    return apply_tv_calibration_and_dither(rgb, rng_seed=601)

def render_proc_photoreal_nature_alpine_4k():
    y = np.linspace(0.0, 1.0, HEIGHT, dtype=np.float32).reshape((HEIGHT, 1, 1))
    x = np.linspace(0.0, 1.0, WIDTH, dtype=np.float32).reshape((1, WIDTH, 1))

    # Alpine night under moon/aurora glow
    sky = (BG_DEEP * 0.8).reshape((1, 1, 3)) + (PETROL * 0.9).reshape((1, 1, 3)) * (1.0 - y * 0.8)
    rgb = np.broadcast_to(sky, (HEIGHT, WIDTH, 3)).copy()

    # Rugged crag mountain slopes
    crag1 = 0.44 + 0.12 * np.sin(2.5 * np.pi * x) + 0.05 * np.cos(7.2 * np.pi * x) + 0.02 * np.sin(19.0 * np.pi * x)
    mask1 = y >= crag1
    slope = np.clip((y - crag1) / 0.35, 0.0, 1.0)
    rock = (PETROL * 0.5 + BG_DEEP * 0.5).reshape((1, 1, 3)) * (1.0 - slope * 0.5)
    # Snow highlights on crest
    snow = np.exp(- ((y - crag1) / 0.012) ** 2) * (np.sin(5.0 * np.pi * x) * 0.5 + 0.5)

    rgb = np.where(mask1, rock, rgb)
    rgb += (CYAN * 0.5 + SILVER * 0.5).reshape((1, 1, 3)) * snow * 0.85

    rgb = 255.0 * (1.0 - np.exp(-rgb / 150.0))
    return apply_tv_calibration_and_dither(rgb, rng_seed=602)

def render_proc_photoreal_modern_cityscape_4k():
    y = np.linspace(0.0, 1.0, HEIGHT, dtype=np.float32).reshape((HEIGHT, 1, 1))
    x = np.linspace(0.0, 1.0, WIDTH, dtype=np.float32).reshape((1, WIDTH, 1))

    sky = BG_DEEP.reshape((1, 1, 3)) + (PETROL * 0.6).reshape((1, 1, 3)) * (1.0 - y * 0.7)
    rgb = np.broadcast_to(sky, (HEIGHT, WIDTH, 3)).copy()

    # Skyscraper building columns
    num_towers = 24
    rng_towers = np.random.default_rng(701)
    for _ in range(num_towers):
        tx = rng_towers.uniform(0.05, 0.95)
        tw = rng_towers.uniform(0.025, 0.065)
        th = rng_towers.uniform(0.35, 0.68)
        tower_mask = (x >= (tx - tw/2)) & (x <= (tx + tw/2)) & (y >= (1.0 - th))
        # Window matrix lit up
        wx = np.abs(np.sin((x - tx) * 450.0)) < 0.35
        wy = np.abs(np.sin((y - (1.0 - th)) * 300.0)) < 0.45
        windows = (wx & wy & tower_mask & (rng_towers.uniform(0, 1, size=(HEIGHT, WIDTH, 1)) > 0.45))
        
        tower_col = (PETROL * 0.4 + BG_DEEP * 0.6).reshape((1, 1, 3))
        rgb = np.where(tower_mask, tower_col, rgb)
        rgb += CYAN.reshape((1, 1, 3)) * windows * 0.8

    rgb = 255.0 * (1.0 - np.exp(-rgb / 155.0))
    return apply_tv_calibration_and_dither(rgb, rng_seed=603)

def render_proc_photoreal_modern_monolith_4k():
    y = np.linspace(-1.0, 1.0, HEIGHT, dtype=np.float32).reshape((HEIGHT, 1, 1))
    x = np.linspace(-1.777, 1.777, WIDTH, dtype=np.float32).reshape((1, WIDTH, 1))

    # Architectural angled glass facade
    p1 = np.maximum(np.abs(x + 0.3) * 0.85 + (y - 0.05) * 0.65, 0.0)
    bevel = np.exp(- ((p1 - 0.8) / 0.025) ** 2)

    ambient = BG_DEEP.reshape((1, 1, 3)) + (PETROL * 0.4).reshape((1, 1, 3)) * (1.0 - np.abs(y))
    rgb = ambient + ACCENT_BLUE.reshape((1, 1, 3)) * bevel * 1.5

    rgb = 255.0 * (1.0 - np.exp(-rgb / 150.0))
    return apply_tv_calibration_and_dither(rgb, rng_seed=604)

print("Module loaded.")
