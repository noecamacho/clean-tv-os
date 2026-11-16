//! The Control Center, as on tvOS 26: no pane, glass controls floating in the top right corner
//! over the dimmed picture. Brightness and volume are sliders the remote moves in place; the
//! rest opens the screen of the TV that owns the setting.

use super::painter::Painter;
use super::{ids, Anim, Effect, LumeApp, PendingFocus, Screen, Tab};
use crate::i18n::StringId;
use crate::keys::SettingShortcut;
use crate::render::Rect;
use crate::text::{Align, TextStyle};
use crate::theme::{Color, Paint};
use crate::ui::focus::{Direction, Focusable, Row};

/// The column of controls, inside the 54-unit side margin of Home.
const WIDTH: f32 = 456.0;
const X: f32 = 1280.0 - 54.0 - WIDTH;
const Y: f32 = 92.0;
const GAP: f32 = 10.0;
const HALF: f32 = (WIDTH - GAP) / 2.0;
/// A pill: a tile of one line, and a slider.
const PILL: f32 = 69.0;
const ROUND: f32 = 56.0;
/// One press of Left or Right on a slider.
const STEP: i16 = 5;
/// What OK on the timer goes through, in minutes.
const TIMER_STEPS: [u16; 5] = [0, 15, 30, 60, 120];
/// The controls come down from the corner they belong to.
const OPEN: crate::theme::Transition = crate::theme::Transition {
    duration_ms: 240.0,
    curve: crate::theme::Curve::CubicBezier(0.2, 0.0, 0.0, 1.0),
};
const OPEN_RISE: f32 = 22.0;
/// They leave faster than they come, back to the corner.
pub(super) const CLOSE: crate::theme::Transition = crate::theme::Transition {
    duration_ms: 160.0,
    curve: crate::theme::Curve::EaseIn,
};

const WHITE_TILE: Color = Color {
    r: 246,
    g: 248,
    b: 252,
    a: 240,
};
const DARK_INK: Color = Color {
    r: 16,
    g: 22,
    b: 32,
    a: 255,
};
const BLUE: Color = Color {
    r: 10,
    g: 132,
    b: 255,
    a: 255,
};

#[derive(Clone, Copy, PartialEq)]
enum Control {
    Sleep,
    Wifi,
    Bluetooth,
    Brightness,
    Volume,
    Airplay,
    Timer,
    Inputs,
    Mirror,
    Picture,
    Sound,
    HomeSettings,
    AndroidSettings,
}

/// The controls in focus order with their place on the canvas; the last ones are the row of
/// round buttons.
fn controls(inputs_available: bool) -> Vec<(Control, Rect)> {
    let row = |n: f32| Y + n * (PILL + GAP);
    let mut items = vec![
        (Control::Sleep, Rect::new(X, Y, HALF, 2.0 * PILL + GAP)),
        (
            Control::Wifi,
            Rect::new(X + HALF + GAP, row(0.0), HALF, PILL),
        ),
        (
            Control::Bluetooth,
            Rect::new(X + HALF + GAP, row(1.0), HALF, PILL),
        ),
        (Control::Brightness, Rect::new(X, row(2.0), WIDTH, PILL)),
        (Control::Volume, Rect::new(X, row(3.0), WIDTH, PILL)),
        (Control::Airplay, Rect::new(X, row(4.0), HALF, PILL)),
        (
            Control::Timer,
            Rect::new(X + HALF + GAP, row(4.0), HALF, PILL),
        ),
    ];
    let mut round = vec![
        Control::Mirror,
        Control::Picture,
        Control::Sound,
        Control::HomeSettings,
        Control::AndroidSettings,
    ];
    if inputs_available {
        round.insert(0, Control::Inputs);
    }
    let pitch = if round.len() > 1 {
        (WIDTH - ROUND) / (round.len() - 1) as f32
    } else {
        0.0
    };
    for (index, control) in round.into_iter().enumerate() {
        items.push((
            control,
            Rect::new(X + index as f32 * pitch, row(5.0) + 3.0, ROUND, ROUND),
        ));
    }
    items
}

/// The rows the focus moves through: the tall Sleep tile shares the first with Wi-Fi.
fn focus_rows(items: &[(Control, Rect)]) -> Vec<Row> {
    let id = |index: usize| Focusable::new(ids::CONTROL_BASE + index as u32, items[index].1);
    vec![
        Row::new(vec![id(0), id(1)]),
        Row::new(vec![id(2)]),
        Row::new(vec![id(3)]),
        Row::new(vec![id(4)]),
        Row::new(vec![id(5), id(6)]),
        Row::new((7..items.len()).map(id).collect()),
    ]
}

impl LumeApp {
    pub(super) fn open_control_center(&mut self) {
        self.control_origin = match self.screen {
            Screen::Main(tab) => tab,
            _ => Tab::Home,
        };
        self.control_focus = self.focus.current_id();
        self.control_closing = false;
        self.screen = Screen::ControlCenter;
        self.anim.set(Anim::Control, 0.0);
        self.anim
            .animate_to(Anim::Control, 0.0, 1.0, self.now_ms, Some(OPEN));
        // On Wi-Fi, not on Sleep: a second press of OK must not turn the TV off.
        self.pending_focus = Some(PendingFocus::Id(ids::CONTROL_BASE + 1));
    }

    fn focused_control(&self) -> Option<Control> {
        let index = self.focus.current_id()?.checked_sub(ids::CONTROL_BASE)?;
        controls(self.inputs_available)
            .get(index as usize)
            .map(|(control, _)| *control)
    }

    /// Left and Right on a slider change its level; `None` anywhere else, where they move the focus.
    pub(super) fn slide_control(&mut self, direction: Direction) -> Option<Vec<Effect>> {
        let step = match direction {
            Direction::Left => -STEP,
            Direction::Right => STEP,
            _ => return None,
        };
        let next = |level: u8| (i16::from(level) + step).clamp(0, 100) as u8;
        match self.focused_control()? {
            Control::Brightness => {
                // The same floor the platform keeps: a black screen cannot be found again.
                self.levels.0 = next(self.levels.0).max(5);
                Some(vec![Effect::SetBrightness(self.levels.0)])
            }
            Control::Volume => {
                self.levels.1 = next(self.levels.1);
                Some(vec![Effect::SetVolume(self.levels.1)])
            }
            _ => None,
        }
    }

    pub(super) fn activate_control_center(&mut self, _id: u32) -> Vec<Effect> {
        if self.control_closing {
            return vec![];
        }
        let setting = |setting| vec![Effect::OpenTvSetting { setting }];
        match self.focused_control() {
            Some(Control::Sleep) => vec![Effect::Sleep],
            Some(Control::Wifi) => setting(SettingShortcut::Wifi),
            Some(Control::Bluetooth) => setting(SettingShortcut::Bluetooth),
            // OK on the slider hands the backlight to the TV's light sensor, or takes it back.
            Some(Control::Brightness) => match self.auto_brightness {
                Some(on) => {
                    self.auto_brightness = Some(!on);
                    vec![Effect::SetAutoBrightness(!on)]
                }
                None => setting(SettingShortcut::Picture),
            },
            Some(Control::Picture) => setting(SettingShortcut::Picture),
            Some(Control::Volume | Control::Sound) => setting(SettingShortcut::Sound),
            Some(Control::Airplay) => vec![Effect::OpenPanel("airplay")],
            Some(Control::Mirror) => vec![Effect::OpenPanel("mirror")],
            Some(Control::Timer) => {
                let at = TIMER_STEPS
                    .iter()
                    .position(|minutes| *minutes == self.sleep_timer)
                    .unwrap_or(0);
                self.sleep_timer = TIMER_STEPS[(at + 1) % TIMER_STEPS.len()];
                vec![Effect::SleepTimer(self.sleep_timer)]
            }
            Some(Control::Inputs) => vec![Effect::OpenInputs],
            Some(Control::HomeSettings) => {
                self.open_settings();
                vec![]
            }
            Some(Control::AndroidSettings) => vec![Effect::OpenAndroidSettings],
            None => vec![],
        }
    }

    /// The glass of a control, white once it has the focus as every focused control of tvOS.
    fn control_surface(&self, p: &mut Painter, rect: Rect, radius: f32, focus: f32) {
        p.wallpaper_glass(rect, radius, self.config.wallpaper);
        p.list.fill(
            rect,
            radius,
            Paint::Solid(crate::ui::style::mix_color(
                Color {
                    r: 14,
                    g: 20,
                    b: 30,
                    a: if self.more_contrast { 230 } else { 120 },
                },
                WHITE_TILE,
                focus,
            )),
        );
        p.list.ring(
            rect,
            radius,
            1.0,
            Color {
                r: 255,
                g: 255,
                b: 255,
                a: 80,
            },
        );
    }

    pub(super) fn draw_control_center(&mut self, p: &mut Painter) {
        let items = controls(self.inputs_available);
        self.update_focus(focus_rows(&items));

        let open = self.anim.get(Anim::Control, 1.0, self.now_ms);
        let start = p.list.len();
        p.text_with(
            &self.clock.text(),
            Rect::new(X, 40.0, WIDTH - 6.0, 32.0),
            Align::End,
            &TextStyle {
                size: 22.0,
                weight: 500,
                letter_spacing: 0.0,
            },
            Color::WHITE,
        );
        let press = self.anim.get(Anim::Press, 0.0, self.now_ms);
        let title = TextStyle {
            size: 17.0,
            weight: 500,
            letter_spacing: 0.0,
        };
        let detail = TextStyle {
            size: 14.0,
            weight: 500,
            letter_spacing: 0.0,
        };
        for (index, (control, at)) in items.iter().enumerate() {
            let focus = self
                .focus_progress(ids::CONTROL_BASE + index as u32)
                .clamp(0.0, 1.0);
            let rect = at.scaled_about_center(1.0 + 0.04 * focus * (1.0 - 1.6 * press));
            let ink = crate::ui::style::mix_color(Color::WHITE, DARK_INK, focus);
            let faint = Color { a: 150, ..ink };
            let (icon, label, value, on) = match control {
                Control::Sleep => (crate::icons::POWER, StringId::ControlSleepNow, None, None),
                Control::Wifi => (
                    crate::icons::WIFI,
                    StringId::ControlWifi,
                    None,
                    Some(self.radios.0),
                ),
                Control::Bluetooth => (
                    crate::icons::BLUETOOTH,
                    StringId::ControlBluetooth,
                    None,
                    Some(self.radios.1),
                ),
                Control::Brightness => (
                    crate::icons::SUN,
                    StringId::ControlBrightness,
                    Some(self.levels.0),
                    None,
                ),
                Control::Volume => (
                    crate::icons::SOUND,
                    StringId::ControlVolume,
                    Some(self.levels.1),
                    None,
                ),
                Control::Airplay => (crate::icons::AIRPLAY, StringId::ControlAirplay, None, None),
                Control::Timer => (
                    crate::icons::TIMER,
                    StringId::ControlTimer,
                    None,
                    Some(self.sleep_timer > 0),
                ),
                Control::Inputs => (crate::icons::INPUTS, StringId::ControlInputs, None, None),
                Control::Mirror => (crate::icons::CAST, StringId::ControlMirror, None, None),
                Control::Picture => (crate::icons::PICTURE, StringId::ControlPicture, None, None),
                Control::Sound => (crate::icons::SOUND, StringId::ControlSound, None, None),
                Control::HomeSettings => (
                    crate::icons::CONTROL_CENTER,
                    StringId::SettingLumeSettings,
                    None,
                    None,
                ),
                Control::AndroidSettings => (
                    crate::icons::SETTINGS,
                    StringId::ControlSettings,
                    None,
                    None,
                ),
            };
            let label = self.translator.get(label);
            if at.w == ROUND {
                // A round button: the glyph alone.
                self.control_surface(p, rect, rect.h / 2.0, focus);
                p.icon(icon, rect, 24.0, ink);
                continue;
            }
            if *control == Control::Sleep {
                self.control_surface(p, rect, 34.0, focus);
                p.icon(
                    icon,
                    Rect::new(rect.x, rect.y + 26.0, rect.w, 56.0),
                    48.0,
                    ink,
                );
                p.text_with(
                    &label,
                    Rect::new(rect.x, rect.bottom() - 50.0, rect.w, 26.0),
                    Align::Center,
                    &title,
                    ink,
                );
                continue;
            }
            let radius = rect.h / 2.0;
            if let Some(level) = value {
                // A slider: the glass, and over it the level, cut straight where it ends.
                self.control_surface(p, rect, radius, 0.0);
                let filled = rect.w * f32::from(level) / 100.0;
                p.list
                    .push_clip(Rect::new(rect.x, rect.y - 40.0, filled, rect.h + 80.0));
                p.list.fill(
                    rect,
                    radius,
                    Paint::Solid(crate::ui::style::mix_color(
                        Color {
                            r: 255,
                            g: 255,
                            b: 255,
                            a: 70,
                        },
                        WHITE_TILE,
                        focus,
                    )),
                );
                p.list.pop_clip();
                // What lies on the level takes its ink; the rest stays white on the glass.
                let over = |x: f32| {
                    if filled > x - rect.x {
                        ink
                    } else {
                        Color::WHITE
                    }
                };
                p.icon(
                    icon,
                    Rect::new(rect.x + 14.0, rect.y, 40.0, rect.h),
                    24.0,
                    over(rect.x + 44.0),
                );
                p.text_with(
                    &label,
                    Rect::new(rect.x + 58.0, rect.y, 200.0, rect.h),
                    Align::Start,
                    &title,
                    over(rect.x + 120.0),
                );
                let shown = if *control == Control::Brightness && self.auto_brightness == Some(true)
                {
                    format!("{} · {level} %", self.translator.get(StringId::ControlAuto))
                } else {
                    format!("{level} %")
                };
                p.text_with(
                    &shown,
                    Rect::new(rect.right() - 190.0, rect.y, 170.0, rect.h),
                    Align::End,
                    &title,
                    over(rect.right() - 30.0),
                );
                continue;
            }
            self.control_surface(p, rect, radius, focus);
            let glyph = Rect::new(rect.x + 12.0, rect.y + (rect.h - 44.0) / 2.0, 44.0, 44.0);
            // What can be on shows it as tvOS toggles do: a blue disc.
            let mut glyph_ink = ink;
            let disc = if on == Some(true) {
                glyph_ink = Color::WHITE;
                BLUE
            } else {
                Color { a: 60, ..ink }
            };
            p.list.fill(glyph, 22.0, Paint::Solid(disc));
            p.icon(icon, glyph, 24.0, glyph_ink);
            let on_off = |on: bool| {
                let id = if on {
                    StringId::ControlOn
                } else {
                    StringId::ControlTimerOff
                };
                Some(self.translator.get(id).into_owned())
            };
            let state = match control {
                Control::Timer if self.sleep_timer > 0 => Some(format!("{} min", self.sleep_timer)),
                Control::Timer => on_off(false),
                Control::Wifi => on_off(self.radios.0),
                Control::Bluetooth => on_off(self.radios.1),
                _ => None,
            };
            let text_x = rect.x + 66.0;
            match state {
                Some(state) => {
                    p.text_with(
                        &label,
                        Rect::new(text_x, rect.y + 12.0, rect.w - 72.0, 24.0),
                        Align::Start,
                        &title,
                        ink,
                    );
                    p.text_with(
                        &state,
                        Rect::new(text_x, rect.y + 36.0, rect.w - 72.0, 20.0),
                        Align::Start,
                        &detail,
                        faint,
                    );
                }
                None => p.text_with(
                    &label,
                    Rect::new(text_x, rect.y, rect.w - 72.0, rect.h),
                    Align::Start,
                    &title,
                    ink,
                ),
            }
        }
        if open < 1.0 {
            let end = p.list.len();
            p.list.fade_from(start, open);
            // Reduce Motion: a fade alone, nothing travels.
            if !self.reduce_motion {
                p.list
                    .translate_range(start, end, -(1.0 - open) * OPEN_RISE);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::tests::Rig;
    use crate::app::Key;

    #[test]
    fn controls_open_from_the_top_bar_and_back_restores_it() {
        let mut rig = Rig::new();
        rig.app.focus.focus(ids::CONTROL_CENTER);
        rig.press(Key::Ok);
        assert_eq!(rig.app.screen(), &Screen::ControlCenter);
        assert_eq!(rig.focused(), Some(ids::CONTROL_BASE + 1));
        rig.press(Key::Back);
        // The tiles go back to their corner first.
        assert_eq!(rig.app.screen(), &Screen::ControlCenter);
        rig.advance(400);
        assert_eq!(rig.app.screen(), &Screen::Main(Tab::Home));
        assert_eq!(rig.focused(), Some(ids::CONTROL_CENTER));
    }

    #[test]
    fn home_at_the_start_of_home_toggles_the_controls() {
        let mut rig = Rig::new();
        rig.press(Key::Home);
        assert_eq!(rig.app.screen(), &Screen::ControlCenter);
        rig.press(Key::Home);
        rig.advance(400);
        assert_eq!(rig.app.screen(), &Screen::Main(Tab::Home));
        // The focus is back on the banner it left, so Home opens the controls again.
        rig.press(Key::Home);
        assert_eq!(rig.app.screen(), &Screen::ControlCenter);
    }

    #[test]
    fn wifi_control_opens_tv_network_settings() {
        let mut rig = Rig::new();
        rig.app.open_control_center();
        rig.frame();
        assert_eq!(
            rig.press(Key::Ok),
            vec![Effect::OpenTvSetting {
                setting: SettingShortcut::Wifi
            }]
        );
    }

    #[test]
    fn left_and_right_on_a_slider_change_its_level_instead_of_the_focus() {
        let mut rig = Rig::new();
        rig.app.open_control_center();
        rig.frame();
        rig.press(Key::Down);
        rig.press(Key::Down);
        let brightness = rig.focused();
        rig.app.levels = (40, 20);
        assert_eq!(rig.press(Key::Right), vec![Effect::SetBrightness(45)]);
        assert_eq!(rig.focused(), brightness);
        rig.app.levels.0 = 5;
        // Never to a black screen.
        assert_eq!(rig.press(Key::Left), vec![Effect::SetBrightness(5)]);
        rig.press(Key::Down);
        assert_eq!(rig.press(Key::Left), vec![Effect::SetVolume(15)]);
    }

    #[test]
    fn ok_on_the_brightness_slider_toggles_the_light_sensor_where_the_tv_has_one() {
        let mut rig = Rig::new();
        rig.app.open_control_center();
        rig.frame();
        rig.press(Key::Down);
        rig.press(Key::Down);
        // No sensor known: the TV's own picture screen.
        assert_eq!(
            rig.press(Key::Ok),
            vec![Effect::OpenTvSetting {
                setting: SettingShortcut::Picture
            }]
        );
        rig.app.auto_brightness = Some(false);
        assert_eq!(rig.press(Key::Ok), vec![Effect::SetAutoBrightness(true)]);
        assert_eq!(rig.press(Key::Ok), vec![Effect::SetAutoBrightness(false)]);
    }

    #[test]
    fn ok_on_the_timer_goes_through_its_times_and_back_to_off() {
        let mut rig = Rig::new();
        rig.app.open_control_center();
        rig.frame();
        for _ in 0..4 {
            rig.press(Key::Down);
        }
        rig.press(Key::Right);
        assert_eq!(rig.press(Key::Ok), vec![Effect::SleepTimer(15)]);
        for _ in 0..3 {
            rig.press(Key::Ok);
        }
        assert_eq!(rig.press(Key::Ok), vec![Effect::SleepTimer(0)]);
    }

    #[test]
    fn the_controls_open_on_wifi_so_a_second_ok_does_not_sleep_the_tv() {
        let mut rig = Rig::new();
        rig.app.open_control_center();
        rig.frame();
        assert_ne!(rig.press(Key::Ok), vec![Effect::Sleep]);
    }
}
