//! The demonstration data the harness runs with: apps of the design mockup and a fixed clock.

use lume_core::app::{Key, LumeApp};
use lume_core::app_icons::Art;
use lume_core::catalog::color::ICON_SIZE;
use lume_core::catalog::{Catalog, Discovered};
use lume_core::clock::WallClock;
use lume_core::home::config::HomeConfig;
use lume_core::i18n::Translator;
use lume_core::usage::UsageHistory;

/// 2026-10-19 21:45:00 UTC: the time shown in the design mockups.
pub const MOCKUP_TIME: u64 = 1_792_446_300;

/// A solid-color icon, so the catalog derives that color for apps lume does not know.
fn icon(rgb: u32) -> Vec<u32> {
    vec![0xFF00_0000 | rgb; ICON_SIZE * ICON_SIZE]
}

fn app(package: &str, label: &str, rgb: Option<u32>) -> Discovered {
    Discovered {
        package: package.to_string(),
        label: label.to_string(),
        activity: format!("{package}.Main"),
        leanback: true,
        system: false,
        icon: rgb.map(icon),
    }
}

/// The apps of the "Todos os apps" mockup (the known ones get their colors from the theme).
pub fn mockup_apps() -> Vec<Discovered> {
    vec![
        app("com.netflix.ninja", "Netflix", None),
        app("com.google.android.youtube.tv", "YouTube", None),
        app("com.amazon.amazonvideo.livingroom", "Prime Video", None),
        app("com.disney.disneyplus", "Disney+", None),
        app("com.wbd.stream", "Max", None),
        app("com.spotify.tv.android", "Spotify", None),
        app("com.globo.globotv", "Globoplay", None),
        app("com.android.vending", "Play Store", None),
        app("com.android.chrome", "Chrome", None),
        app("com.demo.games", "Jogos", Some(0x6B3FA5)),
        app("com.demo.gallery", "Galeria", Some(0xA3446E)),
        app("com.demo.music", "Música", Some(0xB3601A)),
        app("com.demo.cast", "Cast", Some(0x27757C)),
        app("com.demo.files", "Arquivos", Some(0x3E4A5C)),
    ]
}

pub struct Setup {
    pub locale: Option<String>,
    pub pseudo_locale: bool,
    pub is_24h: bool,
    /// The first N apps of [`mockup_apps`] as favorites, instead of the first-use ones.
    pub favorites: Option<usize>,
}

impl Default for Setup {
    fn default() -> Setup {
        Setup {
            locale: None,
            pseudo_locale: false,
            is_24h: true,
            favorites: None,
        }
    }
}

/// Some shortcuts for the key mapping screen: settings with one or two keys, a key without a name
/// and apps, so the screen is seen with rows and keycaps of different widths.
pub fn map_some_keys(app: &mut LumeApp) {
    use lume_core::keys::{KeyTarget, SettingShortcut};
    let settings = [
        (SettingShortcut::Sleep, vec![183]),
        (SettingShortcut::Picture, vec![184, 165]),
        (SettingShortcut::Wifi, vec![999]),
    ];
    for (setting, codes) in settings {
        for code in codes {
            app.keys.add_code(&KeyTarget::Setting(setting), code);
        }
    }
    let packages: Vec<_> = app
        .catalog
        .apps()
        .iter()
        .take(2)
        .map(|a| a.package.clone())
        .collect();
    for (package, code) in packages.into_iter().zip([185, 186]) {
        app.keys.add_code(&KeyTarget::App(package), code);
    }
}

/// A launcher with the mockup apps, as it looks on a first run.
pub fn app_with(setup: &Setup) -> LumeApp {
    let catalog = Catalog::from_discovered(&mockup_apps(), "app.lume.tv");
    let mut usage = UsageHistory::new();
    usage.mark_baseline(catalog.apps().iter().map(|a| a.package.as_str()));
    let mut config = HomeConfig::first_use(&catalog);
    if let Some(count) = setup.favorites {
        config.favorites = mockup_apps()
            .into_iter()
            .take(count)
            .map(|app| app.package)
            .collect();
    }
    let preferred: Vec<&str> = setup.locale.as_deref().into_iter().collect();
    let mut translator = Translator::new(&preferred);
    if setup.pseudo_locale {
        translator = translator.pseudo();
    }
    let clock = WallClock {
        unix_secs: MOCKUP_TIME,
        utc_offset_secs: 0,
        is_24h: setup.is_24h,
    };
    let mut app = LumeApp::new(catalog, usage, config, translator, clock);
    app.inputs_available = true;
    app
}

/// A made-up TV's information sheet, with a few lines left out as a real system sometimes does.
fn demo_device_info() -> lume_core::device_info::DeviceInfo {
    use lume_core::device_info::{DeviceInfo, Field, Link, Value};
    let mut info = DeviceInfo::default();
    for (field, text) in [
        (Field::Brand, "TCL"),
        (Field::Model, "55C715"),
        (Field::Manufacturer, "TCL"),
        (Field::Board, "t962x3"),
        (Field::Soc, "Amlogic T962X3"),
        (Field::Cpu, "Cortex-A55"),
        (Field::Abi, "arm64-v8a"),
        (Field::Gpu, "Mali-G31"),
        (Field::OpenGl, "OpenGL ES 3.2"),
        (Field::Android, "11"),
        (Field::Api, "30"),
        (Field::Firmware, "RTVA.211202.0204 release-keys"),
        (Field::Kernel, "4.9.269"),
        (Field::Resolution, "1920 × 1080"),
        (Field::Ip, "192.168.0.24"),
    ] {
        info.set_text(field, text);
    }
    info.set(Field::Cores, Value::Count(4));
    info.set(Field::Ram, Value::Bytes(2_000_000_000));
    info.set(Field::MemoryProfile, Value::LowRam(false));
    info.set(Field::StorageTotal, Value::Bytes(12_000_000_000));
    info.set(Field::StorageFree, Value::Bytes(7_300_000_000));
    info.set(Field::RefreshRate, Value::Hertz(60.0));
    info.set(Field::Density, Value::Dpi(213));
    info.set(Field::Connection, Value::Link(Link::WiFi));
    info
}

/// Feeds the settings panel the numbers of a made-up TV, as the platform would every 2 seconds,
/// and the TV information screen its sheet when it opens.
pub fn feed_stats(app: &mut LumeApp, now_ms: u64) {
    use lume_core::stats::{CpuReading, Stats};
    if app.wants_device_info() {
        app.apply_device_info(demo_device_info());
    }
    if app.stats_deadline(now_ms).is_some_and(|at| at <= now_ms) {
        app.apply_stats(
            Stats {
                memory_total_kb: 2_000_000,
                memory_available_kb: 820_000,
                disk_total: 12_000_000_000,
                disk_free: 7_300_000_000,
                cpu: CpuReading::Freq {
                    current: 1_350_000,
                    max: 1_500_000,
                },
            },
            now_ms,
        );
    }
}

/// The color of every synthetic banner: a light yellow no app here has, so a ring of the tile
/// color around it shows in a capture.
pub const SYNTHETIC_BANNER: u32 = 0xF0D060;

/// Stand-ins for the pictures the platform fetches, which the desktop cannot read: a solid banner
/// for every app tile and an opaque white icon for every favorite. Reserves their cells and
/// returns each region (`[x, y, width, height]`) with its premultiplied RGBA pixels.
pub fn synthetic_icons(app: &mut LumeApp) -> Vec<([u32; 4], Vec<u8>)> {
    let solid = |region: [u32; 4], rgb: u32| {
        let pixel = [(rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8, 0xFF];
        (region, pixel.repeat((region[2] * region[3]) as usize))
    };
    let packages: Vec<String> = app
        .catalog
        .apps()
        .iter()
        .map(|a| a.package.clone())
        .collect();
    let mut uploads = Vec::new();
    for package in &packages {
        if let Some(region) = app.icons.reserve(package, Art::Banner) {
            uploads.push(solid(region, SYNTHETIC_BANNER));
        }
    }
    for package in app.config.favorites.clone() {
        if let Some(region) = app.icons.reserve_dock(&package) {
            uploads.push(solid(region, 0xFFFFFF));
        }
    }
    uploads
}

/// Parses names such as `down`, `right` and `ok`.
pub fn parse_keys(text: &str) -> Result<Vec<Key>, String> {
    text.split(',')
        .filter(|name| !name.trim().is_empty())
        .map(|name| {
            Key::parse_name(name).ok_or_else(|| {
                format!(
                    "unknown key '{}' (use up, down, left, right, ok, back, home, menu)",
                    name.trim()
                )
            })
        })
        .collect()
}
