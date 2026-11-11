//! Validates translations against the reference language and emits Rust tables.

use crate::android::Strings;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

/// Plural quantities, in the order of the generated `PluralForms` arrays.
pub const QUANTITIES: [&str; 6] = ["zero", "one", "two", "few", "many", "other"];

pub struct Compiled {
    /// Rust source with `StringId`, `PluralId`, `LanguageTable` and `LANGUAGES`.
    pub code: String,
    /// Problems found in translations, ready to print.
    pub warnings: Vec<String>,
    /// Tags of the generated languages; the reference language `en` comes first.
    pub languages: Vec<String>,
}

/// A `%N$c` placeholder of a format string.
type Placeholder = (u32, char);

/// The placeholders of `text`; `%%` is a literal percent sign and `%s`/`%d` without a position
/// count from 1 in order.
pub fn placeholders(text: &str) -> BTreeSet<(u32, char)> {
    let mut found = BTreeSet::new();
    let mut next_implicit = 1;
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '%' {
            continue;
        }
        if chars.peek() == Some(&'%') {
            chars.next();
            continue;
        }
        let mut digits = String::new();
        let mut explicit = None;
        while let Some(&d) = chars.peek() {
            if d.is_ascii_digit() {
                digits.push(d);
                chars.next();
            } else {
                break;
            }
        }
        if chars.peek() == Some(&'$') && !digits.is_empty() {
            chars.next();
            explicit = digits.parse().ok();
        }
        // Flags and width between the position and the conversion.
        while let Some(&f) = chars.peek() {
            if f.is_ascii_digit() || matches!(f, '-' | '+' | ' ' | '#' | '.' | ',') {
                chars.next();
            } else {
                break;
            }
        }
        if let Some(conversion) = chars.next() {
            if conversion.is_ascii_alphabetic() {
                let index = explicit.unwrap_or_else(|| {
                    let i = next_implicit;
                    next_implicit += 1;
                    i
                });
                found.insert((index, conversion));
            }
        }
    }
    found
}

fn is_valid_key(key: &str) -> bool {
    let mut chars = key.chars();
    chars.next().is_some_and(|c| c.is_ascii_lowercase())
        && chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}

fn camel_case(key: &str) -> String {
    key.split('_')
        .filter(|p| !p.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            chars
                .next()
                .map(|c| c.to_ascii_uppercase().to_string() + chars.as_str())
                .unwrap_or_default()
        })
        .collect()
}

/// Compiles the languages found in the resource directories.
/// `languages` holds `(tag, texts)`; the tag is `None` for the default `values` directory.
pub fn compile(languages: Vec<(Option<String>, Strings)>) -> Result<Compiled, String> {
    let mut reference = None;
    let mut english_overlay = None;
    let mut others: BTreeMap<String, Strings> = BTreeMap::new();
    for (tag, strings) in languages {
        match tag.as_deref() {
            None => reference = Some(strings),
            Some("en") => english_overlay = Some(strings),
            Some(tag) => {
                others.insert(tag.to_string(), strings);
            }
        }
    }
    let mut reference = reference.ok_or("the default language (values/strings.xml) is missing")?;
    if let Some(overlay) = english_overlay {
        reference.strings.extend(overlay.strings);
        reference.plurals.extend(overlay.plurals);
        reference.untranslatable.extend(overlay.untranslatable);
    }

    // The reference language must be sound: every plural needs "other", and keys must be usable.
    for key in reference.strings.keys().chain(reference.plurals.keys()) {
        if !is_valid_key(key) {
            return Err(format!(
                "invalid key '{key}': use lowercase letters, digits and underscores"
            ));
        }
    }
    if let Some(key) = reference
        .strings
        .keys()
        .find(|k| reference.plurals.contains_key(*k))
    {
        return Err(format!("'{key}' is both a string and a plurals"));
    }
    for (key, items) in &reference.plurals {
        validate_quantities(items)
            .map_err(|e| format!("default language: plurals '{key}': {e}"))?;
    }
    let mut identifiers = BTreeSet::new();
    for key in reference.strings.keys().chain(reference.plurals.keys()) {
        if !identifiers.insert(camel_case(key)) {
            return Err(format!("the key '{key}' produces a duplicate identifier"));
        }
    }

    let mut warnings = Vec::new();
    let mut tables = vec![("en".to_string(), reference_table(&reference))];
    for (tag, strings) in &others {
        tables.push((
            tag.clone(),
            translated_table(tag, strings, &reference, &mut warnings),
        ));
    }

    let string_keys: Vec<&String> = reference.strings.keys().collect();
    let plural_keys: Vec<&String> = reference.plurals.keys().collect();
    let code = generate(&string_keys, &plural_keys, &tables);
    Ok(Compiled {
        code,
        warnings,
        languages: tables.into_iter().map(|(tag, _)| tag).collect(),
    })
}

type StringEntries = Vec<Option<String>>;
type PluralEntries = Vec<Option<[Option<String>; 6]>>;
type Table = (StringEntries, PluralEntries);

fn forms(items: &BTreeMap<String, String>) -> [Option<String>; 6] {
    QUANTITIES.map(|q| items.get(q).cloned())
}

fn reference_table(reference: &Strings) -> Table {
    (
        reference.strings.values().cloned().map(Some).collect(),
        reference
            .plurals
            .values()
            .map(|items| Some(forms(items)))
            .collect(),
    )
}

fn validate_quantities(items: &BTreeMap<String, String>) -> Result<(), String> {
    if let Some(bad) = items.keys().find(|q| !QUANTITIES.contains(&q.as_str())) {
        return Err(format!("unknown quantity '{bad}'"));
    }
    if !items.contains_key("other") {
        return Err("the 'other' quantity is required".into());
    }
    Ok(())
}

fn translated_table(
    tag: &str,
    strings: &Strings,
    reference: &Strings,
    warnings: &mut Vec<String>,
) -> Table {
    for key in strings.strings.keys() {
        if !reference.strings.contains_key(key) {
            warnings.push(format!(
                "strings: {tag}: unknown key '{key}' (not in the reference language); ignored"
            ));
        }
    }
    for key in strings.plurals.keys() {
        if !reference.plurals.contains_key(key) {
            warnings.push(format!(
                "strings: {tag}: unknown plurals '{key}' (not in the reference language); ignored"
            ));
        }
    }

    let mut string_entries = Vec::new();
    for (key, reference_text) in &reference.strings {
        let untranslatable = reference.untranslatable.contains(key);
        let entry = match strings.strings.get(key) {
            _ if untranslatable => {
                if strings.strings.contains_key(key) {
                    warnings.push(format!(
                        "strings: {tag}: '{key}' is not translatable; ignored"
                    ));
                }
                Some(reference_text.clone())
            }
            None => {
                warnings.push(format!(
                    "strings: {tag}: missing string '{key}' (the English text is used)"
                ));
                None
            }
            Some(text) if placeholders(text) != placeholders(reference_text) => {
                warnings.push(format!(
                    "strings: {tag}: '{key}' has placeholders {} but the reference has {} (the English text is used)",
                    describe(&placeholders(text)),
                    describe(&placeholders(reference_text))
                ));
                None
            }
            Some(text) => Some(text.clone()),
        };
        string_entries.push(entry);
    }

    let mut plural_entries = Vec::new();
    for (key, reference_items) in &reference.plurals {
        let entry = match strings.plurals.get(key) {
            None => {
                warnings.push(format!(
                    "strings: {tag}: missing plurals '{key}' (the English text is used)"
                ));
                None
            }
            Some(items) => match validate_quantities(items) {
                Err(e) => {
                    warnings.push(format!(
                        "strings: {tag}: plurals '{key}': {e} (the English text is used)"
                    ));
                    None
                }
                Ok(()) => {
                    // Every form may use only the placeholders of the reference's "other" form.
                    let allowed = placeholders(&reference_items["other"]);
                    match items
                        .iter()
                        .find(|(_, text)| !placeholders(text).is_subset(&allowed))
                    {
                        Some((quantity, text)) => {
                            warnings.push(format!(
                                "strings: {tag}: plurals '{key}' item '{quantity}' has placeholders {} but the reference allows {} (the English text is used)",
                                describe(&placeholders(text)),
                                describe(&allowed)
                            ));
                            None
                        }
                        None => Some(forms(items)),
                    }
                }
            },
        };
        plural_entries.push(entry);
    }
    (string_entries, plural_entries)
}

fn describe(set: &BTreeSet<Placeholder>) -> String {
    if set.is_empty() {
        return "none".to_string();
    }
    set.iter()
        .map(|(i, c)| format!("%{i}${c}"))
        .collect::<Vec<_>>()
        .join(", ")
}

fn generate(
    string_keys: &[&String],
    plural_keys: &[&String],
    tables: &[(String, Table)],
) -> String {
    let mut out = String::from("// @generated by build.rs from android/app/src/main/res/values*/strings.xml. Do not edit.\n\n");

    write_enum(
        &mut out,
        "StringId",
        "Every plain string of the app.",
        string_keys,
    );
    write_enum(
        &mut out,
        "PluralId",
        "Every plurals resource of the app.",
        plural_keys,
    );
    writeln!(
        out,
        "/// Plural forms in the order zero, one, two, few, many, other."
    )
    .unwrap();
    writeln!(out, "pub type PluralForms = [Option<&'static str>; 6];\n").unwrap();
    writeln!(
        out,
        "/// The texts of one language. `None` entries fall back to the reference language.\npub struct LanguageTable {{\n    pub tag: &'static str,\n    pub strings: [Option<&'static str>; StringId::COUNT],\n    pub plurals: [Option<PluralForms>; PluralId::COUNT],\n}}\n"
    )
    .unwrap();
    writeln!(out, "/// Every language, the reference language `en` first.\npub static LANGUAGES: [LanguageTable; {}] = [", tables.len()).unwrap();
    for (tag, (strings, plurals)) in tables {
        writeln!(
            out,
            "    LanguageTable {{\n        tag: {tag:?},\n        strings: ["
        )
        .unwrap();
        for entry in strings {
            writeln!(out, "            {},", option_str(entry.as_deref())).unwrap();
        }
        writeln!(out, "        ],\n        plurals: [").unwrap();
        for entry in plurals {
            match entry {
                None => writeln!(out, "            None,").unwrap(),
                Some(forms) => {
                    let items: Vec<String> =
                        forms.iter().map(|f| option_str(f.as_deref())).collect();
                    writeln!(out, "            Some([{}]),", items.join(", ")).unwrap();
                }
            }
        }
        writeln!(out, "        ],\n    }},").unwrap();
    }
    out.push_str("];\n");
    out
}

fn option_str(value: Option<&str>) -> String {
    match value {
        Some(text) => format!("Some({text:?})"),
        None => "None".to_string(),
    }
}

fn write_enum(out: &mut String, name: &str, doc: &str, keys: &[&String]) {
    writeln!(
        out,
        "/// {doc}\n#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]\npub enum {name} {{"
    )
    .unwrap();
    for key in keys {
        writeln!(out, "    {},", camel_case(key)).unwrap();
    }
    writeln!(
        out,
        "}}\n\nimpl {name} {{\n    pub const COUNT: usize = {};\n",
        keys.len()
    )
    .unwrap();
    writeln!(out, "    pub const ALL: [{name}; {}] = [", keys.len()).unwrap();
    for key in keys {
        writeln!(out, "        {name}::{},", camel_case(key)).unwrap();
    }
    writeln!(out, "    ];\n\n    /// The resource name, as written in strings.xml.\n    pub const fn key(self) -> &'static str {{\n        match self {{").unwrap();
    for key in keys {
        writeln!(out, "            {name}::{} => {key:?},", camel_case(key)).unwrap();
    }
    writeln!(out, "        }}\n    }}\n}}\n").unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn strings(pairs: &[(&str, &str)], plurals: &[(&str, &[(&str, &str)])]) -> Strings {
        Strings {
            strings: pairs
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            plurals: plurals
                .iter()
                .map(|(k, items)| {
                    (
                        k.to_string(),
                        items
                            .iter()
                            .map(|(q, t)| (q.to_string(), t.to_string()))
                            .collect(),
                    )
                })
                .collect(),
            untranslatable: Vec::new(),
        }
    }

    fn reference() -> Strings {
        let mut s = strings(
            &[
                ("app_name", "lume"),
                ("greeting", "Hello %1$s"),
                ("title", "Title"),
            ],
            &[("days", &[("one", "%1$d day"), ("other", "%1$d days")])],
        );
        s.untranslatable.push("app_name".into());
        s
    }

    fn compile_with(translation: Strings) -> Compiled {
        compile(vec![(None, reference()), (Some("xx".into()), translation)]).unwrap()
    }

    #[test]
    fn placeholders_are_extracted() {
        assert_eq!(
            placeholders("Hello %1$s, you have %2$d"),
            [(1, 's'), (2, 'd')].into()
        );
        assert_eq!(placeholders("100%% sure"), BTreeSet::new());
        assert_eq!(placeholders("%s and %d"), [(1, 's'), (2, 'd')].into());
        assert_eq!(placeholders("%1$05.2f"), [(1, 'f')].into());
        assert_eq!(placeholders("no placeholders"), BTreeSet::new());
        assert_eq!(placeholders("ends with %"), BTreeSet::new());
    }

    #[test]
    fn a_complete_translation_has_no_warnings() {
        let compiled = compile_with(strings(
            &[("greeting", "Olá %1$s"), ("title", "Título")],
            &[("days", &[("one", "%1$d dia"), ("other", "%1$d dias")])],
        ));
        assert!(compiled.warnings.is_empty(), "{:?}", compiled.warnings);
        assert_eq!(compiled.languages, ["en", "xx"]);
    }

    #[test]
    fn a_missing_key_warns_with_language_and_key() {
        let compiled = compile_with(strings(
            &[("title", "Título")],
            &[("days", &[("other", "%1$d dias")])],
        ));
        assert_eq!(compiled.warnings.len(), 1, "{:?}", compiled.warnings);
        assert!(compiled.warnings[0].contains("xx") && compiled.warnings[0].contains("greeting"));
        // The generated table has no entry for it, so the reference text is used at runtime.
        assert!(compiled.code.contains("tag: \"xx\""));
    }

    #[test]
    fn an_unknown_key_warns_and_is_ignored() {
        let compiled = compile_with(strings(
            &[
                ("greeting", "Olá %1$s"),
                ("title", "Título"),
                ("extra", "Extra"),
            ],
            &[
                ("days", &[("other", "%1$d dias")]),
                ("weeks", &[("other", "x")]),
            ],
        ));
        assert_eq!(compiled.warnings.len(), 2, "{:?}", compiled.warnings);
        assert!(compiled.warnings.iter().any(|w| w.contains("extra")));
        assert!(compiled.warnings.iter().any(|w| w.contains("weeks")));
        assert!(!compiled.code.contains("Extra") && !compiled.code.contains("\"extra\""));
    }

    #[test]
    fn mismatched_placeholders_warn_and_drop_the_text() {
        let compiled = compile_with(strings(
            &[("greeting", "Olá"), ("title", "Título")],
            &[("days", &[("other", "%1$s dias")])],
        ));
        assert_eq!(compiled.warnings.len(), 2, "{:?}", compiled.warnings);
        assert!(
            compiled.warnings[0].contains("greeting")
                && compiled.warnings[0].contains("placeholders")
        );
        assert!(compiled.warnings[1].contains("days"));
        assert!(
            !compiled.code.contains("\"Olá\""),
            "the broken text must not be emitted"
        );
    }

    #[test]
    fn reordered_placeholders_are_accepted() {
        let reference = Strings {
            strings: [("k".to_string(), "%1$s opened %2$d times".to_string())].into(),
            ..Strings::default()
        };
        let translation = strings(&[("k", "%2$d vezes: %1$s")], &[]);
        let compiled = compile(vec![(None, reference), (Some("xx".into()), translation)]).unwrap();
        assert!(compiled.warnings.is_empty(), "{:?}", compiled.warnings);
    }

    #[test]
    fn plurals_need_the_other_form_and_known_quantities() {
        let compiled = compile_with(strings(
            &[("greeting", "Olá %1$s"), ("title", "Título")],
            &[("days", &[("one", "%1$d dia")])],
        ));
        assert!(
            compiled
                .warnings
                .iter()
                .any(|w| w.contains("days") && w.contains("other")),
            "{:?}",
            compiled.warnings
        );
        let compiled = compile_with(strings(
            &[("greeting", "Olá %1$s"), ("title", "Título")],
            &[("days", &[("some", "x"), ("other", "%1$d dias")])],
        ));
        assert!(compiled.warnings.iter().any(|w| w.contains("some")));
    }

    #[test]
    fn plural_forms_may_omit_the_number() {
        let compiled = compile_with(strings(
            &[("greeting", "Olá %1$s"), ("title", "Título")],
            &[("days", &[("one", "um dia"), ("other", "%1$d dias")])],
        ));
        assert!(compiled.warnings.is_empty(), "{:?}", compiled.warnings);
    }

    #[test]
    fn untranslatable_strings_are_copied_from_the_reference() {
        let compiled = compile_with(strings(
            &[
                ("app_name", "other name"),
                ("greeting", "Olá %1$s"),
                ("title", "Título"),
            ],
            &[("days", &[("other", "%1$d dias")])],
        ));
        assert_eq!(compiled.warnings.len(), 1);
        assert!(compiled.warnings[0].contains("app_name"));
        assert_eq!(
            compiled.code.matches("Some(\"lume\")").count(),
            2,
            "both languages use 'lume'"
        );
    }

    #[test]
    fn english_values_override_the_default_directory() {
        let overlay = strings(&[("title", "English title")], &[]);
        let compiled = compile(vec![(None, reference()), (Some("en".into()), overlay)]).unwrap();
        assert!(compiled.code.contains("Some(\"English title\")"));
        assert_eq!(compiled.languages, ["en"]);
    }

    #[test]
    fn languages_are_sorted_with_english_first() {
        let compiled = compile(vec![
            (Some("pt-BR".into()), strings(&[], &[])),
            (None, reference()),
            (Some("es".into()), strings(&[], &[])),
        ])
        .unwrap();
        assert_eq!(compiled.languages, ["en", "es", "pt-BR"]);
    }

    #[test]
    fn the_reference_must_exist_and_be_sound() {
        assert!(compile(vec![(Some("pt".into()), strings(&[], &[]))]).is_err());
        assert!(compile(vec![(None, strings(&[("Bad-Key", "x")], &[]))]).is_err());
        assert!(compile(vec![(None, strings(&[], &[("p", &[("one", "x")])]))]).is_err());
        assert!(compile(vec![(
            None,
            strings(&[("a", "x")], &[("a", &[("other", "x")])])
        )])
        .is_err());
    }

    #[test]
    fn generated_code_has_the_enums_and_tables() {
        let compiled = compile_with(strings(&[], &[]));
        let code = &compiled.code;
        assert!(code.contains("pub enum StringId {\n    AppName,\n    Greeting,\n    Title,\n}"));
        assert!(code.contains("pub enum PluralId {\n    Days,\n}"));
        assert!(code.contains("pub const COUNT: usize = 3;"));
        assert!(code.contains("StringId::Greeting => \"greeting\""));
        assert!(code.contains("pub static LANGUAGES: [LanguageTable; 2]"));
        assert!(code.contains("Some(\"Hello %1$s\")"));
        assert!(code
            .contains("Some([None, Some(\"%1$d day\"), None, None, None, Some(\"%1$d days\")])"));
    }

    #[test]
    fn special_characters_are_escaped_in_the_generated_literals() {
        let r = strings(&[("k", "quote \" backslash \\ newline \n é")], &[]);
        let compiled = compile(vec![(None, r)]).unwrap();
        assert!(
            compiled
                .code
                .contains(r#"Some("quote \" backslash \\ newline \n é")"#),
            "{}",
            compiled.code
        );
    }
}
