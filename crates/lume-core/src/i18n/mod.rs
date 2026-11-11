//! Translations: language choice, texts, plurals and placeholders.
//!
//! The texts come from `android/app/src/main/res/values*/strings.xml`, compiled by
//! `build.rs` into static tables; nothing is parsed at runtime.

mod format;
mod plural;

pub use format::{format, pseudo_localize, Arg};
pub use plural::{category, family_of, Category, Family};

use std::borrow::Cow;

include!(concat!(env!("OUT_DIR"), "/strings_generated.rs"));

/// The reference language and fallback for anything missing.
pub const FALLBACK_LANGUAGE: &str = "en";

/// Tags of the languages that have a translation.
pub fn available_languages() -> impl Iterator<Item = &'static str> {
    LANGUAGES.iter().map(|l| l.tag)
}

/// Normalizes a language tag: `pt_br` -> `pt-BR`, `zh-hans-cn` -> `zh-Hans-CN`, and the
/// obsolete codes Android may still report (`iw`, `in`, `ji`) -> `he`, `id`, `yi`.
pub fn normalize_tag(tag: &str) -> String {
    let mut parts = tag.trim().split(['-', '_']);
    let language = match parts.next().unwrap_or("").to_ascii_lowercase().as_str() {
        "iw" => "he".to_string(),
        "in" => "id".to_string(),
        "ji" => "yi".to_string(),
        other => other.to_string(),
    };
    let mut out = language;
    for part in parts {
        out.push('-');
        match part.len() {
            4 if part.chars().all(|c| c.is_ascii_alphabetic()) => {
                out.push_str(&part[..1].to_ascii_uppercase());
                out.push_str(&part[1..].to_ascii_lowercase());
            }
            2 if part.chars().all(|c| c.is_ascii_alphabetic()) => {
                out.push_str(&part.to_ascii_uppercase())
            }
            _ => out.push_str(part),
        }
    }
    out
}

fn language_subtag(tag: &str) -> &str {
    tag.split('-').next().unwrap_or(tag)
}

/// Picks the language to show: for each preferred tag, in order, an exact match first and
/// then any translation of the same language; `None` when nothing matches.
pub fn pick_language(preferred: &[&str], available: &[&str]) -> Option<usize> {
    let available: Vec<String> = available.iter().map(|t| normalize_tag(t)).collect();
    for tag in preferred {
        let tag = normalize_tag(tag);
        if tag.is_empty() {
            continue;
        }
        if let Some(i) = available.iter().position(|a| *a == tag) {
            return Some(i);
        }
        let language = language_subtag(&tag);
        // The bare language is the best stand-in, then any variant of it.
        if let Some(i) = available.iter().position(|a| a == language) {
            return Some(i);
        }
        if let Some(i) = available
            .iter()
            .position(|a| language_subtag(a) == language)
        {
            return Some(i);
        }
    }
    None
}

/// The translated texts for one language, with English as the fallback.
#[derive(Clone, Copy)]
pub struct Translator {
    language: &'static LanguageTable,
    reference: &'static LanguageTable,
    family: Family,
    pseudo: bool,
}

impl Translator {
    /// Chooses the language from the user's preferred languages, in order.
    pub fn new(preferred: &[&str]) -> Translator {
        let tags: Vec<&str> = available_languages().collect();
        let index = pick_language(preferred, &tags).unwrap_or(0);
        Translator::for_table(&LANGUAGES[index])
    }

    fn for_table(language: &'static LanguageTable) -> Translator {
        Translator {
            language,
            reference: &LANGUAGES[0],
            family: family_of(language.tag),
            pseudo: false,
        }
    }

    /// The same language with every text lengthened and accented, to test layouts.
    pub fn pseudo(mut self) -> Translator {
        self.pseudo = true;
        self
    }

    /// Tag of the language in use, such as `pt-BR`.
    pub fn language(&self) -> &'static str {
        self.language.tag
    }

    fn template(&self, id: StringId) -> &'static str {
        self.language.strings[id as usize]
            .or(self.reference.strings[id as usize])
            .expect("the reference language has every string")
    }

    fn decorate(&self, text: &'static str) -> Cow<'static, str> {
        if self.pseudo {
            Cow::Owned(pseudo_localize(text))
        } else {
            Cow::Borrowed(text)
        }
    }

    /// A string without placeholders.
    pub fn get(&self, id: StringId) -> Cow<'static, str> {
        self.decorate(self.template(id))
    }

    /// A string with positional placeholders.
    pub fn format(&self, id: StringId, args: &[Arg]) -> String {
        format::format(&self.decorate(self.template(id)), args)
    }

    /// The plural form of `id` for `count`. `args` fills the placeholders; the count itself is
    /// usually the first one.
    pub fn plural(&self, id: PluralId, count: i64, args: &[Arg]) -> String {
        let wanted = category(self.family, count) as usize;
        let pick = |table: &'static LanguageTable| -> Option<&'static str> {
            let forms = table.plurals[id as usize]?;
            forms[wanted].or(forms[Category::Other as usize])
        };
        let template = pick(self.language)
            .or_else(|| {
                // Fall back to the reference language, using its own plural rule.
                let forms = self.reference.plurals[id as usize]?;
                let english = category(family_of(self.reference.tag), count) as usize;
                forms[english].or(forms[Category::Other as usize])
            })
            .expect("the reference language has every plural");
        format::format(&self.decorate(template), args)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const AVAILABLE: [&str; 3] = ["en", "es", "pt-BR"];

    fn picked(preferred: &[&str]) -> &'static str {
        // Resolve against the real tables, for the scenarios of the spec.
        Translator::new(preferred).language()
    }

    #[test]
    fn tags_are_normalized() {
        assert_eq!(normalize_tag("pt_br"), "pt-BR");
        assert_eq!(normalize_tag("PT-br"), "pt-BR");
        assert_eq!(normalize_tag("zh-hans-cn"), "zh-Hans-CN");
        assert_eq!(normalize_tag("iw"), "he");
        assert_eq!(normalize_tag("in-ID"), "id-ID");
        assert_eq!(normalize_tag("en"), "en");
        assert_eq!(normalize_tag(" en-US "), "en-US");
        assert_eq!(normalize_tag(""), "");
    }

    #[test]
    fn exact_match_comes_first() {
        assert_eq!(pick_language(&["pt-BR"], &AVAILABLE), Some(2));
        assert_eq!(pick_language(&["es"], &AVAILABLE), Some(1));
        assert_eq!(pick_language(&["pt_BR"], &AVAILABLE), Some(2));
        let with_both = ["pt", "pt-BR", "en"];
        assert_eq!(pick_language(&["pt-BR"], &with_both), Some(1));
    }

    #[test]
    fn language_only_matches_any_variant() {
        assert_eq!(pick_language(&["pt-PT"], &AVAILABLE), Some(2));
        assert_eq!(pick_language(&["pt"], &AVAILABLE), Some(2));
        assert_eq!(pick_language(&["en-GB"], &AVAILABLE), Some(0));
        assert_eq!(pick_language(&["es-MX"], &AVAILABLE), Some(1));
        // The bare language wins over another variant.
        assert_eq!(pick_language(&["pt-PT"], &["pt-BR", "pt"]), Some(1));
    }

    #[test]
    fn the_first_preferred_language_with_a_translation_wins() {
        assert_eq!(pick_language(&["ja", "pt-BR"], &AVAILABLE), Some(2));
        assert_eq!(pick_language(&["ja", "ko"], &AVAILABLE), None);
        assert_eq!(pick_language(&["", "es"], &AVAILABLE), Some(1));
        assert_eq!(pick_language(&[], &AVAILABLE), None);
    }

    #[test]
    fn spec_scenarios_with_the_real_tables() {
        assert_eq!(picked(&["pt-BR"]), "pt-BR");
        assert_eq!(picked(&["pt-PT"]), "pt-BR");
        assert_eq!(picked(&["ja"]), "en");
        assert_eq!(picked(&["ja", "pt-BR"]), "pt-BR");
        assert_eq!(picked(&[]), "en");
        assert_eq!(picked(&["en-US", "pt-BR"]), "en");
    }

    #[test]
    fn the_mockup_labels_in_portuguese() {
        let t = Translator::new(&["pt-BR"]);
        assert_eq!(t.get(StringId::TabHome), "Início");
        assert_eq!(t.get(StringId::TabApps), "Apps");
        assert_eq!(t.get(StringId::HomeFavoritesTitle), "Seus apps favoritos");
        assert_eq!(t.get(StringId::AppsTitle), "Todos os apps");
        assert_eq!(t.get(StringId::EditorDone), "Concluído");
        assert_eq!(Translator::new(&["en"]).get(StringId::TabHome), "Home");
    }

    #[test]
    fn cinematic_home_follows_spanish_or_english_tv_language() {
        let spanish = Translator::new(&["es-ES"]);
        assert_eq!(spanish.get(StringId::TabHome), "Inicio");
        assert_eq!(spanish.get(StringId::WallpaperCinematic), "Cala turquesa");

        let english = Translator::new(&["en-US"]);
        assert_eq!(english.get(StringId::TabHome), "Home");
        assert_eq!(english.get(StringId::WallpaperCinematic), "Beach Cove");
    }

    #[test]
    fn plurals_in_portuguese_and_english() {
        let pt = Translator::new(&["pt-BR"]);
        let one = pt.plural(PluralId::EditorLimitReached, 1, &[Arg::Int(1)]);
        let three = pt.plural(PluralId::EditorLimitReached, 3, &[Arg::Int(3)]);
        assert_eq!(
            (one.as_str(), three.as_str()),
            ("Limite de 1 favorito", "Limite de 3 favoritos")
        );
        let en = Translator::new(&["en"]);
        assert_eq!(
            en.plural(PluralId::EditorLimitReached, 1, &[Arg::Int(1)]),
            "Limit of 1 favorite"
        );
        assert_eq!(
            en.plural(PluralId::EditorLimitReached, 2, &[Arg::Int(2)]),
            "Limit of 2 favorites"
        );
        assert_eq!(
            en.plural(PluralId::EditorLimitReached, 0, &[Arg::Int(0)]),
            "Limit of 0 favorites"
        );
    }

    #[test]
    fn placeholders_follow_the_translation_order() {
        let t = Translator::new(&["pt-BR"]);
        assert_eq!(
            t.format(StringId::AboutSource, &[Arg::Str("github.com/x")]),
            "Código-fonte: github.com/x"
        );
        assert_eq!(
            Translator::new(&["en"]).format(StringId::AboutVersion, &[Arg::Str("0.1.0")]),
            "Version 0.1.0"
        );
    }

    #[test]
    fn missing_translations_fall_back_to_english() {
        // Build a translator over a table with holes, like a partial translation.
        static PARTIAL: LanguageTable = {
            let mut strings = [None; StringId::COUNT];
            strings[StringId::TabHome as usize] = Some("Início");
            LanguageTable {
                tag: "pt-BR",
                strings,
                plurals: [None; PluralId::COUNT],
            }
        };
        let t = Translator::for_table(&PARTIAL);
        assert_eq!(t.get(StringId::TabHome), "Início");
        assert_eq!(t.get(StringId::TabApps), "Apps");
        assert_eq!(t.get(StringId::HomeFavoritesTitle), "Your favorite apps");
        // The plural falls back to the English text with English rules.
        assert_eq!(
            t.plural(PluralId::EditorLimitReached, 1, &[Arg::Int(1)]),
            "Limit of 1 favorite"
        );
        assert_eq!(
            t.plural(PluralId::EditorLimitReached, 0, &[Arg::Int(0)]),
            "Limit of 0 favorites"
        );
    }

    #[test]
    fn plural_forms_pick_the_right_category_for_each_family() {
        // A Russian-style table: one, few, many, other.
        static RU: LanguageTable = {
            let mut plurals = [None; PluralId::COUNT];
            plurals[PluralId::EditorLimitReached as usize] = Some([
                None,
                Some("%1$d день"),
                None,
                Some("%1$d дня"),
                Some("%1$d дней"),
                Some("%1$d дня"),
            ]);
            LanguageTable {
                tag: "ru",
                strings: [None; StringId::COUNT],
                plurals,
            }
        };
        let t = Translator::for_table(&RU);
        let day = |n| t.plural(PluralId::EditorLimitReached, n, &[Arg::Int(n)]);
        assert_eq!(day(1), "1 день");
        assert_eq!(day(3), "3 дня");
        assert_eq!(day(5), "5 дней");
        assert_eq!(day(21), "21 день");
        assert_eq!(day(11), "11 дней");
    }

    #[test]
    fn the_pseudo_locale_stretches_every_text() {
        let t = Translator::new(&["pt-BR"]).pseudo();
        for id in StringId::ALL {
            let plain = Translator::new(&["pt-BR"]).get(id);
            let stretched = t.get(id);
            assert!(
                stretched.chars().count() as f32 >= plain.chars().count() as f32 * 1.4,
                "{}: {stretched:?}",
                id.key()
            );
        }
        let limit = t.plural(PluralId::EditorLimitReached, 3, &[Arg::Int(3)]);
        assert!(limit.contains('3') && limit.ends_with('~'));
    }

    #[test]
    fn every_string_has_text_in_every_language() {
        // The shipped translations are complete: no fallback is needed.
        for language in &LANGUAGES {
            for id in StringId::ALL {
                assert!(
                    language.strings[id as usize].is_some(),
                    "{} lacks {}",
                    language.tag,
                    id.key()
                );
            }
            for id in PluralId::ALL {
                assert!(
                    language.plurals[id as usize].is_some(),
                    "{} lacks {}",
                    language.tag,
                    id.key()
                );
            }
        }
    }

    #[test]
    fn ids_and_keys_round_trip() {
        assert_eq!(StringId::HomeFavoritesTitle.key(), "home_favorites_title");
        assert_eq!(PluralId::EditorLimitReached.key(), "editor_limit_reached");
        assert_eq!(StringId::ALL.len(), StringId::COUNT);
        assert_eq!(
            available_languages().collect::<Vec<_>>(),
            ["en", "es", "id", "pt-BR", "ru"]
        );
    }
}
