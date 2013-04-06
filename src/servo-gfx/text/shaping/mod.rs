//! Shaper encapsulates a specific shaper, such as Harfbuzz, 
/// Uniscribe, Pango, or Coretext.
///
/// Currently, only harfbuzz bindings are implemented.

use text::glyph::GlyphStore;

pub use Shaper = text::shaping::harfbuzz::Shaper;

pub mod harfbuzz;

pub trait ShaperMethods {
    fn shape_text(&self, text: &str, glyphs: &mut GlyphStore);
}

