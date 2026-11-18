//! What HOME asks the TV: apps, launching, levels, radios, clock and stats.
//!
//! Everything goes over the Luna bus (see `luna`) or `/proc`. Every URI here was verified
//! as root on the OLED55C1PSA (webOS 6.5.3); see `docs/llm-wiki/lg-webos.md`.

use crate::icons;
use crate::luna;
use lume_core::catalog::Discovered;
use lume_core::clock::WallClock;
use lume_core::stats::{CpuReading, Stats};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

const TIMEOUT: Duration = Duration::from_millis(6500);

/// The ids SAM hides from launchers; HOME hides itself too.
const OWN_ID: &str = "app.home.tv";

fn call(uri: &str, payload: Value) -> Option<Value> {
    let reply = match luna::call(uri, &payload.to_string(), TIMEOUT) {
        Ok(reply) => reply,
        Err(e) => {
            log::warn!("{uri}: {e}");
            return None;
        }
    };
    let value: Value = match serde_json::from_str(&reply) {
        Ok(value) => value,
        Err(e) => {
            log::warn!("{uri}: invalid JSON ({e})");
            return None;
        }
    };
    if value.get("returnValue").and_then(Value::as_bool) == Some(false) {
        log::warn!("{uri}: refused: {}", &reply[..reply.len().min(240)]);
        return None;
    }
    Some(value)
}

/// One installed app as the launcher needs it, plus where its icon files live.
struct AppIcons {
    icon: Option<PathBuf>,
    large: Option<PathBuf>,
}

#[derive(Default)]
pub struct Bridge {
    /// Icon paths by app id, filled by the scans.
    icons: HashMap<String, AppIcons>,
}

fn icon_path(point: &Value, key: &str) -> Option<PathBuf> {
    let text = point.get(key)?.as_str()?.trim();
    let text = text.strip_prefix("file://").unwrap_or(text);
    (!text.is_empty() && text.ends_with(".png")).then(|| PathBuf::from(text))
}

impl Bridge {
    /// Every visible app, through `listLaunchPoints` (what LG's own Home lists).
    pub fn scan_all(&mut self) -> Vec<Discovered> {
        let Some(reply) = call(
            "luna://com.webos.applicationManager/listLaunchPoints",
            json!({}),
        ) else {
            return Vec::new();
        };
        let Some(points) = reply.get("launchPoints").and_then(Value::as_array) else {
            log::warn!("listLaunchPoints: no launchPoints array");
            return Vec::new();
        };
        let mut apps = Vec::new();
        for point in points {
            let Some(id) = point.get("id").and_then(Value::as_str) else {
                continue;
            };
            if id == OWN_ID {
                continue;
            }
            let title = point
                .get("title")
                .and_then(Value::as_str)
                .unwrap_or(id)
                .to_string();
            let system = point.get("systemApp").and_then(Value::as_bool) == Some(true)
                || point.get("removable").and_then(Value::as_bool) == Some(false)
                || ["com.webos.", "com.palm.", "com.lge."]
                    .iter()
                    .any(|prefix| id.starts_with(prefix));
            let icons = AppIcons {
                icon: icon_path(point, "icon"),
                large: icon_path(point, "largeIcon"),
            };
            // The color thumbnail comes from the best icon file.
            let icon = icons
                .large
                .as_deref()
                .or(icons.icon.as_deref())
                .and_then(icons::decode_png)
                .map(|decoded| {
                    icons::color_thumbnail(&decoded, lume_core::catalog::color::ICON_SIZE)
                });
            self.icons.insert(id.to_string(), icons);
            apps.push(Discovered {
                package: id.to_string(),
                label: title,
                // webOS launches by id alone; the activity field stays empty.
                activity: String::new(),
                leanback: true,
                system,
                icon,
            });
        }
        apps
    }

    /// The icon paths of `package`, scanning once if this process has not scanned yet
    /// (the catalog cache skips the scan on later runs).
    fn icon_paths(&mut self, package: &str) -> Option<(Option<PathBuf>, Option<PathBuf>)> {
        if self.icons.is_empty() {
            self.scan_all();
        }
        self.icons
            .get(package)
            .map(|icons| (icons.large.clone(), icons.icon.clone()))
    }

    /// The tile image of `package`: `cell`-sized premultiplied RGBA, always [`Art::Icon`]
    /// (webOS apps have no TV banners).
    pub fn tile_art(
        &mut self,
        package: &str,
        cell: [u32; 2],
    ) -> Option<(Vec<u8>, lume_core::app_icons::Art)> {
        let (large, small) = self.icon_paths(package)?;
        let decoded = large
            .as_deref()
            .and_then(icons::decode_png)
            .or_else(|| small.as_deref().and_then(icons::decode_png))?;
        Some((
            icons::fit_cell(&decoded, cell[0], cell[1]),
            lume_core::app_icons::Art::Icon,
        ))
    }

    /// The dock icon of `package`: `side` x `side` premultiplied RGBA.
    pub fn dock_icon(&mut self, package: &str, side: u32) -> Option<Vec<u8>> {
        let (large, small) = self.icon_paths(package)?;
        let decoded = large
            .as_deref()
            .and_then(icons::decode_png)
            .or_else(|| small.as_deref().and_then(icons::decode_png))?;
        Some(icons::fit_cell(&decoded, side, side))
    }

    /// Launches an app (or an input: webOS inputs are apps) by id.
    pub fn launch(&self, id: &str) -> bool {
        call(
            "luna://com.webos.applicationManager/launch",
            json!({ "id": id }),
        )
        .is_some()
    }

    /// The media volume (0 to 100), and whether it can be read at all.
    pub fn volume(&self) -> Option<u8> {
        let reply = call("luna://com.webos.service.audio/master/getVolume", json!({}))?;
        let status = reply.get("volumeStatus")?;
        status
            .get("volume")
            .and_then(Value::as_u64)
            .map(|v| v.min(100) as u8)
    }

    pub fn set_volume(&self, percent: u8) -> bool {
        call(
            "luna://com.webos.service.audio/master/setVolume",
            json!({ "volume": percent.min(100) }),
        )
        .is_some()
    }

    /// The OLED light (what webOS still calls `backlight`), 0 to 100.
    pub fn backlight(&self) -> Option<u8> {
        let reply = call(
            "luna://com.webos.settingsservice/getSystemSettings",
            json!({ "category": "picture", "keys": ["backlight"] }),
        )?;
        let value = reply.get("settings")?.get("backlight")?;
        // The settings service stores numbers as strings or numbers depending on the key.
        value
            .as_u64()
            .or_else(|| value.as_str().and_then(|s| s.parse().ok()))
            .map(|v| v.min(100) as u8)
    }

    pub fn set_backlight(&self, percent: u8) -> bool {
        call(
            "luna://com.webos.settingsservice/setSystemSettings",
            json!({ "category": "picture", "settings": { "backlight": percent.min(100).to_string() } }),
        )
        .is_some()
    }

    /// Whether Wi-Fi is connected and Bluetooth powered.
    pub fn radios(&self) -> Option<(bool, bool)> {
        let wifi = call(
            "luna://com.webos.service.connectionmanager/getStatus",
            json!({}),
        )
        .map(|reply| {
            reply
                .get("wifi")
                .and_then(|w| w.get("state"))
                .and_then(Value::as_str)
                == Some("connected")
        })?;
        let bluetooth = call(
            "luna://com.webos.service.bluetooth2/adapter/getStatus",
            json!({}),
        )
        .and_then(|reply| {
            Some(
                reply
                    .get("adapters")?
                    .as_array()?
                    .first()?
                    .get("powered")
                    .and_then(Value::as_bool)
                    == Some(true),
            )
        })
        .unwrap_or(false);
        Some((wifi, bluetooth))
    }

    /// The UI locale list, for the translator.
    pub fn locales(&self) -> Vec<String> {
        let locale = call(
            "luna://com.webos.settingsservice/getSystemSettings",
            json!({ "keys": ["localeInfo"] }),
        )
        .and_then(|reply| {
            Some(
                reply
                    .get("settings")?
                    .get("localeInfo")?
                    .get("locales")?
                    .get("UI")?
                    .as_str()?
                    .to_string(),
            )
        });
        match locale {
            Some(locale) => vec![locale],
            None => Vec::new(),
        }
    }

    /// Wall time, zone offset and the 12/24-hour choice.
    pub fn clock(&self) -> WallClock {
        let clock_setting = call(
            "luna://com.webos.settingsservice/getSystemSettings",
            json!({ "keys": ["localeInfo"] }),
        )
        .and_then(|reply| {
            Some(
                reply
                    .get("settings")?
                    .get("localeInfo")?
                    .get("clock")?
                    .as_str()?
                    .to_string(),
            )
        })
        .unwrap_or_default();
        let locales = self.locales();
        let locale = locales.first().map(String::as_str).unwrap_or("");
        let is_24h = match clock_setting.as_str() {
            "24" => true,
            "12" => false,
            // shortcut: "locale" maps the two formats this TV can realistically show;
            // read CLDR properly if more regions appear.
            _ => !matches!(locale, l if l.starts_with("es-MX") || l.starts_with("en-US")),
        };
        let (unix_secs, utc_offset_secs) = local_time();
        WallClock {
            unix_secs,
            utc_offset_secs,
            is_24h,
        }
    }

    /// Memory, disk and CPU for the settings screen, from `/proc` and `statvfs`.
    pub fn stats(&self, disk_path: &std::path::Path) -> Option<Stats> {
        let meminfo = std::fs::read_to_string("/proc/meminfo").ok()?;
        let field = |name: &str| {
            meminfo
                .lines()
                .find(|l| l.starts_with(name))?
                .split_whitespace()
                .nth(1)?
                .parse::<u64>()
                .ok()
        };
        let memory_total_kb = field("MemTotal:")?;
        let memory_available_kb = field("MemAvailable:")?;
        let (disk_total, disk_free) = disk_space(disk_path)?;
        let cpu = cpu_reading().unwrap_or(CpuReading::Unavailable);
        (memory_total_kb > 0 && disk_total > 0).then_some(Stats {
            memory_total_kb,
            memory_available_kb: memory_available_kb.min(memory_total_kb),
            disk_total,
            disk_free: disk_free.min(disk_total),
            cpu,
        })
    }

    /// Model, firmware and webOS version for the TV information screen.
    pub fn system_info(&self) -> Option<(String, String, String)> {
        let reply = call(
            "luna://com.webos.service.tv.systemproperty/getSystemInfo",
            json!({ "keys": ["modelName", "firmwareVersion", "sdkVersion"] }),
        )?;
        let text = |key: &str| {
            reply
                .get(key)
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string()
        };
        Some((
            text("modelName"),
            text("firmwareVersion"),
            text("sdkVersion"),
        ))
    }

    /// Puts the panel to sleep (screen off; the TV stays on and any key wakes it).
    pub fn screen_off(&self) -> bool {
        call(
            "luna://com.webos.service.tvpower/power/turnOffScreen",
            json!({ "standbyMode": "active" }),
        )
        .is_some()
    }
}

/// Seconds since the epoch and the local zone's offset.
fn local_time() -> (u64, i32) {
    // SAFETY: time and localtime_r with valid pointers.
    unsafe {
        let now = libc::time(std::ptr::null_mut());
        let mut tm: libc::tm = std::mem::zeroed();
        if libc::localtime_r(&now, &mut tm).is_null() {
            return (now.max(0) as u64, 0);
        }
        (now.max(0) as u64, tm.tm_gmtoff as i32)
    }
}

fn disk_space(path: &std::path::Path) -> Option<(u64, u64)> {
    let path = std::ffi::CString::new(path.as_os_str().as_encoded_bytes()).ok()?;
    // SAFETY: statvfs with a valid path and out-pointer.
    unsafe {
        let mut stat: libc::statvfs = std::mem::zeroed();
        if libc::statvfs(path.as_ptr(), &mut stat) != 0 {
            return None;
        }
        let block = stat.f_frsize.max(1) as u64;
        Some((stat.f_blocks as u64 * block, stat.f_bavail as u64 * block))
    }
}

/// Jiffies busy and total, from the `cpu` line of `/proc/stat`.
fn cpu_reading() -> Option<CpuReading> {
    let stat = std::fs::read_to_string("/proc/stat").ok()?;
    let line = stat.lines().find(|l| l.starts_with("cpu "))?;
    let values: Vec<u64> = line
        .split_whitespace()
        .skip(1)
        .filter_map(|v| v.parse().ok())
        .collect();
    if values.len() < 4 {
        return None;
    }
    let total: u64 = values.iter().sum();
    let idle = values[3] + values.get(4).copied().unwrap_or(0);
    Some(CpuReading::Busy {
        busy: total.saturating_sub(idle),
        total,
    })
}
