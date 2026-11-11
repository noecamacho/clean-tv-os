//! Animations by interpolation, driven by the theme's durations and curves.
//!
//! Nothing runs on its own: the app passes the current time (milliseconds from a monotonic
//! clock) and draws a frame only while [`Animations::next_deadline`] says something is moving.

use crate::theme::{Curve, Transition};
use std::collections::HashMap;
use std::hash::Hash;

/// Time between animation frames while something is moving (about 60 per second).
pub const FRAME_MS: u64 = 16;

/// Maps linear progress `t` (0 to 1) to eased progress.
pub fn ease(curve: Curve, t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    match curve {
        Curve::Linear => t,
        Curve::Ease => cubic_bezier(0.25, 0.1, 0.25, 1.0, t),
        Curve::EaseIn => cubic_bezier(0.42, 0.0, 1.0, 1.0, t),
        Curve::EaseOut => cubic_bezier(0.0, 0.0, 0.58, 1.0, t),
        Curve::EaseInOut => cubic_bezier(0.42, 0.0, 0.58, 1.0, t),
        Curve::CubicBezier(x1, y1, x2, y2) => cubic_bezier(x1, y1, x2, y2, t),
    }
}

/// The CSS `cubic-bezier(x1, y1, x2, y2)` timing function evaluated at time `x`.
fn cubic_bezier(x1: f32, y1: f32, x2: f32, y2: f32, x: f32) -> f32 {
    let sample = |a: f32, b: f32, t: f32| {
        3.0 * a * (1.0 - t).powi(2) * t + 3.0 * b * (1.0 - t) * t * t + t.powi(3)
    };
    // Solve sample(x1, x2, t) = x for t by bisection (the curve is monotonic in x for CSS inputs).
    let (mut low, mut high) = (0.0f32, 1.0f32);
    for _ in 0..24 {
        let mid = (low + high) / 2.0;
        if sample(x1, x2, mid) < x {
            low = mid;
        } else {
            high = mid;
        }
    }
    sample(y1, y2, (low + high) / 2.0)
}

/// One value moving from where it was to a target.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tween {
    from: f32,
    to: f32,
    start: u64,
    duration: u64,
    curve: Curve,
}

impl Tween {
    /// A value at rest.
    pub fn at(value: f32) -> Tween {
        Tween {
            from: value,
            to: value,
            start: 0,
            duration: 0,
            curve: Curve::Linear,
        }
    }

    pub fn value(&self, now: u64) -> f32 {
        if self.duration == 0 || now >= self.start + self.duration {
            return self.to;
        }
        let t = now.saturating_sub(self.start) as f32 / self.duration as f32;
        self.from + (self.to - self.from) * ease(self.curve, t)
    }

    pub fn target(&self) -> f32 {
        self.to
    }

    pub fn is_animating(&self, now: u64) -> bool {
        now < self.start + self.duration
    }

    /// Starts moving to `to` from the current value. Without a transition (or with a zero
    /// duration) the value jumps. Retargeting to the same target keeps the running animation.
    pub fn animate_to(&mut self, now: u64, to: f32, transition: Option<Transition>) {
        if (to - self.to).abs() < f32::EPSILON {
            return;
        }
        let from = self.value(now);
        match transition.filter(|t| t.duration_ms > 0.0) {
            Some(t) => {
                *self = Tween {
                    from,
                    to,
                    start: now,
                    duration: t.duration_ms.round() as u64,
                    curve: t.curve,
                }
            }
            None => *self = Tween::at(to),
        }
    }
}

/// A set of named animated values.
#[derive(Clone, Debug)]
pub struct Animations<K: Eq + Hash> {
    values: HashMap<K, Tween>,
}

impl<K: Eq + Hash> Default for Animations<K> {
    fn default() -> Self {
        Animations {
            values: HashMap::new(),
        }
    }
}

impl<K: Eq + Hash + Copy> Animations<K> {
    pub fn new() -> Self {
        Self::default()
    }

    /// The current value of `key`, or `rest` when it has never been animated.
    pub fn get(&self, key: K, rest: f32, now: u64) -> f32 {
        self.values.get(&key).map_or(rest, |t| t.value(now))
    }

    /// Moves `key` toward `target`. A key that has never been animated starts at `rest`.
    pub fn animate_to(
        &mut self,
        key: K,
        rest: f32,
        target: f32,
        now: u64,
        transition: Option<Transition>,
    ) {
        self.values
            .entry(key)
            .or_insert_with(|| Tween::at(rest))
            .animate_to(now, target, transition);
    }

    /// Sets a value without animating it.
    pub fn set(&mut self, key: K, value: f32) {
        self.values.insert(key, Tween::at(value));
    }

    pub fn is_animating(&self, now: u64) -> bool {
        self.values.values().any(|t| t.is_animating(now))
    }

    /// Whether one of the values `which` picks is moving.
    pub fn is_animating_where(&self, now: u64, which: impl Fn(&K) -> bool) -> bool {
        self.values
            .iter()
            .any(|(key, t)| which(key) && t.is_animating(now))
    }

    /// When the next frame is due: one frame from `now` while something moves, `None` when
    /// everything is at rest (so the event loop can sleep).
    pub fn next_deadline(&self, now: u64) -> Option<u64> {
        self.is_animating(now).then_some(now + FRAME_MS)
    }

    /// Ends every animation but those `keep` picks at its target, as if its time had run out.
    pub fn settle(&mut self, keep: impl Fn(&K) -> bool) {
        for (key, tween) in self.values.iter_mut() {
            if !keep(key) {
                *tween = Tween::at(tween.target());
            }
        }
    }

    /// Forgets values that finished at their resting value, to keep the set small.
    pub fn prune(&mut self, now: u64, rest: impl Fn(&K) -> f32) {
        self.values.retain(|key, tween| {
            tween.is_animating(now) || (tween.target() - rest(key)).abs() > f32::EPSILON
        });
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn transition(ms: f32, curve: Curve) -> Option<Transition> {
        Some(Transition {
            duration_ms: ms,
            curve,
        })
    }

    #[test]
    fn every_curve_starts_at_zero_and_ends_at_one() {
        for curve in [
            Curve::Linear,
            Curve::Ease,
            Curve::EaseIn,
            Curve::EaseOut,
            Curve::EaseInOut,
            Curve::CubicBezier(0.2, 0.0, 0.0, 1.0),
        ] {
            assert!(ease(curve, 0.0).abs() < 1e-4, "{curve:?}");
            assert!((ease(curve, 1.0) - 1.0).abs() < 1e-4, "{curve:?}");
        }
    }

    #[test]
    fn curves_are_monotonic_and_shaped_as_named() {
        for curve in [
            Curve::Linear,
            Curve::Ease,
            Curve::EaseIn,
            Curve::EaseOut,
            Curve::EaseInOut,
            Curve::CubicBezier(0.2, 0.0, 0.0, 1.0),
        ] {
            let mut previous = -1.0;
            for i in 0..=100 {
                let value = ease(curve, i as f32 / 100.0);
                assert!(value >= previous - 1e-4, "{curve:?} goes backwards at {i}");
                previous = value;
            }
        }
        // Ease-out is fast at first, ease-in slow at first.
        assert!(ease(Curve::EaseOut, 0.5) > 0.6);
        assert!(ease(Curve::EaseIn, 0.5) < 0.4);
        assert!((ease(Curve::EaseInOut, 0.5) - 0.5).abs() < 0.01);
        assert!((ease(Curve::Linear, 0.3) - 0.3).abs() < 1e-6);
    }

    #[test]
    fn out_of_range_progress_is_clamped() {
        assert_eq!(ease(Curve::Linear, -1.0), 0.0);
        assert_eq!(ease(Curve::Linear, 2.0), 1.0);
    }

    #[test]
    fn a_tween_moves_from_the_start_to_the_target_over_its_duration() {
        let mut tween = Tween::at(1.0);
        tween.animate_to(1000, 1.05, transition(200.0, Curve::Linear));
        assert_eq!(tween.value(1000), 1.0);
        assert!((tween.value(1100) - 1.025).abs() < 1e-5);
        assert_eq!(tween.value(1200), 1.05);
        assert_eq!(tween.value(5000), 1.05);
        assert!(tween.is_animating(1199) && !tween.is_animating(1200));
    }

    #[test]
    fn retargeting_continues_from_the_current_value() {
        let mut tween = Tween::at(0.0);
        tween.animate_to(0, 100.0, transition(100.0, Curve::Linear));
        // Halfway there, the target flips back.
        tween.animate_to(50, 0.0, transition(100.0, Curve::Linear));
        assert!(
            (tween.value(50) - 50.0).abs() < 1e-3,
            "no jump at the change"
        );
        assert!((tween.value(100) - 25.0).abs() < 1e-3);
        assert_eq!(tween.value(150), 0.0);
    }

    #[test]
    fn the_same_target_does_not_restart_the_animation() {
        let mut tween = Tween::at(0.0);
        tween.animate_to(0, 10.0, transition(100.0, Curve::Linear));
        tween.animate_to(60, 10.0, transition(100.0, Curve::Linear));
        assert!((tween.value(80) - 8.0).abs() < 1e-3);
        assert!(!tween.is_animating(100));
    }

    #[test]
    fn no_transition_jumps_straight_to_the_target() {
        let mut tween = Tween::at(0.0);
        tween.animate_to(10, 5.0, None);
        assert_eq!(tween.value(10), 5.0);
        assert!(!tween.is_animating(10));
        tween.animate_to(20, 9.0, transition(0.0, Curve::Ease));
        assert_eq!(tween.value(20), 9.0);
    }

    #[test]
    fn the_set_reports_when_to_draw_the_next_frame() {
        let mut anim: Animations<u32> = Animations::new();
        assert_eq!(anim.next_deadline(0), None);
        anim.animate_to(7, 1.0, 1.05, 100, transition(150.0, Curve::EaseOut));
        assert_eq!(anim.next_deadline(100), Some(100 + FRAME_MS));
        assert_eq!(anim.next_deadline(240), Some(240 + FRAME_MS));
        assert_eq!(
            anim.next_deadline(250),
            None,
            "the animation ended: the loop can sleep"
        );
        assert_eq!(anim.get(7, 1.0, 250), 1.05);
        assert_eq!(anim.get(8, 1.0, 250), 1.0, "unknown keys are at rest");
    }

    #[test]
    fn several_values_animate_independently() {
        let mut anim: Animations<&str> = Animations::new();
        anim.animate_to("scale", 1.0, 1.05, 0, transition(100.0, Curve::Linear));
        anim.animate_to("scroll", 0.0, 300.0, 0, transition(300.0, Curve::Linear));
        assert!((anim.get("scale", 1.0, 50) - 1.025).abs() < 1e-5);
        assert!((anim.get("scroll", 0.0, 150) - 150.0).abs() < 1e-3);
        assert!(
            anim.is_animating(150),
            "the scroll is still going after the scale ended"
        );
        assert!(!anim.is_animating(300));
    }

    #[test]
    fn finished_values_at_rest_are_pruned() {
        let mut anim: Animations<u32> = Animations::new();
        anim.animate_to(1, 1.0, 1.05, 0, transition(100.0, Curve::Linear));
        anim.animate_to(2, 1.0, 1.0, 0, transition(100.0, Curve::Linear));
        anim.set(3, 1.0);
        anim.prune(500, |_| 1.0);
        assert_eq!(anim.len(), 1, "only the value away from rest stays");
        anim.animate_to(1, 1.0, 1.0, 500, transition(100.0, Curve::Linear));
        anim.prune(700, |_| 1.0);
        assert!(anim.is_empty());
    }
}
