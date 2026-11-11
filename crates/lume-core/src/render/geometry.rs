//! Basic 2D geometry in logical canvas units.

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Rect {
    pub const fn new(x: f32, y: f32, w: f32, h: f32) -> Rect {
        Rect { x, y, w, h }
    }

    pub fn right(&self) -> f32 {
        self.x + self.w
    }

    pub fn bottom(&self) -> f32 {
        self.y + self.h
    }

    pub fn center(&self) -> (f32, f32) {
        (self.x + self.w / 2.0, self.y + self.h / 2.0)
    }

    pub fn is_empty(&self) -> bool {
        self.w <= 0.0 || self.h <= 0.0
    }

    /// Grows the rect by `amount` on every side (shrinks when negative).
    pub fn grow(&self, amount: f32) -> Rect {
        Rect::new(
            self.x - amount,
            self.y - amount,
            self.w + 2.0 * amount,
            self.h + 2.0 * amount,
        )
    }

    pub fn translate(&self, dx: f32, dy: f32) -> Rect {
        Rect::new(self.x + dx, self.y + dy, self.w, self.h)
    }

    /// Scales the rect around its center, as CSS `transform: scale()` does.
    pub fn scaled_about_center(&self, factor: f32) -> Rect {
        let (cx, cy) = self.center();
        let (w, h) = (self.w * factor, self.h * factor);
        Rect::new(cx - w / 2.0, cy - h / 2.0, w, h)
    }

    pub fn intersects(&self, other: &Rect) -> bool {
        self.x < other.right()
            && other.x < self.right()
            && self.y < other.bottom()
            && other.y < self.bottom()
    }

    /// The overlapping area, or `None` when the rects do not overlap.
    pub fn intersection(&self, other: &Rect) -> Option<Rect> {
        let x = self.x.max(other.x);
        let y = self.y.max(other.y);
        let right = self.right().min(other.right());
        let bottom = self.bottom().min(other.bottom());
        (right > x && bottom > y).then(|| Rect::new(x, y, right - x, bottom - y))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn intersection_of_overlapping_and_disjoint_rects() {
        let a = Rect::new(0.0, 0.0, 10.0, 10.0);
        assert_eq!(
            a.intersection(&Rect::new(5.0, 5.0, 10.0, 10.0)),
            Some(Rect::new(5.0, 5.0, 5.0, 5.0))
        );
        assert_eq!(a.intersection(&Rect::new(10.0, 0.0, 5.0, 5.0)), None);
        assert!(!a.intersects(&Rect::new(20.0, 20.0, 1.0, 1.0)));
    }

    #[test]
    fn scaling_keeps_the_center() {
        let r = Rect::new(100.0, 100.0, 200.0, 100.0).scaled_about_center(1.1);
        assert!((r.center().0 - 200.0).abs() < 1e-4 && (r.center().1 - 150.0).abs() < 1e-4);
        assert!((r.w - 220.0).abs() < 1e-3);
    }
}
