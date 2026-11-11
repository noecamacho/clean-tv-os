//! Apps lume knows by package name: their tile colors come from the theme, not from the icon.

use crate::theme::{vars, Color};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum KnownApp {
    Netflix,
    YouTube,
    PrimeVideo,
    DisneyPlus,
    Max,
    Spotify,
    Globoplay,
    PlayStore,
    Chrome,
}

impl KnownApp {
    /// Apps lume suggests as favorites on first use, in this order.
    pub const DEFAULT_FAVORITES: [KnownApp; 7] = [
        KnownApp::Netflix,
        KnownApp::YouTube,
        KnownApp::PrimeVideo,
        KnownApp::Spotify,
        KnownApp::DisneyPlus,
        KnownApp::Max,
        KnownApp::Globoplay,
    ];

    /// Every known app.
    pub const ALL: [KnownApp; 9] = [
        KnownApp::Netflix,
        KnownApp::YouTube,
        KnownApp::PrimeVideo,
        KnownApp::DisneyPlus,
        KnownApp::Max,
        KnownApp::Spotify,
        KnownApp::Globoplay,
        KnownApp::PlayStore,
        KnownApp::Chrome,
    ];

    /// Package names of the app's TV and mobile builds.
    pub const fn packages(self) -> &'static [&'static str] {
        match self {
            KnownApp::Netflix => &["com.netflix.ninja", "com.netflix.mediaclient"],
            KnownApp::YouTube => &[
                "com.google.android.youtube.tv",
                "com.google.android.youtube",
            ],
            KnownApp::PrimeVideo => &[
                "com.amazon.amazonvideo.livingroom",
                "com.amazon.avod.thirdpartyclient",
            ],
            KnownApp::DisneyPlus => &["com.disney.disneyplus"],
            KnownApp::Max => &["com.wbd.stream", "com.hbo.hbonow", "com.hbo.hbomax"],
            KnownApp::Spotify => &["com.spotify.tv.android", "com.spotify.music"],
            KnownApp::Globoplay => &["com.globo.globotv", "com.globo.globoplay"],
            KnownApp::PlayStore => &["com.android.vending"],
            KnownApp::Chrome => &["com.android.chrome", "com.google.android.apps.chrome"],
        }
    }

    /// The tile color defined by the theme.
    pub const fn color(self) -> Color {
        match self {
            KnownApp::Netflix => vars::BRAND_NETFLIX,
            KnownApp::YouTube => vars::BRAND_YOUTUBE,
            KnownApp::PrimeVideo => vars::BRAND_PRIME_VIDEO,
            KnownApp::DisneyPlus => vars::BRAND_DISNEY_PLUS,
            KnownApp::Max => vars::BRAND_MAX,
            KnownApp::Spotify => vars::BRAND_SPOTIFY,
            KnownApp::Globoplay => vars::BRAND_GLOBOPLAY,
            KnownApp::PlayStore => vars::BRAND_PLAY_STORE,
            KnownApp::Chrome => vars::BRAND_CHROME,
        }
    }

    pub fn from_package(package: &str) -> Option<KnownApp> {
        KnownApp::ALL
            .into_iter()
            .find(|app| app.packages().contains(&package))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::color::{contrast_with_white, MIN_CONTRAST};

    #[test]
    fn packages_map_to_apps() {
        assert_eq!(
            KnownApp::from_package("com.netflix.ninja"),
            Some(KnownApp::Netflix)
        );
        assert_eq!(
            KnownApp::from_package("com.netflix.mediaclient"),
            Some(KnownApp::Netflix)
        );
        assert_eq!(
            KnownApp::from_package("com.spotify.music"),
            Some(KnownApp::Spotify)
        );
        assert_eq!(KnownApp::from_package("com.example.unknown"), None);
        assert_eq!(KnownApp::from_package(""), None);
    }

    #[test]
    fn no_package_belongs_to_two_apps() {
        let mut seen = std::collections::HashSet::new();
        for app in KnownApp::ALL {
            for package in app.packages() {
                assert!(seen.insert(*package), "{package} is listed twice");
            }
        }
    }

    #[test]
    fn spotify_uses_the_green_of_the_theme() {
        assert_eq!(KnownApp::Spotify.color(), Color::rgb(0x15, 0x7A, 0x3E));
    }

    #[test]
    fn every_known_color_reads_well_with_white_text() {
        for app in KnownApp::ALL {
            assert!(contrast_with_white(app.color()) >= MIN_CONTRAST, "{app:?}");
        }
    }

    #[test]
    fn the_default_favorites_follow_the_spec_order() {
        assert_eq!(
            KnownApp::DEFAULT_FAVORITES,
            [
                KnownApp::Netflix,
                KnownApp::YouTube,
                KnownApp::PrimeVideo,
                KnownApp::Spotify,
                KnownApp::DisneyPlus,
                KnownApp::Max,
                KnownApp::Globoplay
            ]
        );
    }
}
