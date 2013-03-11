/**
Shaper encapsulates a specific shaper, such as Harfbuzz, 
Uniscribe, Pango, or Coretext.

Currently, only harfbuzz bindings are implemented.
*/
use gfx_font::Font;
use text::glyph::GlyphStore;
use text::harfbuzz;

pub type Shaper/& = harfbuzz::shaper::HarfbuzzShaper;

pub trait ShaperMethods {
    fn shape_text(&self, text: &str, glyphs: &mut GlyphStore);
}

// TODO(Issue #163): this is a workaround for static methods and
// typedefs not working well together. It should be removed.
pub impl Shaper {
    static pub fn new(font: @mut Font) -> Shaper {
        harfbuzz::shaper::HarfbuzzShaper::new(font)
    }
}
