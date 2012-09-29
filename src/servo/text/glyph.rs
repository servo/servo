export GlyphIndex, GlyphPos, Glyph;

use gfx::geometry::au;
use geom::point::Point2D;

/** The index of a particular glyph within a font */
type GlyphIndex = uint;

/** The position of a glyph on the screen. */
struct GlyphPos {
    advance: Point2D<au>,
    offset: Point2D<au>,
}

fn GlyphPos(advance: Point2D<au>, offset: Point2D<au>) -> GlyphPos {
    GlyphPos {
        advance : advance,
        offset : offset,
    }
}

/** A single glyph. */
struct Glyph {
    index: GlyphIndex,
    pos: GlyphPos,
}

fn Glyph(index: GlyphIndex, pos: GlyphPos) -> Glyph {
    Glyph {
        index : index,
        pos : copy pos,
    }
}
