import libc::{c_void};
import libc::types::common::c99::int32_t;
import text::glyph::{glyph, glyph_pos};

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
        let mut glyphs = [];
        let mut cur_x = 0u;
        for self.text.each_char {
            |ch|
            // TODO: Use HarfBuzz!
            let hb_pos = {
                x_advance: 10 as int32_t,
                y_advance: 0 as int32_t,
                x_offset: cur_x as int32_t,
                y_offset: 0 as int32_t,
                var: 0
            };

            vec::push(glyphs, glyph(ch as uint, glyph_pos(hb_pos)));
            cur_x += 10u;
        };

        self.glyphs = some(/* move */ glyphs);
    }
}

