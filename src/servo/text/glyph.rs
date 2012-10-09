use au = gfx::geometry;
use au::au;
use core::cmp::{Ord, Eq};
use core::dvec::DVec;
use core::u16;
use geom::point::Point2D;
use std::sort;
use servo_util::vec::*;
use num::from_int;

// GlyphEntry is a port of Gecko's CompressedGlyph scheme for storing
// glyph data compactly.
//
// In the common case (reasonable glyph advances, no offsets from the
// font em-box, and one glyph per character), we pack glyph advance,
// glyph id, and some flags into a single u32.
//
// In the uncommon case (multiple glyphs per unicode character, large
// glyph index/advance, or glyph offsets), we pack the glyph count
// into GlyphEntry, and store the other glyph information in
// DetailedGlyphStore.
struct GlyphEntry {
    value : u32
}

pure fn GlyphEntry(value: u32) -> GlyphEntry { GlyphEntry { value: value } }

/// The index of a particular glyph within a font
type GlyphIndex = u32;

// TODO: unify with bit flags?
enum BreakType {
    BreakTypeNone,
    BreakTypeNormal,
    BreakTypeHyphen
}

const BREAK_TYPE_NONE   : u8 = 0x0u8;
const BREAK_TYPE_NORMAL : u8 = 0x1u8;
const BREAK_TYPE_HYPHEN  : u8 = 0x2u8;

pure fn break_flag_to_enum(flag: u8) -> BreakType {
    if (flag & BREAK_TYPE_NONE) as bool   { return BreakTypeNone; }
    if (flag & BREAK_TYPE_NORMAL) as bool { return BreakTypeNormal; }
    if (flag & BREAK_TYPE_HYPHEN) as bool { return  BreakTypeHyphen; }
    fail ~"Unknown break setting"
}

pure fn break_enum_to_flag(e: BreakType) -> u8 {
    match e {
        BreakTypeNone => BREAK_TYPE_NONE,
        BreakTypeNormal => BREAK_TYPE_NORMAL,
        BreakTypeHyphen => BREAK_TYPE_HYPHEN,
    }
}

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
// side array so that there is a 1:1 mapping of GlyphEntry to
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

pure fn InitialGlyphEntry() -> GlyphEntry {
    GlyphEntry { value: 0 }
}

// Creates a GlyphEntry for the common case
pure fn SimpleGlyphEntry(index: GlyphIndex, advance: au) -> GlyphEntry {
    assert is_simple_glyph_id(index);
    assert is_simple_advance(advance);

    let index_mask = index as u32;
    let advance_mask = (*advance as u32) << GLYPH_ADVANCE_SHIFT;

    GlyphEntry {
        value: index_mask | advance_mask | FLAG_IS_SIMPLE_GLYPH
    }
}

// Create a GlyphEntry for uncommon case; should be accompanied by
// initialization of the actual DetailedGlyph data in DetailedGlyphStore
pure fn ComplexGlyphEntry(startsCluster: bool, startsLigature: bool, glyphCount: uint) -> GlyphEntry {
    assert glyphCount <= u16::max_value as uint;

    let mut val = FLAG_NOT_MISSING;

    if !startsCluster {
        val |= FLAG_NOT_CLUSTER_START;
    }
    if !startsLigature {
        val |= FLAG_NOT_LIGATURE_GROUP_START;
    }
    val |= (glyphCount as u32) << GLYPH_COUNT_SHIFT;

    GlyphEntry {
        value: val
    }
}

// Create a GlyphEntry for the case where glyphs couldn't be found
// for the specified character.
pure fn MissingGlyphsEntry(glyphCount: uint) -> GlyphEntry {
    assert glyphCount <= u16::max_value as uint;

    GlyphEntry {
        value: (glyphCount as u32) << GLYPH_COUNT_SHIFT
    }
}

// Getters and setters for GlyphEntry. Setter methods are functional,
// because GlyphEntry is immutable and only a u32 in size.
impl GlyphEntry {
    // getter methods
    pure fn advance() -> au {
        assert self.is_simple();
        from_int(((self.value & GLYPH_ADVANCE_MASK) >> GLYPH_ADVANCE_SHIFT) as int)
    }

    pure fn index() -> GlyphIndex {
        assert self.is_simple();
        self.value & GLYPH_ID_MASK
    }

    pure fn offset() -> Point2D<au> {
        assert self.is_simple();
        Point2D(au(0), au(0))
    }
    
    pure fn is_ligature_start() -> bool {
        self.has_flag(!FLAG_NOT_LIGATURE_GROUP_START)
    }

    pure fn is_cluster_start() -> bool {
        self.has_flag(!FLAG_NOT_CLUSTER_START)
    }
    
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

    pure fn can_break_before() -> BreakType {
        let flag = ((self.value & FLAG_CAN_BREAK_MASK) >> FLAG_CAN_BREAK_SHIFT) as u8;
        break_flag_to_enum(flag)
    }

    // setter methods
    pure fn set_char_is_space() -> GlyphEntry {
        GlyphEntry(self.value | FLAG_CHAR_IS_SPACE)
    }

    pure fn set_char_is_tab() -> GlyphEntry {
        assert !self.is_simple();
        GlyphEntry(self.value | FLAG_CHAR_IS_TAB)
    }

    pure fn set_char_is_newline() -> GlyphEntry {
        assert !self.is_simple();
        GlyphEntry(self.value | FLAG_CHAR_IS_NEWLINE)
    }

    // returns a glyph entry only if the setting had changed.
    pure fn set_can_break_before(e: BreakType) -> Option<GlyphEntry> {
        let flag = break_enum_to_flag(e);
        let mask = (flag as u32) << FLAG_CAN_BREAK_SHIFT;
        let toggle = mask ^ (self.value & FLAG_CAN_BREAK_MASK);

        match (toggle as bool) {
            true  => Some(GlyphEntry(self.value ^ toggle)),
            false => None
        }
    }

    // helper methods

    /*priv*/ pure fn glyph_count() -> u16 {
        assert !self.is_simple();
        ((self.value & GLYPH_COUNT_MASK) >> GLYPH_COUNT_SHIFT) as u16
    }

    pure fn is_simple() -> bool {
        self.has_flag(FLAG_IS_SIMPLE_GLYPH)
    }

    /*priv*/ pure fn has_flag(flag: u32) -> bool {
        (self.value & flag) != 0
    }
}

// Stores data for a detailed glyph, in the case that several glyphs
// correspond to one character, or the glyph's data couldn't be packed.
struct DetailedGlyph {
    index: GlyphIndex,
    // glyph's advance, in the text's direction (RTL or RTL)
    advance: au,
    // glyph's offset from the font's em-box (from top-left)
    offset: Point2D<au>
}


fn DetailedGlyph(index: GlyphIndex,
                 advance: au, offset: Point2D<au>) -> DetailedGlyph {
    DetailedGlyph {
        index: index,
        advance: advance,
        offset: offset
    }
}

struct DetailedGlyphRecord {
    // source string offset/GlyphEntry offset in the TextRun
    entry_offset: uint,
    // offset into the detailed glyphs buffer
    detail_offset: uint
}

impl DetailedGlyphRecord : Ord {
    pure fn lt(other: &DetailedGlyphRecord) -> bool { self.entry_offset <  other.entry_offset }
    pure fn le(other: &DetailedGlyphRecord) -> bool { self.entry_offset <= other.entry_offset }
    pure fn ge(other: &DetailedGlyphRecord) -> bool { self.entry_offset >= other.entry_offset }
    pure fn gt(other: &DetailedGlyphRecord) -> bool { self.entry_offset >  other.entry_offset }
}

impl DetailedGlyphRecord : Eq {
    pure fn eq(other : &DetailedGlyphRecord) -> bool { self.entry_offset == other.entry_offset }
    pure fn ne(other : &DetailedGlyphRecord) -> bool { self.entry_offset != other.entry_offset }
}

// Manages the lookup table for detailed glyphs. Sorting is deferred
// until a lookup is actually performed; this matches the expected
// usage pattern of setting/appending all the detailed glyphs, and
// then querying without setting.
struct DetailedGlyphStore {
    detail_buffer: DVec<DetailedGlyph>,
    detail_lookup: DVec<DetailedGlyphRecord>,
    mut lookup_is_sorted: bool,
}

fn DetailedGlyphStore() -> DetailedGlyphStore {
    DetailedGlyphStore {
        detail_buffer: DVec(),
        detail_lookup: DVec(),
        lookup_is_sorted: false
    }
}

impl DetailedGlyphStore {
    fn add_detailed_glyphs_for_entry(entry_offset: uint, glyphs: &[DetailedGlyph]) {
        let entry = DetailedGlyphRecord {
            entry_offset: entry_offset,
            detail_offset: self.detail_buffer.len()
        };

        /* TODO: don't actually assert this until asserts are compiled
        in/out based on severity, debug/release, etc. This assertion
        would wreck the complexity of the lookup.

        See Rust Issue #3647, #2228, #3627 for related information.

        do self.detail_lookup.borrow |arr| {
            assert !arr.contains(entry)
        }
        */

        self.detail_lookup.push(entry);
        self.detail_buffer.push_all(glyphs);
        self.lookup_is_sorted = false;
    }

    // not pure; may perform a deferred sort.
    fn get_detailed_glyphs_for_entry(&self, entry_offset: uint, count: u16) -> &[DetailedGlyph] {
        assert count > 0;
        assert (count as uint) <= self.detail_buffer.len();
        self.ensure_sorted();

        let key = DetailedGlyphRecord {
            entry_offset: entry_offset,
            detail_offset: 0 // unused
        };

        do self.detail_lookup.borrow |records : &[DetailedGlyphRecord]| {
            match records.binary_search_index(&key) {
                None => fail ~"Invalid index not found in detailed glyph lookup table!",
                Some(i) => {
                    do self.detail_buffer.borrow |glyphs : &[DetailedGlyph]| {
                        assert i + (count as uint)  < glyphs.len();
                        // return a view into the buffer
                        vec::view(glyphs, i, count as uint)
                    }
                }
            }
        }
    }

    fn get_detailed_glyph_with_index(&self, entry_offset: uint, detail_offset: u16) -> &DetailedGlyph {
        assert (detail_offset as uint) <= self.detail_buffer.len();
        self.ensure_sorted();

        let key = DetailedGlyphRecord {
            entry_offset: entry_offset,
            detail_offset: 0 // unused
        };

        do self.detail_lookup.borrow |records : &[DetailedGlyphRecord]| {
            match records.binary_search_index(&key) {
                None => fail ~"Invalid index not found in detailed glyph lookup table!",
                Some(i) => {
                    do self.detail_buffer.borrow |glyphs : &[DetailedGlyph]| {
                        assert i + (detail_offset as uint)  < glyphs.len();
                        &glyphs[i+(detail_offset as uint)]
                    }
                }
            }
        }
    }

    /*priv*/ fn ensure_sorted() {
        if self.lookup_is_sorted {
            return;
        }

        do self.detail_lookup.borrow_mut |arr| {
            sort::quick_sort3(arr);
        };
        self.lookup_is_sorted = true;
    }
}

// This struct is used by GlyphStore clients to provide new glyph data.
// It should be allocated on the stack and passed by reference to GlyphStore.
struct GlyphData {
    index: GlyphIndex,
    advance: au,
    offset: Point2D<au>,
    is_missing: bool,
    cluster_start: bool,
    ligature_start: bool,
}

pure fn GlyphData(index: GlyphIndex, 
                   advance: au,
                   offset: Option<Point2D<au>>,
                   is_missing: bool,
                   cluster_start: bool,
                   ligature_start: bool) -> GlyphData {
    
    let _offset = match offset {
        None => au::zero_point(),
        Some(o) => o
    };

    GlyphData {
        index: index,
        advance: advance,
        offset: _offset,
        is_missing: is_missing,
        cluster_start: cluster_start,
        ligature_start: ligature_start,
    }
}

// This enum is a proxy that's provided to GlyphStore clients when iterating
// through glyphs (either for a particular TextRun offset, or all glyphs).
// Rather than eagerly assembling and copying glyph data, it only retrieves
// values as they are needed from the GlyphStore, using provided offsets.
enum GlyphInfo {
    SimpleGlyphInfo(&GlyphStore, uint),
    DetailGlyphInfo(&GlyphStore, uint, u16)
}

impl GlyphInfo {
    fn index() -> GlyphIndex {
        match self {
            SimpleGlyphInfo(store, entry_i) => store.entry_buffer[entry_i].index(),
            DetailGlyphInfo(store, entry_i, detail_j) => store.detail_store.get_detailed_glyph_with_index(entry_i, detail_j).index
        }
    }

    fn advance() -> au {
        match self {
            SimpleGlyphInfo(store, entry_i) => store.entry_buffer[entry_i].advance(),
            DetailGlyphInfo(store, entry_i, detail_j) => store.detail_store.get_detailed_glyph_with_index(entry_i, detail_j).advance
        }
    }

    fn offset() -> Option<Point2D<au>> {
        match self {
            SimpleGlyphInfo(_, _) => None,
            DetailGlyphInfo(store, entry_i, detail_j) => Some(store.detail_store.get_detailed_glyph_with_index(entry_i, detail_j).offset)
        }
    }

    fn is_ligature_start() -> bool {
        match self {
            SimpleGlyphInfo(store, entry_i) => store.entry_buffer[entry_i].is_ligature_start(),
            DetailGlyphInfo(store, entry_i, _) => store.entry_buffer[entry_i].is_ligature_start()
        }
    }

    fn is_cluster_start() -> bool {
        match self {
            SimpleGlyphInfo(store, entry_i) => store.entry_buffer[entry_i].is_cluster_start(),
            DetailGlyphInfo(store, entry_i, _) => store.entry_buffer[entry_i].is_cluster_start()
        }
    }
}

// Public data structure and API for storing and retrieving glyph data
struct GlyphStore {
    // we use a DVec here instead of a mut vec, since this is much safer.
    entry_buffer: DVec<GlyphEntry>,
    detail_store: DetailedGlyphStore,
}

// Initializes the glyph store, but doesn't actually shape anything.
// Use the set_glyph, set_glyphs() methods to store glyph data.
fn GlyphStore(text: &str) -> GlyphStore {
    assert text.len() > 0;

    let buffer = vec::from_elem(text.len(), InitialGlyphEntry());

    GlyphStore {
        entry_buffer: dvec::from_vec(buffer),
        detail_store: DetailedGlyphStore(),
    }
}

impl GlyphStore {
    fn add_glyph_for_index(i: uint, data: &GlyphData) {

        pure fn glyph_is_compressible(data: &GlyphData) -> bool {
            is_simple_glyph_id(data.index)
                && is_simple_advance(data.advance)
                && data.offset == au::zero_point()
        }

        assert i < self.entry_buffer.len();

        let entry = match (data.is_missing, glyph_is_compressible(data)) {
            (true, _) => MissingGlyphsEntry(1),
            (false, true) => { SimpleGlyphEntry(data.index, data.advance) },
            (false, false) => {
                let glyph = [DetailedGlyph(data.index, data.advance, data.offset)];
                self.detail_store.add_detailed_glyphs_for_entry(i, glyph);
                ComplexGlyphEntry(data.cluster_start, data.ligature_start, 1)
            }
        };

        self.entry_buffer.set_elt(i, entry);
    }

    fn add_glyphs_for_index(i: uint, data_for_glyphs: &[GlyphData]) {
        assert i < self.entry_buffer.len();
        assert data_for_glyphs.len() > 0;

        let glyph_count = data_for_glyphs.len();

        let first_glyph_data = data_for_glyphs[0];
        let entry = match first_glyph_data.is_missing {
            true  => MissingGlyphsEntry(glyph_count),
            false => {
                let glyphs_vec = vec::from_fn(glyph_count, |i| {
                    DetailedGlyph(data_for_glyphs[i].index,
                                  data_for_glyphs[i].advance,
                                  data_for_glyphs[i].offset)
                });

                self.detail_store.add_detailed_glyphs_for_entry(i, glyphs_vec);
                ComplexGlyphEntry(first_glyph_data.cluster_start,
                                  first_glyph_data.ligature_start,
                                  glyph_count)
            }
        };

        self.entry_buffer.set_elt(i, entry);
    }

    fn iter_glyphs_for_index<T>(&self, i: uint, cb: fn&(uint, GlyphInfo/&) -> T) {
        assert i < self.entry_buffer.len();

        let entry = &self.entry_buffer[i];
        match entry.is_simple() {
            true => { 
                let proxy = SimpleGlyphInfo(self, i);
                cb(i, proxy);
            },
            false => {
                let glyphs = self.detail_store.get_detailed_glyphs_for_entry(i, entry.glyph_count());
                for uint::range(0, glyphs.len()) |j| {
                    let proxy = DetailGlyphInfo(self, i, j as u16);
                    cb(i, proxy);
                }
            }
        }
    }

    fn iter_glyphs_for_range<T>(&self, offset: uint, len: uint, cb: fn&(uint, GlyphInfo/&) -> T) {
        assert offset < self.entry_buffer.len();
        assert len > 0 && len + offset <= self.entry_buffer.len();

        for uint::range(offset, offset + len) |i| {
            self.iter_glyphs_for_index(i, cb);
        }
    }

    fn iter_all_glyphs<T>(cb: fn&(uint, GlyphInfo/&) -> T) {
        for uint::range(0, self.entry_buffer.len()) |i| {
            self.iter_glyphs_for_index(i, cb);
        }
    }

    // getter methods
    fn char_is_space(i: uint) -> bool {
        assert i < self.entry_buffer.len();
        self.entry_buffer[i].char_is_space()
    }

    fn char_is_tab(i: uint) -> bool {
        assert i < self.entry_buffer.len();
        self.entry_buffer[i].char_is_tab()
    }

    fn char_is_newline(i: uint) -> bool {
        assert i < self.entry_buffer.len();
        self.entry_buffer[i].char_is_newline()
    }

    fn is_ligature_start(i: uint) -> bool {
        assert i < self.entry_buffer.len();
        self.entry_buffer[i].is_ligature_start()
    }

    fn is_cluster_start(i: uint) -> bool {
        assert i < self.entry_buffer.len();
        self.entry_buffer[i].is_cluster_start()
    }

    fn can_break_before(i: uint) -> BreakType {
        assert i < self.entry_buffer.len();
        self.entry_buffer[i].can_break_before()
    }

    // setter methods
    fn set_char_is_space(i: uint) {
        assert i < self.entry_buffer.len();
        let entry = self.entry_buffer[i];
        self.entry_buffer.set_elt(i, entry.set_char_is_space())
    }

    fn set_char_is_tab(i: uint) {
        assert i < self.entry_buffer.len();
        let entry = self.entry_buffer[i];
        self.entry_buffer.set_elt(i, entry.set_char_is_tab())
    }

    fn set_char_is_newline(i: uint) {
        assert i < self.entry_buffer.len();
        let entry = self.entry_buffer[i];
        self.entry_buffer.set_elt(i, entry.set_char_is_newline())
    }

    fn set_can_break_before(i: uint, t: BreakType) {
        assert i < self.entry_buffer.len();
        let entry = self.entry_buffer[i];
        match entry.set_can_break_before(t) {
            Some(e) => self.entry_buffer.set_elt(i, e),
            None => {}
        };
    }
}
