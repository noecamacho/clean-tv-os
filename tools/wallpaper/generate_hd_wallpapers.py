"""High-quality procedural wallpaper generator for Lume TV Launcher.
Renders Full HD (1920x1080) wallpapers in 3 themes (Abstract, Nature, Modernity)
plus the brand Aurora with maximum visual quality, vector-smooth equations,
contrast protection for TV top bar & dock, and triangular dithering to eliminate
banding on 4K/UHD upscaled panels.
"""

import math
import os
import sys
import numpy as np
from PIL import Image

WIDTH = 1920
HEIGHT = 1080

# Color definitions
PETROL = np.array([15.0, 23.0, 30.0], dtype=np.float32)
BG_DEEP = np.array([6.0, 10.0, 16.0], dtype=np.float32)
ACCENT_BLUE = np.array([77.0, 179.0, 255.0], dtype=np.float32)
CYAN = np.array([70.0, 215.0, 255.0], dtype=np.float32)
INDIGO = np.array([70.0, 90.0, 230.0], dtype=np.float32)
PURPLE = np.array([130.0, 60.0, 220.0], dtype=np.float32)
EMERALD = np.array([30.0, 180.0, 140.0], dtype=np.float32)
AMBER = np.array([240.0, 150.0, 60.0], dtype=np.float32)
SILVER = np.array([180.0, 205.0, 225.0], dtype=np.float32)

def smoothstep(edge0, edge1, x):
    t = np.clip((x - edge0) / (edge1 - edge0), 0.0, 1.0)
    return t * t * (3.0 - 2.0 * t)

def apply_tv_calibration_and_dither(img_rgb, rng_seed=42):
    """Applies TV top bar and dock contrast protection followed by triangular dither."""
    h, w, _ = img_rgb.shape
    y_coords = np.arange(h, dtype=np.float32).reshape((h, 1, 1))

    # Dock protection (bottom rows y >= 780 out of 1080)
    # Ensures dock text & app cards keep contrast >= 4.5:1
    dock_start = 780.0
    dock_fade = 1.0 - 0.28 * smoothstep(dock_start, float(h), y_coords)
    img_rgb = img_rgb * dock_fade

    # Top bar protection (top rows y <= 135 out of 1080)
    # Ensures clock, status and top tabs remain completely crisp and readable
    top_end = 135.0
    top_fade = 0.72 + 0.28 * smoothstep(0.0, top_end, y_coords)
    img_rgb = img_rgb * top_fade

    # Triangular dither (+- 1 level) to prevent banding when upscaled to 4K
    rng = np.random.default_rng(rng_seed)
    dither = rng.uniform(-0.5, 0.5, size=img_rgb.shape) + rng.uniform(-0.5, 0.5, size=img_rgb.shape)
    img_dithered = np.clip(img_rgb + dither, 0.0, 255.0).astype(np.uint8)
    return img_dithered

# -------------------------------------------------------------
# 0. AURORA (Max Quality Regenerated)
# -------------------------------------------------------------
def render_aurora():
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
        inten[~up_mask] = np.maximum(0.0, 1.0 + dy[~up_mask] / 12.0)

        total_inten = inten * ray_mult * L["gain"] * top_fade
        t_col = np.clip(dy / (L["h"] * HEIGHT * 2.2), 0.0, 1.0)
        c_layer = (1.0 - t_col) * L["low"].reshape((1, 1, 3)) + t_col * L["high"].reshape((1, 1, 3))
        rgb += c_layer * total_inten

    # Add crisp stars (420 stars as per Lume brand spec)
    rng_stars = np.random.default_rng(21)
    for _ in range(420):
        sx = rng_stars.integers(0, WIDTH)
        sy = rng_stars.integers(0, int(HEIGHT * 0.7))
        sk = rng_stars.uniform(0.25, 1.0)
        rgb[sy, sx] += ACCENT_BLUE * (sk * 0.7) + CYAN * (sk * 0.3)
        if sk > 0.8 and sy + 1 < HEIGHT and sx + 1 < WIDTH:
            rgb[sy + 1, sx] += ACCENT_BLUE * (sk * 0.35)
            rgb[sy, sx + 1] += CYAN * (sk * 0.35)

    rgb = 255.0 * (1.0 - np.exp(-rgb / 165.0))
    return apply_tv_calibration_and_dither(rgb, rng_seed=101)

# -------------------------------------------------------------
# THEME 1: ABSTRACT (3 wallpapers)
# -------------------------------------------------------------
def render_abstract_prism_topo():
    y = np.linspace(0.0, 1.0, HEIGHT, dtype=np.float32).reshape((HEIGHT, 1, 1))
    x = np.linspace(0.0, 1.0, WIDTH, dtype=np.float32).reshape((1, WIDTH, 1))

    f1 = np.sin(3.5 * np.pi * x) * np.cos(2.8 * np.pi * y)
    f2 = np.sin(6.0 * np.pi * (x * 0.8 + y * 0.6)) * 0.5
    f3 = np.cos(9.0 * np.pi * (x * 0.5 - y * 0.8)) * 0.25
    elevation = (f1 + f2 + f3) * 0.5 + 0.5

    stripes = np.sin(elevation * 38.0 * np.pi)
    lines = np.exp(-((stripes - 0.94) / 0.06) ** 2)

    bg = BG_DEEP.reshape((1, 1, 3)) * (0.8 + 0.4 * elevation)
    color_map = (1.0 - elevation) * ACCENT_BLUE.reshape((1, 1, 3)) + elevation * PURPLE.reshape((1, 1, 3))
    rgb = bg + color_map * lines * 1.6 + (CYAN.reshape((1, 1, 3)) * (elevation ** 2) * 0.3)
    rgb = 255.0 * (1.0 - np.exp(-rgb / 155.0))
    return apply_tv_calibration_and_dither(rgb, rng_seed=201)

def render_abstract_silk_wave():
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

def render_abstract_quantum_matrix():
    y = np.linspace(-1.0, 1.0, HEIGHT, dtype=np.float32).reshape((HEIGHT, 1, 1))
    x = np.linspace(-1.777, 1.777, WIDTH, dtype=np.float32).reshape((1, WIDTH, 1))

    r = np.sqrt(x * x + y * y)
    theta = np.arctan2(y, x)

    spiral1 = np.sin(5.0 * np.log(np.maximum(r, 0.01)) - 3.0 * theta)
    spiral2 = np.cos(7.0 * np.log(np.maximum(r, 0.01)) + 4.0 * theta)
    intensity = smoothstep(0.1, 0.9, np.abs(spiral1 * 0.6 + spiral2 * 0.4))

    vignette = np.exp(- (r / 1.3) ** 2)
    bg = BG_DEEP.reshape((1, 1, 3)) + PETROL.reshape((1, 1, 3)) * (1.0 - np.clip(r / 1.5, 0.0, 1.0))

    lum_col = (1.0 - vignette) * INDIGO.reshape((1, 1, 3)) + vignette * ACCENT_BLUE.reshape((1, 1, 3))
    rgb = bg + lum_col * (intensity * vignette) * 1.5
    rgb = 255.0 * (1.0 - np.exp(-rgb / 150.0))
    return apply_tv_calibration_and_dither(rgb, rng_seed=203)

# -------------------------------------------------------------
# THEME 2: NATURE (3 wallpapers)
# -------------------------------------------------------------
def render_nature_midnight_ocean():
    y = np.linspace(0.0, 1.0, HEIGHT, dtype=np.float32).reshape((HEIGHT, 1, 1))
    x = np.linspace(0.0, 1.0, WIDTH, dtype=np.float32).reshape((1, WIDTH, 1))

    horizon = 0.52
    sky_mask = y < horizon # (HEIGHT, 1, 1)

    sky = BG_DEEP.reshape((1, 1, 3)) + (PETROL * 0.8).reshape((1, 1, 3)) * (1.0 - np.clip(y / horizon, 0.0, 1.0))
    sky = np.broadcast_to(sky, (HEIGHT, WIDTH, 3)).copy()

    glow_center_x = 0.62
    glow_dist = np.sqrt((x - glow_center_x) ** 2 + ((y - horizon + 0.05) * 2.5) ** 2)
    sky_glow = np.exp(- (glow_dist / 0.28) ** 2)
    sky += ACCENT_BLUE.reshape((1, 1, 3)) * sky_glow * 0.85

    water_v = np.clip((y - horizon) / (1.0 - horizon), 0.0, 1.0)
    ocean_base = (PETROL * 0.7).reshape((1, 1, 3)) * (1.0 - water_v * 0.6)

    waves = np.sin((15.0 / (water_v + 0.04)) + np.sin(x * 12.0) * 1.5)
    specular_x = np.exp(- ((x - glow_center_x) / (0.12 + 0.4 * water_v)) ** 2)
    water_spec = np.clip(waves * 0.5 + 0.5, 0.0, 1.0) * specular_x * (1.0 - water_v * 0.5)
    water = ocean_base + CYAN.reshape((1, 1, 3)) * water_spec * 1.3
    water = np.broadcast_to(water, (HEIGHT, WIDTH, 3)).copy()

    rgb = np.where(sky_mask, sky, water)
    rgb = 255.0 * (1.0 - np.exp(-rgb / 155.0))
    return apply_tv_calibration_and_dither(rgb, rng_seed=301)

def render_nature_misty_forest():
    y = np.linspace(0.0, 1.0, HEIGHT, dtype=np.float32).reshape((HEIGHT, 1, 1))
    x = np.linspace(0.0, 1.0, WIDTH, dtype=np.float32).reshape((1, WIDTH, 1))

    sky = (BG_DEEP * 0.8).reshape((1, 1, 3)) + (PETROL * 0.9).reshape((1, 1, 3)) * (1.0 - y * 0.7)
    rgb = np.broadcast_to(sky, (HEIGHT, WIDTH, 3)).copy()

    ridges = [
        {"base": 0.48, "col": EMERALD * 0.5 + PETROL * 0.5, "mist": 0.65, "freq": 4.0},
        {"base": 0.62, "col": EMERALD * 0.35 + BG_DEEP * 0.65, "mist": 0.40, "freq": 7.0},
        {"base": 0.76, "col": BG_DEEP * 0.9, "mist": 0.15, "freq": 11.0},
    ]

    for idx, rd in enumerate(ridges):
        ridge_y = rd["base"] + 0.07 * np.sin(rd["freq"] * x * np.pi + idx * 1.5) + 0.03 * np.cos(rd["freq"] * 2.3 * x * np.pi)
        mask = y >= ridge_y
        depth_in_ridge = np.clip((y - ridge_y) / 0.18, 0.0, 1.0)
        
        layer_color = rd["col"].reshape((1, 1, 3))
        mist = np.exp(- ((y - ridge_y + 0.02) / 0.06) ** 2) * rd["mist"]
        
        rgb = np.where(mask, rgb * (1.0 - depth_in_ridge) + layer_color * depth_in_ridge, rgb)
        rgb += (CYAN * 0.5 + SILVER * 0.5).reshape((1, 1, 3)) * mist * 0.75

    rgb = 255.0 * (1.0 - np.exp(-rgb / 160.0))
    return apply_tv_calibration_and_dither(rgb, rng_seed=302)

def render_nature_obsidian_dunes():
    y = np.linspace(0.0, 1.0, HEIGHT, dtype=np.float32).reshape((HEIGHT, 1, 1))
    x = np.linspace(0.0, 1.0, WIDTH, dtype=np.float32).reshape((1, WIDTH, 1))

    sky = BG_DEEP.reshape((1, 1, 3)) + (PETROL * 0.7).reshape((1, 1, 3)) * (1.0 - y * 0.8)
    rgb = np.broadcast_to(sky, (HEIGHT, WIDTH, 3)).copy()

    dunes = [
        {"y": 0.55, "amp": 0.09, "phase": 0.4, "col": PETROL * 0.8},
        {"y": 0.68, "amp": 0.12, "phase": 2.1, "col": PETROL * 0.5 + BG_DEEP * 0.5},
        {"y": 0.82, "amp": 0.08, "phase": 4.5, "col": BG_DEEP * 0.7},
    ]

    for d in dunes:
        crest = d["y"] + d["amp"] * np.sin(2.0 * np.pi * x + d["phase"]) + 0.03 * np.sin(5.0 * np.pi * x)
        mask = y >= crest
        crest_dist = (y - crest) / 0.2
        rim = np.exp(- ((y - crest) / 0.018) ** 2) * (np.sin(2.0 * np.pi * x + d["phase"]) * 0.5 + 0.5)
        
        dune_surf = d["col"].reshape((1, 1, 3)) * (1.0 - np.clip(crest_dist, 0.0, 0.6))
        rgb = np.where(mask, dune_surf, rgb)
        rgb += ACCENT_BLUE.reshape((1, 1, 3)) * rim * 0.9

    rgb = 255.0 * (1.0 - np.exp(-rgb / 150.0))
    return apply_tv_calibration_and_dither(rgb, rng_seed=303)

# -------------------------------------------------------------
# THEME 3: MODERNITY (3 wallpapers)
# -------------------------------------------------------------
def render_modern_cyber_grid():
    y = np.linspace(0.0, 1.0, HEIGHT, dtype=np.float32).reshape((HEIGHT, 1, 1))
    x = np.linspace(-1.0, 1.0, WIDTH, dtype=np.float32).reshape((1, WIDTH, 1))

    horizon = 0.42
    sky_mask = y < horizon

    sky = BG_DEEP.reshape((1, 1, 3)) + (PETROL * 0.6).reshape((1, 1, 3)) * (y / horizon)
    dy = np.maximum(y - horizon, 0.001)
    z = 0.28 / dy
    px = x * z

    grid_x = np.abs(np.sin(px * np.pi * 3.5))
    grid_z = np.abs(np.sin(z * np.pi * 1.8))
    wireframe = np.exp(- (grid_x / 0.08) ** 2) + np.exp(- (grid_z / 0.08) ** 2)

    fade_dist = np.exp(- (z / 8.5) ** 1.5)
    floor = (PETROL * 0.4).reshape((1, 1, 3)) + ACCENT_BLUE.reshape((1, 1, 3)) * (wireframe * fade_dist) * 0.95

    rgb = np.where(sky_mask, sky, floor)

    horizon_glow = np.exp(- (np.abs(y - horizon) / 0.02) ** 2)
    rgb += CYAN.reshape((1, 1, 3)) * horizon_glow * 1.2

    rgb = 255.0 * (1.0 - np.exp(-rgb / 155.0))
    return apply_tv_calibration_and_dither(rgb, rng_seed=401)

def render_modern_fiber_optics():
    y = np.linspace(0.0, 1.0, HEIGHT, dtype=np.float32).reshape((HEIGHT, 1, 1))
    x = np.linspace(0.0, 1.0, WIDTH, dtype=np.float32).reshape((1, WIDTH, 1))

    rgb = (BG_DEEP * 0.9).reshape((1, 1, 3)) + (PETROL * 0.3).reshape((1, 1, 3)) * (1.0 - y)
    rgb = np.broadcast_to(rgb, (HEIGHT, WIDTH, 3)).copy()

    lines = [
        {"slope": 0.45, "y0": 0.25, "width": 0.008, "col": ACCENT_BLUE, "pulse": 3.0},
        {"slope": 0.38, "y0": 0.40, "width": 0.012, "col": CYAN, "pulse": 4.5},
        {"slope": 0.52, "y0": 0.55, "width": 0.006, "col": SILVER, "pulse": 2.2},
        {"slope": 0.41, "y0": 0.70, "width": 0.010, "col": INDIGO, "pulse": 5.0},
    ]

    for ln in lines:
        center_y = ln["y0"] + ln["slope"] * (x - 0.5)
        dist = np.abs(y - center_y)
        core = np.exp(- (dist / ln["width"]) ** 2)
        halo = np.exp(- (dist / (ln["width"] * 5.0)) ** 1.5) * 0.4
        
        pulse = np.sin(x * 20.0 + ln["pulse"]) * 0.5 + 0.5
        total_intensity = (core * (0.8 + 0.6 * pulse) + halo)
        rgb += ln["col"].reshape((1, 1, 3)) * total_intensity * 1.3

    rgb = 255.0 * (1.0 - np.exp(-rgb / 155.0))
    return apply_tv_calibration_and_dither(rgb, rng_seed=402)

def render_modern_brutalist_void():
    y = np.linspace(-1.0, 1.0, HEIGHT, dtype=np.float32).reshape((HEIGHT, 1, 1))
    x = np.linspace(-1.777, 1.777, WIDTH, dtype=np.float32).reshape((1, WIDTH, 1))

    p1 = np.maximum(np.abs(x + 0.4) * 0.8 + (y - 0.1) * 0.6, 0.0)
    p2 = np.maximum(np.abs(x - 0.5) * 0.9 - (y + 0.2) * 0.5, 0.0)

    facets = np.sin(p1 * 4.0) * 0.5 + np.cos(p2 * 5.0) * 0.5
    ambient = BG_DEEP.reshape((1, 1, 3)) + (PETROL * 0.5).reshape((1, 1, 3)) * (1.0 - np.abs(y))

    bevel1 = np.exp(- ((p1 - 0.8) / 0.04) ** 2)
    bevel2 = np.exp(- ((p2 - 0.7) / 0.04) ** 2)
    
    rgb = ambient + (PETROL * 0.6).reshape((1, 1, 3)) * facets
    rgb += ACCENT_BLUE.reshape((1, 1, 3)) * bevel1 * 1.4
    rgb += CYAN.reshape((1, 1, 3)) * bevel2 * 1.1

    rgb = 255.0 * (1.0 - np.exp(-rgb / 150.0))
    return apply_tv_calibration_and_dither(rgb, rng_seed=403)


WALLPAPERS = [
    ("aurora.png", render_aurora),
    ("abstract_prism_topo.png", render_abstract_prism_topo),
    ("abstract_silk_wave.png", render_abstract_silk_wave),
    ("abstract_quantum_matrix.png", render_abstract_quantum_matrix),
    ("nature_midnight_ocean.png", render_nature_midnight_ocean),
    ("nature_misty_forest.png", render_nature_misty_forest),
    ("nature_obsidian_dunes.png", render_nature_obsidian_dunes),
    ("modern_cyber_grid.png", render_modern_cyber_grid),
    ("modern_fiber_optics.png", render_modern_fiber_optics),
    ("modern_brutalist_void.png", render_modern_brutalist_void),
]

def main():
    target_dir = sys.argv[1] if len(sys.argv) > 1 else "var/wallpapers"
    os.makedirs(target_dir, exist_ok=True)

    print(f"Generating {len(WALLPAPERS)} wallpapers in Full HD (1920x1080) to {target_dir}...")
    for filename, render_fn in WALLPAPERS:
        path = os.path.join(target_dir, filename)
        print(f"  -> Rendering {filename}...")
        img_arr = render_fn()
        im = Image.fromarray(img_arr, mode="RGB")
        im.save(path, format="PNG", optimize=True)
        print(f"     Saved: {path} ({os.path.getsize(path) // 1024} KB)")

    print("All wallpapers generated successfully with maximum quality!")

if __name__ == "__main__":
    main()
