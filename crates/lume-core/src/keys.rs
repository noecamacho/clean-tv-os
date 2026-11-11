//! Android remote control keys, keycode mappings, and persistent configuration.

use crate::i18n::{Arg, StringId, Translator};
use crate::storage::{self, Kv, StorageError};
use std::path::Path;

/// Remote control keys.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Key {
    Up = 0,
    Down = 1,
    Left = 2,
    Right = 3,
    /// OK, Enter or the center of the D-pad.
    Ok = 4,
    Back = 5,
    Home = 6,
    /// Opens the context menu of the focused item, where it has one.
    Menu = 7,
}

impl Key {
    pub const COUNT: usize = 8;
    pub const ALL: [Key; Key::COUNT] = [
        Key::Up,
        Key::Down,
        Key::Left,
        Key::Right,
        Key::Ok,
        Key::Back,
        Key::Home,
        Key::Menu,
    ];

    #[inline]
    pub const fn index(self) -> usize {
        self as usize
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Key::Up => "up",
            Key::Down => "down",
            Key::Left => "left",
            Key::Right => "right",
            Key::Ok => "ok",
            Key::Back => "back",
            Key::Home => "home",
            Key::Menu => "menu",
        }
    }

    pub fn parse_name(name: &str) -> Option<Key> {
        match name.trim().to_ascii_lowercase().as_str() {
            "up" => Some(Key::Up),
            "down" => Some(Key::Down),
            "left" => Some(Key::Left),
            "right" => Some(Key::Right),
            "ok" | "enter" => Some(Key::Ok),
            "back" => Some(Key::Back),
            "home" => Some(Key::Home),
            "menu" => Some(Key::Menu),
            _ => None,
        }
    }

    pub fn default_codes(self) -> &'static [i32] {
        match self {
            Key::Up => DEFAULT_UP,
            Key::Down => DEFAULT_DOWN,
            Key::Left => DEFAULT_LEFT,
            Key::Right => DEFAULT_RIGHT,
            Key::Ok => DEFAULT_OK,
            Key::Back => DEFAULT_BACK,
            Key::Home => DEFAULT_HOME,
            Key::Menu => DEFAULT_MENU,
        }
    }
}

pub const KEYCODE_HOME: i32 = 3;
pub const KEYCODE_BACK: i32 = 4;
pub const KEYCODE_0: i32 = 7;
pub const KEYCODE_1: i32 = 8;
pub const KEYCODE_2: i32 = 9;
pub const KEYCODE_3: i32 = 10;
pub const KEYCODE_4: i32 = 11;
pub const KEYCODE_5: i32 = 12;
pub const KEYCODE_6: i32 = 13;
pub const KEYCODE_7: i32 = 14;
pub const KEYCODE_8: i32 = 15;
pub const KEYCODE_9: i32 = 16;
pub const KEYCODE_DPAD_UP: i32 = 19;
pub const KEYCODE_DPAD_DOWN: i32 = 20;
pub const KEYCODE_DPAD_LEFT: i32 = 21;
pub const KEYCODE_DPAD_RIGHT: i32 = 22;
pub const KEYCODE_DPAD_CENTER: i32 = 23;
pub const KEYCODE_VOLUME_UP: i32 = 24;
pub const KEYCODE_VOLUME_DOWN: i32 = 25;
pub const KEYCODE_POWER: i32 = 26;
pub const KEYCODE_ENTER: i32 = 66;
/// The Menu button of the remote and of a keyboard.
pub const KEYCODE_MENU: i32 = 82;
pub const KEYCODE_MEDIA_PLAY_PAUSE: i32 = 85;
pub const KEYCODE_MEDIA_STOP: i32 = 86;
pub const KEYCODE_MEDIA_NEXT: i32 = 87;
pub const KEYCODE_MEDIA_PREVIOUS: i32 = 88;
pub const KEYCODE_MEDIA_REWIND: i32 = 89;
pub const KEYCODE_MEDIA_FAST_FORWARD: i32 = 90;
/// "Select" on some remotes and game controllers.
pub const KEYCODE_BUTTON_A: i32 = 96;
pub const KEYCODE_BUTTON_B: i32 = 97;
pub const KEYCODE_ESCAPE: i32 = 111;
pub const KEYCODE_MEDIA_PLAY: i32 = 126;
pub const KEYCODE_MEDIA_PAUSE: i32 = 127;
pub const KEYCODE_F1: i32 = 131;
pub const KEYCODE_F2: i32 = 132;
pub const KEYCODE_F3: i32 = 133;
pub const KEYCODE_F4: i32 = 134;
pub const KEYCODE_F5: i32 = 135;
pub const KEYCODE_F6: i32 = 136;
pub const KEYCODE_F7: i32 = 137;
pub const KEYCODE_F8: i32 = 138;
pub const KEYCODE_F9: i32 = 139;
pub const KEYCODE_F10: i32 = 140;
pub const KEYCODE_F11: i32 = 141;
pub const KEYCODE_F12: i32 = 142;
pub const KEYCODE_NUMPAD_ENTER: i32 = 160;
pub const KEYCODE_VOLUME_MUTE: i32 = 164;
pub const KEYCODE_INFO: i32 = 165;
pub const KEYCODE_CHANNEL_UP: i32 = 166;
pub const KEYCODE_CHANNEL_DOWN: i32 = 167;
pub const KEYCODE_TV: i32 = 170;
pub const KEYCODE_GUIDE: i32 = 172;
/// What the Menu button of the AIWA (KTC) remote sends; its firmware labels it "bookmark".
pub const KEYCODE_BOOKMARK: i32 = 174;
pub const KEYCODE_CAPTIONS: i32 = 175;
pub const KEYCODE_SETTINGS: i32 = 176;
pub const KEYCODE_TV_POWER: i32 = 177;
pub const KEYCODE_TV_INPUT: i32 = 178;
pub const KEYCODE_PROG_RED: i32 = 183;
pub const KEYCODE_PROG_GREEN: i32 = 184;
pub const KEYCODE_PROG_YELLOW: i32 = 185;
pub const KEYCODE_PROG_BLUE: i32 = 186;
pub const KEYCODE_SLEEP: i32 = 223;
pub const KEYCODE_WAKEUP: i32 = 224;
pub const KEYCODE_SOFT_SLEEP: i32 = 276;

/// Hardware power and sleep keys reserved by the TV system.
pub const POWER_KEYCODES: &[i32] = &[
    KEYCODE_POWER,
    KEYCODE_TV_POWER,
    KEYCODE_SLEEP,
    KEYCODE_WAKEUP,
    KEYCODE_SOFT_SLEEP,
];

/// Returns whether `code` is a D-pad directional arrow (Up, Down, Left, Right).
pub fn is_dpad_code(code: i32) -> bool {
    DEFAULT_UP.contains(&code)
        || DEFAULT_DOWN.contains(&code)
        || DEFAULT_LEFT.contains(&code)
        || DEFAULT_RIGHT.contains(&code)
}

/// Returns whether `code` is a Back key (the only way to cancel the key listening).
pub fn is_back_code(code: i32) -> bool {
    DEFAULT_BACK.contains(&code)
}

/// Returns whether `code` is an OK/confirm key.
pub fn is_ok_code(code: i32) -> bool {
    DEFAULT_OK.contains(&code)
}

/// Returns whether `code` is a reserved key: it keeps its native behavior and is never recorded
/// by the key listening (D-pad arrows, OK, Home, Power, Sleep, Wake).
pub fn is_reserved_code(code: i32) -> bool {
    is_dpad_code(code)
        || is_ok_code(code)
        || DEFAULT_HOME.contains(&code)
        || POWER_KEYCODES.contains(&code)
}

/// Returns whether `code` is protected against custom mapping (reserved or Back).
pub fn is_protected_code(code: i32) -> bool {
    is_reserved_code(code) || is_back_code(code)
}

/// Returns the friendly name for an Android keycode, if known.
pub fn keycode_name(code: i32) -> Option<&'static str> {
    match code {
        KEYCODE_MENU => Some("Menu"),
        KEYCODE_BOOKMARK => Some("Favorite"),
        KEYCODE_TV_INPUT => Some("Input"),
        KEYCODE_GUIDE => Some("Guide"),
        KEYCODE_INFO => Some("Info"),
        KEYCODE_CAPTIONS => Some("Captions"),
        KEYCODE_TV => Some("TV"),
        KEYCODE_PROG_RED => Some("Red"),
        KEYCODE_PROG_GREEN => Some("Green"),
        KEYCODE_PROG_YELLOW => Some("Yellow"),
        KEYCODE_PROG_BLUE => Some("Blue"),
        KEYCODE_CHANNEL_UP => Some("Channel +"),
        KEYCODE_CHANNEL_DOWN => Some("Channel -"),
        KEYCODE_VOLUME_UP => Some("Volume +"),
        KEYCODE_VOLUME_DOWN => Some("Volume -"),
        KEYCODE_VOLUME_MUTE => Some("Mute"),
        KEYCODE_MEDIA_PLAY_PAUSE => Some("Play/Pause"),
        KEYCODE_MEDIA_STOP => Some("Stop"),
        KEYCODE_MEDIA_FAST_FORWARD => Some("Fast Forward"),
        KEYCODE_MEDIA_REWIND => Some("Rewind"),
        KEYCODE_MEDIA_PLAY => Some("Play"),
        KEYCODE_MEDIA_PAUSE => Some("Pause"),
        KEYCODE_MEDIA_NEXT => Some("Next"),
        KEYCODE_MEDIA_PREVIOUS => Some("Previous"),
        KEYCODE_SETTINGS => Some("Settings"),
        KEYCODE_0 => Some("0"),
        KEYCODE_1 => Some("1"),
        KEYCODE_2 => Some("2"),
        KEYCODE_3 => Some("3"),
        KEYCODE_4 => Some("4"),
        KEYCODE_5 => Some("5"),
        KEYCODE_6 => Some("6"),
        KEYCODE_7 => Some("7"),
        KEYCODE_8 => Some("8"),
        KEYCODE_9 => Some("9"),
        KEYCODE_F1 => Some("F1"),
        KEYCODE_F2 => Some("F2"),
        KEYCODE_F3 => Some("F3"),
        KEYCODE_F4 => Some("F4"),
        KEYCODE_F5 => Some("F5"),
        KEYCODE_F6 => Some("F6"),
        KEYCODE_F7 => Some("F7"),
        KEYCODE_F8 => Some("F8"),
        KEYCODE_F9 => Some("F9"),
        KEYCODE_F10 => Some("F10"),
        KEYCODE_F11 => Some("F11"),
        KEYCODE_F12 => Some("F12"),
        _ => None,
    }
}

/// Returns a human-readable label for a keycode, e.g. "Menu (82)" or "Key 999".
pub fn keycode_label(code: i32) -> String {
    if let Some(name) = keycode_name(code) {
        format!("{name} ({code})")
    } else {
        format!("Key {code}")
    }
}

/// The localized name of a keycode, when it has one.
fn localized_name(code: i32, t: &Translator) -> Option<String> {
    let id = match code {
        KEYCODE_MENU => StringId::KeyMenu,
        KEYCODE_BOOKMARK => StringId::KeyFavorite,
        KEYCODE_TV_INPUT => StringId::KeyInput,
        KEYCODE_GUIDE => StringId::KeyGuide,
        KEYCODE_INFO => StringId::KeyInfo,
        KEYCODE_CAPTIONS => StringId::KeyCaptions,
        KEYCODE_TV => StringId::KeyTv,
        KEYCODE_PROG_RED => StringId::KeyRed,
        KEYCODE_PROG_GREEN => StringId::KeyGreen,
        KEYCODE_PROG_YELLOW => StringId::KeyYellow,
        KEYCODE_PROG_BLUE => StringId::KeyBlue,
        KEYCODE_CHANNEL_UP => StringId::KeyChannelUp,
        KEYCODE_CHANNEL_DOWN => StringId::KeyChannelDown,
        KEYCODE_VOLUME_UP => StringId::KeyVolumeUp,
        KEYCODE_VOLUME_DOWN => StringId::KeyVolumeDown,
        KEYCODE_VOLUME_MUTE => StringId::KeyVolumeMute,
        KEYCODE_MEDIA_PLAY_PAUSE => StringId::KeyPlayPause,
        KEYCODE_MEDIA_STOP => StringId::KeyStop,
        KEYCODE_MEDIA_FAST_FORWARD => StringId::KeyFastForward,
        KEYCODE_MEDIA_REWIND => StringId::KeyRewind,
        KEYCODE_SETTINGS => StringId::KeySettings,
        _ => return keycode_name(code).map(str::to_string),
    };
    Some(t.get(id).into_owned())
}

/// The two parts of a key's caption: its name and its code (`("Favorito", Some("174"))`). A key
/// without a name is shown whole in the first part (`("Tecla 999", None)`).
pub fn keycode_parts(code: i32, t: &Translator) -> (String, Option<String>) {
    match localized_name(code, t) {
        Some(name) => (name, Some(code.to_string())),
        None => (
            t.format(StringId::KeyFallback, &[Arg::Int(code as i64)]),
            None,
        ),
    }
}

/// Returns the localized display label for a keycode using the given translator.
pub fn keycode_display(code: i32, t: &Translator) -> String {
    match keycode_parts(code, t) {
        (name, Some(code)) => format!("{name} ({code})"),
        (name, None) => name,
    }
}

/// Commands of the Lume launcher that can be mapped to remote keys.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum CommandTarget {
    Menu,
    Inputs,
}

impl CommandTarget {
    pub const ALL: [CommandTarget; 2] = [CommandTarget::Menu, CommandTarget::Inputs];

    pub const fn as_str(self) -> &'static str {
        match self {
            CommandTarget::Menu => "menu",
            CommandTarget::Inputs => "inputs",
        }
    }

    pub fn parse_name(name: &str) -> Option<CommandTarget> {
        match name.trim().to_ascii_lowercase().as_str() {
            "menu" => Some(CommandTarget::Menu),
            "inputs" => Some(CommandTarget::Inputs),
            _ => None,
        }
    }

    pub fn default_codes(self) -> &'static [i32] {
        match self {
            CommandTarget::Menu => DEFAULT_MENU,
            CommandTarget::Inputs => DEFAULT_INPUTS,
        }
    }

    pub fn title_id(self) -> StringId {
        match self {
            CommandTarget::Menu => StringId::CommandMenu,
            CommandTarget::Inputs => StringId::CommandInputs,
        }
    }
}

/// Shortcuts to specific Android TV or Lume settings screens.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum SettingShortcut {
    Sleep,
    Picture,
    Sound,
    Wifi,
    Bluetooth,
    Apps,
    Android,
    LumeSettings,
    LumeEditor,
    LumeTvInfo,
    Storage,
    Language,
    DateTime,
    Keyboard,
    Captions,
    Location,
    Security,
    Developer,
    HomeApp,
    About,
}

/// Every shortcut with its name in `keys.v1` (`cfg.<name>`), in the order the "Choose a shortcut"
/// screen lists them: the TV's own settings first, then the system's, then Lume's.
const SETTING_NAMES: [(SettingShortcut, &str); 20] = [
    (SettingShortcut::Sleep, "sleep"),
    (SettingShortcut::Picture, "picture"),
    (SettingShortcut::Sound, "sound"),
    (SettingShortcut::Wifi, "wifi"),
    (SettingShortcut::Bluetooth, "bluetooth"),
    (SettingShortcut::Apps, "apps"),
    (SettingShortcut::Storage, "storage"),
    (SettingShortcut::Language, "language"),
    (SettingShortcut::DateTime, "datetime"),
    (SettingShortcut::Keyboard, "keyboard"),
    (SettingShortcut::Captions, "captions"),
    (SettingShortcut::Location, "location"),
    (SettingShortcut::Security, "security"),
    (SettingShortcut::Developer, "developer"),
    (SettingShortcut::HomeApp, "home_app"),
    (SettingShortcut::About, "about"),
    (SettingShortcut::Android, "android"),
    (SettingShortcut::LumeSettings, "lume_settings"),
    (SettingShortcut::LumeEditor, "lume_editor"),
    (SettingShortcut::LumeTvInfo, "lume_tvinfo"),
];

impl SettingShortcut {
    pub const COUNT: usize = SETTING_NAMES.len();

    pub const ALL: [SettingShortcut; SettingShortcut::COUNT] = {
        let mut all = [SettingShortcut::Sleep; SettingShortcut::COUNT];
        let mut i = 0;
        while i < SETTING_NAMES.len() {
            all[i] = SETTING_NAMES[i].0;
            i += 1;
        }
        all
    };

    pub const fn as_str(self) -> &'static str {
        let mut i = 0;
        while i < SETTING_NAMES.len() {
            if SETTING_NAMES[i].0 as u8 == self as u8 {
                return SETTING_NAMES[i].1;
            }
            i += 1;
        }
        "android"
    }

    pub fn parse_name(name: &str) -> Option<SettingShortcut> {
        let name = name.trim().to_ascii_lowercase();
        SETTING_NAMES
            .iter()
            .find(|(_, known)| *known == name)
            .map(|(setting, _)| *setting)
    }

    pub fn is_tv_setting(self) -> bool {
        !matches!(
            self,
            SettingShortcut::LumeSettings
                | SettingShortcut::LumeEditor
                | SettingShortcut::LumeTvInfo
        )
    }

    pub fn title_id(self) -> StringId {
        match self {
            SettingShortcut::Sleep => StringId::SettingSleep,
            SettingShortcut::Picture => StringId::SettingPicture,
            SettingShortcut::Sound => StringId::SettingSound,
            SettingShortcut::Wifi => StringId::SettingWifi,
            SettingShortcut::Bluetooth => StringId::SettingBluetooth,
            SettingShortcut::Apps => StringId::SettingApps,
            SettingShortcut::Android => StringId::SettingAndroid,
            SettingShortcut::LumeSettings => StringId::SettingLumeSettings,
            SettingShortcut::LumeEditor => StringId::SettingLumeEditor,
            SettingShortcut::LumeTvInfo => StringId::SettingLumeTvinfo,
            SettingShortcut::Storage => StringId::SettingStorage,
            SettingShortcut::Language => StringId::SettingLanguage,
            SettingShortcut::DateTime => StringId::SettingDatetime,
            SettingShortcut::Keyboard => StringId::SettingKeyboard,
            SettingShortcut::Captions => StringId::SettingCaptions,
            SettingShortcut::Location => StringId::SettingLocation,
            SettingShortcut::Security => StringId::SettingSecurity,
            SettingShortcut::Developer => StringId::SettingDeveloper,
            SettingShortcut::HomeApp => StringId::SettingHomeApp,
            SettingShortcut::About => StringId::SettingAbout,
        }
    }
}

/// A target that can be mapped to one or more remote control keys.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum KeyTarget {
    Command(CommandTarget),
    Setting(SettingShortcut),
    App(String),
}

impl KeyTarget {
    pub fn config_key(&self) -> String {
        match self {
            KeyTarget::Command(c) => c.as_str().to_string(),
            KeyTarget::Setting(s) => format!("cfg.{}", s.as_str()),
            KeyTarget::App(pkg) => format!("app.{}", pkg),
        }
    }

    pub fn parse_config_key(key: &str) -> Option<KeyTarget> {
        if let Some(cmd) = CommandTarget::parse_name(key) {
            return Some(KeyTarget::Command(cmd));
        }
        if let Some(rest) = key.strip_prefix("cfg.") {
            if let Some(s) = SettingShortcut::parse_name(rest) {
                return Some(KeyTarget::Setting(s));
            }
        }
        if let Some(rest) = key.strip_prefix("app.") {
            let pkg = rest.trim();
            if !pkg.is_empty() {
                return Some(KeyTarget::App(pkg.to_string()));
            }
        }
        None
    }
}

/// High-level action triggered by a key press.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum KeyAction {
    Key(Key),
    Inputs,
    Setting(SettingShortcut),
    App(String),
}

include!(concat!(env!("OUT_DIR"), "/keys_generated.rs"));

/// Maps an Android `KeyEvent` key code to a launcher key using default mappings.
pub fn from_android(code: i32) -> Option<Key> {
    Some(match code {
        c if DEFAULT_UP.contains(&c) => Key::Up,
        c if DEFAULT_DOWN.contains(&c) => Key::Down,
        c if DEFAULT_LEFT.contains(&c) => Key::Left,
        c if DEFAULT_RIGHT.contains(&c) => Key::Right,
        c if DEFAULT_OK.contains(&c) => Key::Ok,
        c if DEFAULT_BACK.contains(&c) => Key::Back,
        c if DEFAULT_HOME.contains(&c) => Key::Home,
        c if DEFAULT_MENU.contains(&c) => Key::Menu,
        _ => return None,
    })
}

const FAST_TABLE_SIZE: usize = 320;
const VERSION: u32 = 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FastEntry {
    Nav(Key),
    Inputs,
    Setting(SettingShortcut),
    App(u16),
}

/// In-memory remote control key mappings with fast O(1) translation.
#[derive(Clone, Debug)]
pub struct KeyMap {
    /// Active keycodes per logical Key, indexed by `key.index()` (0..8).
    bindings: [Vec<i32>; Key::COUNT],
    /// Active keycodes for the TV inputs command.
    inputs: Vec<i32>,
    /// Active setting shortcuts: (SettingShortcut, Vec<i32>).
    settings: Vec<(SettingShortcut, Vec<i32>)>,
    /// Active app shortcuts: (package_name, Vec<i32>).
    apps: Vec<(String, Vec<i32>)>,
    /// O(1) lookup table for Android keycodes in 0..320.
    fast_table: [Option<FastEntry>; FAST_TABLE_SIZE],
    /// Fallback list only for custom keycodes outside 0..320.
    overflow: Vec<(i32, FastEntry)>,
}

impl PartialEq for KeyMap {
    fn eq(&self, other: &Self) -> bool {
        self.bindings == other.bindings
            && self.inputs == other.inputs
            && self.settings == other.settings
            && self.apps == other.apps
    }
}

impl Eq for KeyMap {}

impl Default for KeyMap {
    fn default() -> Self {
        let bindings = [
            Key::Up.default_codes().to_vec(),
            Key::Down.default_codes().to_vec(),
            Key::Left.default_codes().to_vec(),
            Key::Right.default_codes().to_vec(),
            Key::Ok.default_codes().to_vec(),
            Key::Back.default_codes().to_vec(),
            Key::Home.default_codes().to_vec(),
            Key::Menu.default_codes().to_vec(),
        ];
        let mut map = KeyMap {
            bindings,
            inputs: DEFAULT_INPUTS.to_vec(),
            settings: Vec::new(),
            apps: Vec::new(),
            fast_table: [None; FAST_TABLE_SIZE],
            overflow: Vec::new(),
        };
        map.rebuild_lookup();
        map
    }
}

impl KeyMap {
    fn rebuild_lookup(&mut self) {
        self.fast_table = [None; FAST_TABLE_SIZE];
        self.overflow.clear();

        for key in Key::ALL {
            for &code in &self.bindings[key.index()] {
                Self::insert_lookup(
                    &mut self.fast_table,
                    &mut self.overflow,
                    code,
                    FastEntry::Nav(key),
                );
            }
        }
        for &code in &self.inputs {
            Self::insert_lookup(
                &mut self.fast_table,
                &mut self.overflow,
                code,
                FastEntry::Inputs,
            );
        }
        for (setting, codes) in &self.settings {
            for &code in codes {
                Self::insert_lookup(
                    &mut self.fast_table,
                    &mut self.overflow,
                    code,
                    FastEntry::Setting(*setting),
                );
            }
        }
        for (idx, (_, codes)) in self.apps.iter().enumerate() {
            for &code in codes {
                Self::insert_lookup(
                    &mut self.fast_table,
                    &mut self.overflow,
                    code,
                    FastEntry::App(idx as u16),
                );
            }
        }
    }

    fn insert_lookup(
        fast_table: &mut [Option<FastEntry>; FAST_TABLE_SIZE],
        overflow: &mut Vec<(i32, FastEntry)>,
        code: i32,
        entry: FastEntry,
    ) {
        if code >= 0 && (code as usize) < FAST_TABLE_SIZE {
            if fast_table[code as usize].is_none() {
                fast_table[code as usize] = Some(entry);
            }
        } else if !overflow.iter().any(|&(c, _)| c == code) {
            overflow.push((code, entry));
        }
    }

    #[inline]
    fn lookup(&self, code: i32) -> Option<FastEntry> {
        if code >= 0 && (code as usize) < FAST_TABLE_SIZE {
            return self.fast_table[code as usize];
        }
        self.overflow
            .iter()
            .find_map(|&(c, entry)| (c == code).then_some(entry))
    }

    /// Maps an Android keycode to a launcher Key (Up..=Menu).
    #[inline]
    pub fn from_android(&self, code: i32) -> Option<Key> {
        match self.lookup(code) {
            Some(FastEntry::Nav(key)) => Some(key),
            _ => None,
        }
    }

    /// Resolves an Android keycode to a high-level KeyAction.
    pub fn action_for(&self, code: i32) -> Option<KeyAction> {
        match self.lookup(code) {
            Some(FastEntry::Nav(key)) => Some(KeyAction::Key(key)),
            Some(FastEntry::Inputs) => Some(KeyAction::Inputs),
            Some(FastEntry::Setting(s)) => Some(KeyAction::Setting(s)),
            Some(FastEntry::App(idx)) => self
                .apps
                .get(idx as usize)
                .map(|(pkg, _)| KeyAction::App(pkg.clone())),
            None => None,
        }
    }

    /// Returns the target mapped to `code`, if any.
    pub fn target_for(&self, code: i32) -> Option<KeyTarget> {
        match self.lookup(code) {
            Some(FastEntry::Nav(Key::Menu)) => Some(KeyTarget::Command(CommandTarget::Menu)),
            Some(FastEntry::Inputs) => Some(KeyTarget::Command(CommandTarget::Inputs)),
            Some(FastEntry::Setting(s)) => Some(KeyTarget::Setting(s)),
            Some(FastEntry::App(idx)) => self
                .apps
                .get(idx as usize)
                .map(|(pkg, _)| KeyTarget::App(pkg.clone())),
            _ => None,
        }
    }

    /// Returns active keycodes for `key`.
    pub fn codes(&self, key: Key) -> &[i32] {
        &self.bindings[key.index()]
    }

    /// Returns active keycodes for the TV inputs command.
    pub fn inputs(&self) -> &[i32] {
        &self.inputs
    }

    /// Returns active keycodes for `target`.
    pub fn codes_for(&self, target: &KeyTarget) -> &[i32] {
        match target {
            KeyTarget::Command(CommandTarget::Menu) => &self.bindings[Key::Menu.index()],
            KeyTarget::Command(CommandTarget::Inputs) => &self.inputs,
            KeyTarget::Setting(s) => self
                .settings
                .iter()
                .find(|(setting, _)| setting == s)
                .map(|(_, c)| c.as_slice())
                .unwrap_or(&[]),
            KeyTarget::App(pkg) => self
                .apps
                .iter()
                .find(|(p, _)| p == pkg)
                .map(|(_, c)| c.as_slice())
                .unwrap_or(&[]),
        }
    }

    /// Returns all configured setting shortcuts.
    pub fn setting_shortcuts(&self) -> &[(SettingShortcut, Vec<i32>)] {
        &self.settings
    }

    /// Returns all configured app shortcuts.
    pub fn app_shortcuts(&self) -> &[(String, Vec<i32>)] {
        &self.apps
    }

    /// Checks if a setting shortcut is currently configured with at least one key.
    pub fn has_setting(&self, s: SettingShortcut) -> bool {
        self.settings
            .iter()
            .any(|(setting, codes)| *setting == s && !codes.is_empty())
    }

    /// Checks if an app shortcut is currently configured with at least one key.
    pub fn has_app(&self, pkg: &str) -> bool {
        self.apps
            .iter()
            .any(|(p, codes)| p == pkg && !codes.is_empty())
    }

    /// Removes `code` from any target other than the given one.
    fn resolve_conflicts(&mut self, code: i32) {
        self.bindings[Key::Menu.index()].retain(|&c| c != code);
        self.inputs.retain(|&c| c != code);
        for (_, codes) in &mut self.settings {
            codes.retain(|&c| c != code);
        }
        self.settings.retain(|(_, codes)| !codes.is_empty());
        for (_, codes) in &mut self.apps {
            codes.retain(|&c| c != code);
        }
        self.apps.retain(|(_, codes)| !codes.is_empty());
    }

    /// Adds a keycode to `target`, resolving conflicts with any other target.
    /// Returns false if `code` is protected against mapping.
    pub fn add_code(&mut self, target: &KeyTarget, code: i32) -> bool {
        if is_protected_code(code) {
            return false;
        }
        self.resolve_conflicts(code);
        match target {
            KeyTarget::Command(CommandTarget::Menu) => {
                if !self.bindings[Key::Menu.index()].contains(&code) {
                    self.bindings[Key::Menu.index()].push(code);
                }
            }
            KeyTarget::Command(CommandTarget::Inputs) => {
                if !self.inputs.contains(&code) {
                    self.inputs.push(code);
                }
            }
            KeyTarget::Setting(s) => {
                if let Some((_, codes)) = self.settings.iter_mut().find(|(setting, _)| setting == s)
                {
                    if !codes.contains(&code) {
                        codes.push(code);
                    }
                } else {
                    self.settings.push((*s, vec![code]));
                }
            }
            KeyTarget::App(pkg) => {
                if let Some((_, codes)) = self.apps.iter_mut().find(|(p, _)| p == pkg) {
                    if !codes.contains(&code) {
                        codes.push(code);
                    }
                } else {
                    self.apps.push((pkg.clone(), vec![code]));
                }
            }
        }
        self.rebuild_lookup();
        true
    }

    /// Removes a single keycode from `target`.
    pub fn remove_code(&mut self, target: &KeyTarget, code: i32) {
        match target {
            KeyTarget::Command(CommandTarget::Menu) => {
                self.bindings[Key::Menu.index()].retain(|&c| c != code);
            }
            KeyTarget::Command(CommandTarget::Inputs) => {
                self.inputs.retain(|&c| c != code);
            }
            KeyTarget::Setting(s) => {
                if let Some((_, codes)) = self.settings.iter_mut().find(|(setting, _)| setting == s)
                {
                    codes.retain(|&c| c != code);
                }
                self.settings.retain(|(_, codes)| !codes.is_empty());
            }
            KeyTarget::App(pkg) => {
                if let Some((_, codes)) = self.apps.iter_mut().find(|(p, _)| p == pkg) {
                    codes.retain(|&c| c != code);
                }
                self.apps.retain(|(_, codes)| !codes.is_empty());
            }
        }
        self.rebuild_lookup();
    }

    /// Resets or removes a target.
    /// Commands return to defaults; setting and app shortcuts are removed.
    pub fn reset_target(&mut self, target: &KeyTarget) {
        match target {
            KeyTarget::Command(CommandTarget::Menu) => {
                let defaults = DEFAULT_MENU.to_vec();
                for &c in &defaults {
                    self.resolve_conflicts(c);
                }
                self.bindings[Key::Menu.index()] = defaults;
            }
            KeyTarget::Command(CommandTarget::Inputs) => {
                let defaults = DEFAULT_INPUTS.to_vec();
                for &c in &defaults {
                    self.resolve_conflicts(c);
                }
                self.inputs = defaults;
            }
            KeyTarget::Setting(s) => {
                self.settings.retain(|(setting, _)| setting != s);
            }
            KeyTarget::App(pkg) => {
                self.apps.retain(|(p, _)| p != pkg);
            }
        }
        self.rebuild_lookup();
    }

    /// Checks if `target` matches its default state.
    pub fn is_target_default(&self, target: &KeyTarget) -> bool {
        match target {
            KeyTarget::Command(CommandTarget::Menu) => self.codes(Key::Menu) == DEFAULT_MENU,
            KeyTarget::Command(CommandTarget::Inputs) => self.inputs.as_slice() == DEFAULT_INPUTS,
            KeyTarget::Setting(s) => !self.has_setting(*s),
            KeyTarget::App(pkg) => !self.has_app(pkg),
        }
    }

    /// Sets active keycodes for `key`, deduplicating and removing conflicts from other keys.
    pub fn set(&mut self, key: Key, codes: Vec<i32>) {
        let mut deduped: Vec<i32> = Vec::with_capacity(codes.len());
        for c in codes {
            if !deduped.contains(&c) {
                deduped.push(c);
            }
        }
        for other in Key::ALL {
            if other != key {
                self.bindings[other.index()].retain(|c| !deduped.contains(c));
            }
        }
        self.bindings[key.index()] = deduped;
        self.rebuild_lookup();
    }

    /// Clears keycodes for `key`, disabling it.
    pub fn clear(&mut self, key: Key) {
        self.set(key, Vec::new());
    }

    /// Resets `key` to its default keycodes.
    pub fn reset_key(&mut self, key: Key) {
        self.set(key, key.default_codes().to_vec());
    }

    /// Resets all keys and shortcuts to factory default.
    pub fn reset(&mut self) {
        *self = KeyMap::default();
    }

    /// Checks if `key` matches its default keycodes.
    pub fn is_default(&self, key: Key) -> bool {
        self.codes(key) == key.default_codes()
    }

    /// Checks if all keys and shortcuts match their default states.
    pub fn is_all_default(&self) -> bool {
        Key::ALL.into_iter().all(|k| self.is_default(k))
            && self.inputs.as_slice() == DEFAULT_INPUTS
            && self.settings.is_empty()
            && self.apps.is_empty()
    }

    fn to_kv(&self) -> Kv {
        let mut kv = Kv::with_capacity(Key::COUNT + 1 + self.settings.len() + self.apps.len());
        for key in Key::ALL {
            let codes: Vec<String> = self.codes(key).iter().map(i32::to_string).collect();
            kv.set_list(key.as_str(), codes.iter().map(String::as_str));
        }
        let inputs: Vec<String> = self.inputs.iter().map(i32::to_string).collect();
        kv.set_list("inputs", inputs.iter().map(String::as_str));

        for (setting, codes) in &self.settings {
            let key = format!("cfg.{}", setting.as_str());
            let list: Vec<String> = codes.iter().map(i32::to_string).collect();
            kv.set_list(&key, list.iter().map(String::as_str));
        }
        for (pkg, codes) in &self.apps {
            let key = format!("app.{}", pkg);
            let list: Vec<String> = codes.iter().map(i32::to_string).collect();
            kv.set_list(&key, list.iter().map(String::as_str));
        }
        kv
    }

    /// Parses a KeyMap from Kv, returning the map and whether healing/saving is needed.
    pub fn from_kv(kv: &Kv) -> (KeyMap, bool) {
        let mut map = KeyMap::default();
        let mut needs_save = false;

        for key in Key::ALL {
            match kv.get(key.as_str()) {
                None => {
                    needs_save = true;
                }
                Some(raw) => {
                    let trimmed = raw.trim();
                    if trimmed.is_empty() {
                        if key != Key::Menu {
                            map.bindings[key.index()] = key.default_codes().to_vec();
                            needs_save = true;
                        } else {
                            map.bindings[key.index()] = Vec::new();
                        }
                    } else {
                        let mut parsed = Vec::new();
                        let mut ok = true;
                        for item in trimmed.split(',') {
                            let item_trimmed = item.trim();
                            if item_trimmed.is_empty() {
                                ok = false;
                                break;
                            }
                            match item_trimmed.parse::<i32>() {
                                Ok(c) => {
                                    if !parsed.contains(&c) {
                                        parsed.push(c);
                                    }
                                }
                                Err(_) => {
                                    ok = false;
                                    break;
                                }
                            }
                        }

                        if ok {
                            if key != Key::Menu && parsed != key.default_codes() {
                                map.bindings[key.index()] = key.default_codes().to_vec();
                                needs_save = true;
                            } else {
                                map.bindings[key.index()] = parsed;
                            }
                        } else {
                            log::warn!("ignoring invalid keycodes for {}: {:?}", key.as_str(), raw);
                            needs_save = true;
                        }
                    }
                }
            }
        }

        if let Some(raw) = kv.get("inputs") {
            let trimmed = raw.trim();
            if trimmed.is_empty() {
                map.inputs = Vec::new();
            } else {
                let mut parsed = Vec::new();
                let mut ok = true;
                for item in trimmed.split(',') {
                    let item_trimmed = item.trim();
                    if item_trimmed.is_empty() {
                        ok = false;
                        break;
                    }
                    match item_trimmed.parse::<i32>() {
                        Ok(c) => {
                            if !parsed.contains(&c) {
                                parsed.push(c);
                            }
                        }
                        Err(_) => {
                            ok = false;
                            break;
                        }
                    }
                }
                if ok {
                    map.inputs = parsed;
                } else {
                    map.inputs = DEFAULT_INPUTS.to_vec();
                    needs_save = true;
                }
            }
        } else {
            map.inputs = DEFAULT_INPUTS.to_vec();
        }

        for (k, val) in kv.iter() {
            if let Some(rest) = k.strip_prefix("cfg.") {
                if let Some(setting) = SettingShortcut::parse_name(rest) {
                    let mut parsed = Vec::new();
                    let mut ok = true;
                    for item in val.split(',') {
                        let item_trimmed = item.trim();
                        if item_trimmed.is_empty() {
                            continue;
                        }
                        match item_trimmed.parse::<i32>() {
                            Ok(c) => {
                                if !parsed.contains(&c) {
                                    parsed.push(c);
                                }
                            }
                            Err(_) => {
                                ok = false;
                                break;
                            }
                        }
                    }
                    if ok && !parsed.is_empty() {
                        map.settings.push((setting, parsed));
                    } else if !ok {
                        needs_save = true;
                    }
                } else {
                    needs_save = true;
                }
            } else if let Some(rest) = k.strip_prefix("app.") {
                let pkg = rest.trim();
                if !pkg.is_empty() {
                    let mut parsed = Vec::new();
                    let mut ok = true;
                    for item in val.split(',') {
                        let item_trimmed = item.trim();
                        if item_trimmed.is_empty() {
                            continue;
                        }
                        match item_trimmed.parse::<i32>() {
                            Ok(c) => {
                                if !parsed.contains(&c) {
                                    parsed.push(c);
                                }
                            }
                            Err(_) => {
                                ok = false;
                                break;
                            }
                        }
                    }
                    if ok && !parsed.is_empty() {
                        map.apps.push((pkg.to_string(), parsed));
                    } else if !ok {
                        needs_save = true;
                    }
                } else {
                    needs_save = true;
                }
            }
        }

        map.rebuild_lookup();
        (map, needs_save)
    }

    /// Saves the key mapping to disk.
    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        storage::save(path, VERSION, &self.to_kv())
    }

    /// Loads `keys.v1` from `path`. If the file is missing, corrupt, or had missing/invalid entries,
    /// repairs it in memory and atomically writes the clean `keys.v1` to disk.
    pub fn load_or_heal(path: &Path) -> KeyMap {
        match storage::load(path, VERSION) {
            Ok(kv) => {
                let (map, needs_save) = KeyMap::from_kv(&kv);
                if needs_save {
                    if let Err(e) = map.save(path) {
                        log::warn!("cannot write repaired {}: {e}", path.display());
                    }
                }
                map
            }
            Err(StorageError::Missing) => {
                let map = KeyMap::default();
                if let Err(e) = map.save(path) {
                    log::warn!("cannot write initial {}: {e}", path.display());
                }
                map
            }
            Err(e) => {
                log::warn!("healing {}: {e}", path.display());
                let map = KeyMap::default();
                if let Err(err) = map.save(path) {
                    log::warn!("cannot write healed {}: {err}", path.display());
                }
                map
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    fn dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("lume-keys-{}-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn size_of_option_key_is_one_byte() {
        assert_eq!(std::mem::size_of::<Option<Key>>(), 1);
    }

    #[test]
    fn default_mappings_from_android() {
        let default_map = KeyMap::default();

        for &(code, expected) in &[
            (19, Some(Key::Up)),
            (20, Some(Key::Down)),
            (21, Some(Key::Left)),
            (22, Some(Key::Right)),
            (23, Some(Key::Ok)),
            (66, Some(Key::Ok)),
            (160, Some(Key::Ok)),
            (96, Some(Key::Ok)),
            (4, Some(Key::Back)),
            (111, Some(Key::Back)),
            (97, Some(Key::Back)),
            (3, Some(Key::Home)),
            (82, Some(Key::Menu)),
            (174, Some(Key::Menu)),
        ] {
            assert_eq!(from_android(code), expected, "code {code}");
            assert_eq!(
                default_map.from_android(code),
                expected,
                "default_map code {code}"
            );
        }

        for code in [0, 7, 24, 25, 85, 126, 164, 9999, -1] {
            assert_eq!(from_android(code), None, "unmapped code {code}");
            assert_eq!(
                default_map.from_android(code),
                None,
                "default_map unmapped code {code}"
            );
        }
    }

    #[test]
    fn overflow_custom_keycodes() {
        let mut map = KeyMap::default();
        map.set(Key::Menu, vec![500, -10]);
        assert_eq!(map.from_android(500), Some(Key::Menu));
        assert_eq!(map.from_android(-10), Some(Key::Menu));
        assert_eq!(map.from_android(82), None);
        assert_eq!(map.from_android(174), None);
    }

    #[test]
    fn auto_create_when_missing() {
        let d = dir("missing");
        let path = d.join("keys.v1");
        assert!(!path.exists());

        let map = KeyMap::load_or_heal(&path);
        assert!(path.exists());
        assert!(map.is_all_default());

        let content = fs::read_to_string(&path).unwrap();
        assert!(content.starts_with("lume=1\n"));
        assert!(content.ends_with("#end\n"));
    }

    #[test]
    fn load_existing_valid_file_without_rewriting() {
        let d = dir("valid");
        let path = d.join("keys.v1");
        let custom_content = "lume=1\nup=19\ndown=20\nleft=21\nright=22\nok=23,66,160,96\nback=4,111,97\nhome=3\nmenu=165\n#end\n";
        fs::write(&path, custom_content).unwrap();

        let map = KeyMap::load_or_heal(&path);
        assert_eq!(map.codes(Key::Menu), &[165]);
        assert_eq!(map.from_android(165), Some(Key::Menu));
        assert_eq!(map.from_android(82), None);

        let after_content = fs::read_to_string(&path).unwrap();
        assert_eq!(after_content, custom_content, "file was not rewritten");
    }

    #[test]
    fn multiple_keycodes_for_single_key() {
        let d = dir("multi_key");
        let path = d.join("keys.v1");
        let custom_content = "lume=1\nup=19\ndown=20\nleft=21\nright=22\nok=23,66,160,96\nback=4,111,97\nhome=3\nmenu=82,174,165\n#end\n";
        fs::write(&path, custom_content).unwrap();

        let map = KeyMap::load_or_heal(&path);
        assert_eq!(map.codes(Key::Menu), &[82, 174, 165]);
        assert_eq!(map.from_android(82), Some(Key::Menu));
        assert_eq!(map.from_android(174), Some(Key::Menu));
        assert_eq!(map.from_android(165), Some(Key::Menu));
    }

    #[test]
    fn empty_value_disables_key() {
        let d = dir("disabled");
        let path = d.join("keys.v1");
        let content = "lume=1\nup=19\ndown=20\nleft=21\nright=22\nok=23,66,160,96\nback=4,111,97\nhome=3\nmenu=\n#end\n";
        fs::write(&path, content).unwrap();

        let map = KeyMap::load_or_heal(&path);
        assert!(map.codes(Key::Menu).is_empty());
        assert_eq!(map.from_android(82), None);
        assert_eq!(map.from_android(174), None);

        let after_content = fs::read_to_string(&path).unwrap();
        assert_eq!(
            after_content, content,
            "file was not rewritten for disabled key"
        );
    }

    #[test]
    fn healing_corrupted_file() {
        let d = dir("corrupt");
        let path = d.join("keys.v1");
        fs::write(&path, "lume=1\nup=19\n").unwrap(); // missing #end

        let map = KeyMap::load_or_heal(&path);
        assert!(map.is_all_default());

        let healed = fs::read_to_string(&path).unwrap();
        assert!(healed.starts_with("lume=1\n"));
        assert!(healed.ends_with("#end\n"));
        assert!(healed.contains("menu=82,174"));
    }

    #[test]
    fn partial_healing_invalid_token_or_missing_key() {
        let d = dir("partial_heal");
        let path = d.join("keys.v1");
        // menu has custom 165, ok has invalid 23,abc, home is missing
        let partial_content =
            "lume=1\nup=19\ndown=20\nleft=21\nright=22\nok=23,abc\nback=4,111,97\nmenu=165\n#end\n";
        fs::write(&path, partial_content).unwrap();

        let map = KeyMap::load_or_heal(&path);
        assert_eq!(map.codes(Key::Menu), &[165], "custom preserved");
        assert_eq!(
            map.codes(Key::Ok),
            Key::Ok.default_codes(),
            "repaired ok to default"
        );
        assert_eq!(
            map.codes(Key::Home),
            Key::Home.default_codes(),
            "repaired missing home to default"
        );

        let healed = fs::read_to_string(&path).unwrap();
        assert!(healed.contains("menu=165"));
        assert!(healed.contains("ok=23,66,160,96"));
        assert!(healed.contains("home=3"));
        assert!(healed.ends_with("#end\n"));
    }

    #[test]
    fn editing_and_reset_methods() {
        let mut map = KeyMap::default();
        assert!(map.is_all_default());

        // set with deduplication and removing conflict from other keys
        map.set(Key::Menu, vec![19, 19, 200]);
        assert_eq!(map.codes(Key::Menu), &[19, 200]);
        assert!(
            !map.codes(Key::Up).contains(&19),
            "19 removed from Up to avoid conflict"
        );
        assert_eq!(map.from_android(19), Some(Key::Menu));
        assert_eq!(map.from_android(200), Some(Key::Menu));
        assert!(!map.is_default(Key::Menu));
        assert!(!map.is_default(Key::Up));
        assert!(!map.is_all_default());

        // reset_key
        map.reset_key(Key::Up);
        assert!(map.is_default(Key::Up));
        assert!(
            !map.codes(Key::Menu).contains(&19),
            "19 removed from Menu when Up reset"
        );
        assert_eq!(map.from_android(19), Some(Key::Up));

        // clear
        map.clear(Key::Back);
        assert!(map.codes(Key::Back).is_empty());
        assert_eq!(map.from_android(4), None);
        assert!(!map.is_default(Key::Back));

        // reset
        map.reset();
        assert!(map.is_all_default());
        assert_eq!(map.from_android(4), Some(Key::Back));
        assert_eq!(map.from_android(82), Some(Key::Menu));
    }

    #[test]
    fn test_protected_and_reserved_keys() {
        for &code in &[19, 20, 21, 22] {
            assert!(is_dpad_code(code));
            assert!(is_reserved_code(code));
            assert!(is_protected_code(code));
        }

        for &code in &[4, 111, 97] {
            assert!(is_back_code(code));
            assert!(is_protected_code(code));
            assert!(!is_reserved_code(code), "Back cancels, it is not rejected");
            assert!(!is_dpad_code(code));
        }

        for &code in &[23, 66, 160, 96] {
            assert!(is_ok_code(code));
            assert!(is_reserved_code(code), "OK is rejected by the listening");
            assert!(is_protected_code(code));
            assert!(!is_back_code(code));
            assert!(!is_dpad_code(code));
        }

        for &code in &[3, 26, 177, 223, 224, 276] {
            assert!(is_reserved_code(code));
            assert!(is_protected_code(code));
            assert!(!is_back_code(code));
        }

        for &code in &[82, 174, 178, 165, 172, 183, 184, 185, 186, 24, 25, 164] {
            assert!(!is_protected_code(code));
        }
    }

    #[test]
    fn test_key_targets_and_shortcuts() {
        assert_eq!(CommandTarget::Menu.as_str(), "menu");
        assert_eq!(CommandTarget::Inputs.as_str(), "inputs");
        assert_eq!(CommandTarget::parse_name("menu"), Some(CommandTarget::Menu));
        assert_eq!(
            CommandTarget::parse_name("inputs"),
            Some(CommandTarget::Inputs)
        );
        assert_eq!(CommandTarget::Menu.default_codes(), DEFAULT_MENU);
        assert_eq!(CommandTarget::Inputs.default_codes(), DEFAULT_INPUTS);

        assert_eq!(SettingShortcut::ALL.len(), 20);
        let mut names: Vec<_> = SettingShortcut::ALL.iter().map(|s| s.as_str()).collect();
        names.sort_unstable();
        names.dedup();
        assert_eq!(
            names.len(),
            20,
            "every shortcut has its own name in keys.v1"
        );
        for s in SettingShortcut::ALL {
            assert_eq!(SettingShortcut::parse_name(s.as_str()), Some(s));
        }
        for s in [
            SettingShortcut::Storage,
            SettingShortcut::Language,
            SettingShortcut::HomeApp,
            SettingShortcut::About,
        ] {
            assert!(s.is_tv_setting());
        }
        assert!(SettingShortcut::Sleep.is_tv_setting());
        assert!(SettingShortcut::Picture.is_tv_setting());
        assert!(SettingShortcut::Sound.is_tv_setting());
        assert!(SettingShortcut::Wifi.is_tv_setting());
        assert!(SettingShortcut::Bluetooth.is_tv_setting());
        assert!(SettingShortcut::Apps.is_tv_setting());
        assert!(SettingShortcut::Android.is_tv_setting());
        assert!(!SettingShortcut::LumeSettings.is_tv_setting());
        assert!(!SettingShortcut::LumeEditor.is_tv_setting());
        assert!(!SettingShortcut::LumeTvInfo.is_tv_setting());

        let cmd_target = KeyTarget::Command(CommandTarget::Inputs);
        assert_eq!(cmd_target.config_key(), "inputs");
        assert_eq!(KeyTarget::parse_config_key("inputs"), Some(cmd_target));

        let setting_target = KeyTarget::Setting(SettingShortcut::Sleep);
        assert_eq!(setting_target.config_key(), "cfg.sleep");
        assert_eq!(
            KeyTarget::parse_config_key("cfg.sleep"),
            Some(setting_target)
        );

        let app_target = KeyTarget::App("com.google.android.youtube.tv".to_string());
        assert_eq!(app_target.config_key(), "app.com.google.android.youtube.tv");
        assert_eq!(
            KeyTarget::parse_config_key("app.com.google.android.youtube.tv"),
            Some(app_target)
        );
    }

    #[test]
    fn test_add_and_remove_code_with_conflict_resolution() {
        let mut map = KeyMap::default();
        let target_menu = KeyTarget::Command(CommandTarget::Menu);
        let target_inputs = KeyTarget::Command(CommandTarget::Inputs);
        let target_sleep = KeyTarget::Setting(SettingShortcut::Sleep);
        let target_app = KeyTarget::App("com.google.android.youtube.tv".to_string());

        // Protected keys are rejected
        assert!(!map.add_code(&target_menu, 19)); // Up
        assert!(!map.add_code(&target_inputs, 4)); // Back
        assert!(!map.add_code(&target_sleep, 23)); // OK
        assert!(!map.add_code(&target_app, 26)); // Power

        // Menu starts with 82 and 174
        assert_eq!(map.codes_for(&target_menu), &[82, 174]);

        // Add 174 to sleep: 174 is removed from Menu (conflict resolution) and assigned to Sleep
        assert!(map.add_code(&target_sleep, 174));
        assert_eq!(map.codes_for(&target_menu), &[82]);
        assert_eq!(map.codes_for(&target_sleep), &[174]);
        assert!(!map.is_target_default(&target_menu));
        assert!(!map.is_target_default(&target_sleep));

        assert_eq!(
            map.action_for(174),
            Some(KeyAction::Setting(SettingShortcut::Sleep))
        );
        assert_eq!(
            map.target_for(174),
            Some(KeyTarget::Setting(SettingShortcut::Sleep))
        );
        assert_eq!(map.from_android(174), None); // Not a nav key anymore

        // Add 174 to App: moves from Sleep to App
        assert!(map.add_code(&target_app, 174));
        assert_eq!(map.codes_for(&target_sleep), &[]);
        assert!(!map.has_setting(SettingShortcut::Sleep));
        assert_eq!(map.codes_for(&target_app), &[174]);
        assert_eq!(
            map.action_for(174),
            Some(KeyAction::App("com.google.android.youtube.tv".to_string()))
        );

        // Reset target Menu restores default
        map.reset_target(&target_menu);
        assert_eq!(map.codes_for(&target_menu), DEFAULT_MENU);
        assert!(map.is_target_default(&target_menu));
        // And conflict resolution removes 174 from App
        assert!(!map.has_app("com.google.android.youtube.tv"));

        // Remove code from Inputs
        assert_eq!(map.codes_for(&target_inputs), &[178]);
        map.remove_code(&target_inputs, 178);
        assert_eq!(map.codes_for(&target_inputs), &[]);
        assert!(!map.is_target_default(&target_inputs));

        // Reset target Inputs
        map.reset_target(&target_inputs);
        assert_eq!(map.codes_for(&target_inputs), &[178]);
        assert!(map.is_target_default(&target_inputs));
    }

    #[test]
    fn test_persistence_with_settings_and_apps() {
        let d = dir("persistence_shortcuts");
        let path = d.join("keys.v1");

        let mut map = KeyMap::default();
        map.add_code(&KeyTarget::Setting(SettingShortcut::Sleep), 183);
        map.add_code(
            &KeyTarget::App("com.google.android.youtube.tv".to_string()),
            184,
        );
        map.save(&path).unwrap();

        let loaded = KeyMap::load_or_heal(&path);
        assert_eq!(
            loaded.codes_for(&KeyTarget::Setting(SettingShortcut::Sleep)),
            &[183]
        );
        assert_eq!(
            loaded.codes_for(&KeyTarget::App("com.google.android.youtube.tv".to_string())),
            &[184]
        );
        assert_eq!(
            loaded.action_for(183),
            Some(KeyAction::Setting(SettingShortcut::Sleep))
        );
        assert_eq!(
            loaded.action_for(184),
            Some(KeyAction::App("com.google.android.youtube.tv".to_string()))
        );

        // Healing: if a protected key is modified in the file, it is repaired
        let modified = "lume=1\nup=999\ndown=20\nleft=21\nright=22\nok=23,66,160,96\nback=4,111,97\nhome=3\nmenu=82\ninputs=178\n#end\n";
        fs::write(&path, modified).unwrap();
        let healed = KeyMap::load_or_heal(&path);
        assert_eq!(healed.codes(Key::Up), DEFAULT_UP, "repaired up to default");
    }

    #[test]
    fn test_friendly_names() {
        assert_eq!(keycode_name(82), Some("Menu"));
        assert_eq!(keycode_name(174), Some("Favorite"));
        assert_eq!(keycode_name(178), Some("Input"));
        assert_eq!(keycode_name(172), Some("Guide"));
        assert_eq!(keycode_name(165), Some("Info"));
        assert_eq!(keycode_name(175), Some("Captions"));
        assert_eq!(keycode_name(170), Some("TV"));
        assert_eq!(keycode_name(183), Some("Red"));
        assert_eq!(keycode_name(184), Some("Green"));
        assert_eq!(keycode_name(185), Some("Yellow"));
        assert_eq!(keycode_name(186), Some("Blue"));
        assert_eq!(keycode_name(166), Some("Channel +"));
        assert_eq!(keycode_name(167), Some("Channel -"));
        assert_eq!(keycode_name(24), Some("Volume +"));
        assert_eq!(keycode_name(25), Some("Volume -"));
        assert_eq!(keycode_name(164), Some("Mute"));
        assert_eq!(keycode_name(85), Some("Play/Pause"));
        assert_eq!(keycode_name(86), Some("Stop"));
        assert_eq!(keycode_name(90), Some("Fast Forward"));
        assert_eq!(keycode_name(89), Some("Rewind"));
        assert_eq!(keycode_name(7), Some("0"));
        assert_eq!(keycode_name(16), Some("9"));
        assert_eq!(keycode_name(131), Some("F1"));
        assert_eq!(keycode_name(142), Some("F12"));

        assert_eq!(keycode_label(82), "Menu (82)");
        assert_eq!(keycode_label(174), "Favorite (174)");
        assert_eq!(keycode_label(9999), "Key 9999");

        let t = Translator::new(&["pt-BR"]);
        assert_eq!(keycode_display(82, &t), "Menu (82)");
        assert_eq!(keycode_display(174, &t), "Favorito (174)");
        assert_eq!(keycode_display(178, &t), "Entrada (178)");
        assert_eq!(keycode_display(183, &t), "Vermelho (183)");
        assert_eq!(keycode_display(9999, &t), "Tecla 9999");
    }
}
