import gfx::geom::{au, point};

#[doc="The position of a glyph on the screen."]
class glyph_pos {
    let advance: point<au>;
    let offset: point<au>;
    new(advance: point<au>, offset: point<au>) {
        self.advance = advance;
        self.offset = offset;
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

