//! CLDR plural categories.
//!
//! Only the rules of the languages lume is likely to be translated into are listed;
//! other languages use the English rule (`one` for exactly 1). Adding a language that
//! follows an existing rule is one line in [`family_of`].

/// The CLDR plural categories, in the order of the generated `PluralForms` arrays.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Category {
    Zero = 0,
    One = 1,
    Two = 2,
    Few = 3,
    Many = 4,
    Other = 5,
}

/// A group of languages that share a plural rule.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Family {
    /// No plural forms: Japanese, Chinese, Korean, Vietnamese, Thai, Indonesian...
    NoPlural,
    /// `one` for exactly 1: English, German, Dutch, Swedish, Spanish, Italian, Turkish...
    OneIsOne,
    /// `one` for 0 and 1: French, Brazilian Portuguese.
    OneIsZeroOrOne,
    /// East Slavic (Russian, Ukrainian): one, few, many.
    EastSlavic,
    /// Polish: one, few, many.
    Polish,
    /// Czech and Slovak: one, few, other.
    CzechSlovak,
    /// Arabic: zero, one, two, few, many, other.
    Arabic,
}

/// The plural family of a language tag such as `pt-BR`, `ru` or `zh-Hans-CN`.
pub fn family_of(tag: &str) -> Family {
    let normalized = tag.replace('_', "-").to_ascii_lowercase();
    let mut parts = normalized.split('-');
    let language = parts.next().unwrap_or("");
    match language {
        "ja" | "zh" | "ko" | "vi" | "th" | "id" | "ms" | "lo" | "km" | "my" => Family::NoPlural,
        // European Portuguese only treats exactly 1 as singular.
        "pt" if parts.any(|p| p == "pt") => Family::OneIsOne,
        "pt" | "fr" | "hy" | "kab" => Family::OneIsZeroOrOne,
        "ru" | "uk" | "be" => Family::EastSlavic,
        "pl" => Family::Polish,
        "cs" | "sk" => Family::CzechSlovak,
        "ar" => Family::Arabic,
        _ => Family::OneIsOne,
    }
}

/// The plural category of the integer `count` in `family`.
pub fn category(family: Family, count: i64) -> Category {
    let n = count.unsigned_abs();
    let (mod10, mod100) = (n % 10, n % 100);
    match family {
        Family::NoPlural => Category::Other,
        Family::OneIsOne => {
            if n == 1 {
                Category::One
            } else {
                Category::Other
            }
        }
        Family::OneIsZeroOrOne => {
            if n <= 1 {
                Category::One
            } else {
                Category::Other
            }
        }
        Family::EastSlavic => {
            if mod10 == 1 && mod100 != 11 {
                Category::One
            } else if (2..=4).contains(&mod10) && !(12..=14).contains(&mod100) {
                Category::Few
            } else {
                Category::Many
            }
        }
        Family::Polish => {
            if n == 1 {
                Category::One
            } else if (2..=4).contains(&mod10) && !(12..=14).contains(&mod100) {
                Category::Few
            } else {
                Category::Many
            }
        }
        Family::CzechSlovak => match n {
            1 => Category::One,
            2..=4 => Category::Few,
            _ => Category::Other,
        },
        Family::Arabic => match n {
            0 => Category::Zero,
            1 => Category::One,
            2 => Category::Two,
            _ if (3..=10).contains(&mod100) => Category::Few,
            _ if (11..=99).contains(&mod100) => Category::Many,
            _ => Category::Other,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cat(tag: &str, n: i64) -> Category {
        category(family_of(tag), n)
    }

    #[test]
    fn portuguese_treats_zero_and_one_as_singular() {
        assert_eq!(cat("pt-BR", 0), Category::One);
        assert_eq!(cat("pt-BR", 1), Category::One);
        assert_eq!(cat("pt-BR", 2), Category::Other);
        assert_eq!(cat("pt", 21), Category::Other);
        // European Portuguese: only exactly 1.
        assert_eq!(cat("pt-PT", 0), Category::Other);
        assert_eq!(cat("pt-PT", 1), Category::One);
        assert_eq!(cat("pt_PT", 5), Category::Other);
    }

    #[test]
    fn english_and_friends() {
        for tag in ["en", "en-US", "de", "es", "nl", "tr", "unknown"] {
            assert_eq!(cat(tag, 1), Category::One, "{tag}");
            assert_eq!(cat(tag, 0), Category::Other, "{tag}");
            assert_eq!(cat(tag, 2), Category::Other, "{tag}");
        }
        assert_eq!(cat("fr", 0), Category::One);
    }

    #[test]
    fn japanese_has_a_single_form() {
        for n in [0, 1, 2, 100] {
            assert_eq!(cat("ja", n), Category::Other);
            assert_eq!(cat("zh-Hans-CN", n), Category::Other);
        }
    }

    #[test]
    fn russian_follows_the_cldr_table() {
        let expected = [
            (1, Category::One),
            (2, Category::Few),
            (4, Category::Few),
            (5, Category::Many),
            (11, Category::Many),
            (12, Category::Many),
            (14, Category::Many),
            (21, Category::One),
            (22, Category::Few),
            (25, Category::Many),
            (101, Category::One),
            (111, Category::Many),
            (0, Category::Many),
        ];
        for (n, want) in expected {
            assert_eq!(cat("ru", n), want, "ru {n}");
            assert_eq!(cat("uk", n), want, "uk {n}");
        }
    }

    #[test]
    fn polish_and_czech() {
        assert_eq!(cat("pl", 1), Category::One);
        assert_eq!(cat("pl", 3), Category::Few);
        assert_eq!(cat("pl", 5), Category::Many);
        assert_eq!(cat("pl", 22), Category::Few);
        assert_eq!(cat("pl", 12), Category::Many);
        assert_eq!(cat("cs", 1), Category::One);
        assert_eq!(cat("cs", 3), Category::Few);
        assert_eq!(cat("cs", 5), Category::Other);
    }

    #[test]
    fn arabic_uses_every_category() {
        let expected = [
            (0, Category::Zero),
            (1, Category::One),
            (2, Category::Two),
            (3, Category::Few),
            (10, Category::Few),
            (11, Category::Many),
            (99, Category::Many),
            (100, Category::Other),
            (102, Category::Other),
            (103, Category::Few),
        ];
        for (n, want) in expected {
            assert_eq!(cat("ar", n), want, "ar {n}");
        }
    }

    #[test]
    fn negative_counts_use_the_magnitude() {
        assert_eq!(cat("en", -1), Category::One);
        assert_eq!(cat("ru", -21), Category::One);
    }
}
