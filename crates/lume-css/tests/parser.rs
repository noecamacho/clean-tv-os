use lume_css::*;

fn parse_ok(src: &str) -> Stylesheet {
    let p = parse(src, "t.css");
    assert!(
        p.diagnostics.is_empty(),
        "unexpected diagnostics: {:#?}",
        p.diagnostics
    );
    p.stylesheet
}

fn only(src: &str) -> (Severity, u32, String) {
    let p = parse(src, "t.css");
    assert_eq!(p.diagnostics.len(), 1, "{:#?}", p.diagnostics);
    let d = &p.diagnostics[0];
    assert_eq!(d.file, "t.css");
    (d.severity, d.line, d.message.clone())
}

fn single_decl(prop: &str, value: &str) -> Decl {
    let sheet = parse_ok(&format!(".tile {{ {prop}: {value}; }}"));
    assert_eq!(sheet.rules.len(), 1);
    assert_eq!(sheet.rules[0].decls.len(), 1);
    sheet.rules[0].decls[0].clone()
}

// ---- structure -----------------------------------------------------------

#[test]
fn parses_root_variables_rules_and_states() {
    let sheet = parse_ok(
        r#"
        /* test theme */
        :root { --accent: #4DB3FF; --radius: 12px; }
        .tile { border-radius: var(--radius); color: white; }
        .tile:focus { outline: 3px solid var(--accent); transform: scale(1.06); }
        .button:active { opacity: 0.8; }
        .button:disabled { opacity: 0.4; }
        "#,
    );
    assert_eq!(sheet.variable("--accent"), Some("#4DB3FF"));
    assert_eq!(
        sheet.color_variable("--accent"),
        Some(Color::rgb(0x4d, 0xb3, 0xff))
    );
    assert_eq!(sheet.rules.len(), 4);
    let states: Vec<_> = sheet
        .rules
        .iter()
        .map(|r| (r.component.as_str(), r.state))
        .collect();
    assert_eq!(
        states,
        [
            ("tile", State::Base),
            ("tile", State::Focus),
            ("button", State::Active),
            ("button", State::Disabled)
        ]
    );
    assert_eq!(sheet.rules[0].decls[0], Decl::BorderRadius(12.0));
}

#[test]
fn records_rule_lines() {
    let sheet = parse_ok("\n\n.tile {\n  color: white;\n}\n\n.chip {\n  color: black;\n}\n");
    assert_eq!(sheet.rules[0].line, 3);
    assert_eq!(sheet.rules[1].line, 7);
}

#[test]
fn selector_list_expands_to_one_rule_each() {
    let sheet = parse_ok(".tile, .chip:focus { opacity: 0.5; }");
    assert_eq!(sheet.rules.len(), 2);
    assert_eq!(sheet.rules[1].state, State::Focus);
}

#[test]
fn decls_lookup_combines_rules() {
    let sheet = parse_ok(".tile { opacity: 0.5; } .tile { opacity: 0.9; }");
    let all: Vec<_> = sheet.decls("tile", State::Base).collect();
    assert_eq!(all, [&Decl::Opacity(0.5), &Decl::Opacity(0.9)]);
}

#[test]
fn variable_declared_after_use_and_last_definition_wins() {
    let sheet = parse_ok(".tile { color: var(--c); } :root { --c: #000; } :root { --c: #fff; }");
    assert_eq!(
        sheet.rules[0].decls[0],
        Decl::Color(Color::rgb(255, 255, 255))
    );
}

#[test]
fn empty_and_comment_only_sources_are_valid() {
    parse_ok("");
    parse_ok("/* nada */\n");
}

// ---- values --------------------------------------------------------------

#[test]
fn colors() {
    assert_eq!(
        single_decl("color", "#fff"),
        Decl::Color(Color::rgb(255, 255, 255))
    );
    assert_eq!(
        single_decl("color", "#4DB3FF"),
        Decl::Color(Color::rgb(0x4d, 0xb3, 0xff))
    );
    assert_eq!(
        single_decl("color", "#00000080"),
        Decl::Color(Color {
            r: 0,
            g: 0,
            b: 0,
            a: 128
        })
    );
    assert_eq!(
        single_decl("color", "#f008"),
        Decl::Color(Color {
            r: 255,
            g: 0,
            b: 0,
            a: 136
        })
    );
    assert_eq!(
        single_decl("color", "rgb(10, 20, 30)"),
        Decl::Color(Color::rgb(10, 20, 30))
    );
    assert_eq!(
        single_decl("color", "rgba(255, 255, 255, 0.5)"),
        Decl::Color(Color {
            r: 255,
            g: 255,
            b: 255,
            a: 128
        })
    );
    assert_eq!(
        single_decl("color", "rgb(0 0 0 / 50%)"),
        Decl::Color(Color {
            r: 0,
            g: 0,
            b: 0,
            a: 128
        })
    );
    assert_eq!(
        single_decl("color", "transparent"),
        Decl::Color(Color::TRANSPARENT)
    );
}

#[test]
fn lengths_and_numbers() {
    assert_eq!(
        single_decl("border-radius", "16px"),
        Decl::BorderRadius(16.0)
    );
    assert_eq!(single_decl("border-radius", "0"), Decl::BorderRadius(0.0));
    assert_eq!(single_decl("font-size", "22px"), Decl::FontSize(22.0));
    assert_eq!(single_decl("font-weight", "700"), Decl::FontWeight(700));
    assert_eq!(
        single_decl("letter-spacing", "-0.5px"),
        Decl::LetterSpacing(-0.5)
    );
    assert_eq!(single_decl("opacity", ".4"), Decl::Opacity(0.4));
    assert_eq!(single_decl("width", "320px"), Decl::Width(320.0));
    assert_eq!(single_decl("height", "180px"), Decl::Height(180.0));
    assert_eq!(single_decl("gap", "20.5px"), Decl::Gap(20.5));
    assert_eq!(
        single_decl("outline-offset", "4px"),
        Decl::OutlineOffset(4.0)
    );
}

#[test]
fn padding_shorthand() {
    assert_eq!(single_decl("padding", "8px"), Decl::Padding([8.0; 4]));
    assert_eq!(
        single_decl("padding", "8px 16px"),
        Decl::Padding([8.0, 16.0, 8.0, 16.0])
    );
    assert_eq!(
        single_decl("padding", "1px 2px 3px"),
        Decl::Padding([1.0, 2.0, 3.0, 2.0])
    );
    assert_eq!(
        single_decl("padding", "1px 2px 3px 4px"),
        Decl::Padding([1.0, 2.0, 3.0, 4.0])
    );
}

#[test]
fn linear_gradients() {
    let Decl::Background(Paint::Gradient(g)) = single_decl(
        "background",
        "linear-gradient(135deg, #1a1040 0%, #3b1f6e 60%, rgba(0,0,0,0.5) 100%)",
    ) else {
        panic!("expected a gradient");
    };
    assert_eq!(g.angle_deg, 135.0);
    let pos: Vec<f32> = g.stops.iter().map(|s| s.position).collect();
    assert_eq!(pos, [0.0, 0.6, 1.0]);
    assert_eq!(g.stops[2].color.a, 128);

    // No angle: downwards; positions spread evenly.
    let Decl::Background(Paint::Gradient(g)) =
        single_decl("background", "linear-gradient(#000, #777, #fff)")
    else {
        panic!()
    };
    assert_eq!(g.angle_deg, 180.0);
    let pos: Vec<f32> = g.stops.iter().map(|s| s.position).collect();
    assert_eq!(pos, [0.0, 0.5, 1.0]);

    let Decl::Background(Paint::Gradient(g)) =
        single_decl("background", "linear-gradient(to right, #000, #fff)")
    else {
        panic!()
    };
    assert_eq!(g.angle_deg, 90.0);
}

#[test]
fn solid_background() {
    assert_eq!(
        single_decl("background", "#102030"),
        Decl::Background(Paint::Solid(Color::rgb(0x10, 0x20, 0x30)))
    );
}

#[test]
fn outline_and_shadow() {
    assert_eq!(
        single_decl("outline", "3px solid #fff"),
        Decl::Outline(Outline {
            width: 3.0,
            color: Color::rgb(255, 255, 255)
        })
    );
    assert_eq!(
        single_decl("outline", "none"),
        Decl::Outline(Outline {
            width: 0.0,
            color: Color::TRANSPARENT
        })
    );
    assert_eq!(
        single_decl("box-shadow", "0 8px 24px rgba(0,0,0,0.4)"),
        Decl::BoxShadow(Some(Shadow {
            offset_x: 0.0,
            offset_y: 8.0,
            blur: 24.0,
            spread: 0.0,
            color: Color {
                r: 0,
                g: 0,
                b: 0,
                a: 102
            }
        }))
    );
    assert_eq!(single_decl("box-shadow", "none"), Decl::BoxShadow(None));
}

#[test]
fn transform_and_transition() {
    assert_eq!(
        single_decl("transform", "scale(1.06)"),
        Decl::TransformScale(1.06)
    );
    assert_eq!(
        single_decl("transition", "150ms ease-out"),
        Decl::Transition(Transition {
            duration_ms: 150.0,
            curve: Curve::EaseOut
        })
    );
    assert_eq!(
        single_decl("transition", "0.2s linear"),
        Decl::Transition(Transition {
            duration_ms: 200.0,
            curve: Curve::Linear
        })
    );
    assert_eq!(
        single_decl("transition", "all 300ms cubic-bezier(0.2, 0, 0, 1)"),
        Decl::Transition(Transition {
            duration_ms: 300.0,
            curve: Curve::CubicBezier(0.2, 0.0, 0.0, 1.0)
        })
    );
}

#[test]
fn var_with_fallback_and_nested_vars() {
    let sheet = parse_ok(
        ":root { --a: #111; --b: var(--a); }\n.tile { color: var(--b); background: var(--missing, var(--a)); }",
    );
    assert_eq!(
        sheet.rules[0].decls[0],
        Decl::Color(Color::rgb(0x11, 0x11, 0x11))
    );
    assert_eq!(
        sheet.rules[0].decls[1],
        Decl::Background(Paint::Solid(Color::rgb(0x11, 0x11, 0x11)))
    );
    assert_eq!(sheet.variable("--b"), Some("#111"));
}

#[test]
fn var_inside_function() {
    let sheet = parse_ok(":root { --a: #000; --b: #fff; }\n.panel { background: linear-gradient(90deg, var(--a), var(--b)); }");
    assert!(matches!(
        sheet.rules[0].decls[0],
        Decl::Background(Paint::Gradient(_))
    ));
}

#[test]
fn undefined_variable_is_an_error_with_line() {
    let (sev, line, msg) = only(".tile {\n  color: var(--nope);\n}");
    assert_eq!(sev, Severity::Error);
    assert_eq!(line, 2);
    assert!(msg.contains("--nope"), "{msg}");
}

#[test]
fn circular_variables_are_an_error() {
    let p = parse(":root { --a: var(--b); --b: var(--a); }", "t.css");
    assert!(p.has_errors());
    assert!(p.errors().any(|d| d.message.contains("circular")));
}

#[test]
fn invalid_values_are_errors() {
    for (prop, v) in [
        ("color", "#12"),
        ("color", "azul"),
        ("opacity", "2"),
        ("border-radius", "12"),
        ("border-radius", "1em"),
        ("transform", "rotate(10deg)"),
        ("transition", "ease"),
        ("outline", "3px"),
        ("box-shadow", "0 0 #000, 1px 1px #fff"),
        ("background", "linear-gradient(#000)"),
        ("background", "linear-gradient(#000, #111, #222, #333)"),
        ("font-weight", "bold"),
    ] {
        let p = parse(&format!(".tile {{ {prop}: {v}; }}"), "t.css");
        assert!(
            p.has_errors(),
            "{prop}: {v} should be an error: {:?}",
            p.diagnostics
        );
    }
}

// ---- diagnostics ---------------------------------------------------------

#[test]
fn unsupported_property_warns_with_file_line_and_name() {
    let p = parse(
        ".tile {\n  color: white;\n  filter: blur(4px);\n}",
        "tema.css",
    );
    assert_eq!(p.diagnostics.len(), 1);
    let d = &p.diagnostics[0];
    assert_eq!(
        (d.severity, d.file.as_str(), d.line),
        (Severity::Warning, "tema.css", 3)
    );
    assert!(d.message.contains("filter"), "{}", d.message);
    // The property is ignored, the rest stays.
    assert_eq!(p.stylesheet.rules[0].decls.len(), 1);
    assert!(!p.has_errors());
}

#[test]
fn unsupported_selectors_warn_and_are_ignored() {
    for sel in [
        "div",
        "#id",
        ".tile .chip",
        ".desconhecido",
        ".tile:hover",
        "*",
        ".tile > .chip",
    ] {
        let p = parse(&format!("{sel} {{ color: white; }}"), "t.css");
        assert_eq!(p.diagnostics.len(), 1, "{sel}: {:?}", p.diagnostics);
        assert_eq!(p.diagnostics[0].severity, Severity::Warning, "{sel}");
        assert!(p.stylesheet.rules.is_empty(), "{sel}");
    }
}

#[test]
fn import_warns_and_is_ignored() {
    let p = parse(
        "@import url(https://exemplo.com/a.css);\n.tile { color: white; }",
        "t.css",
    );
    assert_eq!(p.diagnostics.len(), 1);
    assert_eq!(p.diagnostics[0].severity, Severity::Warning);
    assert_eq!(p.diagnostics[0].line, 1);
    assert!(p.diagnostics[0].message.contains("@import"));
    // The rest of the theme is still valid.
    assert_eq!(p.stylesheet.rules.len(), 1);
}

#[test]
fn other_at_rules_warn_including_blocks() {
    let p = parse("@media (min-width: 1px) { .tile { color: white; } }\n@font-face { font-family: x; }\n.chip { color: white; }", "t.css");
    assert_eq!(p.diagnostics.len(), 2);
    assert!(p
        .diagnostics
        .iter()
        .all(|d| d.severity == Severity::Warning));
    assert_eq!(p.diagnostics[0].line, 1);
    assert_eq!(p.diagnostics[1].line, 2);
    assert_eq!(p.stylesheet.rules.len(), 1);
}

#[test]
fn url_in_value_warns() {
    let (sev, line, msg) = only(".panel {\n  background: url(imagem.png);\n}");
    assert_eq!((sev, line), (Severity::Warning, 2));
    assert!(msg.contains("url()"), "{msg}");
}

#[test]
fn small_font_warns_but_keeps_declaration() {
    let p = parse(".tile-label {\n  font-size: 12px;\n}", "t.css");
    assert_eq!(p.diagnostics.len(), 1);
    assert_eq!(
        (p.diagnostics[0].severity, p.diagnostics[0].line),
        (Severity::Warning, 2)
    );
    assert_eq!(p.stylesheet.rules[0].decls[0], Decl::FontSize(12.0));
    assert_eq!(
        parse(".tile-label { font-size: 16px; }", "t.css")
            .diagnostics
            .len(),
        0
    );
}

#[test]
fn font_size_outside_the_type_scale_warns() {
    for size in ["15px", "17px", "24px", "22.5px"] {
        let p = parse(&format!(".tile-label {{ font-size: {size}; }}"), "t.css");
        assert_eq!(p.diagnostics.len(), 1, "{size}");
        assert_eq!(p.diagnostics[0].severity, Severity::Warning);
    }
    for size in ["16px", "18px", "20px", "22px", "28px", "40px"] {
        let p = parse(&format!(".tile-label {{ font-size: {size}; }}"), "t.css");
        assert!(p.diagnostics.is_empty(), "{size}");
    }
}

#[test]
fn non_variable_in_root_and_variable_in_component_warn() {
    let p = parse(
        ":root {\n  color: white;\n}\n.tile {\n  --x: 1px;\n}",
        "t.css",
    );
    assert_eq!(p.diagnostics.len(), 2);
    assert_eq!(p.diagnostics[0].line, 2);
    assert_eq!(p.diagnostics[1].line, 5);
    assert!(p
        .diagnostics
        .iter()
        .all(|d| d.severity == Severity::Warning));
}

#[test]
fn unclosed_block_is_an_error_at_the_opening_line() {
    let (sev, line, msg) = only("\n.tile {\n  color: white;\n");
    assert_eq!((sev, line), (Severity::Error, 2));
    assert!(msg.contains("'}'"), "{msg}");
}

#[test]
fn syntax_errors() {
    for src in [
        ".tile color: white; }",
        ".tile { color white; }",
        ".tile { : white; }",
        ".tile { color: ; }",
        ".tile { .chip { color: white; } }",
        "/* sem fim\n.tile { color: white; }",
        ".tile { color: var(--a }",
    ] {
        let p = parse(src, "t.css");
        assert!(
            p.has_errors(),
            "{src:?} should be an error: {:?}",
            p.diagnostics
        );
    }
}

#[test]
fn error_reports_correct_line_after_comment() {
    let (sev, line, _) = only("/* a\n b */\n.tile {\n  color white;\n}");
    assert_eq!((sev, line), (Severity::Error, 4));
}

#[test]
fn diagnostic_display_has_file_and_line() {
    let p = parse(".tile { filter: none; }", "themes/x.css");
    assert!(
        p.diagnostics[0]
            .to_string()
            .starts_with("themes/x.css:1: warning:"),
        "{}",
        p.diagnostics[0]
    );
}

#[test]
fn the_removed_carousel_selectors_warn_as_unsupported() {
    for selector in [
        ".hero",
        ".hero-badge",
        ".hero-title",
        ".hero-subtitle",
        ".pager-dot",
    ] {
        let p = parse(&format!("{selector} {{\n  color: #fff;\n}}"), "t.css");
        assert_eq!(p.diagnostics.len(), 1, "{selector}");
        assert_eq!(p.diagnostics[0].severity, Severity::Warning);
        assert!(
            p.diagnostics[0].message.contains("unsupported selector"),
            "{selector}: {}",
            p.diagnostics[0].message
        );
        assert!(p.stylesheet.rules.is_empty());
    }
}
