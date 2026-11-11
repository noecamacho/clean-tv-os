//! Parses a theme file and prints its diagnostics: `cargo run -p lume-css --example check -- <file>`.

fn main() {
    let path = std::env::args().nth(1).expect("usage: check <theme.css>");
    let source = std::fs::read_to_string(&path).expect("cannot read the theme file");
    let parsed = lume_css::parse(&source, &path);
    for d in &parsed.diagnostics {
        println!("{d}");
    }
    println!(
        "{} rules, {} variables, {} diagnostics",
        parsed.stylesheet.rules.len(),
        parsed.stylesheet.variables.len(),
        parsed.diagnostics.len()
    );
    if parsed.has_errors() {
        std::process::exit(1);
    }
}
