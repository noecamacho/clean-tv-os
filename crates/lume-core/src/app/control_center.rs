//! Quick controls for the TV, presented as a glass panel over the current wallpaper.

use super::painter::Painter;
use super::{ids, Anim, Effect, LumeApp, PendingFocus, Screen, Tab};
use crate::i18n::StringId;
use crate::keys::SettingShortcut;
use crate::render::Rect;
use crate::text::{Align, TextStyle};
use crate::theme::{Color, Paint};
use crate::ui::focus::{Focusable, Row};

/// tvOS 26 has no pane: the picture dims a little and glass tiles float in the top right
/// corner, two to a row, inside the 54-unit side margin of Home.
const CARD_WIDTH: f32 = 214.0;
const CARD_HEIGHT: f32 = 84.0;
const CARD_GAP: f32 = 14.0;
const CARD_X: f32 = 1280.0 - 54.0 - 2.0 * CARD_WIDTH - CARD_GAP;
const CARD_Y: f32 = 56.0;
const CARD_RADIUS: f32 = 26.0;
/// The tiles come down from the corner they belong to.
const OPEN: crate::theme::Transition = crate::theme::Transition {
    duration_ms: 240.0,
    curve: crate::theme::Curve::CubicBezier(0.2, 0.0, 0.0, 1.0),
};
const OPEN_RISE: f32 = 22.0;

#[derive(Clone, Copy)]
enum Action {
    Inputs,
    Setting(SettingShortcut),
}

#[derive(Clone, Copy)]
struct Control {
    label: StringId,
    icon: char,
    action: Action,
}

fn controls(inputs_available: bool) -> Vec<Control> {
    let mut items = vec![
        Control {
            label: StringId::ControlWifi,
            icon: crate::icons::WIFI,
            action: Action::Setting(SettingShortcut::Wifi),
        },
        Control {
            label: StringId::ControlBluetooth,
            icon: crate::icons::BLUETOOTH,
            action: Action::Setting(SettingShortcut::Bluetooth),
        },
    ];
    if inputs_available {
        items.push(Control {
            label: StringId::ControlInputs,
            icon: crate::icons::INPUTS,
            action: Action::Inputs,
        });
    }
    items.extend([
        Control {
            label: StringId::ControlPicture,
            icon: crate::icons::PICTURE,
            action: Action::Setting(SettingShortcut::Picture),
        },
        Control {
            label: StringId::ControlSound,
            icon: crate::icons::SOUND,
            action: Action::Setting(SettingShortcut::Sound),
        },
        Control {
            label: StringId::ControlAmbient,
            icon: crate::icons::SLEEP,
            action: Action::Setting(SettingShortcut::Sleep),
        },
        Control {
            label: StringId::ControlApps,
            icon: crate::icons::SETTINGS,
            action: Action::Setting(SettingShortcut::Apps),
        },
        Control {
            label: StringId::ControlSettings,
            icon: crate::icons::SETTINGS,
            action: Action::Setting(SettingShortcut::Android),
        },
    ]);
    items
}

fn card(index: usize) -> Rect {
    Rect::new(
        CARD_X + (index % 2) as f32 * (CARD_WIDTH + CARD_GAP),
        CARD_Y + (index / 2) as f32 * (CARD_HEIGHT + CARD_GAP),
        CARD_WIDTH,
        CARD_HEIGHT,
    )
}

impl LumeApp {
    pub(super) fn open_control_center(&mut self) {
        self.control_origin = match self.screen {
            Screen::Main(tab) => tab,
            _ => Tab::Home,
        };
        self.control_focus = self.focus.current_id();
        self.screen = Screen::ControlCenter;
        self.anim.set(Anim::Control, 0.0);
        self.anim
            .animate_to(Anim::Control, 0.0, 1.0, self.now_ms, Some(OPEN));
        self.pending_focus = Some(PendingFocus::Id(ids::CONTROL_BASE));
    }

    pub(super) fn activate_control_center(&mut self, id: u32) -> Vec<Effect> {
        let Some(control) = controls(self.inputs_available)
            .get(id.checked_sub(ids::CONTROL_BASE).unwrap_or(u32::MAX) as usize)
            .copied()
        else {
            return vec![];
        };
        match control.action {
            Action::Inputs => vec![Effect::OpenInputs],
            Action::Setting(SettingShortcut::Android) => vec![Effect::OpenAndroidSettings],
            Action::Setting(setting) => vec![Effect::OpenTvSetting { setting }],
        }
    }

    pub(super) fn draw_control_center(&mut self, p: &mut Painter) {
        let controls = controls(self.inputs_available);
        self.update_focus(
            (0..controls.len().div_ceil(2))
                .map(|row| {
                    Row::new(
                        (0..2)
                            .filter_map(|col| {
                                let index = row * 2 + col;
                                controls.get(index).map(|_| {
                                    Focusable::new(ids::CONTROL_BASE + index as u32, card(index))
                                })
                            })
                            .collect(),
                    )
                })
                .collect(),
        );

        let open = self.anim.get(Anim::Control, 1.0, self.now_ms);
        p.list.fill(
            p.display.visible_rect(),
            0.0,
            Paint::Solid(Color {
                r: 3,
                g: 8,
                b: 19,
                a: (open * 90.0).round() as u8,
            }),
        );
        let start = p.list.len();
        for (index, control) in controls.iter().enumerate() {
            let focus = self
                .focus_progress(ids::CONTROL_BASE + index as u32)
                .clamp(0.0, 1.0);
            let rect = card(index).scaled_about_center(1.0 + 0.04 * focus);
            // Glass at rest; the tile with the focus turns white with dark content, as on tvOS.
            p.wallpaper_glass(rect, CARD_RADIUS, self.config.wallpaper);
            p.list.fill(
                rect,
                CARD_RADIUS,
                Paint::Solid(crate::ui::style::mix_color(
                    Color {
                        r: 14,
                        g: 20,
                        b: 30,
                        a: 120,
                    },
                    Color {
                        r: 246,
                        g: 248,
                        b: 252,
                        a: 240,
                    },
                    focus,
                )),
            );
            p.list.ring(
                rect,
                CARD_RADIUS,
                1.0,
                Color {
                    r: 255,
                    g: 255,
                    b: 255,
                    a: 80,
                },
            );
            let ink = crate::ui::style::mix_color(
                Color::WHITE,
                Color {
                    r: 16,
                    g: 22,
                    b: 32,
                    a: 255,
                },
                focus,
            );
            p.icon(
                control.icon,
                Rect::new(rect.x + 14.0, rect.y + (rect.h - 44.0) / 2.0, 44.0, 44.0),
                28.0,
                ink,
            );
            p.text_with(
                &self.translator.get(control.label),
                Rect::new(rect.x + 60.0, rect.y, rect.w - 68.0, rect.h),
                Align::Start,
                &TextStyle {
                    size: 18.0,
                    weight: 700,
                    letter_spacing: 0.0,
                },
                ink,
            );
        }
        if open < 1.0 {
            let end = p.list.len();
            p.list.fade_from(start, open);
            p.list
                .translate_range(start, end, -(1.0 - open) * OPEN_RISE);
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
        assert_eq!(rig.focused(), Some(ids::CONTROL_BASE));
        rig.press(Key::Back);
        assert_eq!(rig.app.screen(), &Screen::Main(Tab::Home));
        assert_eq!(rig.focused(), Some(ids::CONTROL_CENTER));
    }

    #[test]
    fn home_at_the_start_of_home_toggles_the_controls() {
        let mut rig = Rig::new();
        rig.press(Key::Home);
        assert_eq!(rig.app.screen(), &Screen::ControlCenter);
        rig.press(Key::Home);
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
}
