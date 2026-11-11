//! The wallpaper: the single place the renderer gets it from.
//!
//! A catalog of 6 high definition wallpapers (1920x1080, lossless QOI) read from the platform's
//! assets (the APK's `assets/` on Android, the repository on the desktop), not embedded in the
//! library: the APK carries one copy of each. Sharp stars drawn as points on top when present.

use crate::i18n::StringId;
use std::sync::OnceLock;

/// Reads a wallpaper asset (a file name such as `aurora-1920x1080.qoi`) from the platform.
pub type AssetReader = fn(&str) -> Option<Vec<u8>>;

static ASSET_READER: OnceLock<AssetReader> = OnceLock::new();

/// Sets the platform's asset reader (Android: the `AAssetManager`). Without one, desktop builds
/// read the repository's `assets/wallpaper/` directory.
pub fn set_asset_reader(reader: AssetReader) {
    let _ = ASSET_READER.set(reader);
}

#[cfg(not(target_os = "android"))]
fn default_reader() -> Option<AssetReader> {
    Some(|name| {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/wallpaper");
        std::fs::read(dir.join(name)).ok()
    })
}

#[cfg(target_os = "android")]
fn default_reader() -> Option<AssetReader> {
    None
}

static CACHE_DIR: OnceLock<std::path::PathBuf> = OnceLock::new();

/// Sets the directory where the decoded wallpaper in use is kept between wake-ups (Android: the app's
/// cache directory, which the system may clear and does not back up). Without one there is no cache.
pub fn set_cache_dir(dir: std::path::PathBuf) {
    let _ = CACHE_DIR.set(dir);
}

static READY_NOTIFIER: OnceLock<fn()> = OnceLock::new();

/// Sets what the platform does when a wallpaper prepared off the frame loop is ready (Android: wake the
/// loop, so it draws the frame that swaps it in).
pub fn set_ready_notifier(notifier: fn()) {
    let _ = READY_NOTIFIER.set(notifier);
}

/// Reads and decodes the wallpaper on a worker thread; the receiver gets the RGB (or `None` when the
/// asset is missing or invalid) and then the platform is notified.
pub fn load_in_background(id: WallpaperId) -> std::sync::mpsc::Receiver<Option<Vec<u8>>> {
    let (sender, receiver) = std::sync::mpsc::channel();
    let spawned = std::thread::Builder::new()
        .name("lume-wallpaper".into())
        .spawn(move || {
            let _ = sender.send(id.wallpaper().load_rgb());
            if let Some(notify) = READY_NOTIFIER.get() {
                notify();
            }
        });
    if let Err(e) = spawned {
        log::error!("cannot start the wallpaper thread: {e}");
    }
    receiver
}

fn read_asset(name: &str) -> Option<Vec<u8>> {
    let reader = ASSET_READER.get().copied().or_else(default_reader)?;
    reader(name)
}

/// An RGB image the renderer uploads as the background texture, and its stars.
#[derive(Debug)]
pub struct Wallpaper {
    pub width: u32,
    pub height: u32,
    /// The asset's file name (QOI, `width * height` RGB pixels).
    pub asset: &'static str,
    /// Stars to draw as crisp points over the image: `x`, `y` in a 1920x1080 picture and a
    /// brightness from 0 to 255.
    pub stars: &'static [(u16, u16, u8)],
}

impl Wallpaper {
    /// Reads the 24-bit raw RGB buffer (`width * height * 3` bytes), from the cache when it has it and
    /// otherwise by decoding the asset (and then keeping the result in the cache). `None` when the asset
    /// is missing or invalid.
    pub fn load_rgb(&self) -> Option<Vec<u8>> {
        self.load_rgb_cached(
            CACHE_DIR.get().map(|dir| dir.as_path()),
            lume_version::VERSION,
        )
    }

    /// The cache file of this wallpaper: the asset and the version of the build in the name, so a new
    /// version of the app never reads the image of an old one.
    fn cache_file(&self, dir: &std::path::Path, version: &str) -> std::path::PathBuf {
        dir.join(format!("wallpaper-{}-{version}.rgb", self.asset))
    }

    fn load_rgb_cached(&self, cache: Option<&std::path::Path>, version: &str) -> Option<Vec<u8>> {
        let started = std::time::Instant::now();
        let size = (self.width * self.height * 3) as usize;
        let file = cache.map(|dir| self.cache_file(dir, version));
        if let Some(rgb) = file.as_ref().and_then(|file| std::fs::read(file).ok()) {
            if rgb.len() == size {
                log::info!(
                    "wallpaper {}: from the cache in {} ms",
                    self.asset,
                    started.elapsed().as_millis()
                );
                return Some(rgb);
            }
            // A broken file (cut short by a crash): made again below.
        }
        let rgb = self.decode_asset()?;
        log::info!(
            "wallpaper {}: decoded in {} ms",
            self.asset,
            started.elapsed().as_millis()
        );
        if let (Some(dir), Some(file)) = (cache, file) {
            // Only the wallpaper in use stays: the others (and the old versions) are deleted. Written
            // to a temporary name and renamed, so a reader never sees half of a file.
            let _ = std::fs::create_dir_all(dir);
            if let Ok(entries) = std::fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let name = entry.file_name();
                    if name.to_string_lossy().starts_with("wallpaper-") && entry.path() != file {
                        let _ = std::fs::remove_file(entry.path());
                    }
                }
            }
            let temporary = file.with_extension("tmp");
            if std::fs::write(&temporary, &rgb).is_ok()
                && std::fs::rename(&temporary, &file).is_err()
            {
                let _ = std::fs::remove_file(&temporary);
            }
        }
        Some(rgb)
    }

    fn decode_asset(&self) -> Option<Vec<u8>> {
        let bytes = read_asset(self.asset)?;
        let (header, rgb) = qoi::decode_to_vec(&bytes).ok()?;
        let valid = header.width == self.width
            && header.height == self.height
            && rgb.len() == (self.width * self.height * 3) as usize;
        valid.then_some(rgb)
    }

    /// Like [`Wallpaper::load_rgb`], for tests.
    pub fn decompress_rgb(&self) -> Vec<u8> {
        self.load_rgb()
            .unwrap_or_else(|| panic!("cannot read the wallpaper asset {}", self.asset))
    }
}

const WIDTH: u32 = 1920;
const HEIGHT: u32 = 1080;

pub const THUMBNAILS_WIDTH: u32 = 640;
pub const THUMBNAILS_HEIGHT: u32 = 144;
pub const THUMBNAILS_DEFLATED: &[u8] = include_bytes!(concat!(
    env!("OUT_DIR"),
    "/wallpaper_thumbnails_640x144.deflate"
));

/// Decompresses the 640x144 RGBA thumbnail atlas.
pub fn decompress_thumbnails() -> Vec<u8> {
    miniz_oxide::inflate::decompress_to_vec(THUMBNAILS_DEFLATED).expect("corrupt thumbnails")
}

/// Identifiers for the wallpapers in the catalog.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum WallpaperId {
    #[default]
    Cinematic,
    BeachSunset,
    BeachDawn,
    Aurora,
    MountainStars,
    MirrorLake,
    MistyForest,
    GlassFacade,
    SynthwaveGrid,
}

impl WallpaperId {
    /// All wallpapers in catalog display order.
    pub const ALL: [WallpaperId; 9] = [
        WallpaperId::Cinematic,
        WallpaperId::BeachSunset,
        WallpaperId::BeachDawn,
        WallpaperId::Aurora,
        WallpaperId::MountainStars,
        WallpaperId::MirrorLake,
        WallpaperId::MistyForest,
        WallpaperId::GlassFacade,
        WallpaperId::SynthwaveGrid,
    ];

    /// The 0-based index of this wallpaper in the catalog (0 to 5).
    pub const fn index(&self) -> usize {
        match self {
            WallpaperId::Cinematic => 0,
            WallpaperId::BeachSunset => 1,
            WallpaperId::BeachDawn => 2,
            WallpaperId::Aurora => 3,
            WallpaperId::MountainStars => 4,
            WallpaperId::MirrorLake => 5,
            WallpaperId::MistyForest => 6,
            WallpaperId::GlassFacade => 7,
            WallpaperId::SynthwaveGrid => 8,
        }
    }

    /// The string identifier stored in configuration (`config.v1`).
    pub const fn as_str(&self) -> &'static str {
        match self {
            WallpaperId::Cinematic => "cinematic",
            WallpaperId::BeachSunset => "beach_sunset",
            WallpaperId::BeachDawn => "beach_dawn",
            WallpaperId::Aurora => "aurora",
            WallpaperId::MountainStars => "mountain_stars",
            WallpaperId::MirrorLake => "mirror_lake",
            WallpaperId::MistyForest => "misty_forest",
            WallpaperId::GlassFacade => "glass_facade",
            WallpaperId::SynthwaveGrid => "synthwave_grid",
        }
    }

    /// Parses a string identifier from configuration.
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<WallpaperId> {
        match s {
            "cinematic" => Some(WallpaperId::Cinematic),
            "beach_sunset" => Some(WallpaperId::BeachSunset),
            "beach_dawn" => Some(WallpaperId::BeachDawn),
            "aurora" => Some(WallpaperId::Aurora),
            "mountain_stars" => Some(WallpaperId::MountainStars),
            "mirror_lake" => Some(WallpaperId::MirrorLake),
            "misty_forest" => Some(WallpaperId::MistyForest),
            "glass_facade" => Some(WallpaperId::GlassFacade),
            "synthwave_grid" => Some(WallpaperId::SynthwaveGrid),
            _ => None,
        }
    }

    /// The localized name string ID.
    pub const fn title_string_id(&self) -> StringId {
        match self {
            WallpaperId::Cinematic => StringId::WallpaperCinematic,
            WallpaperId::BeachSunset => StringId::WallpaperBeachSunset,
            WallpaperId::BeachDawn => StringId::WallpaperBeachDawn,
            WallpaperId::Aurora => StringId::WallpaperAurora,
            WallpaperId::MountainStars => StringId::WallpaperMountainStars,
            WallpaperId::MirrorLake => StringId::WallpaperMirrorLake,
            WallpaperId::MistyForest => StringId::WallpaperMistyForest,
            WallpaperId::GlassFacade => StringId::WallpaperGlassFacade,
            WallpaperId::SynthwaveGrid => StringId::WallpaperSynthwaveGrid,
        }
    }

    /// UV rectangle (x, y, w, h) in the 640x144 thumbnail atlas.
    pub fn thumb_uv(self) -> crate::render::Rect {
        let i = self.index();
        let col = (i % 5) as f32;
        let row = (i / 5) as f32;
        crate::render::Rect::new(
            col * 128.0 / 640.0,
            row * 72.0 / 144.0,
            128.0 / 640.0,
            72.0 / 144.0,
        )
    }

    /// The wallpaper metadata, RGB buffer and stars.
    pub fn wallpaper(&self) -> &'static Wallpaper {
        match self {
            WallpaperId::Cinematic => &CINEMATIC,
            WallpaperId::BeachSunset => &BEACH_SUNSET,
            WallpaperId::BeachDawn => &BEACH_DAWN,
            WallpaperId::Aurora => &AURORA,
            WallpaperId::MountainStars => &MOUNTAIN_STARS,
            WallpaperId::MirrorLake => &MIRROR_LAKE,
            WallpaperId::MistyForest => &MISTY_FOREST,
            WallpaperId::GlassFacade => &GLASS_FACADE,
            WallpaperId::SynthwaveGrid => &SYNTHWAVE_GRID,
        }
    }
}

static CINEMATIC: Wallpaper = Wallpaper {
    width: WIDTH,
    height: HEIGHT,
    asset: "cinematic-1920x1080.qoi",
    stars: include!(concat!(env!("OUT_DIR"), "/wallpaper_cinematic_stars.rs")),
};

static BEACH_SUNSET: Wallpaper = Wallpaper {
    width: WIDTH,
    height: HEIGHT,
    asset: "beach_sunset-1920x1080.qoi",
    stars: include!(concat!(env!("OUT_DIR"), "/wallpaper_beach_sunset_stars.rs")),
};

static BEACH_DAWN: Wallpaper = Wallpaper {
    width: WIDTH,
    height: HEIGHT,
    asset: "beach_dawn-1920x1080.qoi",
    stars: include!(concat!(env!("OUT_DIR"), "/wallpaper_beach_dawn_stars.rs")),
};

static AURORA: Wallpaper = Wallpaper {
    width: WIDTH,
    height: HEIGHT,
    asset: "aurora-1920x1080.qoi",
    stars: include!(concat!(env!("OUT_DIR"), "/wallpaper_aurora_stars.rs")),
};

static MOUNTAIN_STARS: Wallpaper = Wallpaper {
    width: WIDTH,
    height: HEIGHT,
    asset: "mountain_stars-1920x1080.qoi",
    stars: include!(concat!(
        env!("OUT_DIR"),
        "/wallpaper_mountain_stars_stars.rs"
    )),
};

static MIRROR_LAKE: Wallpaper = Wallpaper {
    width: WIDTH,
    height: HEIGHT,
    asset: "mirror_lake-1920x1080.qoi",
    stars: include!(concat!(env!("OUT_DIR"), "/wallpaper_mirror_lake_stars.rs")),
};

static MISTY_FOREST: Wallpaper = Wallpaper {
    width: WIDTH,
    height: HEIGHT,
    asset: "misty_forest-1920x1080.qoi",
    stars: include!(concat!(env!("OUT_DIR"), "/wallpaper_misty_forest_stars.rs")),
};

static GLASS_FACADE: Wallpaper = Wallpaper {
    width: WIDTH,
    height: HEIGHT,
    asset: "glass_facade-1920x1080.qoi",
    stars: include!(concat!(env!("OUT_DIR"), "/wallpaper_glass_facade_stars.rs")),
};

static SYNTHWAVE_GRID: Wallpaper = Wallpaper {
    width: WIDTH,
    height: HEIGHT,
    asset: "synthwave_grid-1920x1080.qoi",
    stars: include!(concat!(
        env!("OUT_DIR"),
        "/wallpaper_synthwave_grid_stars.rs"
    )),
};

/// The default wallpaper.
pub fn current() -> &'static Wallpaper {
    WallpaperId::default().wallpaper()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Average luminance (0 to 255) of the rows `rows` of a wallpaper, given in a 360-row scale
    /// (the bands the contrast limits were set on), whatever the stored height.
    fn luma(w: &Wallpaper, rows: std::ops::Range<usize>) -> f32 {
        let rgb = w.decompress_rgb();
        let k = w.height as usize / 360;
        let row = w.width as usize * 3;
        let slice = &rgb[rows.start * k * row..rows.end * k * row];
        slice
            .chunks(3)
            .map(|p| 0.2126 * f32::from(p[0]) + 0.7152 * f32::from(p[1]) + 0.0722 * f32::from(p[2]))
            .sum::<f32>()
            / (slice.len() / 3) as f32
    }

    #[test]
    fn all_wallpapers_have_declared_size() {
        for id in WallpaperId::ALL {
            let w = id.wallpaper();
            assert_eq!((w.width, w.height), (1920, 1080));
            assert_eq!(w.decompress_rgb().len(), 1920 * 1080 * 3);
        }
    }

    #[test]
    #[ignore = "the wallpapers no longer carry the top and dock darkening (author's decision, 2026-10-21; see CLAUDE.md); see the wallpapers-full-hd change"]
    fn all_wallpapers_dark_top_and_bottom_for_ui() {
        for id in WallpaperId::ALL {
            let w = id.wallpaper();
            let top = luma(w, 0..30);
            let bottom = luma(w, 300..360);
            let mid = luma(w, 120..240);
            assert!(
                top < 40.0,
                "{}: top luma {top} must be < 40 for top bar contrast",
                id.as_str()
            );
            assert!(
                bottom < 25.0,
                "{}: bottom luma {bottom} must be < 25 for dock contrast",
                id.as_str()
            );
            assert!(
                mid > bottom,
                "{}: mid luma {mid} must be > bottom luma {bottom}",
                id.as_str()
            );
        }
    }

    #[test]
    fn stars_stay_in_the_upper_part_of_the_picture() {
        for id in WallpaperId::ALL {
            let w = id.wallpaper();
            for &(x, y, k) in w.stars {
                assert!(x < 1920 && y < 1080 && k > 0);
                assert!(
                    y < (1080.0 * 0.7) as u16,
                    "{}: star at ({x}, {y}) is in bottom 30%",
                    id.as_str()
                );
            }
        }
    }

    fn temp_dir(name: &str) -> std::path::PathBuf {
        let dir =
            std::env::temp_dir().join(format!("lume-wallpaper-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn cache_files(dir: &std::path::Path) -> Vec<String> {
        let mut names: Vec<String> = std::fs::read_dir(dir)
            .unwrap()
            .flatten()
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .collect();
        names.sort();
        names
    }

    #[test]
    fn the_cache_misses_then_hits_and_keeps_the_same_image() {
        let dir = temp_dir("hit");
        let w = WallpaperId::Aurora.wallpaper();
        assert!(cache_files(&dir).is_empty());
        let first = w.load_rgb_cached(Some(&dir), "1.0").unwrap();
        assert_eq!(cache_files(&dir).len(), 1, "the decoded image is kept");
        let second = w.load_rgb_cached(Some(&dir), "1.0").unwrap();
        assert!(first == second, "the cache returns the decoded image");
        assert_eq!(first, w.decode_asset().unwrap());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn choosing_another_wallpaper_or_version_leaves_only_the_active_file() {
        let dir = temp_dir("swap");
        WallpaperId::Aurora
            .wallpaper()
            .load_rgb_cached(Some(&dir), "1.0")
            .unwrap();
        WallpaperId::MirrorLake
            .wallpaper()
            .load_rgb_cached(Some(&dir), "1.0")
            .unwrap();
        assert_eq!(
            cache_files(&dir),
            ["wallpaper-mirror_lake-1920x1080.qoi-1.0.rgb"]
        );
        WallpaperId::MirrorLake
            .wallpaper()
            .load_rgb_cached(Some(&dir), "1.1")
            .unwrap();
        assert_eq!(
            cache_files(&dir),
            ["wallpaper-mirror_lake-1920x1080.qoi-1.1.rgb"]
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_broken_or_missing_cache_is_made_again_and_no_cache_dir_still_works() {
        let dir = temp_dir("broken");
        let w = WallpaperId::GlassFacade.wallpaper();
        std::fs::write(w.cache_file(&dir, "1.0"), b"cut short").unwrap();
        let rgb = w.load_rgb_cached(Some(&dir), "1.0").unwrap();
        assert_eq!(rgb.len(), 1920 * 1080 * 3);
        assert_eq!(
            std::fs::read(w.cache_file(&dir, "1.0")).unwrap().len(),
            rgb.len()
        );
        // The system cleared the cache: the directory itself is gone.
        let _ = std::fs::remove_dir_all(&dir);
        assert_eq!(
            w.load_rgb_cached(Some(&dir), "1.0").unwrap().len(),
            rgb.len()
        );
        assert_eq!(w.load_rgb_cached(None, "1.0").unwrap().len(), rgb.len());
    }

    #[test]
    fn wallpaper_id_string_round_trip() {
        for id in WallpaperId::ALL {
            let s = id.as_str();
            assert_eq!(WallpaperId::from_str(s), Some(id));
        }
        assert_eq!(WallpaperId::from_str("unknown"), None);
    }
}
