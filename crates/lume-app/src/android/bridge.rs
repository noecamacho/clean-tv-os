//! The Java side, reached through one JNI call: `LumeActivity.bridge(command, argument)`.

use android_activity::AndroidApp;
use jni::objects::{JByteArray, JObject, JString};
use jni::{jni_sig, jni_str, JValue, JavaVM};
use lume_core::app_icons::Art;
use lume_core::catalog::known::KnownApp;
use lume_core::catalog::{Catalog, Discovered};
use lume_core::clock::WallClock;
use lume_core::shell::protocol::{self, Changed};
use std::ffi::c_void;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct Bridge {
    vm: JavaVM,
    /// The activity, as an unowned JNI global reference kept alive by android-activity.
    activity: *mut c_void,
}

impl Bridge {
    pub fn new(app: &AndroidApp) -> Bridge {
        // SAFETY: the pointer comes from android-activity and stays valid while the app runs.
        let vm = unsafe { JavaVM::from_raw(app.vm_as_ptr().cast()) };
        Bridge {
            vm,
            activity: app.activity_as_ptr(),
        }
    }

    /// Runs a bridge command. `None` when the call failed (the reason is logged).
    pub fn call(&self, command: &str, argument: &str) -> Option<String> {
        let result = self
            .vm
            .attach_current_thread(|env| -> jni::errors::Result<Option<String>> {
                // SAFETY: `activity` is a valid global reference to the activity object.
                let activity = unsafe { JObject::from_raw(env, self.activity.cast()) };
                let command_arg = env.new_string(command)?;
                let argument_arg = env.new_string(argument)?;
                let answer = env.call_method(
                    &activity,
                    jni_str!("bridge"),
                    jni_sig!((command: JString, argument: JString) -> JString),
                    &[JValue::Object(&command_arg), JValue::Object(&argument_arg)],
                )?;
                let object = answer.l()?;
                if object.is_null() {
                    return Ok(None);
                }
                let text = env.cast_local::<JString>(object)?;
                Ok(Some(text.try_to_string(env)?))
            });
        match result {
            Ok(answer) => answer,
            Err(e) => {
                log::warn!("bridge command '{command}' failed: {e}");
                None
            }
        }
    }

    /// What a tile shows for an app: `cell` (width and height) premultiplied RGBA bytes and whether
    /// they are the TV banner or the icon.
    pub fn tile_art(
        &self,
        package: &str,
        activity: &str,
        cell: [u32; 2],
    ) -> Option<(Vec<u8>, Art)> {
        let result = self
            .vm
            .attach_current_thread(|env| -> jni::errors::Result<Option<(Vec<u8>, Art)>> {
                // SAFETY: `activity` is a valid global reference to the activity object.
                let activity_object = unsafe { JObject::from_raw(env, self.activity.cast()) };
                let package_arg = env.new_string(package)?;
                let activity_arg = env.new_string(activity)?;
                let answer = env.call_method(
                    &activity_object,
                    jni_str!("iconPixels"),
                    jni_sig!((package: JString, activity: JString, width: jint, height: jint) -> jbyte[]),
                    &[
                        JValue::Object(&package_arg),
                        JValue::Object(&activity_arg),
                        JValue::Int(cell[0] as i32),
                        JValue::Int(cell[1] as i32),
                    ],
                )?;
                let object = answer.l()?;
                if object.is_null() {
                    return Ok(None);
                }
                let bytes = env.cast_local::<JByteArray>(object)?;
                let mut pixels = env.convert_byte_array(&bytes)?;
                // The last byte says what the pixels are.
                let art = match pixels.pop() {
                    Some(1) => Art::Banner,
                    _ => Art::Icon,
                };
                Ok(Some((pixels, art)))
            });
        match result {
            Ok(pixels) => pixels,
            Err(e) => {
                log::warn!("icon of {package} failed: {e}");
                None
            }
        }
    }

    /// The square icon of an app for the dock: `side` x `side` premultiplied RGBA bytes.
    pub fn dock_icon(&self, package: &str, activity: &str, side: u32) -> Option<Vec<u8>> {
        let result = self
            .vm
            .attach_current_thread(|env| -> jni::errors::Result<Option<Vec<u8>>> {
                // SAFETY: `activity` is a valid global reference to the activity object.
                let activity_object = unsafe { JObject::from_raw(env, self.activity.cast()) };
                let package_arg = env.new_string(package)?;
                let activity_arg = env.new_string(activity)?;
                let answer = env.call_method(
                    &activity_object,
                    jni_str!("dockIcon"),
                    jni_sig!((package: JString, activity: JString, size: jint) -> jbyte[]),
                    &[
                        JValue::Object(&package_arg),
                        JValue::Object(&activity_arg),
                        JValue::Int(side as i32),
                    ],
                )?;
                let object = answer.l()?;
                if object.is_null() {
                    return Ok(None);
                }
                let bytes = env.cast_local::<JByteArray>(object)?;
                Ok(Some(env.convert_byte_array(&bytes)?))
            });
        match result {
            Ok(pixels) => pixels.filter(|p| protocol::is_dock_icon(p, side)),
            Err(e) => {
                log::warn!("dock icon of {package} failed: {e}");
                None
            }
        }
    }

    pub fn package_name(&self) -> String {
        self.call("package", "")
            .filter(|p| !p.is_empty())
            .unwrap_or_else(|| "app.lume.tv".to_string())
    }

    pub fn locales(&self) -> Vec<String> {
        self.call("locales", "")
            .map(|t| protocol::parse_locales(&t))
            .unwrap_or_default()
    }

    /// The wall clock: the system's seconds, with the zone and 12/24-hour setting from Java.
    pub fn clock(&self) -> WallClock {
        let (is_24h, offset) = self
            .call("clock", "")
            .and_then(|t| protocol::parse_clock(&t))
            .unwrap_or((true, 0));
        WallClock {
            unix_secs: unix_seconds(),
            utc_offset_secs: offset,
            is_24h,
        }
    }

    /// Opens an app. Returns `false` when it cannot be started (for example it was uninstalled).
    pub fn start_app(&self, package: &str, activity: &str) -> bool {
        self.call("start", &protocol::start_argument(package, activity))
            .as_deref()
            == Some("ok")
    }

    pub fn open_android_settings(&self) -> bool {
        self.call("settings", "").as_deref() == Some("ok")
    }

    /// Memory, disk and CPU of the TV, for the settings panel.
    pub fn stats(&self) -> Option<lume_core::stats::Stats> {
        let mut stats = lume_core::stats::Stats::parse(&self.call("stats", "")?)?;
        if let Some(freq) = lume_core::stats::read_sysfs_cpu_freq() {
            stats.cpu = freq;
        }
        Some(stats)
    }

    /// The interface size, refresh rate and density, for the TV information screen.
    pub fn display(&self) -> Option<[lume_core::device_info::Value; 3]> {
        lume_core::device_info::parse_display(&self.call("display", "")?)
    }

    /// Whether the system has a TV inputs panel to open.
    pub fn inputs_available(&self) -> bool {
        self.call("inputs", "").as_deref() == Some("1")
    }

    pub fn open_inputs(&self) -> bool {
        self.call("open-inputs", "").as_deref() == Some("ok")
    }

    pub fn open_setting(&self, name: &str) -> bool {
        self.call("open-setting", name).as_deref() == Some("ok")
    }

    /// Opens the app's page in the Android settings.
    pub fn open_app_info(&self, package: &str) -> bool {
        self.call("app-info", package).as_deref() == Some("ok")
    }

    /// Opens the system uninstaller for the app. Its result arrives as a package change.
    pub fn uninstall(&self, package: &str) -> bool {
        self.call("uninstall", package).as_deref() == Some("ok")
    }

    /// Every launchable app. Icons are fetched only for apps that need a color derived from them:
    /// not the well-known ones and not the ones already in `known`.
    pub fn scan_all(&self, known: Option<&Catalog>) -> Vec<Discovered> {
        let apps = self
            .call("apps", "")
            .map(|t| protocol::parse_apps(&t))
            .unwrap_or_default();
        self.with_icons(apps, known)
    }

    /// The launchable activities of one package, with icons where needed.
    pub fn query(&self, package: &str) -> Vec<Discovered> {
        let apps = self
            .call("query", package)
            .map(|t| protocol::parse_apps(&t))
            .unwrap_or_default();
        self.with_icons(apps, None)
    }

    pub fn changed(&self) -> Changed {
        match self.call("changed", "") {
            Some(text) => protocol::parse_changed(&text),
            None => Changed::Packages(Vec::new()),
        }
    }

    fn with_icons(&self, mut apps: Vec<Discovered>, known: Option<&Catalog>) -> Vec<Discovered> {
        let mut icons: Vec<(String, Option<Vec<u32>>)> = Vec::new();
        for app in &mut apps {
            if KnownApp::from_package(&app.package).is_some()
                || known.is_some_and(|c| c.contains(&app.package))
            {
                continue;
            }
            let icon = match icons.iter().find(|(package, _)| *package == app.package) {
                Some((_, icon)) => icon.clone(),
                None => {
                    let icon = self
                        .call("icon", &app.package)
                        .and_then(|t| protocol::parse_icon(&t));
                    icons.push((app.package.clone(), icon.clone()));
                    icon
                }
            };
            app.icon = icon;
        }
        apps
    }
}

pub fn unix_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}
