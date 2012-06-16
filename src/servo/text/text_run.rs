import libc::{c_void};
import font::font;
import glyph::glyph;
import shaper::shape_text;

#[doc="A single, unbroken line of text."]
class text_run {
    let glyphs: [glyph];

    new(font: &font, text: str) {
        self.glyphs = shape_text(font, text);
    }
}

