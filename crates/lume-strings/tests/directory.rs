//! Compiles real directory layouts, like the Android `res/` folder.

use lume_strings::compile_directory;
use std::fs;
use std::path::PathBuf;

fn res_dir(name: &str, files: &[(&str, &str)]) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("lume-strings-{}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    for (path, content) in files {
        let path = dir.join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, content).unwrap();
    }
    dir
}

const REFERENCE: &str = r#"<resources>
    <string name="home_favorites_title">Your favorite apps</string>
    <string name="greeting">Hello %1$s</string>
    <plurals name="days"><item quantity="one">%1$d day</item><item quantity="other">%1$d days</item></plurals>
</resources>"#;

#[test]
fn a_new_language_needs_only_a_file() {
    let dir = res_dir(
        "new-language",
        &[
            ("values/strings.xml", REFERENCE),
            (
                "values-es/strings.xml",
                r#"<resources>
                    <string name="home_favorites_title">Tus apps favoritas</string>
                    <string name="greeting">Hola %1$s</string>
                    <plurals name="days"><item quantity="one">%1$d día</item><item quantity="other">%1$d días</item></plurals>
                </resources>"#,
            ),
        ],
    );
    let compiled = compile_directory(&dir).unwrap();
    assert!(compiled.warnings.is_empty(), "{:?}", compiled.warnings);
    assert_eq!(compiled.languages, ["en", "es"]);
    assert!(compiled.code.contains("Some(\"Tus apps favoritas\")"));
}

#[test]
fn a_translation_with_errors_produces_the_expected_warnings() {
    let dir = res_dir(
        "with-errors",
        &[
            ("values/strings.xml", REFERENCE),
            (
                "values-xx/strings.xml",
                // Missing 'home_favorites_title', unknown 'bogus', wrong placeholder in 'greeting'.
                r#"<resources>
                    <string name="greeting">Hola</string>
                    <string name="bogus">?</string>
                    <plurals name="days"><item quantity="other">%1$d días</item></plurals>
                </resources>"#,
            ),
        ],
    );
    let compiled = compile_directory(&dir).unwrap();
    let w = compiled.warnings.join("\n");
    assert_eq!(compiled.warnings.len(), 3, "{w}");
    assert!(
        w.contains("xx") && w.contains("home_favorites_title") && w.contains("missing"),
        "{w}"
    );
    assert!(w.contains("bogus") && w.contains("unknown"), "{w}");
    assert!(w.contains("greeting") && w.contains("placeholders"), "{w}");
}

#[test]
fn other_resource_qualifiers_are_not_translations() {
    let dir = res_dir(
        "qualifiers",
        &[
            ("values/strings.xml", REFERENCE),
            (
                "values-night/strings.xml",
                "<resources><string name=\"zzz\">x</string></resources>",
            ),
            ("values-v21/strings.xml", "<resources/>"),
            ("drawable/strings.xml", "not xml"),
        ],
    );
    let compiled = compile_directory(&dir).unwrap();
    assert_eq!(compiled.languages, ["en"]);
    assert!(compiled.warnings.is_empty());
}

#[test]
fn a_broken_xml_file_is_an_error_naming_the_file() {
    let dir = res_dir(
        "broken",
        &[
            ("values/strings.xml", REFERENCE),
            (
                "values-es/strings.xml",
                "<resources>\n<string name=\"a\">x\n</resources>",
            ),
        ],
    );
    let error = compile_directory(&dir)
        .err()
        .expect("a broken file is an error");
    assert!(
        error.contains("values-es/strings.xml") && error.contains("line"),
        "{error}"
    );
}

#[test]
fn the_real_project_strings_compile_without_warnings() {
    let dir =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../android/app/src/main/res");
    let compiled = compile_directory(&dir).unwrap();
    assert!(compiled.warnings.is_empty(), "{:#?}", compiled.warnings);
    assert_eq!(compiled.languages, ["en", "es", "id", "pt-BR", "ru"]);
}
