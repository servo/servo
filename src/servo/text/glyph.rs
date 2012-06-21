import gfx::geometry::au;
import geom::point::Point2D;

#[doc="The position of a glyph on the screen."]
class GlyphPos {
    let advance: Point2D<au>;
    let offset: Point2D<au>;
    new(advance: Point2D<au>, offset: Point2D<au>) {
        self.advance = advance;
        self.offset = offset;
    }
}

#[doc="A single glyph."]
class Glyph {
    let codepoint: uint;
    let pos: GlyphPos;

    new(codepoint: uint, pos: GlyphPos) {
        self.codepoint = codepoint;
        self.pos = copy pos;
    }
}
