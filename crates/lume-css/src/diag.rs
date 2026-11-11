//! Diagnostics (warnings and errors) emitted while parsing a theme.

use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Severity {
    /// Something was ignored, but the theme is still usable.
    Warning,
    /// The theme is invalid; the build must fail.
    Error,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Diagnostic {
    pub severity: Severity,
    pub file: String,
    /// Line (starting at 1) the diagnostic refers to.
    pub line: u32,
    pub message: String,
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let kind = match self.severity {
            Severity::Warning => "warning",
            Severity::Error => "error",
        };
        write!(f, "{}:{}: {}: {}", self.file, self.line, kind, self.message)
    }
}
