//! Scrolling that keeps the focused element visible.

/// The new scroll offset along one axis so that the item `[item_start, item_end]` is fully
/// inside the view `[offset, offset + view_length]`, with `margin` of breathing room.
/// The offset never goes below 0 or above `max_offset`, and does not change when the item
/// already fits.
pub fn reveal(
    offset: f32,
    view_length: f32,
    item_start: f32,
    item_end: f32,
    margin: f32,
    max_offset: f32,
) -> f32 {
    let mut target = offset;
    if item_start - margin < target {
        target = item_start - margin;
    }
    if item_end + margin > target + view_length {
        target = item_end + margin - view_length;
    }
    // An item taller than the view shows its start.
    if item_end - item_start + 2.0 * margin > view_length {
        target = item_start - margin;
    }
    target.clamp(0.0, max_offset.max(0.0))
}

/// The largest useful scroll offset for content of `content_length` in a view of `view_length`.
pub fn max_offset(content_length: f32, view_length: f32) -> f32 {
    (content_length - view_length).max(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn visible_items_do_not_scroll() {
        assert_eq!(reveal(100.0, 600.0, 200.0, 300.0, 20.0, 2000.0), 100.0);
    }

    #[test]
    fn items_below_the_view_scroll_down_just_enough() {
        // View 0..600, item 650..750, margin 20: the bottom edge lands at 770.
        assert_eq!(reveal(0.0, 600.0, 650.0, 750.0, 20.0, 2000.0), 170.0);
    }

    #[test]
    fn items_above_the_view_scroll_up() {
        assert_eq!(reveal(500.0, 600.0, 400.0, 480.0, 20.0, 2000.0), 380.0);
    }

    #[test]
    fn the_offset_is_clamped() {
        assert_eq!(
            reveal(0.0, 600.0, 5.0, 50.0, 20.0, 2000.0),
            0.0,
            "never negative"
        );
        assert_eq!(
            reveal(0.0, 600.0, 1900.0, 1990.0, 20.0, 1400.0),
            1400.0,
            "never past the end"
        );
        assert_eq!(
            reveal(0.0, 600.0, 900.0, 950.0, 20.0, 0.0),
            0.0,
            "content that fits does not scroll"
        );
    }

    #[test]
    fn an_item_taller_than_the_view_shows_its_start() {
        assert_eq!(reveal(0.0, 300.0, 500.0, 1000.0, 10.0, 5000.0), 490.0);
    }

    #[test]
    fn max_offset_of_short_content_is_zero() {
        assert_eq!(max_offset(500.0, 720.0), 0.0);
        assert_eq!(max_offset(1400.0, 720.0), 680.0);
    }
}
