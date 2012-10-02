export GlyphIndex, GlyphPos, Glyph;

use au = gfx::geometry;
use au::au;
use geom::point::Point2D;

/** The index of a particular glyph within a font */
struct CompressedGlyph {
    mut value : u32
}
type GlyphIndex = u32;

enum BreakTypeFlags {
    BREAK_TYPE_NONE   = 0x0,
    BREAK_TYPE_NORMAL = 0x1,
    BREAK_TYPE_HYPEN  = 0x2,
}

// TODO: make this more type-safe.

const FLAG_CHAR_IS_SPACE : u32             = 0x10000000u32;
// These two bits store a BreakTypeFlags
const FLAG_CAN_BREAK_MASK : u32            = 0x60000000u32;
const FLAG_CAN_BREAK_SHIFT : u32           = 29u32;
const FLAG_IS_SIMPLE_GLYPH : u32           = 0x80000000u32;

// glyph advance; in au's.
const GLYPH_ADVANCE_MASK : u32             = 0x0FFF0000u32;
const GLYPH_ADVANCE_SHIFT : u32            = 16;
const GLYPH_ID_MASK : u32                  = 0x0000FFFFu32;

// Non-simple glyphs (more than one glyph per char; missing glyph,
// newline, tab, large advance, or nonzero x/y offsets) may have one
// or more detailed glyphs associated with them. They are stored in a
// side array so that there is a 1:1 mapping of CompressedGlyph to
// unicode char.

// The number of detailed glyphs for this char. If the char couldn't
// be mapped to a glyph (!FLAG_NOT_MISSING), then this actually holds
// the UTF8 code point instead.
const GLYPH_COUNT_MASK : u32               = 0x00FFFF00u32;
const GLYPH_COUNT_SHIFT : u32              = 8;
// N.B. following Gecko, these are all inverted so that a lot of
// missing chars can be memset with zeros in one fell swoop.
const FLAG_NOT_MISSING : u32               = 0x00000001u32;
const FLAG_NOT_CLUSTER_START : u32         = 0x00000002u32;
const FLAG_NOT_LIGATURE_GROUP_START : u32  = 0x00000004u32;
 
const FLAG_CHAR_IS_TAB : u32               = 0x00000008u32;
const FLAG_CHAR_IS_NEWLINE : u32           = 0x00000010u32;
const FLAG_CHAR_IS_LOW_SURROGATE : u32     = 0x00000020u32;
const CHAR_IDENTITY_FLAGS_MASK : u32       = 0x00000038u32;


pure fn is_simple_glyph_id(glyphId: GlyphIndex) -> bool {
    ((glyphId as u32) & GLYPH_ID_MASK) == glyphId
}

pure fn is_simple_advance(advance: au) -> bool {
    let unsignedAu = advance.to_int() as u32;
    (unsignedAu & (GLYPH_ADVANCE_MASK >> GLYPH_ADVANCE_SHIFT)) == unsignedAu
}

type DetailedGlyphCount = u16;

enum GlyphStoreResult<T> {
    Simple(T),
    Detailed(u32)
}

fn SimpleGlyph(index: GlyphIndex, advance: au) -> CompressedGlyph {
    assert is_simple_glyph_id(index);
    assert is_simple_advance(advance);

    let index_mask = index as u32;
    let advance_mask = (*advance as u32) << GLYPH_ADVANCE_SHIFT;

    CompressedGlyph {
        value: index_mask | advance_mask | FLAG_IS_SIMPLE_GLYPH
    }
}

fn ComplexGlyph(startsCluster: bool, startsLigature: bool, glyphCount: u16) -> CompressedGlyph {
    let mut val = FLAG_NOT_MISSING;

    if !startsCluster {
        val |= FLAG_NOT_CLUSTER_START;
    }
    if !startsLigature {
        val |= FLAG_NOT_LIGATURE_GROUP_START;
    }
    val |= (glyphCount as u32) << GLYPH_COUNT_SHIFT;

    CompressedGlyph {
        value: val
    }
}

fn MissingGlyphs(glyphCount: u16) -> CompressedGlyph {
    CompressedGlyph {
        value: (glyphCount as u32) << GLYPH_COUNT_SHIFT
    }
}

impl CompressedGlyph {
    pure fn advance() -> GlyphStoreResult<au> {
        match self.is_simple() {
            true  => Simple(au::from_int(((self.value & GLYPH_ADVANCE_MASK) >> GLYPH_ADVANCE_SHIFT) as int)),
            false => Detailed(self.glyph_count())
        }
    }

    pure fn glyph() -> GlyphStoreResult<GlyphIndex> {
        match self.is_simple() {
            true  => Simple(self.value & GLYPH_ID_MASK),
            false => Detailed(self.glyph_count())
        }
    }

    pure fn offset() -> GlyphStoreResult<Point2D<au>> {
        match self.is_simple() {
            true  => Simple(Point2D(au(0), au(0))),
            false => Detailed(self.glyph_count())
        }
    }
    
    // getter methods

    // TODO: some getters are still missing; add them as needed.
    
    // True if original char was normal (U+0020) space. Other chars may
    // map to space glyph, but this does not account for them.
    pure fn char_is_space() -> bool {
        self.has_flag(FLAG_CHAR_IS_SPACE)
    }

    pure fn char_is_tab() -> bool {
        !self.is_simple() && self.has_flag(FLAG_CHAR_IS_TAB)
    }

    pure fn char_is_newline() -> bool {
        !self.is_simple() && self.has_flag(FLAG_CHAR_IS_NEWLINE)
    }

    // TODO: make typesafe break enum
    pure fn can_break_before() -> u8 {
        ((self.value & FLAG_CAN_BREAK_MASK) >> FLAG_CAN_BREAK_SHIFT) as u8
    }

    // setter methods

    fn set_is_space() {
        self.value |= FLAG_CHAR_IS_SPACE;
    }

    fn set_is_tab() {
        assert !self.is_simple();
        self.value |= FLAG_CHAR_IS_TAB;
    }

    fn set_is_newline() {
        assert !self.is_simple();
        self.value |= FLAG_CHAR_IS_NEWLINE;
    }

    // returns whether the setting had changed.
    fn set_can_break_before(flags: u8) -> bool {
        assert flags <= 0x2;
        let mask = (flags as u32) << FLAG_CAN_BREAK_SHIFT;
        let toggle = mask ^ (self.value & FLAG_CAN_BREAK_MASK);
        self.value ^= toggle;

        toggle as bool
    }

    // helper methods

    /*priv*/ pure fn glyph_count() -> u32 {
        assert !self.is_simple();
        (self.value & GLYPH_COUNT_MASK) >> GLYPH_COUNT_SHIFT as u32
    }

    /*priv*/ pure fn is_simple() -> bool {
        (self.value & FLAG_IS_SIMPLE_GLYPH) == self.value
    }

    /*priv*/ pure fn has_flag(flag: u32) -> bool {
        (self.value & flag) != 0
    }
}

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
    index: u32,
    pos: GlyphPos,
}

fn Glyph(index: u32, pos: GlyphPos) -> Glyph {
    Glyph {
        index : index,
        pos : copy pos,
    }
}
