"""Build three original beach QOI wallpapers and the expanded thumbnail atlas.

Run from the repository root:
    uv run --with pillow --with numpy tools/wallpaper/build_beach_assets.py

The source PNGs are generated artwork. The app renders the wallpapers at 1920x1080;
the original source files remain available for future higher resolution builds.
"""

from pathlib import Path
import importlib.util

import numpy as np
from PIL import Image

ROOT = Path(__file__).resolve().parents[2]
GENERATED = ROOT / "tools/wallpaper/generated"
ASSETS = ROOT / "crates/lume-core/assets/wallpaper"

spec = importlib.util.spec_from_file_location(
    "wallpaper_generator", ROOT / "tools/wallpaper/generate_wallpapers.py"
)
generator = importlib.util.module_from_spec(spec)
spec.loader.exec_module(generator)

beaches = [
    ("cinematic", "beach-cove-source.png"),
    ("beach_sunset", "beach-sunset-source.png"),
    ("beach_dawn", "beach-dawn-source.png"),
]

atlas = Image.new("RGBA", (640, 144), (0, 0, 0, 0))
for index, (name, filename) in enumerate(beaches):
    image = Image.open(GENERATED / filename).convert("RGB")
    image = image.resize((1920, 1080), Image.Resampling.LANCZOS)
    rgb = np.asarray(image)
    (ASSETS / f"{name}-1920x1080.qoi").write_bytes(generator.qoi_encode(rgb, 1920, 1080))
    (ASSETS / f"{name}-stars.txt").write_text("# no stars\n")
    tile = image.resize((128, 72), Image.Resampling.LANCZOS).convert("RGBA")
    atlas.paste(tile, ((index % 5) * 128, (index // 5) * 72))

legacy = Image.open(GENERATED / "legacy-thumbnails.png").convert("RGBA")
for old_index in range(1, 7):
    new_index = old_index + 2
    tile = legacy.crop(
        ((old_index % 5) * 128, (old_index // 5) * 72,
         (old_index % 5 + 1) * 128, (old_index // 5 + 1) * 72)
    )
    atlas.paste(tile, ((new_index % 5) * 128, (new_index // 5) * 72))

(ASSETS / "thumbnails-640x144.rgba").write_bytes(atlas.tobytes())
print("Built three 1920x1080 QOI beach wallpapers and a nine-image atlas")
