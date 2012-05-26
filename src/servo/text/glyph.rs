import gfx::geom::{au, point, px_to_au};

#[doc="The position of a glyph on the screen."]
class glyph_pos {
    let advance: point<au>;
    let offset: point<au>;

    new(hb_pos: harfbuzz::hb_glyph_position_t) {
        self.advance = {
            mut x: px_to_au(hb_pos.x_advance as int),
            mut y: px_to_au(hb_pos.y_advance as int)
        };
        self.offset = {
            mut x: px_to_au(hb_pos.x_offset as int),
            mut y: px_to_au(hb_pos.y_offset as int)
        };
    }
}

#[doc="A single glyph."]
class glyph {
    let codepoint: uint;
    let pos: glyph_pos;

    new(codepoint: uint, pos: glyph_pos) {
        self.codepoint = codepoint;
        self.pos = copy pos;
    }
}

