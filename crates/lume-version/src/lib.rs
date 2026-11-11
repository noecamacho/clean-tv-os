//! The version of the build, resolved by `build.rs` (see the `platform-compatibility` spec).

/// The version of the build: a release (`0.3.0`), a commit (`4d8438b`, `4d8438b-dirty`) or `dev`.
pub const VERSION: &str = env!("LUME_BUILD_VERSION");

/// The version as shown to the user: a release gets a leading "v", anything else is as is.
pub fn display() -> String {
    display_of(VERSION)
}

fn display_of(version: &str) -> String {
    if is_release(version) {
        format!("v{version}")
    } else {
        version.to_string()
    }
}

/// True for `X.Y.Z` with an optional pre-release suffix (`0.3.0`, `0.3.0-rc1`).
fn is_release(version: &str) -> bool {
    let numbers = version.split('-').next().unwrap_or("");
    let mut parts = numbers.split('.');
    let mut count = 0;
    for part in parts.by_ref().take(3) {
        if part.is_empty() || !part.bytes().all(|b| b.is_ascii_digit()) {
            return false;
        }
        count += 1;
    }
    count == 3 && parts.next().is_none()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn release_gets_a_v() {
        assert_eq!(display_of("0.3.0"), "v0.3.0");
        assert_eq!(display_of("0.3.0-rc1"), "v0.3.0-rc1");
    }

    #[test]
    fn commit_and_dev_stay_as_is() {
        assert_eq!(display_of("4d8438b"), "4d8438b");
        assert_eq!(display_of("4d8438b-dirty"), "4d8438b-dirty");
        assert_eq!(display_of("dev"), "dev");
        assert_eq!(display_of("1234567"), "1234567");
    }

    #[test]
    fn version_has_no_leading_v() {
        assert!(!VERSION.starts_with('v'));
        assert!(!VERSION.is_empty());
    }
}
