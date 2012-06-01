import libc::{c_void};
import text::glyph::glyph;
import shaper::shape_text;

#[doc="A single, unbroken line of text."]
class text_run {
    let text: str;
    let mut glyphs: option<[glyph]>;

    new(text: str) {
        self.text = text;
        self.glyphs = none;
    }

    #[doc="
        Shapes text. This determines the location of each glyph and determines
        line break positions.
    "]
    fn shape() {
        let font = font::create();
        self.glyphs = some(shape_text(&font, self.text));
    }
}

