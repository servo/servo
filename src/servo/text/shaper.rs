import libc::types::common::c99::int32_t;
import font::font;
import glyph::{glyph, glyph_pos};

#[doc = "
Calculate the layout metrics associated with a some given text
when rendered in a specific font.
"]
fn shape_text(_font: &font, text: str) -> [glyph] {
    let mut glyphs = [];
    let mut cur_x = 0u;
    for text.each_char {
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

    ret glyphs;
}
