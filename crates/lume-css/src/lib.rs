//! Parser for the CSS subset used by lume themes.
//!
//! The format accepts only variables in `:root`, a fixed set of component selectors with
//! `:focus`/`:active`/`:disabled` and a small set of properties
//! (see `docs/theme-format.md`). Anything unsupported produces a warning and is
//! ignored; syntax errors and invalid values produce an error.

mod diag;
mod value;

use std::collections::HashMap;

pub use diag::{Diagnostic, Severity};
pub use value::{
    parse_color, parse_length, parse_transition, Color, Curve, Decl, GradientStop, LinearGradient,
    Outline, Paint, Shadow, Transition, MAX_GRADIENT_STOPS, PROPERTIES,
};

/// Accepted component selectors.
pub const COMPONENTS: &[&str] = &[
    "app",
    "topbar",
    "tab",
    "clock",
    "icon-button",
    "button",
    "button-secondary",
    "button-danger",
    "section-title",
    "tile",
    "tile-label",
    "dock",
    "dock-label",
    "editor-row",
    "chip",
    "keycap",
    "panel",
    "breadcrumb",
    "settings-item",
    "menu",
    "menu-item",
    "menu-note",
    "menu-button",
    "menu-button-danger",
    "toast",
    "brand",
    "page-title",
    "page-subtitle",
    "card-heading",
    "about-heading",
    "about-body",
    "about-license",
];

/// Smallest font size (in logical px) that does not produce a warning.
pub const MIN_FONT_SIZE: f32 = 16.0;

/// The type scale: the only font sizes (in logical px) that do not produce a warning. All are
/// even, so the text lands on whole pixels at 1.5x (1080p).
pub const FONT_SCALE: &[f32] = &[16.0, 18.0, 20.0, 22.0, 28.0, 40.0];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum State {
    Base,
    Focus,
    Active,
    Disabled,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Rule {
    pub component: String,
    pub state: State,
    pub decls: Vec<Decl>,
    pub line: u32,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Stylesheet {
    /// `:root` variables with `var()` already resolved, in declaration order.
    pub variables: Vec<(String, String)>,
    pub rules: Vec<Rule>,
}

impl Stylesheet {
    /// Resolved value of a variable such as `--accent`.
    pub fn variable(&self, name: &str) -> Option<&str> {
        self.variables
            .iter()
            .rev()
            .find(|(n, _)| n == name)
            .map(|(_, v)| v.as_str())
    }

    /// Color of a variable such as `--accent`.
    pub fn color_variable(&self, name: &str) -> Option<Color> {
        value::parse_color(self.variable(name)?)
    }

    /// Length in px of a variable such as `--tab-underline`.
    pub fn length_variable(&self, name: &str) -> Option<f32> {
        value::parse_length(self.variable(name)?)
    }

    /// Duration and curve of a variable such as `--focus-time`.
    pub fn transition_variable(&self, name: &str) -> Option<Transition> {
        value::parse_transition(self.variable(name)?).ok()
    }

    /// All declarations of a component in a state (rules add up, the last one wins).
    pub fn decls<'a>(&'a self, component: &'a str, state: State) -> impl Iterator<Item = &'a Decl> {
        self.rules
            .iter()
            .filter(move |r| r.component == component && r.state == state)
            .flat_map(|r| r.decls.iter())
    }
}

#[derive(Clone, Debug, Default)]
pub struct Parsed {
    pub stylesheet: Stylesheet,
    pub diagnostics: Vec<Diagnostic>,
}

impl Parsed {
    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|d| d.severity == Severity::Error)
    }

    pub fn warnings(&self) -> impl Iterator<Item = &Diagnostic> {
        self.diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Warning)
    }

    pub fn errors(&self) -> impl Iterator<Item = &Diagnostic> {
        self.diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Error)
    }
}

/// Parses `source`. `file` is only used in messages.
pub fn parse(source: &str, file: &str) -> Parsed {
    let mut p = Parser::new(source, file);
    p.run();
    p.finish()
}

enum Selector {
    Root,
    Component(String, State),
}

struct RawDecl {
    name: String,
    value: String,
    line: u32,
}

struct RawRule {
    selector: Selector,
    decls: Vec<RawDecl>,
    line: u32,
}

struct Parser<'a> {
    /// Source without comments (same length and same line breaks).
    src: String,
    file: &'a str,
    line_starts: Vec<usize>,
    diags: Vec<Diagnostic>,
    rules: Vec<RawRule>,
}

impl<'a> Parser<'a> {
    fn new(source: &str, file: &'a str) -> Self {
        let mut line_starts = vec![0];
        line_starts.extend(source.match_indices('\n').map(|(i, _)| i + 1));
        let mut p = Parser {
            src: String::new(),
            file,
            line_starts,
            diags: Vec::new(),
            rules: Vec::new(),
        };
        p.src = p.strip_comments(source);
        p
    }

    fn line_of(&self, offset: usize) -> u32 {
        self.line_starts.partition_point(|&s| s <= offset) as u32
    }

    fn report(&mut self, severity: Severity, line: u32, message: impl Into<String>) {
        self.diags.push(Diagnostic {
            severity,
            file: self.file.to_string(),
            line,
            message: message.into(),
        });
    }

    fn warn(&mut self, line: u32, message: impl Into<String>) {
        self.report(Severity::Warning, line, message);
    }

    fn error(&mut self, line: u32, message: impl Into<String>) {
        self.report(Severity::Error, line, message);
    }

    /// Replaces comments with spaces, preserving line breaks.
    fn strip_comments(&mut self, source: &str) -> String {
        let mut out = String::with_capacity(source.len());
        let mut rest = source;
        let mut offset = 0;
        while let Some(start) = rest.find("/*") {
            out.push_str(&rest[..start]);
            let after = &rest[start + 2..];
            match after.find("*/") {
                Some(end) => {
                    let comment = &rest[start..start + 2 + end + 2];
                    out.extend(comment.chars().map(|c| if c == '\n' { '\n' } else { ' ' }));
                    offset += start + comment.len();
                    rest = &rest[start + comment.len()..];
                }
                None => {
                    let line = self.line_of(offset + start);
                    self.error(line, "unterminated '/*' comment");
                    return out;
                }
            }
        }
        out.push_str(rest);
        out
    }

    fn run(&mut self) {
        if self.diags.iter().any(|d| d.severity == Severity::Error) {
            return;
        }
        let bytes = self.src.clone().into_bytes();
        let n = bytes.len();
        let mut i = 0;
        while i < n {
            if bytes[i].is_ascii_whitespace() {
                i += 1;
                continue;
            }
            let start = i;
            if bytes[i] == b'@' {
                match self.skip_at_rule(&bytes, i) {
                    Some(next) => i = next,
                    None => return,
                }
                continue;
            }
            // Selector up to '{'.
            while i < n && !matches!(bytes[i], b'{' | b'}' | b';') {
                i += 1;
            }
            if i >= n || bytes[i] != b'{' {
                let line = self.line_of(start);
                self.error(line, "expected '{' after the selector");
                return;
            }
            let prelude_end = i;
            let open_line = self.line_of(i);
            // Body up to the matching '}'.
            let body_start = i + 1;
            i += 1;
            let mut quote: Option<u8> = None;
            while i < n {
                let c = bytes[i];
                match (quote, c) {
                    (Some(q), c) if c == q => quote = None,
                    (Some(_), _) => {}
                    (None, b'"' | b'\'') => quote = Some(c),
                    (None, b'{') => {
                        let line = self.line_of(i);
                        self.error(line, "nested blocks are not supported");
                        return;
                    }
                    (None, b'}') => break,
                    _ => {}
                }
                i += 1;
            }
            if i >= n {
                self.error(open_line, "block without a closing '}'");
                return;
            }
            let body_end = i;
            i += 1;
            let prelude = self.src[start..prelude_end].to_string();
            let body = self.src[body_start..body_end].to_string();
            self.handle_rule(&prelude, start, &body, body_start);
        }
    }

    /// Skips an at-rule (with a warning) and returns the next position; `None` on error.
    fn skip_at_rule(&mut self, bytes: &[u8], start: usize) -> Option<usize> {
        let n = bytes.len();
        let mut i = start + 1;
        while i < n && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'-') {
            i += 1;
        }
        let name = self.src[start..i].to_string();
        let line = self.line_of(start);
        let mut depth = 0i32;
        while i < n {
            match bytes[i] {
                b'(' => depth += 1,
                b')' => depth -= 1,
                b';' if depth <= 0 => {
                    i += 1;
                    break;
                }
                b'{' => {
                    let open_line = self.line_of(i);
                    let mut braces = 0;
                    while i < n {
                        match bytes[i] {
                            b'{' => braces += 1,
                            b'}' => {
                                braces -= 1;
                                if braces == 0 {
                                    break;
                                }
                            }
                            _ => {}
                        }
                        i += 1;
                    }
                    if i >= n {
                        self.error(open_line, "block without a closing '}'");
                        return None;
                    }
                    i += 1;
                    break;
                }
                _ => {}
            }
            i += 1;
        }
        let message = if name == "@import" {
            "'@import' is not supported (themes cannot import files or access the network); rule ignored".to_string()
        } else {
            format!("unsupported at-rule: {name}; rule ignored")
        };
        self.warn(line, message);
        Some(i)
    }

    fn handle_rule(
        &mut self,
        prelude: &str,
        prelude_offset: usize,
        body: &str,
        body_offset: usize,
    ) {
        let rule_line = self.line_of(prelude_offset + prelude.len() - prelude.trim_start().len());
        let mut selectors = Vec::new();
        for part in prelude.split(',') {
            if let Some(sel) = self.parse_selector(part.trim(), rule_line) {
                selectors.push(sel);
            }
        }
        let decls = self.split_declarations(body, body_offset);
        for sel in selectors {
            let decls = decls
                .iter()
                .map(|d| RawDecl {
                    name: d.name.clone(),
                    value: d.value.clone(),
                    line: d.line,
                })
                .collect();
            self.rules.push(RawRule {
                selector: sel,
                decls,
                line: rule_line,
            });
        }
    }

    fn parse_selector(&mut self, sel: &str, line: u32) -> Option<Selector> {
        if sel == ":root" {
            return Some(Selector::Root);
        }
        let ident = |s: &str| {
            !s.is_empty()
                && s.chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        };
        let (name, pseudo) = match sel.strip_prefix('.') {
            Some(rest) => match rest.split_once(':') {
                Some((n, p)) => (n, Some(p)),
                None => (rest, None),
            },
            None => {
                self.warn(line, format!("unsupported selector: '{sel}'; rule ignored"));
                return None;
            }
        };
        if !ident(name) || !COMPONENTS.contains(&name) {
            self.warn(
                line,
                format!("unsupported selector: '{sel}' (unknown component); rule ignored"),
            );
            return None;
        }
        let state = match pseudo {
            None => State::Base,
            Some("focus") => State::Focus,
            Some("active") => State::Active,
            Some("disabled") => State::Disabled,
            Some(p) => {
                self.warn(
                    line,
                    format!("unsupported pseudo-class: ':{p}' in '{sel}'; rule ignored"),
                );
                return None;
            }
        };
        Some(Selector::Component(name.to_string(), state))
    }

    fn split_declarations(&mut self, body: &str, offset: usize) -> Vec<RawDecl> {
        let mut out = Vec::new();
        let mut depth = 0i32;
        let mut quote: Option<char> = None;
        let mut start = 0;
        let mut chunks: Vec<(usize, &str)> = Vec::new();
        for (i, c) in body.char_indices() {
            match (quote, c) {
                (Some(q), c) if c == q => quote = None,
                (Some(_), _) => {}
                (None, '"' | '\'') => quote = Some(c),
                (None, '(') => depth += 1,
                (None, ')') => depth -= 1,
                (None, ';') if depth <= 0 => {
                    chunks.push((start, &body[start..i]));
                    start = i + 1;
                }
                _ => {}
            }
        }
        chunks.push((start, &body[start..]));
        for (chunk_start, chunk) in chunks {
            let trimmed = chunk.trim();
            if trimmed.is_empty() {
                continue;
            }
            let lead = chunk.len() - chunk.trim_start().len();
            let line = self.line_of(offset + chunk_start + lead);
            let Some((name, value)) = trimmed.split_once(':') else {
                self.error(
                    line,
                    format!("invalid declaration '{trimmed}' (expected 'property: value')"),
                );
                continue;
            };
            let (name, value) = (name.trim().to_ascii_lowercase(), value.trim());
            let valid_name = !name.is_empty()
                && name
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');
            if !valid_name || value.is_empty() {
                self.error(
                    line,
                    format!("invalid declaration '{trimmed}' (expected 'property: value')"),
                );
                continue;
            }
            out.push(RawDecl {
                name,
                value: value.to_string(),
                line,
            });
        }
        out
    }

    fn finish(mut self) -> Parsed {
        // Variables from :root (the last definition wins).
        let mut raw_vars: Vec<(String, String, u32)> = Vec::new();
        let rules = std::mem::take(&mut self.rules);
        for rule in &rules {
            for d in &rule.decls {
                let is_var = d.name.starts_with("--");
                match (&rule.selector, is_var) {
                    (Selector::Root, true) => {
                        raw_vars.retain(|(n, _, _)| n != &d.name);
                        raw_vars.push((d.name.clone(), d.value.clone(), d.line));
                    }
                    (Selector::Root, false) => self.warn(
                        d.line,
                        format!(
                            "only variables (--name) are allowed in :root; property '{}' ignored",
                            d.name
                        ),
                    ),
                    (Selector::Component(..), true) => self.warn(
                        d.line,
                        format!(
                            "variables can only be declared in :root; '{}' ignored",
                            d.name
                        ),
                    ),
                    _ => {}
                }
            }
        }

        let mut resolver = Resolver {
            raw: &raw_vars,
            done: HashMap::new(),
        };
        let mut variables = Vec::new();
        let mut var_errors = Vec::new();
        for (name, _, line) in &raw_vars {
            match resolver.resolve_var(name, &mut Vec::new()) {
                Ok(v) => variables.push((name.clone(), v)),
                Err(ResolveError::Failed) => {}
                Err(e) => var_errors.push((*line, e.message(name))),
            }
        }
        for (line, msg) in var_errors {
            self.error(line, msg);
        }

        let mut sheet_rules = Vec::new();
        for rule in &rules {
            let Selector::Component(component, state) = &rule.selector else {
                continue;
            };
            let mut decls = Vec::new();
            for d in &rule.decls {
                if d.name.starts_with("--") {
                    continue;
                }
                if !PROPERTIES.contains(&d.name.as_str()) {
                    self.warn(
                        d.line,
                        format!("unsupported property: '{}'; ignored", d.name),
                    );
                    continue;
                }
                if d.value.contains("url(") {
                    self.warn(
                        d.line,
                        format!("url() is not supported (property '{}'); ignored", d.name),
                    );
                    continue;
                }
                let value = match resolver.substitute(&d.value, &mut Vec::new()) {
                    Ok(v) => v,
                    Err(ResolveError::Failed) => continue, // already reported at the variable definition
                    Err(e) => {
                        self.error(d.line, e.message(&d.name));
                        continue;
                    }
                };
                match value::parse_decl(&d.name, &value) {
                    Ok(decl) => {
                        if let Decl::FontSize(px) = decl {
                            if px < MIN_FONT_SIZE {
                                self.warn(
                                    d.line,
                                    format!("font-size of {px}px is below {MIN_FONT_SIZE}px and is hard to read on a TV"),
                                );
                            } else if !FONT_SCALE.contains(&px) {
                                self.warn(
                                    d.line,
                                    format!("font-size of {px}px is not in the type scale {FONT_SCALE:?}; the nearest size is used"),
                                );
                            }
                        }
                        decls.push(decl);
                    }
                    Err(msg) => self.error(d.line, format!("{}: {msg}", d.name)),
                }
            }
            sheet_rules.push(Rule {
                component: component.clone(),
                state: *state,
                decls,
                line: rule.line,
            });
        }

        Parsed {
            stylesheet: Stylesheet {
                variables,
                rules: sheet_rules,
            },
            diagnostics: self.diags,
        }
    }
}

enum ResolveError {
    Undefined(String),
    Cycle(String),
    Malformed,
    /// The variable used already has an error reported at its own definition.
    Failed,
}

impl ResolveError {
    fn message(&self, context: &str) -> String {
        match self {
            ResolveError::Undefined(n) => {
                format!("undefined variable: {n} (used in '{context}')")
            }
            ResolveError::Cycle(n) => format!("circular reference between variables at {n}"),
            ResolveError::Malformed => format!("malformed var() in '{context}'"),
            ResolveError::Failed => String::new(),
        }
    }
}

struct Resolver<'a> {
    raw: &'a [(String, String, u32)],
    done: HashMap<String, Result<String, ()>>,
}

impl Resolver<'_> {
    fn resolve_var(&mut self, name: &str, stack: &mut Vec<String>) -> Result<String, ResolveError> {
        if let Some(r) = self.done.get(name) {
            return r.clone().map_err(|_| ResolveError::Failed);
        }
        if stack.iter().any(|s| s == name) {
            return Err(ResolveError::Cycle(name.to_string()));
        }
        let Some((_, raw, _)) = self.raw.iter().find(|(n, _, _)| n == name) else {
            return Err(ResolveError::Undefined(name.to_string()));
        };
        let raw = raw.clone();
        stack.push(name.to_string());
        let result = self.substitute(&raw, stack);
        stack.pop();
        self.done.insert(
            name.to_string(),
            result.as_ref().map(|v| v.clone()).map_err(|_| ()),
        );
        result
    }

    /// Replaces each `var(--x[, fallback])` with its value.
    fn substitute(&mut self, value: &str, stack: &mut Vec<String>) -> Result<String, ResolveError> {
        let mut out = String::new();
        let mut rest = value;
        while let Some(pos) = rest.find("var(") {
            let preceded_by_ident = rest[..pos]
                .chars()
                .next_back()
                .is_some_and(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');
            if preceded_by_ident {
                out.push_str(&rest[..pos + 4]);
                rest = &rest[pos + 4..];
                continue;
            }
            out.push_str(&rest[..pos]);
            let args_start = pos + 4;
            let mut depth = 1;
            let mut end = None;
            for (i, c) in rest[args_start..].char_indices() {
                match c {
                    '(' => depth += 1,
                    ')' => {
                        depth -= 1;
                        if depth == 0 {
                            end = Some(args_start + i);
                            break;
                        }
                    }
                    _ => {}
                }
            }
            let end = end.ok_or(ResolveError::Malformed)?;
            let args = &rest[args_start..end];
            let (name, fallback) = match args.split_once(',') {
                Some((n, f)) => (n.trim(), Some(f.trim())),
                None => (args.trim(), None),
            };
            if !name.starts_with("--") {
                return Err(ResolveError::Malformed);
            }
            let replacement = match self.resolve_var(name, stack) {
                Ok(v) => v,
                Err(ResolveError::Undefined(n)) => match fallback {
                    Some(f) => self.substitute(f, stack)?,
                    None => return Err(ResolveError::Undefined(n)),
                },
                Err(e) => return Err(e),
            };
            out.push_str(&replacement);
            rest = &rest[end + 1..];
        }
        out.push_str(rest);
        Ok(out)
    }
}
