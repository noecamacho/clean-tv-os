//! Local usage history: when each app was first seen and opened by lume.
//!
//! The data never leaves the device. It is kept in `usage.v1` (see [`crate::storage`]).
//! Times are seconds since the Unix epoch and are always passed in, so tests control the clock.

use crate::storage::{self, Kv, StorageError};
use std::collections::BTreeMap;
use std::path::Path;

const VERSION: u32 = 1;
const WEEK: u64 = 7 * 24 * 60 * 60;
/// Most opens remembered per app; far above what a TV sees in a week.
const MAX_OPENS_PER_APP: usize = 100;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct AppUsage {
    first_seen: Option<u64>,
    last_opened: Option<u64>,
    /// Open times of the last 7 days, oldest first.
    opens: Vec<u64>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct UsageHistory {
    apps: BTreeMap<String, AppUsage>,
}

impl UsageHistory {
    pub fn new() -> UsageHistory {
        UsageHistory::default()
    }

    /// Remembers the first time an app was seen; later calls keep the earlier time.
    /// Returns `true` when the app had never been seen before.
    pub fn mark_seen(&mut self, package: &str, now: u64) -> bool {
        let usage = self.apps.entry(package.to_string()).or_default();
        if usage.first_seen.is_none() {
            usage.first_seen = Some(now);
            true
        } else {
            false
        }
    }

    /// Marks apps that were already installed when lume first ran as seen long ago, so
    /// that only apps installed later count as new.
    pub fn mark_baseline<'a>(&mut self, packages: impl IntoIterator<Item = &'a str>) {
        for package in packages {
            self.apps
                .entry(package.to_string())
                .or_default()
                .first_seen
                .get_or_insert(0);
        }
    }

    /// Records that the user opened `package` from lume.
    pub fn record_open(&mut self, package: &str, now: u64) {
        let usage = self.apps.entry(package.to_string()).or_default();
        usage.first_seen.get_or_insert(now);
        usage.last_opened = Some(usage.last_opened.map_or(now, |last| last.max(now)));
        usage.opens.push(now);
        prune(usage, now);
    }

    pub fn first_seen(&self, package: &str) -> Option<u64> {
        self.apps.get(package)?.first_seen
    }

    pub fn last_opened(&self, package: &str) -> Option<u64> {
        self.apps.get(package)?.last_opened
    }

    /// How many times `package` was opened in the 7 days before `now`.
    pub fn opens_in_last_7_days(&self, package: &str, now: u64) -> u32 {
        let since = now.saturating_sub(WEEK);
        self.apps.get(package).map_or(0, |u| {
            u.opens.iter().filter(|&&t| t > since && t <= now).count() as u32
        })
    }

    /// Whether the app was first seen less than 7 days before `now`.
    pub fn is_new(&self, package: &str, now: u64) -> bool {
        self.first_seen(package)
            .is_some_and(|seen| now.saturating_sub(seen) < WEEK)
    }

    /// The app with the most opens in the last 7 days among `packages`, if any has at least one.
    /// Ties go to the app listed first.
    pub fn most_used<'a>(
        &self,
        packages: impl IntoIterator<Item = &'a str>,
        now: u64,
    ) -> Option<&'a str> {
        let mut best: Option<(&str, u32)> = None;
        for package in packages {
            let count = self.opens_in_last_7_days(package, now);
            if count > 0 && best.is_none_or(|(_, top)| count > top) {
                best = Some((package, count));
            }
        }
        best.map(|(package, _)| package)
    }

    /// Forgets an app that was uninstalled.
    pub fn forget(&mut self, package: &str) {
        self.apps.remove(package);
    }

    /// Drops opens older than 7 days everywhere.
    pub fn prune(&mut self, now: u64) {
        for usage in self.apps.values_mut() {
            prune(usage, now);
        }
    }

    fn to_kv(&self) -> Kv {
        let mut kv = Kv::new();
        kv.set("count", self.apps.len().to_string());
        for (i, (package, usage)) in self.apps.iter().enumerate() {
            kv.set(&format!("u{i}.package"), package.as_str());
            if let Some(t) = usage.first_seen {
                kv.set(&format!("u{i}.first_seen"), t.to_string());
            }
            if let Some(t) = usage.last_opened {
                kv.set(&format!("u{i}.last_opened"), t.to_string());
            }
            let opens: Vec<String> = usage.opens.iter().map(u64::to_string).collect();
            kv.set_list(&format!("u{i}.opens"), opens.iter().map(String::as_str));
        }
        kv
    }

    fn from_kv(kv: &Kv) -> Option<UsageHistory> {
        let count: usize = kv.parse("count")?;
        let mut history = UsageHistory::new();
        for i in 0..count {
            let package = kv.get(&format!("u{i}.package"))?.to_string();
            let mut opens: Vec<u64> = kv
                .list(&format!("u{i}.opens"))?
                .into_iter()
                .map(|t| t.parse().ok())
                .collect::<Option<_>>()?;
            opens.sort_unstable();
            history.apps.insert(
                package,
                AppUsage {
                    first_seen: kv.parse(&format!("u{i}.first_seen")),
                    last_opened: kv.parse(&format!("u{i}.last_opened")),
                    opens,
                },
            );
        }
        Some(history)
    }

    /// Writes the history file (`usage.v1`).
    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        storage::save(path, VERSION, &self.to_kv())
    }

    /// Reads the history; a missing or unusable file starts an empty history.
    pub fn load(path: &Path) -> UsageHistory {
        match storage::load(path, VERSION) {
            Ok(kv) => UsageHistory::from_kv(&kv).unwrap_or_else(|| {
                log::warn!("ignoring {}: incomplete usage entries", path.display());
                UsageHistory::new()
            }),
            Err(StorageError::Missing) => UsageHistory::new(),
            Err(e) => {
                log::warn!("ignoring {}: {e}", path.display());
                UsageHistory::new()
            }
        }
    }
}

fn prune(usage: &mut AppUsage, now: u64) {
    let since = now.saturating_sub(WEEK);
    usage.opens.retain(|&t| t > since);
    if usage.opens.len() > MAX_OPENS_PER_APP {
        let excess = usage.opens.len() - MAX_OPENS_PER_APP;
        usage.opens.drain(..excess);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DAY: u64 = 24 * 60 * 60;
    const T0: u64 = 1_800_000_000;

    #[test]
    fn opening_an_app_records_the_time_and_counts() {
        let mut history = UsageHistory::new();
        history.record_open("com.youtube", T0);
        assert_eq!(history.last_opened("com.youtube"), Some(T0));
        assert_eq!(history.opens_in_last_7_days("com.youtube", T0), 1);
        history.record_open("com.youtube", T0 + 60);
        assert_eq!(history.last_opened("com.youtube"), Some(T0 + 60));
        assert_eq!(history.opens_in_last_7_days("com.youtube", T0 + 60), 2);
        assert_eq!(history.opens_in_last_7_days("com.other", T0), 0);
        assert_eq!(history.last_opened("com.other"), None);
    }

    #[test]
    fn counts_cover_only_the_last_seven_days() {
        let mut history = UsageHistory::new();
        history.record_open("a", T0);
        history.record_open("a", T0 + 3 * DAY);
        history.record_open("a", T0 + 6 * DAY);
        assert_eq!(history.opens_in_last_7_days("a", T0 + 6 * DAY), 3);
        // The first open is exactly 7 days old: outside the window.
        assert_eq!(history.opens_in_last_7_days("a", T0 + 7 * DAY), 2);
        assert_eq!(history.opens_in_last_7_days("a", T0 + 8 * DAY), 2);
        assert_eq!(history.opens_in_last_7_days("a", T0 + 20 * DAY), 0);
        // The last opening time is kept after the counts expire.
        assert_eq!(history.last_opened("a"), Some(T0 + 6 * DAY));
    }

    #[test]
    fn first_seen_is_set_once() {
        let mut history = UsageHistory::new();
        assert!(history.mark_seen("a", T0));
        assert!(!history.mark_seen("a", T0 + 100));
        assert_eq!(history.first_seen("a"), Some(T0));
        // Opening an app never seen before also counts as seeing it.
        history.record_open("b", T0 + 5);
        assert_eq!(history.first_seen("b"), Some(T0 + 5));
        assert!(!history.mark_seen("b", T0 + 9));
    }

    #[test]
    fn the_baseline_makes_existing_apps_old() {
        let mut history = UsageHistory::new();
        history.mark_baseline(["a", "b"]);
        assert!(!history.is_new("a", T0));
        assert_eq!(history.first_seen("b"), Some(0));
        // An app seen earlier keeps its time.
        history.mark_seen("c", T0);
        history.mark_baseline(["c"]);
        assert_eq!(history.first_seen("c"), Some(T0));
        // A later install is new.
        assert!(history.mark_seen("d", T0 + 5));
        assert!(history.is_new("d", T0 + 6));
    }

    #[test]
    fn an_app_is_new_for_seven_days() {
        let mut history = UsageHistory::new();
        history.mark_seen("a", T0);
        assert!(history.is_new("a", T0));
        assert!(history.is_new("a", T0 + 7 * DAY - 1));
        assert!(!history.is_new("a", T0 + 7 * DAY));
        assert!(!history.is_new("unknown", T0));
    }

    #[test]
    fn most_used_picks_the_highest_weekly_count() {
        let mut history = UsageHistory::new();
        for i in 0..3 {
            history.record_open("youtube", T0 + i);
        }
        history.record_open("netflix", T0);
        history.record_open("prime", T0 - 8 * DAY);
        let apps = ["netflix", "youtube", "prime", "unused"];
        assert_eq!(history.most_used(apps, T0 + 10), Some("youtube"));
        // Ties go to the app listed first.
        history.record_open("netflix", T0 + 1);
        history.record_open("netflix", T0 + 2);
        assert_eq!(history.most_used(apps, T0 + 10), Some("netflix"));
        assert_eq!(history.most_used(["prime", "unused"], T0 + 10), None);
        assert_eq!(history.most_used([], T0), None);
    }

    #[test]
    fn forgetting_and_pruning() {
        let mut history = UsageHistory::new();
        history.record_open("a", T0);
        history.record_open("b", T0 + 6 * DAY);
        history.prune(T0 + 8 * DAY);
        assert_eq!(history.opens_in_last_7_days("a", T0 + 8 * DAY), 0);
        assert_eq!(history.opens_in_last_7_days("b", T0 + 8 * DAY), 1);
        history.forget("a");
        assert_eq!(history.last_opened("a"), None);
        assert_eq!(history.first_seen("a"), None);
    }

    #[test]
    fn the_number_of_remembered_opens_is_bounded() {
        let mut history = UsageHistory::new();
        for i in 0..300 {
            history.record_open("a", T0 + i);
        }
        assert_eq!(
            history.opens_in_last_7_days("a", T0 + 300),
            MAX_OPENS_PER_APP as u32
        );
    }

    fn temp(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("lume-usage-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir.join("usage.v1")
    }

    #[test]
    fn the_history_round_trips_through_the_file() {
        let mut history = UsageHistory::new();
        history.mark_seen("netflix", T0 - DAY);
        history.record_open("youtube", T0);
        history.record_open("youtube", T0 + 90);
        history.mark_seen("empty.opens", T0);
        let path = temp("roundtrip");
        history.save(&path).unwrap();
        assert_eq!(UsageHistory::load(&path), history);
    }

    #[test]
    fn a_missing_or_damaged_file_starts_empty() {
        let path = temp("damaged");
        assert_eq!(UsageHistory::load(&path), UsageHistory::new());
        std::fs::write(&path, "lume=1\ncount=1\n#end\n").unwrap();
        assert_eq!(UsageHistory::load(&path), UsageHistory::new());
        std::fs::write(&path, "not a usage file").unwrap();
        assert_eq!(UsageHistory::load(&path), UsageHistory::new());
        // A bad timestamp makes the whole file unusable rather than half-loaded.
        let mut kv = Kv::new();
        kv.set("count", "1");
        kv.set("u0.package", "a");
        kv.set("u0.opens", "12,oops");
        storage::save(&path, 1, &kv).unwrap();
        assert_eq!(UsageHistory::load(&path), UsageHistory::new());
    }

    #[test]
    fn clearing_app_data_means_no_history() {
        // The file lives in the app's private storage; deleting it (what "clear data" does) leaves nothing.
        let mut history = UsageHistory::new();
        history.record_open("a", T0);
        let path = temp("clear");
        history.save(&path).unwrap();
        std::fs::remove_file(&path).unwrap();
        assert_eq!(UsageHistory::load(&path), UsageHistory::new());
    }
}
