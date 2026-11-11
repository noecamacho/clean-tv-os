//! Supported value types and their parsers.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const TRANSPARENT: Color = Color {
        r: 0,
        g: 0,
        b: 0,
        a: 0,
    };

    pub const fn rgb(r: u8, g: u8, b: u8) -> Color {
        Color { r, g, b, a: 255 }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GradientStop {
    pub color: Color,
    /// Position from 0.0 to 1.0 along the gradient line.
    pub position: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LinearGradient {
    /// Angle in degrees, as in CSS: 0 = up, 90 = right, 180 = down.
    pub angle_deg: f32,
    pub stops: Vec<GradientStop>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Paint {
    Solid(Color),
    Gradient(LinearGradient),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Outline {
    pub width: f32,
    pub color: Color,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Shadow {
    pub offset_x: f32,
    pub offset_y: f32,
    pub blur: f32,
    pub spread: f32,
    pub color: Color,
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

/// A declaration that has been validated and given its proper type.
#[derive(Clone, Debug, PartialEq)]
pub enum Decl {
    Background(Paint),
    Color(Color),
    Opacity(f32),
    BorderRadius(f32),
    Outline(Outline),
    OutlineOffset(f32),
    BoxShadow(Option<Shadow>),
    FontSize(f32),
    FontWeight(u16),
    LetterSpacing(f32),
    /// Top, right, bottom, left.
    Padding([f32; 4]),
    Gap(f32),
    Width(f32),
    Height(f32),
    TransformScale(f32),
    Transition(Transition),
}

/// Splits on whitespace at the outermost level (respecting parentheses).
pub(crate) fn split_ws(s: &str) -> Vec<&str> {
    split_top(s, |c| c.is_whitespace())
}

/// Splits on commas at the outermost level (respecting parentheses).
pub(crate) fn split_commas(s: &str) -> Vec<&str> {
    split_top(s, |c| c == ',')
        .into_iter()
        .map(str::trim)
        .collect()
}

fn split_top(s: &str, is_sep: impl Fn(char) -> bool) -> Vec<&str> {
    let mut out = Vec::new();
    let mut depth = 0i32;
    let mut start = 0;
    for (i, c) in s.char_indices() {
        match c {
            '(' => depth += 1,
            ')' => depth -= 1,
            _ if depth == 0 && is_sep(c) => {
                if i > start || !c.is_whitespace() {
                    out.push(&s[start..i]);
                }
                start = i + c.len_utf8();
            }
            _ => {}
        }
    }
    if start < s.len() {
        out.push(&s[start..]);
    }
    out
}

fn number(s: &str) -> Option<f32> {
    let v: f32 = s.parse().ok()?;
    v.is_finite().then_some(v)
}

fn percent(s: &str) -> Option<f32> {
    number(s.strip_suffix('%')?).map(|v| v / 100.0)
}

/// Parses a length: `0` or `<n>px`.
pub fn parse_length(s: &str) -> Option<f32> {
    if s == "0" {
        return Some(0.0);
    }
    number(s.strip_suffix("px")?)
}

fn parse_time_ms(s: &str) -> Option<f32> {
    if let Some(ms) = s.strip_suffix("ms") {
        number(ms).filter(|v| *v >= 0.0)
    } else {
        number(s.strip_suffix('s')?)
            .filter(|v| *v >= 0.0)
            .map(|v| v * 1000.0)
    }
}

fn hex_digit(c: u8) -> Option<u8> {
    (c as char).to_digit(16).map(|d| d as u8)
}

/// Parses a color: hex, `rgb()`/`rgba()` or a keyword (`transparent`, `white`, `black`).
pub fn parse_color(s: &str) -> Option<Color> {
    let s = s.trim();
    if let Some(hex) = s.strip_prefix('#') {
        let d: Option<Vec<u8>> = hex.bytes().map(hex_digit).collect();
        let d = d?;
        return match d.len() {
            3 | 4 => {
                let c = |i: usize| d[i] * 17;
                Some(Color {
                    r: c(0),
                    g: c(1),
                    b: c(2),
                    a: if d.len() == 4 { c(3) } else { 255 },
                })
            }
            6 | 8 => {
                let c = |i: usize| d[i] * 16 + d[i + 1];
                Some(Color {
                    r: c(0),
                    g: c(2),
                    b: c(4),
                    a: if d.len() == 8 { c(6) } else { 255 },
                })
            }
            _ => None,
        };
    }
    match s {
        "transparent" => return Some(Color::TRANSPARENT),
        "white" => return Some(Color::rgb(255, 255, 255)),
        "black" => return Some(Color::rgb(0, 0, 0)),
        _ => {}
    }
    let inner = s
        .strip_prefix("rgba(")
        .or_else(|| s.strip_prefix("rgb("))?
        .strip_suffix(')')?;
    let normalized = inner.replace([',', '/'], " ");
    let parts: Vec<&str> = normalized.split_whitespace().collect();
    if parts.len() != 3 && parts.len() != 4 {
        return None;
    }
    let channel = |p: &str| -> Option<u8> {
        let v = match percent(p) {
            Some(f) => f * 255.0,
            None => number(p)?,
        };
        Some(v.clamp(0.0, 255.0).round() as u8)
    };
    let a = match parts.get(3) {
        None => 255,
        Some(p) => {
            let f = percent(p).or_else(|| number(p))?;
            (f.clamp(0.0, 1.0) * 255.0).round() as u8
        }
    };
    Some(Color {
        r: channel(parts[0])?,
        g: channel(parts[1])?,
        b: channel(parts[2])?,
        a,
    })
}

fn parse_angle(s: &str) -> Option<f32> {
    if let Some(deg) = s.strip_suffix("deg") {
        return number(deg);
    }
    match s {
        "to top" => Some(0.0),
        "to right" => Some(90.0),
        "to bottom" => Some(180.0),
        "to left" => Some(270.0),
        _ => None,
    }
}

/// Maximum number of stops the shader accepts.
pub const MAX_GRADIENT_STOPS: usize = 3;

fn parse_gradient(s: &str) -> Result<LinearGradient, String> {
    let inner = s
        .strip_prefix("linear-gradient(")
        .and_then(|r| r.strip_suffix(')'))
        .ok_or_else(|| format!("invalid gradient: '{s}'"))?;
    let mut parts = split_commas(inner);
    let mut angle = 180.0;
    if let Some(a) = parts.first().and_then(|p| parse_angle(p)) {
        angle = a;
        parts.remove(0);
    }
    if parts.len() < 2 {
        return Err("a gradient needs at least 2 colors".into());
    }
    if parts.len() > MAX_GRADIENT_STOPS {
        return Err(format!(
            "a gradient accepts at most {MAX_GRADIENT_STOPS} stops"
        ));
    }
    let mut raw: Vec<(Color, Option<f32>)> = Vec::new();
    for p in parts {
        let toks = split_ws(p);
        let (color_tok, pos) = match toks.as_slice() {
            [c] => (*c, None),
            [c, pos] => (
                *c,
                Some(percent(pos).ok_or_else(|| format!("invalid stop position: '{pos}'"))?),
            ),
            _ => return Err(format!("invalid gradient stop: '{p}'")),
        };
        let color = parse_color(color_tok)
            .ok_or_else(|| format!("invalid color in gradient: '{color_tok}'"))?;
        raw.push((color, pos));
    }
    let last = raw.len() - 1;
    raw[0].1.get_or_insert(0.0);
    raw[last].1.get_or_insert(1.0);
    // Stops without a position are spaced evenly between their neighbors that have one.
    let mut i = 0;
    while i < raw.len() {
        if raw[i].1.is_none() {
            let prev = raw[i - 1].1.unwrap();
            let next_idx = (i..raw.len()).find(|&j| raw[j].1.is_some()).unwrap();
            let next = raw[next_idx].1.unwrap();
            let span = (next_idx - (i - 1)) as f32;
            for (k, j) in (i..next_idx).enumerate() {
                raw[j].1 = Some(prev + (next - prev) * (k + 1) as f32 / span);
            }
            i = next_idx;
        }
        i += 1;
    }
    let mut stops = Vec::new();
    let mut prev_pos = 0.0f32;
    for (color, pos) in raw {
        let position = pos.unwrap().clamp(0.0, 1.0).max(prev_pos);
        prev_pos = position;
        stops.push(GradientStop { color, position });
    }
    Ok(LinearGradient {
        angle_deg: angle,
        stops,
    })
}

fn parse_paint(v: &str) -> Result<Paint, String> {
    if v.starts_with("linear-gradient(") {
        parse_gradient(v).map(Paint::Gradient)
    } else {
        parse_color(v)
            .map(Paint::Solid)
            .ok_or_else(|| format!("invalid color or gradient: '{v}'"))
    }
}

fn length(v: &str) -> Result<f32, String> {
    parse_length(v).ok_or_else(|| format!("invalid length: '{v}' (use px)"))
}

fn parse_outline(v: &str) -> Result<Outline, String> {
    if v == "none" {
        return Ok(Outline {
            width: 0.0,
            color: Color::TRANSPARENT,
        });
    }
    let (mut width, mut color) = (None, None);
    for tok in split_ws(v) {
        if tok == "solid" {
            continue;
        }
        if let Some(w) = parse_length(tok) {
            width = Some(w);
        } else if let Some(c) = parse_color(tok) {
            color = Some(c);
        } else {
            return Err(format!("invalid outline value: '{tok}'"));
        }
    }
    match (width, color) {
        (Some(width), Some(color)) => Ok(Outline { width, color }),
        _ => Err("outline needs a width and a color (e.g. 3px solid #fff)".into()),
    }
}

fn parse_shadow(v: &str) -> Result<Option<Shadow>, String> {
    if v == "none" {
        return Ok(None);
    }
    if split_commas(v).len() > 1 {
        return Err("box-shadow accepts a single shadow".into());
    }
    let mut lengths = Vec::new();
    let mut color = None;
    for tok in split_ws(v) {
        if let Some(l) = parse_length(tok) {
            lengths.push(l);
        } else if let Some(c) = parse_color(tok) {
            color = Some(c);
        } else {
            return Err(format!("invalid box-shadow value: '{tok}'"));
        }
    }
    let color = color.ok_or("box-shadow needs a color")?;
    match lengths.as_slice() {
        [x, y] => Ok(Some(Shadow {
            offset_x: *x,
            offset_y: *y,
            blur: 0.0,
            spread: 0.0,
            color,
        })),
        [x, y, b] => Ok(Some(Shadow {
            offset_x: *x,
            offset_y: *y,
            blur: *b,
            spread: 0.0,
            color,
        })),
        [x, y, b, s] => Ok(Some(Shadow {
            offset_x: *x,
            offset_y: *y,
            blur: *b,
            spread: *s,
            color,
        })),
        _ => Err("box-shadow needs 2 to 4 lengths".into()),
    }
}

fn parse_curve(tok: &str) -> Option<Curve> {
    match tok {
        "linear" => Some(Curve::Linear),
        "ease" => Some(Curve::Ease),
        "ease-in" => Some(Curve::EaseIn),
        "ease-out" => Some(Curve::EaseOut),
        "ease-in-out" => Some(Curve::EaseInOut),
        _ => {
            let inner = tok.strip_prefix("cubic-bezier(")?.strip_suffix(')')?;
            let p: Vec<f32> = split_commas(inner)
                .into_iter()
                .map(number)
                .collect::<Option<_>>()?;
            if p.len() == 4 && (0.0..=1.0).contains(&p[0]) && (0.0..=1.0).contains(&p[2]) {
                Some(Curve::CubicBezier(p[0], p[1], p[2], p[3]))
            } else {
                None
            }
        }
    }
}

/// Parses a transition such as `150ms ease-out`.
pub fn parse_transition(v: &str) -> Result<Transition, String> {
    let (mut duration, mut curve) = (None, Curve::Ease);
    for tok in split_ws(v) {
        if tok == "all" {
            continue;
        }
        if let Some(d) = parse_time_ms(tok) {
            duration = Some(d);
        } else if let Some(c) = parse_curve(tok) {
            curve = c;
        } else {
            return Err(format!("invalid transition value: '{tok}'"));
        }
    }
    let duration_ms = duration.ok_or("transition needs a duration (e.g. 150ms)")?;
    Ok(Transition { duration_ms, curve })
}

fn parse_padding(v: &str) -> Result<[f32; 4], String> {
    let l: Vec<f32> = split_ws(v)
        .into_iter()
        .map(length)
        .collect::<Result<_, _>>()?;
    match l.as_slice() {
        [a] => Ok([*a; 4]),
        [v, h] => Ok([*v, *h, *v, *h]),
        [t, h, b] => Ok([*t, *h, *b, *h]),
        [t, r, b, l] => Ok([*t, *r, *b, *l]),
        _ => Err("padding accepts 1 to 4 lengths".into()),
    }
}

/// Names of the accepted properties.
pub const PROPERTIES: &[&str] = &[
    "background",
    "color",
    "opacity",
    "border-radius",
    "outline",
    "outline-offset",
    "box-shadow",
    "font-size",
    "font-weight",
    "letter-spacing",
    "padding",
    "gap",
    "width",
    "height",
    "transform",
    "transition",
];

/// Converts `property: value` (with `var()` already resolved) into a [`Decl`].
/// Unknown properties are rejected with an error here; callers filter them first.
pub(crate) fn parse_decl(property: &str, v: &str) -> Result<Decl, String> {
    match property {
        "background" => parse_paint(v).map(Decl::Background),
        "color" => parse_color(v)
            .map(Decl::Color)
            .ok_or_else(|| format!("invalid color: '{v}'")),
        "opacity" => number(v)
            .filter(|n| (0.0..=1.0).contains(n))
            .map(Decl::Opacity)
            .ok_or_else(|| format!("opacity must be between 0 and 1: '{v}'")),
        "border-radius" => length(v).map(Decl::BorderRadius),
        "outline" => parse_outline(v).map(Decl::Outline),
        "outline-offset" => length(v).map(Decl::OutlineOffset),
        "box-shadow" => parse_shadow(v).map(Decl::BoxShadow),
        "font-size" => length(v).map(Decl::FontSize),
        "font-weight" => number(v)
            .filter(|n| (1.0..=1000.0).contains(n) && n.fract() == 0.0)
            .map(|n| Decl::FontWeight(n as u16))
            .ok_or_else(|| format!("invalid font-weight: '{v}'")),
        "letter-spacing" => length(v).map(Decl::LetterSpacing),
        "padding" => parse_padding(v).map(Decl::Padding),
        "gap" => length(v).map(Decl::Gap),
        "width" => length(v).map(Decl::Width),
        "height" => length(v).map(Decl::Height),
        "transform" => {
            let inner = v
                .strip_prefix("scale(")
                .and_then(|r| r.strip_suffix(')'))
                .ok_or_else(|| format!("transform only supports scale(): '{v}'"))?;
            number(inner)
                .filter(|n| *n > 0.0)
                .map(Decl::TransformScale)
                .ok_or_else(|| format!("invalid scale() factor: '{inner}'"))
        }
        "transition" => parse_transition(v).map(Decl::Transition),
        _ => Err(format!("unsupported property: {property}")),
    }
}
