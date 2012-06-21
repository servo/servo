export GlyphIndex, GlyphPos, Glyph;

import gfx::geometry::au;
import geom::point::Point2D;

#[doc = "The index of a particular glyph within a font"]
type GlyphIndex = uint;

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
    let index: GlyphIndex;
    let pos: GlyphPos;

    new(index: GlyphIndex, pos: GlyphPos) {
        self.index = index;
        self.pos = copy pos;
    }
}
