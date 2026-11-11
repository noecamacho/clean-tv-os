//! The installed apps lume can open: discovery results, ordering, changes and a disk cache.

pub mod color;
pub mod known;
pub mod normalize;

use crate::storage::{self, Kv, StorageError};
use crate::theme::{vars, Color};
use known::KnownApp;
use std::collections::HashMap;
use std::fmt::Write;
use std::path::Path;

/// Version of the `catalog.v1` file format.
const CACHE_VERSION: u32 = 2;

/// One entry the platform reports for an app's launcher activity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Discovered {
    pub package: String,
    /// The label the system shows for the app.
    pub label: String,
    /// Class name of the activity to launch.
    pub activity: String,
    /// `true` for a LEANBACK_LAUNCHER activity, `false` for a plain LAUNCHER one.
    pub leanback: bool,
    /// A system app (preinstalled or an update of one), which cannot be uninstalled.
    pub system: bool,
    /// The icon as ARGB pixels, [`color::ICON_SIZE`] squared, used to derive a color.
    pub icon: Option<Vec<u32>>,
}

/// An app in the catalog.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct App {
    pub package: String,
    pub label: String,
    pub activity: String,
    /// Tile background; white text on it has a contrast of at least 4.5:1.
    pub color: Color,
    /// A system app, which cannot be uninstalled.
    pub system: bool,
}

impl App {
    /// Builds an app from a discovery entry, choosing its tile color.
    pub fn from_discovered(entry: &Discovered) -> App {
        let color = match KnownApp::from_package(&entry.package) {
            Some(known) => known.color(),
            None => {
                let derived = entry
                    .icon
                    .as_deref()
                    .and_then(color::dominant_color)
                    .unwrap_or(vars::TILE_DEFAULT);
                color::ensure_white_text_contrast(derived)
            }
        };
        App {
            package: entry.package.clone(),
            label: entry.label.clone(),
            activity: entry.activity.clone(),
            color,
            system: entry.system,
        }
    }
}

/// What changed in the catalog.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Changes {
    pub added: Vec<String>,
    pub removed: Vec<String>,
    pub updated: Vec<String>,
}

impl Changes {
    pub fn is_empty(&self) -> bool {
        self.added.is_empty() && self.removed.is_empty() && self.updated.is_empty()
    }
}

/// All apps, in alphabetical order of their labels (ignoring case and accents).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Catalog {
    apps: Vec<App>,
    /// Package of lume itself, which never appears in the catalog.
    own_package: String,
}

impl Catalog {
    pub fn new(own_package: &str) -> Catalog {
        Catalog {
            apps: Vec::new(),
            own_package: own_package.to_string(),
        }
    }

    /// Builds the catalog from a full discovery. An app with several launcher activities
    /// appears once, using its TV activity when it has one.
    pub fn from_discovered(entries: &[Discovered], own_package: &str) -> Catalog {
        let mut catalog = Catalog::new(own_package);
        for app in Catalog::choose_activities(entries) {
            catalog.apps.push(App::from_discovered(app));
        }
        catalog.apps.retain(|a| a.package != own_package);
        catalog.sort();
        catalog
    }

    fn choose_activities(entries: &[Discovered]) -> Vec<&Discovered> {
        let mut chosen: Vec<&Discovered> = Vec::with_capacity(entries.len());
        let mut index_by_package: HashMap<&str, usize> = HashMap::with_capacity(entries.len());
        for entry in entries {
            match index_by_package.get(entry.package.as_str()) {
                Some(&pos) => {
                    if entry.leanback && !chosen[pos].leanback {
                        chosen[pos] = entry;
                    }
                }
                None => {
                    index_by_package.insert(entry.package.as_str(), chosen.len());
                    chosen.push(entry);
                }
            }
        }
        chosen
    }

    fn sort(&mut self) {
        self.apps.sort_by_cached_key(|a| {
            (
                normalize::sort_key(&a.label),
                a.label.clone(),
                a.package.clone(),
            )
        });
    }

    pub fn apps(&self) -> &[App] {
        &self.apps
    }

    pub fn len(&self) -> usize {
        self.apps.len()
    }

    pub fn is_empty(&self) -> bool {
        self.apps.is_empty()
    }

    pub fn get(&self, package: &str) -> Option<&App> {
        self.apps.iter().find(|a| a.package == package)
    }

    pub fn contains(&self, package: &str) -> bool {
        self.get(package).is_some()
    }

    /// Applies the packages that were installed, updated or removed since the last time.
    /// `changed` replaces or adds apps; lume itself is ignored.
    pub fn apply(&mut self, changed: &[Discovered], removed: &[String]) -> Changes {
        let mut changes = Changes::default();
        for package in removed {
            if let Some(position) = self.apps.iter().position(|a| &a.package == package) {
                self.apps.remove(position);
                changes.removed.push(package.clone());
            }
        }
        for entry in Catalog::choose_activities(changed) {
            if entry.package == self.own_package {
                continue;
            }
            let app = App::from_discovered(entry);
            match self.apps.iter_mut().find(|a| a.package == app.package) {
                Some(existing) if *existing == app => {}
                Some(existing) => {
                    *existing = app;
                    changes.updated.push(entry.package.clone());
                }
                None => {
                    self.apps.push(app);
                    changes.added.push(entry.package.clone());
                }
            }
        }
        self.sort();
        changes
    }

    /// Replaces the catalog with a fresh full discovery and reports what differed.
    pub fn replace_with(&mut self, entries: &[Discovered]) -> Changes {
        let fresh = Catalog::from_discovered(entries, &self.own_package);
        let mut changes = Changes::default();
        let old_by_pkg: HashMap<&str, &App> =
            self.apps.iter().map(|a| (a.package.as_str(), a)).collect();
        let fresh_by_pkg: HashMap<&str, &App> =
            fresh.apps.iter().map(|a| (a.package.as_str(), a)).collect();

        for old in &self.apps {
            match fresh_by_pkg.get(old.package.as_str()) {
                None => changes.removed.push(old.package.clone()),
                Some(new) if *new != old => changes.updated.push(old.package.clone()),
                Some(_) => {}
            }
        }
        for new in &fresh.apps {
            if !old_by_pkg.contains_key(new.package.as_str()) {
                changes.added.push(new.package.clone());
            }
        }
        self.apps = fresh.apps;
        changes
    }

    fn to_kv(&self) -> Kv {
        let mut kv = Kv::with_capacity(1 + self.apps.len() * 5);
        kv.set("count", self.apps.len().to_string());
        let mut key_buf = String::with_capacity(32);
        for (i, app) in self.apps.iter().enumerate() {
            key_buf.clear();
            let _ = write!(key_buf, "a{i}.package");
            kv.set(&key_buf, app.package.as_str());
            key_buf.clear();
            let _ = write!(key_buf, "a{i}.label");
            kv.set(&key_buf, app.label.as_str());
            key_buf.clear();
            let _ = write!(key_buf, "a{i}.activity");
            kv.set(&key_buf, app.activity.as_str());
            key_buf.clear();
            let _ = write!(key_buf, "a{i}.color");
            kv.set(
                &key_buf,
                format!("{:02x}{:02x}{:02x}", app.color.r, app.color.g, app.color.b),
            );
            key_buf.clear();
            let _ = write!(key_buf, "a{i}.system");
            kv.set(&key_buf, if app.system { "1" } else { "0" });
        }
        kv
    }

    fn from_kv(kv: &Kv, own_package: &str) -> Option<Catalog> {
        let count: usize = kv.parse("count")?;
        let mut catalog = Catalog::new(own_package);
        catalog.apps.reserve(count);
        let mut key_buf = String::with_capacity(32);
        for i in 0..count {
            key_buf.clear();
            let _ = write!(key_buf, "a{i}.color");
            let hex = kv.get(&key_buf)?;
            let rgb = u32::from_str_radix(hex, 16)
                .ok()
                .filter(|_| hex.len() == 6)?;
            key_buf.clear();
            let _ = write!(key_buf, "a{i}.package");
            let package = kv.get(&key_buf)?.to_string();
            key_buf.clear();
            let _ = write!(key_buf, "a{i}.label");
            let label = kv.get(&key_buf)?.to_string();
            key_buf.clear();
            let _ = write!(key_buf, "a{i}.activity");
            let activity = kv.get(&key_buf)?.to_string();
            key_buf.clear();
            let _ = write!(key_buf, "a{i}.system");
            let system = match kv.get(&key_buf)? {
                "1" => true,
                "0" => false,
                _ => return None,
            };
            catalog.apps.push(App {
                package,
                label,
                activity,
                color: Color::rgb((rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8),
                system,
            });
        }
        catalog.apps.retain(|a| a.package != own_package);
        catalog.sort();
        Some(catalog)
    }

    /// Writes the cache file (`catalog.v1`).
    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        storage::save(path, CACHE_VERSION, &self.to_kv())
    }

    /// Reads the cache; a missing, unreadable or incompatible file is only a cache miss.
    pub fn load(path: &Path, own_package: &str) -> Option<Catalog> {
        match storage::load(path, CACHE_VERSION) {
            Ok(kv) => {
                let catalog = Catalog::from_kv(&kv, own_package);
                if catalog.is_none() {
                    log::warn!("ignoring {}: incomplete catalog entries", path.display());
                }
                catalog
            }
            Err(StorageError::Missing) => None,
            Err(e) => {
                log::warn!("ignoring {}: {e}", path.display());
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const OWN: &str = "app.lume.tv";

    fn entry(package: &str, label: &str, leanback: bool) -> Discovered {
        Discovered {
            package: package.to_string(),
            label: label.to_string(),
            activity: format!("{package}.Main"),
            leanback,
            system: false,
            icon: None,
        }
    }

    fn labels(catalog: &Catalog) -> Vec<&str> {
        catalog.apps().iter().map(|a| a.label.as_str()).collect()
    }

    #[test]
    fn tv_apps_are_listed() {
        let catalog = Catalog::from_discovered(&[entry("com.netflix.ninja", "Netflix", true)], OWN);
        assert_eq!(labels(&catalog), ["Netflix"]);
        assert_eq!(
            catalog.get("com.netflix.ninja").unwrap().activity,
            "com.netflix.ninja.Main"
        );
    }

    #[test]
    fn lume_is_left_out() {
        let catalog = Catalog::from_discovered(
            &[entry(OWN, "lume", true), entry("a.b", "Other", true)],
            OWN,
        );
        assert_eq!(labels(&catalog), ["Other"]);
        let mut catalog = catalog;
        let changes = catalog.apply(&[entry(OWN, "lume", true)], &[]);
        assert!(changes.is_empty());
        assert!(!catalog.contains(OWN));
    }

    #[test]
    fn an_app_with_several_activities_appears_once_preferring_the_tv_one() {
        let entries = [
            Discovered {
                activity: "phone.Main".into(),
                ..entry("a.b", "App", false)
            },
            Discovered {
                activity: "tv.Main".into(),
                ..entry("a.b", "App", true)
            },
            Discovered {
                activity: "other.Main".into(),
                ..entry("a.b", "App", false)
            },
        ];
        let catalog = Catalog::from_discovered(&entries, OWN);
        assert_eq!(catalog.len(), 1);
        assert_eq!(catalog.get("a.b").unwrap().activity, "tv.Main");
        // Without a TV activity the first plain launcher activity is used.
        let catalog = Catalog::from_discovered(&entries[..1], OWN);
        assert_eq!(catalog.get("a.b").unwrap().activity, "phone.Main");
    }

    #[test]
    fn apps_are_sorted_ignoring_case_and_accents() {
        let catalog = Catalog::from_discovered(
            &[
                entry("p.y", "YouTube", true),
                entry("p.n", "Netflix", true),
                entry("p.g", "Globoplay", true),
                entry("p.m", "Música", true),
                entry("p.c", "Chrome", true),
                entry("p.z", "ábaco", true),
            ],
            OWN,
        );
        assert_eq!(
            labels(&catalog),
            [
                "ábaco",
                "Chrome",
                "Globoplay",
                "Música",
                "Netflix",
                "YouTube"
            ]
        );
    }

    #[test]
    fn equal_names_have_a_stable_order() {
        let a = Catalog::from_discovered(
            &[entry("p.2", "Same", true), entry("p.1", "Same", true)],
            OWN,
        );
        let b = Catalog::from_discovered(
            &[entry("p.1", "Same", true), entry("p.2", "Same", true)],
            OWN,
        );
        assert_eq!(a, b);
    }

    #[test]
    fn known_apps_use_the_theme_color_and_others_derive_one() {
        let icon: Vec<u32> = vec![0xFF_FF_F0_20; color::ICON_SIZE * color::ICON_SIZE];
        let spotify = entry("com.spotify.tv.android", "Spotify", true);
        let unknown = Discovered {
            icon: Some(icon),
            ..entry("com.example.x", "Example", true)
        };
        let catalog = Catalog::from_discovered(&[spotify, unknown], OWN);
        assert_eq!(
            catalog.get("com.spotify.tv.android").unwrap().color,
            vars::BRAND_SPOTIFY
        );
        let derived = catalog.get("com.example.x").unwrap().color;
        assert!(
            color::contrast_with_white(derived) >= color::MIN_CONTRAST,
            "{derived:?}"
        );
        assert!(derived.r > derived.b, "the yellow hue is kept: {derived:?}");
    }

    #[test]
    fn apps_without_a_usable_icon_get_the_default_tile_color() {
        let catalog = Catalog::from_discovered(&[entry("com.example.x", "Example", true)], OWN);
        assert_eq!(
            catalog.get("com.example.x").unwrap().color,
            vars::TILE_DEFAULT
        );
    }

    #[test]
    fn installing_and_removing_apps_is_reported() {
        let mut catalog = Catalog::from_discovered(
            &[entry("p.n", "Netflix", true), entry("p.y", "YouTube", true)],
            OWN,
        );
        let changes = catalog.apply(
            &[entry("p.g", "Globoplay", true)],
            &["p.y".to_string(), "p.unknown".to_string()],
        );
        assert_eq!(
            changes,
            Changes {
                added: vec!["p.g".into()],
                removed: vec!["p.y".into()],
                updated: vec![]
            }
        );
        assert_eq!(labels(&catalog), ["Globoplay", "Netflix"]);
    }

    #[test]
    fn updating_an_app_replaces_it_and_unchanged_apps_are_not_reported() {
        let mut catalog = Catalog::from_discovered(&[entry("p.n", "Netflix", true)], OWN);
        assert!(catalog
            .apply(&[entry("p.n", "Netflix", true)], &[])
            .is_empty());
        let renamed = Discovered {
            label: "Netflix HD".into(),
            ..entry("p.n", "Netflix", true)
        };
        let changes = catalog.apply(&[renamed], &[]);
        assert_eq!(changes.updated, ["p.n"]);
        assert_eq!(labels(&catalog), ["Netflix HD"]);
    }

    #[test]
    fn a_full_rescan_reports_the_difference() {
        let mut catalog = Catalog::from_discovered(
            &[
                entry("p.a", "A", true),
                entry("p.b", "B", true),
                entry("p.c", "C", true),
            ],
            OWN,
        );
        let changed_c = Discovered {
            label: "C2".into(),
            ..entry("p.c", "C", true)
        };
        let changes =
            catalog.replace_with(&[entry("p.b", "B", true), changed_c, entry("p.d", "D", true)]);
        assert_eq!(changes.removed, ["p.a"]);
        assert_eq!(changes.updated, ["p.c"]);
        assert_eq!(changes.added, ["p.d"]);
        assert_eq!(labels(&catalog), ["B", "C2", "D"]);
    }

    fn temp(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("lume-catalog-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir.join("catalog.v1")
    }

    #[test]
    fn the_cache_round_trips() {
        let catalog = Catalog::from_discovered(
            &[
                entry("com.netflix.ninja", "Netflix", true),
                entry("p.m", "Música & Som\nlinha 2", true),
                entry("p.é", "Ürün", false),
                Discovered {
                    system: true,
                    ..entry("com.android.tv.settings", "Settings", true)
                },
            ],
            OWN,
        );
        assert!(catalog.get("com.android.tv.settings").unwrap().system);
        assert!(!catalog.get("p.m").unwrap().system);
        let path = temp("roundtrip");
        catalog.save(&path).unwrap();
        assert_eq!(Catalog::load(&path, OWN), Some(catalog));
    }

    #[test]
    fn a_bad_cache_is_a_miss_not_a_crash() {
        let path = temp("bad");
        assert_eq!(Catalog::load(&path, OWN), None);
        std::fs::write(&path, "garbage").unwrap();
        assert_eq!(Catalog::load(&path, OWN), None);
        // Valid file with a missing field.
        let mut kv = Kv::new();
        kv.set("count", "1");
        kv.set("a0.package", "p.a");
        storage::save(&path, CACHE_VERSION, &kv).unwrap();
        assert_eq!(Catalog::load(&path, OWN), None);
        // Unknown version.
        storage::save(&path, 9, &Kv::new()).unwrap();
        assert_eq!(Catalog::load(&path, OWN), None);
    }

    #[test]
    fn a_cache_of_the_previous_version_is_read_again() {
        // Version 1 had no system mark: it is a miss, so the apps are discovered again.
        let path = temp("v1");
        let mut kv = Kv::new();
        kv.set("count", "1");
        kv.set("a0.package", "p.a");
        kv.set("a0.label", "A");
        kv.set("a0.activity", "p.a.Main");
        kv.set("a0.color", "112233");
        storage::save(&path, 1, &kv).unwrap();
        assert_eq!(Catalog::load(&path, OWN), None);
        // The same entry with the mark, in the current version, is read.
        kv.set("a0.system", "1");
        storage::save(&path, CACHE_VERSION, &kv).unwrap();
        assert!(
            Catalog::load(&path, OWN)
                .unwrap()
                .get("p.a")
                .unwrap()
                .system
        );
    }

    #[test]
    fn a_cached_copy_of_lume_is_dropped() {
        let mut with_self = Catalog::new("other.own");
        with_self.apps.push(App {
            package: OWN.into(),
            label: "lume".into(),
            activity: "x".into(),
            color: Color::rgb(1, 2, 3),
            system: false,
        });
        let path = temp("self");
        with_self.save(&path).unwrap();
        assert!(Catalog::load(&path, OWN).unwrap().is_empty());
    }
}
