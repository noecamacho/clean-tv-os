//! Ensures the examples in `docs/theme-format.md` match the parser.

use lume_css::{parse, Severity};

fn blocks(doc: &str, lang: &str) -> Vec<(usize, String)> {
    let fence = format!("```{lang}");
    let mut out = Vec::new();
    let mut current: Option<(usize, String)> = None;
    for (i, line) in doc.lines().enumerate() {
        match &mut current {
            None if line.trim_end() == fence => current = Some((i + 2, String::new())),
            Some(_) if line.trim_end() == "```" => out.push(current.take().unwrap()),
            Some((_, text)) => {
                text.push_str(line);
                text.push('\n');
            }
            None => {}
        }
    }
    out
}

fn doc() -> String {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../docs/theme-format.md");
    std::fs::read_to_string(path).expect("docs/theme-format.md")
}

#[test]
fn docs_examples_parse_without_diagnostics() {
    let doc = doc();
    let blocks = blocks(&doc, "css");
    assert!(
        blocks.len() >= 5,
        "expected several examples, found {}",
        blocks.len()
    );
    for (line, css) in blocks {
        let p = parse(&css, "docs/theme-format.md");
        assert!(
            p.diagnostics.is_empty(),
            "exemplo na linha {line}: {:#?}",
            p.diagnostics
        );
    }
}

#[test]
fn docs_invalid_examples_do_produce_diagnostics() {
    let doc = doc();
    let blocks = blocks(&doc, "css-invalid");
    assert!(!blocks.is_empty());
    // Each line is an independent example; its comment says which severity to expect.
    for (line, css) in blocks {
        for (offset, example) in css.lines().filter(|l| !l.trim().is_empty()).enumerate() {
            let expected = if example.contains("/* error:") {
                Severity::Error
            } else {
                Severity::Warning
            };
            let p = parse(example, "doc");
            assert!(
                p.diagnostics.iter().any(|d| d.severity == expected),
                "exemplo '{example}' (bloco na linha {line}, item {offset}) deveria gerar {expected:?}: {:?}",
                p.diagnostics
            );
        }
    }
}
