import gfx::geometry::au;
import geom::point::Point2D;

#[doc="The position of a glyph on the screen."]
class glyph_pos {
    let advance: Point2D<au>;
    let offset: Point2D<au>;
    new(advance: Point2D<au>, offset: Point2D<au>) {
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

