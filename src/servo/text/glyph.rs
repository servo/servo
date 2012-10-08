use au = gfx::geometry;
use au::au;
use core::cmp::{Ord, Eq};
use core::dvec::DVec;
use geom::point::Point2D;
use std::sort;
use servo_util::vec::*;

export GlyphIndex, GlyphPos, Glyph;

struct CompressedGlyph {
    mut value : u32
}

/// The index of a particular glyph within a font
type GlyphIndex = u32;

const BREAK_TYPE_NONE   : u8 = 0x0u8;
const BREAK_TYPE_NORMAL : u8 = 0x1u8;
const BREAK_TYPE_HYPEN  : u8 = 0x2u8;

// TODO: make this more type-safe.

const FLAG_CHAR_IS_SPACE : u32             = 0x10000000u32;
// These two bits store some BREAK_TYPE_* flags
const FLAG_CAN_BREAK_MASK : u32            = 0x60000000u32;
const FLAG_CAN_BREAK_SHIFT : u32           = 29;
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
            true  => Simple(num::from_int(((self.value & GLYPH_ADVANCE_MASK) >> GLYPH_ADVANCE_SHIFT) as int)),
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

struct DetailedGlyph {
    // The GlyphIndex, or the unicode codepoint if glyph is missing.
    index: u32,
    // glyph's advance, in the text's direction (RTL or RTL)
    advance: au,
    // glyph's offset from the font's em-box (from top-left)
    offset: Point2D<au>
}

// cg = CompressedGlyph,
// dg = DetailedGlyph

struct DetailedGlyphRecord {
    // source character/CompressedGlyph offset in the TextRun
    cg_offset: u32,
    // offset into the detailed glyphs buffer
    dg_offset: uint
}

impl DetailedGlyphRecord : Ord {
    pure fn lt(other: &DetailedGlyphRecord) -> bool { self.cg_offset <  other.cg_offset }
    pure fn le(other: &DetailedGlyphRecord) -> bool { self.cg_offset <= other.cg_offset }
    pure fn ge(other: &DetailedGlyphRecord) -> bool { self.cg_offset >= other.cg_offset }
    pure fn gt(other: &DetailedGlyphRecord) -> bool { self.cg_offset >  other.cg_offset }
}

impl DetailedGlyphRecord : Eq {
    pure fn eq(other : &DetailedGlyphRecord) -> bool { self.cg_offset == other.cg_offset }
    pure fn ne(other : &DetailedGlyphRecord) -> bool { self.cg_offset != other.cg_offset }
}

// Manages the lookup table for detailed glyphs. 
struct DetailedGlyphStore {
    dg_buffer: DVec<DetailedGlyph>,
    dg_lookup: DVec<DetailedGlyphRecord>,
    mut lookup_is_sorted: bool,
}

fn DetailedGlyphStore() -> DetailedGlyphStore {
    DetailedGlyphStore {
        dg_buffer: DVec(),
        dg_lookup: DVec(),
        lookup_is_sorted: false
    }
}

impl DetailedGlyphStore {
    fn add_glyphs_for_cg(cg_offset: u32, glyphs: &[DetailedGlyph]) {
        let entry = DetailedGlyphRecord {
            cg_offset: cg_offset,
            dg_offset: self.dg_buffer.len()
        };

        /*
        TODO: don't actually assert this until asserts are compiled
        in/out based on severity, debug/release, etc.

        See Rust Issue #3647, #2228, #3627 for related information.

        do self.dg_lookup.borrow |arr| {
            assert !arr.contains(entry)
        }
        */

        self.dg_lookup.push(entry);
        self.dg_buffer.push_all(glyphs);
        self.lookup_is_sorted = false;
    }

    // not pure; may perform a deferred sort.
    fn get_glyphs_for_cg(&self, cg_offset: u32, count: uint) -> &[DetailedGlyph] {
        assert count > 0 && count < self.dg_buffer.len();
        self.ensure_sorted();

        let key = DetailedGlyphRecord {
            cg_offset: cg_offset,
            dg_offset: 0 // unused
        };

        do self.dg_lookup.borrow |records : &[DetailedGlyphRecord]| {
            match records.binary_search_index(&key) {
                None => fail ~"Invalid index not found in detailed glyph lookup table!",
                Some(i) => {
                    do self.dg_buffer.borrow |glyphs : &[DetailedGlyph]| {
                        assert i + count < glyphs.len();
                        // return a view into the buffer
                        vec::view(glyphs, i, count)
                    }
                }
            }
        }
    }

    /*priv*/ fn ensure_sorted() {
        if self.lookup_is_sorted {
            return;
        }

        do self.dg_lookup.borrow_mut |arr| {
            sort::quick_sort3(arr);
        };
        self.lookup_is_sorted = true;
    }
}

// Public data structure and API for storing glyph data
struct GlyphStore {
    mut cg_buffer: ~[CompressedGlyph],
    dg_store: DetailedGlyphStore,
}

// Initializes the glyph store, but doesn't actually shape anything.
// Use the set_glyph, set_glyphs() methods to store glyph data.
// Use the get_glyph_data method to retrieve glyph data for a char..
fn GlyphStore(_text: ~str) {

}

impl GlyphStore {
    
}

// XXX: legacy glyphs below. rip out once converted to new glyphs

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
