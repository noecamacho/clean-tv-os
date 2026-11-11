//! The clock shown in the top bar.

/// The device's wall clock, as the platform reports it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WallClock {
    /// Seconds since the Unix epoch (UTC).
    pub unix_secs: u64,
    /// Offset of the local time zone from UTC, in seconds (includes daylight saving).
    pub utc_offset_secs: i32,
    /// The system's 24-hour setting.
    pub is_24h: bool,
}

impl WallClock {
    fn local_secs(&self) -> u64 {
        (self.unix_secs as i64 + i64::from(self.utc_offset_secs)).max(0) as u64
    }

    /// Local hour (0 to 23) and minute.
    pub fn hour_minute(&self) -> (u32, u32) {
        let seconds_of_day = self.local_secs() % 86_400;
        (
            (seconds_of_day / 3600) as u32,
            ((seconds_of_day % 3600) / 60) as u32,
        )
    }

    /// The text shown on screen: `21:45`, or `9:45 PM` in 12-hour mode.
    pub fn text(&self) -> String {
        let (hour, minute) = self.hour_minute();
        if self.is_24h {
            format!("{hour:02}:{minute:02}")
        } else {
            let suffix = if hour < 12 { "AM" } else { "PM" };
            let hour12 = match hour % 12 {
                0 => 12,
                h => h,
            };
            format!("{hour12}:{minute:02} {suffix}")
        }
    }

    /// Milliseconds until the displayed minute changes.
    pub fn millis_to_next_minute(&self) -> u64 {
        // Time zone offsets are whole minutes in practice, but this holds for any offset.
        (60 - self.local_secs() % 60) * 1000
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 2026-10-19 21:45:59 UTC.
    const EVENING: u64 = 1_792_446_359;

    fn clock(unix_secs: u64, offset_hours: i32, is_24h: bool) -> WallClock {
        WallClock {
            unix_secs,
            utc_offset_secs: offset_hours * 3600,
            is_24h,
        }
    }

    #[test]
    fn shows_the_local_time_in_24_hour_mode() {
        assert_eq!(clock(EVENING, 0, true).text(), "21:45");
        // Brasília, UTC-3.
        assert_eq!(clock(EVENING, -3, true).text(), "18:45");
        // Crossing midnight.
        assert_eq!(clock(EVENING, 3, true).text(), "00:45");
        assert_eq!(clock(EVENING, 5, true).hour_minute(), (2, 45));
    }

    #[test]
    fn shows_12_hour_time_with_am_pm() {
        assert_eq!(clock(EVENING, 0, false).text(), "9:45 PM");
        assert_eq!(clock(EVENING, 3, false).text(), "12:45 AM");
        assert_eq!(clock(EVENING - 9 * 3600, 0, false).text(), "12:45 PM");
        assert_eq!(
            clock(EVENING - 12 * 3600 - 45 * 60, 0, false).text(),
            "9:00 AM"
        );
    }

    #[test]
    fn single_digit_minutes_are_padded() {
        assert_eq!(clock(EVENING + 20 * 60, 0, true).text(), "22:05");
    }

    #[test]
    fn the_minute_turns_over_in_at_most_a_minute() {
        // 21:45:59 -> one second until 21:46:00.
        assert_eq!(clock(EVENING, 0, true).millis_to_next_minute(), 1000);
        let at_the_turn = clock(EVENING + 1, 0, true);
        assert_eq!(at_the_turn.text(), "21:46");
        assert_eq!(at_the_turn.millis_to_next_minute(), 60_000);
    }

    #[test]
    fn the_minute_deadline_follows_the_displayed_text() {
        let start = clock(EVENING - 30, 0, true);
        for step in 0..120u64 {
            let now = clock(start.unix_secs + step, 0, true);
            let later = clock(now.unix_secs + now.millis_to_next_minute() / 1000, 0, true);
            assert_ne!(
                now.text(),
                later.text(),
                "the text changes exactly at the deadline"
            );
            let just_before = clock(later.unix_secs - 1, 0, true);
            assert_eq!(just_before.text(), now.text());
        }
    }

    #[test]
    fn a_time_zone_offset_before_the_epoch_does_not_underflow() {
        assert_eq!(clock(0, -3, true).text(), "00:00");
    }
}
