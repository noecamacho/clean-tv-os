//! The compiled-in theme.
//!
//! `build.rs` parses `themes/lume/theme.css` with `lume-css` and generates
//! [`THEME`] and the [`vars`] constants, so no parsing happens at runtime.

/// No text is rendered smaller than this many logical units.
pub const MIN_FONT_SIZE: f32 = 16.0;

/// The type scale (same as `lume_css::FONT_SCALE`): text is rendered only in these sizes.
pub const FONT_SCALE: [f32; 6] = [16.0, 18.0, 20.0, 22.0, 28.0, 40.0];

/// The size of the scale nearest to `size` (the larger one on a tie); the build already warns
/// about sizes outside the scale.
pub fn nearest_font_size(size: f32) -> f32 {
    FONT_SCALE
        .into_iter()
        .min_by(|a, b| {
            (a - size)
                .abs()
                .total_cmp(&(b - size).abs())
                .then(b.total_cmp(a))
        })
        .unwrap_or(MIN_FONT_SIZE)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const WHITE: Color = Color {
        r: 255,
        g: 255,
        b: 255,
        a: 255,
    };
    pub const TRANSPARENT: Color = Color {
        r: 0,
        g: 0,
        b: 0,
        a: 0,
    };

    pub const fn rgb(r: u8, g: u8, b: u8) -> Color {
        Color { r, g, b, a: 255 }
    }

    /// Straight (non-premultiplied) RGBA as floats from 0.0 to 1.0.
    pub fn to_f32(self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a].map(|c| f32::from(c) / 255.0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GradientStop {
    pub color: Color,
    /// Position from 0.0 to 1.0 along the gradient line.
    pub position: f32,
}

/// The most color stops a gradient has (the rectangle shader takes three).
pub const MAX_GRADIENT_STOPS: usize = 3;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Gradient {
    /// CSS angle: 0 = up, 90 = right, 180 = down.
    pub angle_deg: f32,
    /// The stops; only the first `len` are used.
    pub stops: [GradientStop; MAX_GRADIENT_STOPS],
    pub len: usize,
}

impl Gradient {
    /// A gradient of 2 or 3 stops.
    pub const fn new(angle_deg: f32, stops: &[GradientStop]) -> Gradient {
        assert!(
            stops.len() >= 2 && stops.len() <= MAX_GRADIENT_STOPS,
            "a gradient has 2 or 3 stops"
        );
        let mut padded = [stops[0]; MAX_GRADIENT_STOPS];
        let mut i = 0;
        while i < stops.len() {
            padded[i] = stops[i];
            i += 1;
        }
        Gradient {
            angle_deg,
            stops: padded,
            len: stops.len(),
        }
    }

    pub fn stops(&self) -> &[GradientStop] {
        &self.stops[..self.len]
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Paint {
    Solid(Color),
    Gradient(Gradient),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Outline {
    pub width: f32,
    pub color: Color,
}

impl Outline {
    pub const NONE: Outline = Outline {
        width: 0.0,
        color: Color::TRANSPARENT,
    };
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Shadow {
    pub offset_x: f32,
    pub offset_y: f32,
    pub blur: f32,
    pub spread: f32,
    pub color: Color,
}

impl Shadow {
    pub const NONE: Shadow = Shadow {
        offset_x: 0.0,
        offset_y: 0.0,
        blur: 0.0,
        spread: 0.0,
        color: Color::TRANSPARENT,
    };
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Curve {
    Linear,
    Ease,
    EaseIn,
    EaseOut,
    EaseInOut,
    CubicBezier(f32, f32, f32, f32),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Transition {
    pub duration_ms: f32,
    pub curve: Curve,
}

/// The properties one rule sets. `None` means "not set by this rule".
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Style {
    pub background: Option<Paint>,
    pub color: Option<Color>,
    pub opacity: Option<f32>,
    pub border_radius: Option<f32>,
    pub outline: Option<Outline>,
    pub outline_offset: Option<f32>,
    pub box_shadow: Option<Shadow>,
    pub font_size: Option<f32>,
    pub font_weight: Option<u16>,
    pub letter_spacing: Option<f32>,
    /// Top, right, bottom, left.
    pub padding: Option<[f32; 4]>,
    pub gap: Option<f32>,
    pub width: Option<f32>,
    pub height: Option<f32>,
    pub scale: Option<f32>,
    pub transition: Option<Transition>,
}

impl Style {
    pub const EMPTY: Style = Style {
        background: None,
        color: None,
        opacity: None,
        border_radius: None,
        outline: None,
        outline_offset: None,
        box_shadow: None,
        font_size: None,
        font_weight: None,
        letter_spacing: None,
        padding: None,
        gap: None,
        width: None,
        height: None,
        scale: None,
        transition: None,
    };
}

/// A fully resolved style: every property has a value.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ComputedStyle {
    pub background: Option<Paint>,
    pub color: Color,
    pub opacity: f32,
    pub border_radius: f32,
    /// `None` when there is no visible outline.
    pub outline: Option<Outline>,
    pub outline_offset: f32,
    /// `None` when there is no visible shadow.
    pub box_shadow: Option<Shadow>,
    pub font_size: f32,
    pub font_weight: u16,
    pub letter_spacing: f32,
    pub padding: [f32; 4],
    pub gap: f32,
    pub width: Option<f32>,
    pub height: Option<f32>,
    pub scale: f32,
    pub transition: Option<Transition>,
}

impl ComputedStyle {
    pub const DEFAULT: ComputedStyle = ComputedStyle {
        background: None,
        color: Color::WHITE,
        opacity: 1.0,
        border_radius: 0.0,
        outline: None,
        outline_offset: 0.0,
        box_shadow: None,
        font_size: 20.0,
        font_weight: 500,
        letter_spacing: 0.0,
        padding: [0.0; 4],
        gap: 0.0,
        width: None,
        height: None,
        scale: 1.0,
        transition: None,
    };

    /// Applies the properties `style` sets on top of `self`.
    pub fn overlay(mut self, style: &Style) -> ComputedStyle {
        if let Some(v) = style.background {
            self.background = Some(v);
        }
        if let Some(v) = style.color {
            self.color = v;
        }
        if let Some(v) = style.opacity {
            self.opacity = v;
        }
        if let Some(v) = style.border_radius {
            self.border_radius = v;
        }
        if let Some(v) = style.outline {
            self.outline = (v.width > 0.0 && v.color.a > 0).then_some(v);
        }
        if let Some(v) = style.outline_offset {
            self.outline_offset = v;
        }
        if let Some(v) = style.box_shadow {
            self.box_shadow = (v.color.a > 0).then_some(v);
        }
        if let Some(v) = style.font_size {
            // Smaller text is unreadable on a TV, and sizes off the scale blur at 1.5x.
            self.font_size = nearest_font_size(v);
        }
        if let Some(v) = style.font_weight {
            self.font_weight = v;
        }
        if let Some(v) = style.letter_spacing {
            self.letter_spacing = v;
        }
        if let Some(v) = style.padding {
            self.padding = v;
        }
        if let Some(v) = style.gap {
            self.gap = v;
        }
        if let Some(v) = style.width {
            self.width = Some(v);
        }
        if let Some(v) = style.height {
            self.height = Some(v);
        }
        if let Some(v) = style.scale {
            self.scale = v;
        }
        if let Some(v) = style.transition {
            self.transition = Some(v);
        }
        self
    }
}

/// Which pseudo-classes currently apply to an element.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct StateFlags {
    pub focus: bool,
    pub active: bool,
    pub disabled: bool,
}

impl StateFlags {
    pub const NONE: StateFlags = StateFlags {
        focus: false,
        active: false,
        disabled: false,
    };
    pub const FOCUS: StateFlags = StateFlags {
        focus: true,
        active: false,
        disabled: false,
    };
    pub const ACTIVE: StateFlags = StateFlags {
        focus: false,
        active: true,
        disabled: false,
    };
    pub const DISABLED: StateFlags = StateFlags {
        focus: false,
        active: false,
        disabled: true,
    };
}

/// The rules of one component, one [`Style`] per pseudo-class.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ComponentTheme {
    pub base: Style,
    pub focus: Style,
    pub active: Style,
    pub disabled: Style,
}

impl ComponentTheme {
    /// Resolves the style for `state` on top of `parent`.
    /// Precedence, lowest first: base, active, focus, disabled.
    pub fn resolve_over(&self, parent: ComputedStyle, state: StateFlags) -> ComputedStyle {
        let mut style = parent.overlay(&self.base);
        if state.active {
            style = style.overlay(&self.active);
        }
        if state.focus {
            style = style.overlay(&self.focus);
        }
        if state.disabled {
            style = style.overlay(&self.disabled);
        }
        style
    }
}

include!(concat!(env!("OUT_DIR"), "/theme_generated.rs"));

impl Theme {
    /// Resolves a component's style. Text properties not set by the component
    /// are inherited from `.app`.
    pub fn resolve(&self, component: &ComponentTheme, state: StateFlags) -> ComputedStyle {
        let root = self
            .app
            .resolve_over(ComputedStyle::DEFAULT, StateFlags::NONE);
        // Only text properties are inherited; boxes are not.
        let inherited = ComputedStyle {
            color: root.color,
            font_size: root.font_size,
            font_weight: root.font_weight,
            letter_spacing: root.letter_spacing,
            ..ComputedStyle::DEFAULT
        };
        component.resolve_over(inherited, state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tile_is_resolved_with_inherited_text_defaults() {
        let tile = THEME.resolve(&THEME.tile, StateFlags::NONE);
        assert_eq!(tile.width, Some(172.0));
        assert_eq!(tile.height, Some(97.0));
        assert_eq!(tile.border_radius, 12.0);
        assert_eq!(tile.font_size, 22.0);
        assert_eq!(tile.scale, 1.0);
        assert!(tile.outline.is_none());
    }

    #[test]
    fn focus_overlays_the_base_style() {
        let focused = THEME.resolve(&THEME.tile, StateFlags::FOCUS);
        assert_eq!(focused.scale, 1.12);
        let ring = focused.outline.expect("focus ring");
        assert_eq!(ring.width, 2.0);
        assert_eq!(ring.color, vars::PRIMARY);
        assert!(focused.box_shadow.is_some());
        // Properties the focus rule does not set come from the base rule.
        assert_eq!(focused.border_radius, 12.0);
    }

    #[test]
    fn active_tab_is_bold_and_bright() {
        let inactive = THEME.resolve(&THEME.tab, StateFlags::NONE);
        let active = THEME.resolve(&THEME.tab, StateFlags::ACTIVE);
        assert_eq!(inactive.font_weight, 500);
        assert_eq!(active.font_weight, 700);
        assert_eq!(active.color, vars::TEXT);
        assert_eq!(inactive.color, vars::TEXT_MUTED);
    }

    #[test]
    fn font_sizes_below_the_minimum_are_raised() {
        let small = Style {
            font_size: Some(13.0),
            ..Style::EMPTY
        };
        assert_eq!(ComputedStyle::DEFAULT.overlay(&small).font_size, 16.0);
        let fine = Style {
            font_size: Some(18.0),
            ..Style::EMPTY
        };
        assert_eq!(ComputedStyle::DEFAULT.overlay(&fine).font_size, 18.0);
    }

    #[test]
    fn font_sizes_off_the_scale_take_the_nearest_level() {
        assert_eq!(nearest_font_size(15.0), 16.0);
        assert_eq!(nearest_font_size(17.0), 18.0);
        assert_eq!(nearest_font_size(24.0), 22.0);
        assert_eq!(nearest_font_size(25.0), 28.0);
        assert_eq!(nearest_font_size(100.0), 40.0);
        for size in FONT_SCALE {
            assert_eq!(nearest_font_size(size), size);
        }
    }

    #[test]
    fn every_text_of_the_theme_is_on_the_type_scale() {
        for component in [
            &THEME.app,
            &THEME.tab,
            &THEME.clock,
            &THEME.icon_button,
            &THEME.button,
            &THEME.button_secondary,
            &THEME.button_danger,
            &THEME.section_title,
            &THEME.tile,
            &THEME.tile_label,
            &THEME.dock_label,
            &THEME.editor_row,
            &THEME.chip,
            &THEME.panel,
            &THEME.breadcrumb,
            &THEME.settings_item,
            &THEME.menu_item,
            &THEME.menu_note,
            &THEME.menu_button,
            &THEME.menu_button_danger,
            &THEME.keycap,
            &THEME.toast,
            &THEME.brand,
            &THEME.page_title,
            &THEME.page_subtitle,
            &THEME.card_heading,
            &THEME.about_heading,
            &THEME.about_body,
            &THEME.about_license,
        ] {
            for state in [
                StateFlags::NONE,
                StateFlags::FOCUS,
                StateFlags::ACTIVE,
                StateFlags::DISABLED,
            ] {
                let size = THEME.resolve(component, state).font_size;
                assert!(
                    size >= MIN_FONT_SIZE && FONT_SCALE.contains(&size),
                    "{size}"
                );
            }
        }
    }

    #[test]
    fn the_component_boxes_are_on_the_ruler() {
        let components = [
            ("topbar", &THEME.topbar),
            ("icon-button", &THEME.icon_button),
            ("button", &THEME.button),
            ("button-secondary", &THEME.button_secondary),
            ("button-danger", &THEME.button_danger),
            ("dock", &THEME.dock),
            ("dock-label", &THEME.dock_label),
            ("tile", &THEME.tile),
            ("settings-item", &THEME.settings_item),
            ("menu", &THEME.menu),
            ("menu-item", &THEME.menu_item),
            ("menu-button", &THEME.menu_button),
            ("menu-button-danger", &THEME.menu_button_danger),
            ("keycap", &THEME.keycap),
            ("editor-row", &THEME.editor_row),
            ("panel", &THEME.panel),
            ("chip", &THEME.chip),
            ("toast", &THEME.toast),
        ];
        let on_ruler = |name: &str, what: &str, v: f32| {
            assert!(
                v % 4.0 == 0.0,
                "{name} {what} = {v} is off the 4-unit ruler"
            );
        };
        for (name, component) in components {
            let style = THEME.resolve(component, StateFlags::NONE);
            on_ruler(name, "width", style.width.unwrap_or(0.0));
            // The tile height comes from 16:9 (172 x 96.75), the registered exception.
            if name != "tile" {
                on_ruler(name, "height", style.height.unwrap_or(0.0));
            }
            on_ruler(name, "radius", style.border_radius);
            on_ruler(name, "gap", style.gap);
            for side in style.padding {
                on_ruler(name, "padding", side);
            }
        }
    }

    #[test]
    fn the_type_scale_matches_the_design_system() {
        assert_eq!(
            [
                vars::FONT_CAPTION,
                vars::FONT_LABEL,
                vars::FONT_BODY,
                vars::FONT_PRIMARY,
                vars::FONT_BRAND,
                vars::FONT_TITLE,
            ],
            FONT_SCALE
        );
        let size = |c: &ComponentTheme| THEME.resolve(c, StateFlags::NONE).font_size;
        assert_eq!(size(&THEME.page_title), 40.0);
        assert_eq!(size(&THEME.brand), 28.0);
        // The clock matches the wordmark, at the other end of the top bar.
        assert_eq!(size(&THEME.clock), 28.0);
        for c in [&THEME.tab, &THEME.editor_row, &THEME.about_heading] {
            assert_eq!(size(c), 22.0);
        }
        for c in [
            &THEME.tile_label,
            &THEME.chip,
            &THEME.panel,
            &THEME.breadcrumb,
        ] {
            assert_eq!(size(c), 18.0);
        }
        assert_eq!(size(&THEME.card_heading), 16.0);
        assert_eq!(size(&THEME.about_license), 16.0);
    }

    #[test]
    fn disabled_wins_over_focus_and_active() {
        let state = StateFlags {
            focus: true,
            active: true,
            disabled: true,
        };
        let button = THEME.resolve(&THEME.button, state);
        assert_eq!(button.opacity, 0.4);
    }

    #[test]
    fn components_inherit_text_from_app_but_not_boxes() {
        let toast = THEME.resolve(&THEME.toast, StateFlags::NONE);
        assert_eq!(toast.color, vars::TEXT);
        // `.app` sets a background, but it is not inherited.
        let tab = THEME.resolve(&THEME.tab, StateFlags::NONE);
        assert!(tab.background.is_none());
    }

    #[test]
    fn the_dock_is_a_translucent_rounded_bar_with_a_light_ring() {
        let dock = THEME.resolve(&THEME.dock, StateFlags::NONE);
        let Some(Paint::Gradient(background)) = dock.background else {
            panic!("the dock has a translucent gradient background");
        };
        assert!(
            background
                .stops()
                .iter()
                .all(|stop| stop.color.a > 0 && stop.color.a < 255),
            "every glass layer stays translucent"
        );
        assert!(dock.border_radius >= 16.0);
        assert!(dock.outline.is_some_and(|o| o.width == 1.0));
        assert!(dock.box_shadow.is_some());
    }

    #[test]
    fn the_dock_label_is_a_readable_bubble() {
        let label = THEME.resolve(&THEME.dock_label, StateFlags::NONE);
        assert!(label.background.is_some() && label.border_radius > 0.0);
        assert_eq!(label.color, vars::TEXT);
        assert!(label.font_size >= MIN_FONT_SIZE);
        assert!(label.padding[0] > 0.0 && label.padding[1] > 0.0);
    }

    #[test]
    fn the_color_roles_are_ice_white_and_the_secondary_light_neutral() {
        assert_eq!(vars::PRIMARY, Color::rgb(0xEA, 0xF4, 0xFF));
        assert_eq!(vars::ON_PRIMARY, Color::rgb(0x14, 0x20, 0x33));
        assert_eq!(vars::SECONDARY, Color::rgb(0xF7, 0xFA, 0xFF));
        // The old names are aliases of the roles, so older themes and code keep working.
        assert_eq!(vars::ACCENT, vars::PRIMARY);
        assert_eq!(vars::ON_ACCENT, vars::ON_PRIMARY);
        // The text is the secondary color.
        assert_eq!(vars::TEXT, vars::SECONDARY);
    }

    #[test]
    fn every_focus_outline_of_the_default_theme_is_the_primary_color() {
        for (name, component) in [
            ("tile", &THEME.tile),
            ("settings-item", &THEME.settings_item),
            ("editor-row", &THEME.editor_row),
            ("chip", &THEME.chip),
            ("button", &THEME.button),
            ("button-secondary", &THEME.button_secondary),
            ("button-danger", &THEME.button_danger),
            ("icon-button", &THEME.icon_button),
        ] {
            let focused = THEME.resolve(component, StateFlags::FOCUS);
            let outline = focused
                .outline
                .unwrap_or_else(|| panic!("{name} has no focus outline"));
            assert_eq!(outline.color, vars::PRIMARY, "{name}");
        }
    }

    #[test]
    fn the_primary_button_is_faded_until_it_has_the_focus() {
        let rest = THEME.resolve(&THEME.button, StateFlags::NONE);
        assert_eq!(rest.background, Some(Paint::Solid(vars::CONTROL)));
        assert_eq!(rest.color, vars::TEXT_MUTED);
        let focus = THEME.resolve(&THEME.button, StateFlags::FOCUS);
        assert_eq!(focus.background, Some(Paint::Solid(vars::PRIMARY)));
        assert_eq!(focus.color, vars::ON_PRIMARY);
        // The ring stands off the fill, so the primary color still shows against the background.
        assert!(focus.outline_offset >= 4.0);
    }

    #[test]
    fn variables_become_typed_constants() {
        assert_eq!(vars::ACCENT, Color::rgb(0xEA, 0xF4, 0xFF));
        assert_eq!(vars::BRAND_NETFLIX, Color::rgb(0xB3, 0x12, 0x1C));
        assert_eq!(vars::TAB_UNDERLINE, 2.0);
        assert_eq!(vars::RADIUS, 12.0);
    }

    #[test]
    fn known_app_colors_have_readable_white_text() {
        // White text must keep a contrast of at least 4.5:1 on every brand color.
        fn luminance(c: Color) -> f32 {
            let f = |v: u8| {
                let v = f32::from(v) / 255.0;
                if v <= 0.03928 {
                    v / 12.92
                } else {
                    ((v + 0.055) / 1.055).powf(2.4)
                }
            };
            0.2126 * f(c.r) + 0.7152 * f(c.g) + 0.0722 * f(c.b)
        }
        let contrast = |c: Color| 1.05 / (luminance(c) + 0.05);
        for (name, c) in [
            ("netflix", vars::BRAND_NETFLIX),
            ("youtube", vars::BRAND_YOUTUBE),
            ("prime video", vars::BRAND_PRIME_VIDEO),
            ("disney+", vars::BRAND_DISNEY_PLUS),
            ("max", vars::BRAND_MAX),
            ("spotify", vars::BRAND_SPOTIFY),
            ("globoplay", vars::BRAND_GLOBOPLAY),
            ("play store", vars::BRAND_PLAY_STORE),
            ("chrome", vars::BRAND_CHROME),
        ] {
            assert!(contrast(c) >= 4.5, "{name}: contrast {:.2}", contrast(c));
        }
    }

    #[test]
    fn the_text_on_the_light_card_is_readable() {
        fn luminance(c: Color) -> f32 {
            let f = |v: u8| {
                let v = f32::from(v) / 255.0;
                if v <= 0.03928 {
                    v / 12.92
                } else {
                    ((v + 0.055) / 1.055).powf(2.4)
                }
            };
            0.2126 * f(c.r) + 0.7152 * f(c.g) + 0.0722 * f(c.b)
        }
        let contrast = |a: Color, b: Color| {
            let (a, b) = (luminance(a) + 0.05, luminance(b) + 0.05);
            a.max(b) / a.min(b)
        };
        for (name, c) in [
            ("menu-text", vars::MENU_TEXT),
            ("menu-text-muted", vars::MENU_TEXT_MUTED),
            ("menu-danger", vars::MENU_DANGER),
        ] {
            let ratio = contrast(c, vars::MENU_SURFACE);
            assert!(ratio >= 4.5, "{name}: contrast {ratio:.2}");
        }
        // The primary fill of a focused button keeps its dark text readable.
        assert!(contrast(vars::ON_PRIMARY, vars::PRIMARY) >= 4.5);
    }

    #[test]
    fn the_edge_softness_of_the_theme_reaches_the_renderer_at_every_resolution() {
        // Pixels of the render buffer, the same at 720p and 1080p, from 1 to 1.5 (design-system).
        assert!((1.0..=1.5).contains(&vars::EDGE_SOFTNESS));
        for (width, height) in [(1280, 720), (1920, 1080)] {
            let viewport = crate::display::Display::new(width, height).viewport();
            assert_eq!(viewport.edge_softness, vars::EDGE_SOFTNESS);
        }
    }
}
