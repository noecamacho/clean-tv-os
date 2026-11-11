//! Small versioned `key=value` files with atomic writes.
//!
//! The format is plain text so it can be read with `adb shell run-as`:
//!
//! ```text
//! lume=1
//! carousel.interval=5
//! favorites=com.netflix.ninja,com.google.android.youtube.tv
//! #end
//! ```
//!
//! The first line carries the format version, the last one marks a complete file.
//! Values escape `\` as `\\` and a line break as `\n`.
//! Writing goes through a temporary file that is flushed and then renamed over the
//! target, so a power cut leaves either the whole old file or the whole new one.

use std::fmt;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

const END_MARKER: &str = "#end";

/// An ordered list of `key=value` pairs.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Kv {
    entries: Vec<(String, String)>,
}

impl Kv {
    pub fn new() -> Kv {
        Kv::default()
    }

    pub fn with_capacity(capacity: usize) -> Kv {
        Kv {
            entries: Vec::with_capacity(capacity),
        }
    }

    /// Sets `key`, replacing an earlier value. Keys use letters, digits, `.`, `_` and `-`.
    pub fn set(&mut self, key: &str, value: impl Into<String>) {
        assert!(valid_key(key), "invalid key {key:?}");
        let value = value.into();
        match self.entries.iter_mut().find(|(k, _)| k == key) {
            Some(entry) => entry.1 = value,
            None => self.entries.push((key.to_string(), value)),
        }
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.entries
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.as_str())
    }

    /// The value parsed as `T`; `None` when missing or not parseable.
    pub fn parse<T: std::str::FromStr>(&self, key: &str) -> Option<T> {
        self.get(key)?.parse().ok()
    }

    /// A comma-separated list; an empty value is an empty list.
    pub fn list(&self, key: &str) -> Option<Vec<&str>> {
        let value = self.get(key)?;
        Some(if value.is_empty() {
            Vec::new()
        } else {
            value.split(',').collect()
        })
    }

    pub fn set_list<'a>(&mut self, key: &str, items: impl IntoIterator<Item = &'a str>) {
        let joined: Vec<&str> = items.into_iter().collect();
        assert!(
            joined.iter().all(|i| !i.contains(',')),
            "list items cannot contain commas"
        );
        self.set(key, joined.join(","));
    }

    pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        self.entries.iter().map(|(k, v)| (k.as_str(), v.as_str()))
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

fn valid_key(key: &str) -> bool {
    !key.is_empty()
        && key
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'))
}

#[derive(Debug, PartialEq, Eq)]
pub enum StorageError {
    /// The file does not exist (normal on the first run).
    Missing,
    Io(String),
    /// The text is not a complete, well-formed file.
    Corrupt(String),
    /// A well-formed file of a format this version does not know.
    UnknownVersion(u32),
}

impl fmt::Display for StorageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StorageError::Missing => write!(f, "file not found"),
            StorageError::Io(e) => write!(f, "cannot read: {e}"),
            StorageError::Corrupt(e) => write!(f, "corrupt file: {e}"),
            StorageError::UnknownVersion(v) => write!(f, "unknown format version {v}"),
        }
    }
}

fn escape(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
}

fn unescape(value: &str) -> Result<String, String> {
    let mut out = String::with_capacity(value.len());
    let mut chars = value.chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match chars.next() {
            Some('\\') => out.push('\\'),
            Some('n') => out.push('\n'),
            Some('r') => out.push('\r'),
            other => {
                return Err(format!(
                    "invalid escape sequence \\{}",
                    other.map(String::from).unwrap_or_default()
                ))
            }
        }
    }
    Ok(out)
}

/// Serializes `kv` as format `version`.
pub fn encode(version: u32, kv: &Kv) -> String {
    let mut out = format!("lume={version}\n");
    for (key, value) in kv.iter() {
        out.push_str(key);
        out.push('=');
        out.push_str(&escape(value));
        out.push('\n');
    }
    out.push_str(END_MARKER);
    out.push('\n');
    out
}

/// Parses text written by [`encode`], accepting only format `expected_version`.
pub fn decode(text: &str, expected_version: u32) -> Result<Kv, StorageError> {
    let corrupt = |message: String| StorageError::Corrupt(message);
    let mut lines = text.lines();
    let header = lines.next().ok_or_else(|| corrupt("empty file".into()))?;
    let version: u32 = header
        .strip_prefix("lume=")
        .and_then(|v| v.parse().ok())
        .ok_or_else(|| corrupt(format!("bad header {header:?}")))?;
    if version != expected_version {
        return Err(StorageError::UnknownVersion(version));
    }
    let mut kv = Kv::with_capacity(text.lines().count().saturating_sub(2));
    let mut complete = false;
    for (index, line) in lines.enumerate() {
        let number = index + 2;
        if complete {
            return Err(corrupt(format!(
                "unexpected content after the end marker (line {number})"
            )));
        }
        if line == END_MARKER {
            complete = true;
        } else if line.is_empty() || line.starts_with('#') {
            continue;
        } else {
            let (key, value) = line
                .split_once('=')
                .ok_or_else(|| corrupt(format!("line {number} has no '='")))?;
            if !valid_key(key) {
                return Err(corrupt(format!("line {number} has an invalid key")));
            }
            let value = unescape(value).map_err(|e| corrupt(format!("line {number}: {e}")))?;
            kv.set(key, value);
        }
    }
    if complete {
        Ok(kv)
    } else {
        Err(corrupt("the end marker is missing (truncated file)".into()))
    }
}

fn temp_path(path: &Path) -> PathBuf {
    let mut name = path
        .file_name()
        .map(|n| n.to_os_string())
        .unwrap_or_default();
    name.push(".tmp");
    path.with_file_name(name)
}

/// Writes `kv` to `path` atomically.
pub fn save(path: &Path, version: u32, kv: &Kv) -> io::Result<()> {
    let temp = temp_path(path);
    let result = (|| {
        let mut file = File::create(&temp)?;
        file.write_all(encode(version, kv).as_bytes())?;
        file.sync_all()?;
        drop(file);
        fs::rename(&temp, path)
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temp);
        return result;
    }
    // Make the rename itself durable; not every platform allows opening a directory.
    if let Some(dir) = path.parent().filter(|d| !d.as_os_str().is_empty()) {
        if let Ok(dir) = File::open(dir) {
            let _ = dir.sync_all();
        }
    }
    Ok(())
}

pub fn load(path: &Path, expected_version: u32) -> Result<Kv, StorageError> {
    match fs::read(path) {
        Ok(bytes) => {
            let text = String::from_utf8(bytes)
                .map_err(|_| StorageError::Corrupt("not valid UTF-8".into()))?;
            decode(&text, expected_version)
        }
        Err(e) if e.kind() == io::ErrorKind::NotFound => Err(StorageError::Missing),
        Err(e) => Err(StorageError::Io(e.to_string())),
    }
}

/// Loads `path`, logging a warning and returning `None` when it is unusable.
/// A missing file is the normal first-run case and is not logged.
pub fn load_or_log(path: &Path, expected_version: u32) -> Option<Kv> {
    match load(path, expected_version) {
        Ok(kv) => Some(kv),
        Err(StorageError::Missing) => None,
        Err(e) => {
            log::warn!("ignoring {}: {e}", path.display());
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("lume-storage-{}-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn sample() -> Kv {
        let mut kv = Kv::new();
        kv.set("carousel.interval", "5");
        kv.set_list(
            "favorites",
            ["com.netflix.ninja", "com.google.android.youtube.tv"],
        );
        kv.set("note", "line one\nline two\\end\r");
        kv.set("empty", "");
        kv
    }

    #[test]
    fn encode_and_decode_round_trip() {
        let kv = sample();
        let text = encode(1, &kv);
        assert!(text.starts_with("lume=1\n") && text.ends_with("#end\n"));
        assert_eq!(decode(&text, 1).unwrap(), kv);
    }

    #[test]
    fn values_with_special_characters_survive() {
        let kv = sample();
        let back = decode(&encode(1, &kv), 1).unwrap();
        assert_eq!(back.get("note"), Some("line one\nline two\\end\r"));
        assert_eq!(back.get("empty"), Some(""));
        assert_eq!(
            back.list("favorites").unwrap(),
            ["com.netflix.ninja", "com.google.android.youtube.tv"]
        );
        assert_eq!(back.list("empty"), Some(vec![]));
        assert_eq!(back.parse::<u32>("carousel.interval"), Some(5));
        assert_eq!(back.parse::<u32>("note"), None);
        assert_eq!(back.get("missing"), None);
    }

    #[test]
    fn set_replaces_and_keeps_order() {
        let mut kv = Kv::new();
        kv.set("a", "1");
        kv.set("b", "2");
        kv.set("a", "3");
        assert_eq!(kv.iter().collect::<Vec<_>>(), [("a", "3"), ("b", "2")]);
    }

    #[test]
    fn unknown_versions_are_reported() {
        let text = encode(2, &sample());
        assert_eq!(decode(&text, 1), Err(StorageError::UnknownVersion(2)));
    }

    #[test]
    fn truncated_files_are_rejected_at_every_cut() {
        let text = encode(1, &sample());
        for cut in 0..text.len() - 1 {
            if !text.is_char_boundary(cut) {
                continue;
            }
            assert!(
                decode(&text[..cut], 1).is_err(),
                "a file cut at byte {cut} must not load"
            );
        }
    }

    #[test]
    fn garbage_is_rejected() {
        for text in [
            "",
            "hello",
            "lume=abc\n#end\n",
            "lume=1\nno equals sign\n#end\n",
            "lume=1\nbad key=1\n#end\n",
            "lume=1\nk=\\x\n#end\n",
            "lume=1\n#end\nk=v\n",
        ] {
            assert!(
                matches!(decode(text, 1), Err(StorageError::Corrupt(_))),
                "{text:?}"
            );
        }
        assert!(decode("lume=1\n# a comment\n\nk=v\n#end\n", 1).is_ok());
    }

    #[test]
    fn save_then_load_and_no_temp_file_is_left() {
        let dir = dir("roundtrip");
        let path = dir.join("config.v1");
        save(&path, 1, &sample()).unwrap();
        assert_eq!(load(&path, 1).unwrap(), sample());
        let names: Vec<_> = fs::read_dir(&dir)
            .unwrap()
            .map(|e| e.unwrap().file_name())
            .collect();
        assert_eq!(names, ["config.v1"]);
    }

    #[test]
    fn saving_again_replaces_the_whole_file() {
        let dir = dir("replace");
        let path = dir.join("config.v1");
        save(&path, 1, &sample()).unwrap();
        let mut smaller = Kv::new();
        smaller.set("only", "this");
        save(&path, 1, &smaller).unwrap();
        assert_eq!(load(&path, 1).unwrap(), smaller);
    }

    #[test]
    fn a_failed_write_keeps_the_previous_file() {
        let dir = dir("failed");
        let path = dir.join("config.v1");
        save(&path, 1, &sample()).unwrap();
        // The temporary name is taken by a directory, so creating it fails.
        fs::create_dir(temp_path(&path)).unwrap();
        let mut other = Kv::new();
        other.set("new", "value");
        assert!(save(&path, 1, &other).is_err());
        assert_eq!(load(&path, 1).unwrap(), sample(), "the old file is intact");
    }

    #[test]
    fn a_leftover_temp_file_from_a_crash_is_ignored() {
        let dir = dir("leftover");
        let path = dir.join("config.v1");
        save(&path, 1, &sample()).unwrap();
        fs::write(temp_path(&path), "lume=1\nhalf=writ").unwrap();
        assert_eq!(load(&path, 1).unwrap(), sample());
        // The next save simply overwrites the leftover.
        save(&path, 1, &sample()).unwrap();
        assert!(!temp_path(&path).exists());
    }

    #[test]
    fn missing_and_unreadable_files() {
        let dir = dir("missing");
        assert_eq!(load(&dir.join("nope"), 1), Err(StorageError::Missing));
        assert!(load_or_log(&dir.join("nope"), 1).is_none());
        let path = dir.join("bad");
        fs::write(&path, [0xff, 0xfe, 0xfd]).unwrap();
        assert!(matches!(load(&path, 1), Err(StorageError::Corrupt(_))));
        assert!(load_or_log(&path, 1).is_none());
        // A directory where a file is expected is an I/O error, not a panic.
        assert!(matches!(load(&dir, 1), Err(StorageError::Io(_))));
    }

    #[test]
    fn a_file_of_another_version_is_ignored_by_load_or_log() {
        let dir = dir("version");
        let path = dir.join("config.v1");
        save(&path, 7, &sample()).unwrap();
        assert!(load_or_log(&path, 1).is_none());
    }

    #[test]
    #[should_panic(expected = "invalid key")]
    fn invalid_keys_are_a_programming_error() {
        Kv::new().set("bad key", "x");
    }
}
