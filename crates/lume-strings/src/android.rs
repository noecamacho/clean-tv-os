//! Android resource conventions: `strings.xml` contents and `values-<qualifier>` directories.

use crate::xml::{self, Element};
use std::collections::BTreeMap;
use std::path::Path;

/// The texts of one language (one `values*` directory).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Strings {
    pub strings: BTreeMap<String, String>,
    /// Plural name -> quantity (`one`, `other`, ...) -> text.
    pub plurals: BTreeMap<String, BTreeMap<String, String>>,
    /// Names marked `translatable="false"`.
    pub untranslatable: Vec<String>,
}

/// Parses the text of a `strings.xml` file. Problems that do not stop the parsing
/// (such as a duplicated name) are added to `warnings`.
pub fn parse_strings_xml(
    source: &str,
    file: &str,
    warnings: &mut Vec<String>,
) -> Result<Strings, String> {
    let root = xml::parse(source).map_err(|e| format!("{file}: {e}"))?;
    if root.name != "resources" {
        return Err(format!(
            "{file}: the root element must be <resources>, found <{}>",
            root.name
        ));
    }
    let mut result = Strings::default();
    for element in root.elements() {
        let Some(name) = element.attribute("name") else {
            warnings.push(format!(
                "{file}:{}: <{}> without a name; ignored",
                element.line, element.name
            ));
            continue;
        };
        match element.name.as_str() {
            "string" => {
                let text = unescape(&element.text())
                    .map_err(|e| format!("{file}:{}: {name}: {e}", element.line))?;
                if result.strings.insert(name.to_string(), text).is_some() {
                    warnings.push(format!(
                        "{file}:{}: duplicate string '{name}'; the last one wins",
                        element.line
                    ));
                }
                if element.attribute("translatable") == Some("false") {
                    result.untranslatable.push(name.to_string());
                }
            }
            "plurals" => {
                let items = plural_items(element, file, name)?;
                if result.plurals.insert(name.to_string(), items).is_some() {
                    warnings.push(format!(
                        "{file}:{}: duplicate plurals '{name}'; the last one wins",
                        element.line
                    ));
                }
                if element.attribute("translatable") == Some("false") {
                    result.untranslatable.push(name.to_string());
                }
            }
            "string-array" | "integer" | "bool" | "color" | "dimen" | "item" => {
                warnings.push(format!(
                    "{file}:{}: <{}> '{name}' is not supported; ignored",
                    element.line, element.name
                ));
            }
            other => warnings.push(format!(
                "{file}:{}: unknown element <{other}>; ignored",
                element.line
            )),
        }
    }
    Ok(result)
}

fn plural_items(
    element: &Element,
    file: &str,
    name: &str,
) -> Result<BTreeMap<String, String>, String> {
    let mut items = BTreeMap::new();
    for item in element.elements() {
        if item.name != "item" {
            return Err(format!(
                "{file}:{}: <plurals name=\"{name}\"> may only contain <item>",
                item.line
            ));
        }
        let quantity = item.attribute("quantity").ok_or_else(|| {
            format!(
                "{file}:{}: <item> in plurals '{name}' needs a quantity",
                item.line
            )
        })?;
        let text =
            unescape(&item.text()).map_err(|e| format!("{file}:{}: {name}: {e}", item.line))?;
        items.insert(quantity.to_string(), text);
    }
    Ok(items)
}

/// Applies Android's text rules: surrounding double quotes keep whitespace as is, other
/// whitespace collapses, and backslash escapes are decoded.
pub fn unescape(text: &str) -> Result<String, String> {
    let mut out = String::new();
    let mut in_quotes = false;
    let mut pending_space = false;
    let mut chars = text.trim().chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '"' => in_quotes = !in_quotes,
            '\\' => {
                if pending_space {
                    out.push(' ');
                    pending_space = false;
                }
                match chars.next() {
                    Some('n') => out.push('\n'),
                    Some('t') => out.push('\t'),
                    Some(c @ ('\'' | '"' | '\\' | '@' | '?')) => out.push(c),
                    Some('u') => {
                        let hex: String = (0..4).filter_map(|_| chars.next()).collect();
                        let code = u32::from_str_radix(&hex, 16)
                            .ok()
                            .filter(|_| hex.len() == 4)
                            .ok_or_else(|| format!("invalid \\u escape '\\u{hex}'"))?;
                        out.push(
                            char::from_u32(code)
                                .ok_or_else(|| format!("invalid \\u escape '\\u{hex}'"))?,
                        );
                    }
                    Some(other) => return Err(format!("unknown escape '\\{other}'")),
                    None => return Err("a backslash ends the text".into()),
                }
            }
            c if c.is_whitespace() && !in_quotes => pending_space = true,
            c => {
                if pending_space {
                    out.push(' ');
                    pending_space = false;
                }
                out.push(c);
            }
        }
    }
    if in_quotes {
        return Err("unbalanced double quote".into());
    }
    Ok(out)
}

/// Language tag of a resource directory: `values` is the default language (`None`),
/// `values-pt-rBR` is `pt-BR`, `values-b+sr+Latn` is `sr-Latn`.
/// Directories with other qualifiers (`values-night`, `values-v21`, ...) are not translations.
pub fn directory_language(directory: &str) -> Option<Option<String>> {
    if directory == "values" {
        return Some(None);
    }
    let qualifier = directory.strip_prefix("values-")?;
    if let Some(bcp47) = qualifier.strip_prefix("b+") {
        let parts: Vec<&str> = bcp47.split('+').collect();
        return parts
            .iter()
            .all(|p| p.chars().all(|c| c.is_ascii_alphanumeric()))
            .then(|| Some(parts.join("-")));
    }
    let mut parts = qualifier.split('-');
    let language = parts.next()?;
    if !(2..=3).contains(&language.len()) || !language.chars().all(|c| c.is_ascii_lowercase()) {
        return None;
    }
    match parts.next() {
        None => Some(Some(language.to_string())),
        Some(region) => {
            let region = region.strip_prefix('r')?;
            let is_region = region.len() == 2 && region.chars().all(|c| c.is_ascii_uppercase());
            (is_region && parts.next().is_none()).then(|| Some(format!("{language}-{region}")))
        }
    }
}

/// Every language found under `res_dir`: `(language tag or None for the default, texts)`.
/// Directories are read in name order so the result is deterministic.
pub fn load_directory(
    res_dir: &Path,
    warnings: &mut Vec<String>,
) -> Result<Vec<(Option<String>, Strings)>, String> {
    let mut names: Vec<String> = std::fs::read_dir(res_dir)
        .map_err(|e| format!("cannot read {}: {e}", res_dir.display()))?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().join("strings.xml").is_file())
        .filter_map(|e| e.file_name().into_string().ok())
        .collect();
    names.sort();
    let mut languages = Vec::new();
    for name in names {
        let Some(language) = directory_language(&name) else {
            continue;
        };
        let path = res_dir.join(&name).join("strings.xml");
        let source = std::fs::read_to_string(&path)
            .map_err(|e| format!("cannot read {}: {e}", path.display()))?;
        let shown = format!("{name}/strings.xml");
        languages.push((language, parse_strings_xml(&source, &shown, warnings)?));
    }
    Ok(languages)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(source: &str) -> (Strings, Vec<String>) {
        let mut warnings = Vec::new();
        (
            parse_strings_xml(source, "t.xml", &mut warnings).unwrap(),
            warnings,
        )
    }

    #[test]
    fn reads_strings_and_plurals() {
        let (s, warnings) = parse(
            r#"<resources>
              <string name="a">Hello %1$s</string>
              <string name="b" translatable="false">lume</string>
              <plurals name="p"><item quantity="one">%1$d day</item><item quantity="other">%1$d days</item></plurals>
            </resources>"#,
        );
        assert!(warnings.is_empty());
        assert_eq!(s.strings["a"], "Hello %1$s");
        assert_eq!(s.untranslatable, ["b"]);
        assert_eq!(s.plurals["p"]["one"], "%1$d day");
        assert_eq!(s.plurals["p"]["other"], "%1$d days");
    }

    #[test]
    fn android_text_rules() {
        assert_eq!(
            unescape("  two   words \n here ").unwrap(),
            "two words here"
        );
        assert_eq!(unescape(r"don\'t").unwrap(), "don't");
        assert_eq!(
            unescape(r#""  keep   spaces ""#).unwrap(),
            "  keep   spaces "
        );
        assert_eq!(
            unescape(r"line\nbreak é \@x \\").unwrap(),
            "line\nbreak é @x \\"
        );
        assert!(unescape(r"bad \q").is_err());
        assert!(unescape(r#"open "quote"#).is_err());
        assert!(unescape("trailing \\").is_err());
        assert!(unescape(r"\u12").is_err());
    }

    #[test]
    fn warns_about_duplicates_and_unsupported_elements() {
        let (s, warnings) = parse(
            r#"<resources>
              <string name="a">one</string>
              <string name="a">two</string>
              <string-array name="arr"><item>x</item></string-array>
              <string>no name</string>
            </resources>"#,
        );
        assert_eq!(s.strings["a"], "two");
        assert_eq!(warnings.len(), 3, "{warnings:?}");
    }

    #[test]
    fn rejects_other_roots_and_bad_plurals() {
        let mut w = Vec::new();
        assert!(parse_strings_xml("<other/>", "t.xml", &mut w).is_err());
        assert!(parse_strings_xml(
            "<resources><plurals name=\"p\"><x/></plurals></resources>",
            "t.xml",
            &mut w
        )
        .is_err());
        assert!(parse_strings_xml(
            "<resources><plurals name=\"p\"><item>no quantity</item></plurals></resources>",
            "t.xml",
            &mut w
        )
        .is_err());
    }

    #[test]
    fn directory_names_map_to_language_tags() {
        assert_eq!(directory_language("values"), Some(None));
        assert_eq!(directory_language("values-en"), Some(Some("en".into())));
        assert_eq!(
            directory_language("values-pt-rBR"),
            Some(Some("pt-BR".into()))
        );
        assert_eq!(
            directory_language("values-b+sr+Latn"),
            Some(Some("sr-Latn".into()))
        );
        assert_eq!(
            directory_language("values-b+zh+Hans+CN"),
            Some(Some("zh-Hans-CN".into()))
        );
        for other in [
            "values-night",
            "values-v21",
            "values-sw600dp",
            "values-land",
            "layout",
            "values-pt-BR",
            "values-en-rUSA",
            "values-pt-rBR-night",
        ] {
            assert_eq!(directory_language(other), None, "{other}");
        }
    }
}
