"""Generates all 4K (3840x2160) wallpapers for Option A, Option B, and Option C,
plus the Core 4K (Aurora and the 2 favorite abstracts).
"""

import os
import sys
import numpy as np
from PIL import Image
import generate_4k_all_options as gen

def save_image(img_arr, path):
    im = Image.fromarray(img_arr, mode="RGB")
    im.save(path, format="PNG", optimize=True)
    print(f"Saved: {path} ({os.path.getsize(path) // 1024} KB)")

def main():
    print("=== Generating Core 4K Wallpapers (Aurora + 2 Abstracts) ===")
    for folder in ["var/wallpapers/opcao_a", "var/wallpapers/opcao_b", "var/wallpapers/opcao_c"]:
        os.makedirs(folder, exist_ok=True)

    aurora_4k = gen.render_aurora_4k()
    prism_4k = gen.render_abstract_prism_topo_4k()
    silk_4k = gen.render_abstract_silk_wave_4k()

    # Save shared Core in all 3 options
    for folder in ["var/wallpapers/opcao_a", "var/wallpapers/opcao_b", "var/wallpapers/opcao_c"]:
        save_image(aurora_4k, os.path.join(folder, "0_aurora_4k.png"))
        save_image(prism_4k, os.path.join(folder, "1_abstract_prism_topo_4k.png"))
        save_image(silk_4k, os.path.join(folder, "2_abstract_silk_wave_4k.png"))

    print("\n=== Option A: 4K Real Photography (Unsplash 4K Masters) ===")
    opt_a_dir = "var/wallpapers/opcao_a"
    # Nature (3)
    save_image(gen.process_photo_to_4k("var/wallpapers/downloads/real_mountain_night.jpg", 801),
               os.path.join(opt_a_dir, "nature_1_mountain_stars_4k.png"))
    save_image(gen.process_photo_to_4k("var/wallpapers/downloads/real_lake_stars.jpg", 802),
               os.path.join(opt_a_dir, "nature_2_mirror_lake_4k.png"))
    save_image(gen.process_photo_to_4k("var/wallpapers/downloads/real_dark_forest.jpg", 803),
               os.path.join(opt_a_dir, "nature_3_misty_forest_4k.png"))
    # Modernity (3)
    save_image(gen.process_photo_to_4k("var/wallpapers/downloads/real_skyscrapers_neon.jpg", 804),
               os.path.join(opt_a_dir, "modern_1_cyber_skyscrapers_4k.png"))
    save_image(gen.process_photo_to_4k("var/wallpapers/downloads/real_glass_architecture.jpg", 805),
               os.path.join(opt_a_dir, "modern_2_glass_facade_4k.png"))
    save_image(gen.process_photo_to_4k("var/wallpapers/downloads/real_brutalist_void.jpg", 806),
               os.path.join(opt_a_dir, "modern_3_brutalist_geometry_4k.png"))

    print("\n=== Option B: Curated Master Artworks Resampled to 4K ===")
    opt_b_dir = "var/wallpapers/opcao_b"
    # Nature (3)
    save_image(gen.process_photo_to_4k("tools/wallpaper/masters/1_floresta_noturna.jpg", 901),
               os.path.join(opt_b_dir, "nature_1_floresta_noturna_4k.png"))
    save_image(gen.process_photo_to_4k("tools/wallpaper/masters/2_ondas_meia_noite.jpg", 902),
               os.path.join(opt_b_dir, "nature_2_ondas_meia_noite_4k.png"))
    save_image(gen.process_photo_to_4k("tools/wallpaper/masters/6_lago_espelhado.jpg", 903),
               os.path.join(opt_b_dir, "nature_3_lago_espelhado_4k.png"))
    # Modernity (3)
    save_image(gen.process_photo_to_4k("tools/wallpaper/masters/3_canion_estrelado.jpg", 904),
               os.path.join(opt_b_dir, "modern_1_canion_estrelado_4k.png"))
    save_image(gen.process_photo_to_4k("tools/wallpaper/masters/7_dunas_negras.jpg", 905),
               os.path.join(opt_b_dir, "modern_2_dunas_negras_4k.png"))
    save_image(gen.process_photo_to_4k("tools/wallpaper/masters/9_eter_estelar.jpg", 906),
               os.path.join(opt_b_dir, "modern_3_eter_estelar_4k.png"))

    print("\n=== Option C: Procedural Photorealistic Mathematical Shaders (4K Native) ===")
    opt_c_dir = "var/wallpapers/opcao_c"
    # Nature (3)
    save_image(gen.render_proc_photoreal_nature_lake_4k(),
               os.path.join(opt_c_dir, "nature_1_proc_alpine_lake_4k.png"))
    save_image(gen.render_proc_photoreal_nature_alpine_4k(),
               os.path.join(opt_c_dir, "nature_2_proc_mountain_crag_4k.png"))
    save_image(gen.render_proc_photoreal_modern_cityscape_4k(),
               os.path.join(opt_c_dir, "nature_3_proc_misty_ridge_4k.png"))
    # Modernity (3)
    save_image(gen.render_proc_photoreal_modern_cityscape_4k(),
               os.path.join(opt_c_dir, "modern_1_proc_skyscraper_grid_4k.png"))
    save_image(gen.render_proc_photoreal_modern_monolith_4k(),
               os.path.join(opt_c_dir, "modern_2_proc_glass_monolith_4k.png"))
    save_image(gen.render_proc_photoreal_modern_monolith_4k(),
               os.path.join(opt_c_dir, "modern_3_proc_angular_void_4k.png"))

    print("\nAll 4K wallpapers generated successfully!")

if __name__ == "__main__":
    main()
