"""Builds opcao_d nerd/geek 4K wallpapers for Lume TV Launcher."""

import os
import sys
from PIL import Image
import generate_4k_all_options as gen

def save_image(img_arr, path):
    im = Image.fromarray(img_arr, mode="RGB")
    im.save(path, format="PNG", optimize=True)
    print(f"Saved: {path} ({os.path.getsize(path) // 1024} KB)")

def main():
    opt_d_dir = "var/wallpapers/opcao_d"
    os.makedirs(opt_d_dir, exist_ok=True)

    # Core items (Aurora + 2 Abstracts)
    print("Copying/Rendering Core 4K...")
    aurora_src = "var/wallpapers/opcao_a/0_aurora_4k.png"
    prism_src = "var/wallpapers/opcao_a/1_abstract_prism_topo_4k.png"
    silk_src = "var/wallpapers/opcao_a/2_abstract_silk_wave_4k.png"

    os.system(f"cp {aurora_src} {opt_d_dir}/0_aurora_4k.png")
    os.system(f"cp {prism_src} {opt_d_dir}/1_abstract_prism_topo_4k.png")
    os.system(f"cp {silk_src} {opt_d_dir}/2_abstract_silk_wave_4k.png")

    print("\nProcessing Nerd/Geek 4K Wallpapers with TV contrast calibration & dithering...")
    nerd_items = [
        ("nerd_1_retro_arcade_neon_4k.png", "var/wallpapers/downloads_nerd/retro_arcade_neon.jpg", 1001),
        ("nerd_2_cyber_motherboard_4k.png", "var/wallpapers/downloads_nerd/cyber_motherboard.jpg", 1002),
        ("nerd_3_matrix_code_dark_4k.png", "var/wallpapers/downloads_nerd/matrix_code_dark.jpg", 1003),
        ("nerd_4_arcade_cabinets_hall_4k.png", "var/wallpapers/downloads_nerd/retro_arcade_cabinets.jpg", 1004),
        ("nerd_5_game_controller_neon_4k.png", "var/wallpapers/downloads_nerd/game_controller_neon.jpg", 1005),
        ("nerd_6_synthwave_sunset_grid_4k.png", "var/wallpapers/downloads_nerd/synthwave_grid.jpg", 1006),
    ]

    for filename, src_path, seed in nerd_items:
        dst = os.path.join(opt_d_dir, filename)
        img_arr = gen.process_photo_to_4k(src_path, seed=seed)
        save_image(img_arr, dst)

    os.system("rm -rf var/wallpapers/downloads_nerd")
    print("\nOpcao D successfully built in var/wallpapers/opcao_d!")

if __name__ == "__main__":
    main()
