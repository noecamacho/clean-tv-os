//! Fonts embedded in the binary: Plus Jakarta Sans (OFL) and a Lucide icon subset (ISC).
//!
//! The text faces cover Latin-1 and Latin Extended-A; other scripts come from the
//! system fonts at runtime. Regenerate the files with `scripts/subset-fonts.py`.

/// Weight 500.
pub const MEDIUM: &[u8] = include_bytes!("../assets/fonts/PlusJakartaSans-Medium.ttf");
/// Weight 700.
pub const BOLD: &[u8] = include_bytes!("../assets/fonts/PlusJakartaSans-Bold.ttf");
/// Weight 800.
pub const EXTRA_BOLD: &[u8] = include_bytes!("../assets/fonts/PlusJakartaSans-ExtraBold.ttf");
/// Icons; see [`crate::icons`] for the code points.
pub const ICONS: &[u8] = include_bytes!("../assets/fonts/lucide-subset.ttf");

/// Text faces as `(weight, data)`, lightest first.
pub const TEXT_FACES: [(u16, &[u8]); 3] = [(500, MEDIUM), (700, BOLD), (800, EXTRA_BOLD)];

/// Picks the face closest to the requested CSS weight (the heavier one on a tie).
pub fn face_for_weight(weight: u16) -> &'static [u8] {
    TEXT_FACES
        .iter()
        .rev()
        .min_by_key(|(w, _)| w.abs_diff(weight))
        .map(|(_, data)| *data)
        .expect("TEXT_FACES is not empty")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::icons;
    use fontdue::{Font, FontSettings};

    fn load(data: &[u8]) -> Font {
        Font::from_bytes(data, FontSettings::default()).expect("the font must load")
    }

    #[test]
    fn total_font_size_stays_under_the_budget() {
        let total: usize = [MEDIUM, BOLD, EXTRA_BOLD, ICONS]
            .iter()
            .map(|f| f.len())
            .sum();
        assert!(total < 200 * 1024, "fonts take {total} bytes");
    }

    #[test]
    fn text_faces_cover_latin_1_and_latin_extended_a() {
        for (weight, data) in TEXT_FACES {
            let font = load(data);
            for c in "AaÀÿçãõñÅŁłŠžĐőŰ€…›·–—“”".chars() {
                assert!(
                    font.lookup_glyph_index(c) != 0,
                    "weight {weight} lacks {c:?}"
                );
            }
        }
    }

    #[test]
    fn icon_font_has_every_icon() {
        let font = load(ICONS);
        for c in [
            icons::SETTINGS,
            icons::CHEVRON_UP,
            icons::CHEVRON_DOWN,
            icons::CHEVRON_LEFT,
            icons::CHEVRON_RIGHT,
            icons::TRASH,
            icons::CLOSE,
            icons::CHECK,
            icons::PLUS,
            icons::PLAY,
        ] {
            assert!(font.lookup_glyph_index(c) != 0, "missing icon {c:?}");
            assert!(
                font.rasterize(c, 24.0).1.iter().any(|&p| p > 0),
                "icon {c:?} is blank"
            );
        }
    }

    #[test]
    fn nearest_face_is_chosen_for_a_weight() {
        assert_eq!(face_for_weight(400), MEDIUM);
        assert_eq!(face_for_weight(600), BOLD);
        assert_eq!(face_for_weight(700), BOLD);
        assert_eq!(face_for_weight(740), BOLD);
        assert_eq!(face_for_weight(750), EXTRA_BOLD);
        assert_eq!(face_for_weight(900), EXTRA_BOLD);
    }

    #[test]
    fn kerning_is_available_for_classic_pairs() {
        let font = load(BOLD);
        let kern = font
            .horizontal_kern('A', 'V', 56.0)
            .expect("fontdue found no kerning for A-V");
        // "AV" is tightened, never loosened.
        assert!(kern < -0.5, "A-V kerning at 56px is {kern}");
    }
}
