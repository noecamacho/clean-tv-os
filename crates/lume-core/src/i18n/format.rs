//! Positional placeholders (`%1$s`, `%2$d`) and the pseudo-locale.

use std::fmt::Write as _;

/// A value for a placeholder.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Arg<'a> {
    Str(&'a str),
    Int(i64),
}

impl std::fmt::Display for Arg<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Arg::Str(s) => f.write_str(s),
            Arg::Int(n) => write!(f, "{n}"),
        }
    }
}

/// A piece of a template.
enum Piece<'a> {
    Text(&'a str),
    /// A placeholder: its position (`%2$s` -> `Some(2)`) and as written.
    Placeholder(Option<usize>, &'a str),
}

/// Splits a template into literal text and placeholders. `%%` becomes a literal `%`.
fn tokenize(template: &str) -> Vec<Piece<'_>> {
    let mut pieces = Vec::new();
    let mut rest = template;
    while let Some(percent) = rest.find('%') {
        pieces.push(Piece::Text(&rest[..percent]));
        let after = &rest[percent + 1..];
        if let Some(stripped) = after.strip_prefix('%') {
            pieces.push(Piece::Text("%"));
            rest = stripped;
            continue;
        }
        // %[N$]conversion, with an ASCII letter as the conversion.
        let digits = after.chars().take_while(char::is_ascii_digit).count();
        let (position, tail) = match after[digits..].strip_prefix('$') {
            Some(tail) if digits > 0 => (after[..digits].parse::<usize>().ok(), tail),
            _ => (None, after),
        };
        match tail.chars().next() {
            Some(conversion) if conversion.is_ascii_alphabetic() => {
                let consumed = percent + 1 + (after.len() - tail.len()) + conversion.len_utf8();
                pieces.push(Piece::Placeholder(position, &rest[percent..consumed]));
                rest = &rest[consumed..];
            }
            _ => {
                // A lone '%' is kept as text.
                pieces.push(Piece::Text("%"));
                rest = after;
            }
        }
    }
    pieces.push(Piece::Text(rest));
    pieces
}

/// Fills the placeholders of `template` with `args`. A placeholder without a matching
/// argument is left as written.
pub fn format(template: &str, args: &[Arg]) -> String {
    let mut out = String::with_capacity(template.len() + 16);
    let mut next_implicit = 1;
    for piece in tokenize(template) {
        match piece {
            Piece::Text(text) => out.push_str(text),
            Piece::Placeholder(position, raw) => {
                let index = position.unwrap_or_else(|| {
                    let i = next_implicit;
                    next_implicit += 1;
                    i
                });
                match index.checked_sub(1).and_then(|i| args.get(i)) {
                    Some(arg) => write!(out, "{arg}").unwrap(),
                    None => out.push_str(raw),
                }
            }
        }
    }
    out
}

/// Makes a text look foreign and 40% longer, to test layouts and fonts:
/// vowels get accents, and filler is added at the end. Placeholders are left alone.
pub fn pseudo_localize(template: &str) -> String {
    let mut out = String::new();
    for piece in tokenize(template) {
        match piece {
            Piece::Text("%") => out.push_str("%%"),
            Piece::Text(text) => out.extend(text.chars().map(accent)),
            Piece::Placeholder(_, raw) => out.push_str(raw),
        }
    }
    let length = template.chars().count();
    let filler = (length * 4).div_ceil(10).max(1);
    out.extend(std::iter::repeat_n('~', filler));
    out
}

fn accent(c: char) -> char {
    match c {
        'a' => 'á',
        'e' => 'é',
        'i' => 'í',
        'o' => 'ó',
        'u' => 'ú',
        'c' => 'ç',
        'n' => 'ñ',
        'y' => 'ý',
        'A' => 'Á',
        'E' => 'É',
        'I' => 'Í',
        'O' => 'Ó',
        'U' => 'Ú',
        _ => c,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fills_positional_placeholders_in_any_order() {
        assert_eq!(
            format("Aberto há %1$d horas", &[Arg::Int(3)]),
            "Aberto há 3 horas"
        );
        assert_eq!(
            format("%2$s: %1$d", &[Arg::Int(5), Arg::Str("Globoplay")]),
            "Globoplay: 5"
        );
        assert_eq!(
            format("Novo: %1$s", &[Arg::Str("Globoplay")]),
            "Novo: Globoplay"
        );
        assert_eq!(format("%1$s e %1$s", &[Arg::Str("a")]), "a e a");
    }

    #[test]
    fn handles_implicit_positions_percent_signs_and_missing_args() {
        assert_eq!(
            format("%s tem %d", &[Arg::Str("x"), Arg::Int(2)]),
            "x tem 2"
        );
        assert_eq!(format("100%% de %1$d%%", &[Arg::Int(7)]), "100% de 7%");
        assert_eq!(format("sem %1$s", &[]), "sem %1$s");
        assert_eq!(format("a 50% b", &[]), "a 50% b");
        assert_eq!(format("termina com %", &[]), "termina com %");
        assert_eq!(format("%3$d", &[Arg::Int(1)]), "%3$d");
        assert_eq!(format("sem marcadores", &[Arg::Int(1)]), "sem marcadores");
        assert_eq!(format("", &[]), "");
    }

    #[test]
    fn unicode_text_around_placeholders_is_preserved() {
        assert_eq!(format("爱奇艺 %1$s ✓", &[Arg::Str("é")]), "爱奇艺 é ✓");
    }

    #[test]
    fn a_number_placeholder_accepts_text_and_vice_versa() {
        assert_eq!(format("%1$d", &[Arg::Str("x")]), "x");
        assert_eq!(format("%1$s", &[Arg::Int(4)]), "4");
    }

    #[test]
    fn pseudo_locale_lengthens_by_at_least_forty_percent_and_accents_vowels() {
        for text in ["Home", "Your favorite apps", "Opened 3 hours ago", "x"] {
            let pseudo = pseudo_localize(text);
            assert!(
                pseudo.chars().count() as f32 >= text.chars().count() as f32 * 1.4,
                "{pseudo:?}"
            );
        }
        assert!(pseudo_localize("Home").starts_with("Hómé"));
    }

    #[test]
    fn pseudo_locale_keeps_placeholders_intact() {
        let pseudo = pseudo_localize("Aberto há %1$d horas");
        assert!(pseudo.contains("%1$d"), "{pseudo}");
        assert!(format(&pseudo, &[Arg::Int(3)]).contains('3'));
        assert!(!pseudo_localize("100%% unique").contains("%%%"));
    }
}
