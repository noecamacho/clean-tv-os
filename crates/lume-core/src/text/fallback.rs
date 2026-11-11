//! Fonts used for characters the embedded faces lack.

use fontdue::Font;

/// A source of extra fonts, such as the system fonts on Android.
pub trait FallbackFonts {
    /// Index of a font that has a glyph for `c`, loading it if needed.
    fn find(&mut self, c: char) -> Option<usize>;

    /// The font at `index` (as returned by [`FallbackFonts::find`]).
    fn font(&self, index: usize) -> &Font;

    /// Drops every loaded font to give memory back.
    fn release(&mut self);
}

/// No fallback: missing characters show the embedded font's missing-glyph shape.
pub struct NoFallback;

impl FallbackFonts for NoFallback {
    fn find(&mut self, _c: char) -> Option<usize> {
        None
    }

    fn font(&self, _index: usize) -> &Font {
        unreachable!("NoFallback never returns an index")
    }

    fn release(&mut self) {}
}
