//! Compiles Android `strings.xml` translations into static Rust tables.
//!
//! Used by the `build.rs` of `lume-core`, so the app reads its texts from tables
//! with no XML parsing or JNI calls at runtime.

mod android;
mod compile;
mod xml;

pub use android::{directory_language, load_directory, parse_strings_xml, unescape, Strings};
pub use compile::{compile, placeholders, Compiled, QUANTITIES};

use std::path::Path;

/// Reads every `values*` directory under `res_dir` and compiles them.
/// `Err` means the reference language is unusable; problems in translations are warnings.
pub fn compile_directory(res_dir: &Path) -> Result<Compiled, String> {
    let mut warnings = Vec::new();
    let languages = load_directory(res_dir, &mut warnings)?;
    let mut compiled = compile(languages)?;
    warnings.append(&mut compiled.warnings);
    compiled.warnings = warnings;
    Ok(compiled)
}
