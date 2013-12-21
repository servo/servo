/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use servo_util::vec::*;
use servo_util::range::Range;
use servo_util::geometry::Au;
use servo_util::geometry;

use std::cmp::{Ord, Eq};
use std::num::NumCast;
use std::u16;
use std::vec;
use std::util;
use std::iter;
use geom::point::Point2D;

/// GlyphEntry is a port of Gecko's CompressedGlyph scheme for storing glyph data compactly.
///
/// In the common case (reasonable glyph advances, no offsets from the font em-box, and one glyph
/// per character), we pack glyph advance, glyph id, and some flags into a single u32.
///
/// In the uncommon case (multiple glyphs per unicode character, large glyph index/advance, or
/// glyph offsets), we pack the glyph count into GlyphEntry, and store the other glyph information
/// in DetailedGlyphStore.
#[deriving(Clone)]
struct GlyphEntry {
    value: u32
}

impl GlyphEntry {
    fn new(value: u32) -> GlyphEntry {
        GlyphEntry {
            value: value
        }
    }

    fn initial() -> GlyphEntry {
        GlyphEntry::new(0)
    }

    // Creates a GlyphEntry for the common case
    fn simple(index: GlyphIndex, advance: Au) -> GlyphEntry {
        assert!(is_simple_glyph_id(index));
        assert!(is_simple_advance(advance));

        let index_mask = index as u32;
        let advance_mask = (*advance as u32) << GLYPH_ADVANCE_SHIFT;

        GlyphEntry::new(index_mask | advance_mask | FLAG_IS_SIMPLE_GLYPH)
    }

    // Create a GlyphEntry for uncommon case; should be accompanied by
    // initialization of the actual DetailedGlyph data in DetailedGlyphStore
    fn complex(starts_cluster: bool, starts_ligature: bool, glyph_count: uint) -> GlyphEntry {
        assert!(glyph_count <= u16::max_value as uint);

        debug!("creating complex glyph entry: starts_cluster={}, starts_ligature={}, \
                glyph_count={}",
               starts_cluster,
               starts_ligature,
               glyph_count);

        let mut val = FLAG_NOT_MISSING;

        if !starts_cluster {
            val |= FLAG_NOT_CLUSTER_START;
        }
        if !starts_ligature {
            val |= FLAG_NOT_LIGATURE_GROUP_START;
        }
        val |= (glyph_count as u32) << GLYPH_COUNT_SHIFT;

        GlyphEntry::new(val)
    }

    /// Create a GlyphEntry for the case where glyphs couldn't be found for the specified
    /// character.
    fn missing(glyph_count: uint) -> GlyphEntry {
        assert!(glyph_count <= u16::max_value as uint);

        GlyphEntry::new((glyph_count as u32) << GLYPH_COUNT_SHIFT)
    }
}

/// The index of a particular glyph within a font
pub type GlyphIndex = u32;

// TODO: unify with bit flags?
#[deriving(Eq)]
pub enum BreakType {
    BreakTypeNone,
    BreakTypeNormal,
    BreakTypeHyphen
}

static BREAK_TYPE_NONE:   u8 = 0x0;
static BREAK_TYPE_NORMAL: u8 = 0x1;
static BREAK_TYPE_HYPHEN: u8 = 0x2;

fn break_flag_to_enum(flag: u8) -> BreakType {
    if (flag & BREAK_TYPE_NORMAL) != 0 {
        return BreakTypeNormal;
    }
    if (flag & BREAK_TYPE_HYPHEN) != 0 {
        return BreakTypeHyphen;
    }
    BreakTypeNone
}

fn break_enum_to_flag(e: BreakType) -> u8 {
    match e {
        BreakTypeNone   => BREAK_TYPE_NONE,
        BreakTypeNormal => BREAK_TYPE_NORMAL,
        BreakTypeHyphen => BREAK_TYPE_HYPHEN,
    }
}

// TODO: make this more type-safe.

static FLAG_CHAR_IS_SPACE: u32      = 0x10000000;
// These two bits store some BREAK_TYPE_* flags
static FLAG_CAN_BREAK_MASK: u32     = 0x60000000;
static FLAG_CAN_BREAK_SHIFT: u32    = 29;
static FLAG_IS_SIMPLE_GLYPH: u32    = 0x80000000;

// glyph advance; in Au's.
static GLYPH_ADVANCE_MASK: u32      = 0x0FFF0000;
static GLYPH_ADVANCE_SHIFT: u32     = 16;
static GLYPH_ID_MASK: u32           = 0x0000FFFF;

// Non-simple glyphs (more than one glyph per char; missing glyph,
// newline, tab, large advance, or nonzero x/y offsets) may have one
// or more detailed glyphs associated with them. They are stored in a
// side array so that there is a 1:1 mapping of GlyphEntry to
// unicode char.

// The number of detailed glyphs for this char. If the char couldn't
// be mapped to a glyph (!FLAG_NOT_MISSING), then this actually holds
// the UTF8 code point instead.
static GLYPH_COUNT_MASK:              u32 = 0x00FFFF00;
static GLYPH_COUNT_SHIFT:             u32 = 8;
// N.B. following Gecko, these are all inverted so that a lot of
// missing chars can be memset with zeros in one fell swoop.
static FLAG_NOT_MISSING:              u32 = 0x00000001;
static FLAG_NOT_CLUSTER_START:        u32 = 0x00000002;
static FLAG_NOT_LIGATURE_GROUP_START: u32 = 0x00000004;
 
static FLAG_CHAR_IS_TAB:              u32 = 0x00000008;
static FLAG_CHAR_IS_NEWLINE:          u32 = 0x00000010;
//static FLAG_CHAR_IS_LOW_SURROGATE:    u32 = 0x00000020;
//static CHAR_IDENTITY_FLAGS_MASK:      u32 = 0x00000038;

fn is_simple_glyph_id(glyphId: GlyphIndex) -> bool {
    ((glyphId as u32) & GLYPH_ID_MASK) == glyphId
}

fn is_simple_advance(advance: Au) -> bool {
    let unsignedAu = advance.to_u32().unwrap();
    (unsignedAu & (GLYPH_ADVANCE_MASK >> GLYPH_ADVANCE_SHIFT)) == unsignedAu
}

type DetailedGlyphCount = u16;

// Getters and setters for GlyphEntry. Setter methods are functional,
// because GlyphEntry is immutable and only a u32 in size.
impl GlyphEntry {
    // getter methods
    #[inline(always)]
    fn advance(&self) -> Au {
        NumCast::from((self.value & GLYPH_ADVANCE_MASK) >> GLYPH_ADVANCE_SHIFT).unwrap()
    }

    fn index(&self) -> GlyphIndex {
        self.value & GLYPH_ID_MASK
    }

    fn is_ligature_start(&self) -> bool {
        self.has_flag(!FLAG_NOT_LIGATURE_GROUP_START)
    }

    fn is_cluster_start(&self) -> bool {
        self.has_flag(!FLAG_NOT_CLUSTER_START)
    }
    
    // True if original char was normal (U+0020) space. Other chars may
    // map to space glyph, but this does not account for them.
    fn char_is_space(&self) -> bool {
        self.has_flag(FLAG_CHAR_IS_SPACE)
    }

    fn char_is_tab(&self) -> bool {
        !self.is_simple() && self.has_flag(FLAG_CHAR_IS_TAB)
    }

    fn char_is_newline(&self) -> bool {
        !self.is_simple() && self.has_flag(FLAG_CHAR_IS_NEWLINE)
    }

    fn can_break_before(&self) -> BreakType {
        let flag = ((self.value & FLAG_CAN_BREAK_MASK) >> FLAG_CAN_BREAK_SHIFT) as u8;
        break_flag_to_enum(flag)
    }

    // setter methods
    #[inline(always)]
    fn set_char_is_space(&self) -> GlyphEntry {
        GlyphEntry::new(self.value | FLAG_CHAR_IS_SPACE)
    }

    #[inline(always)]
    fn set_char_is_tab(&self) -> GlyphEntry {
        assert!(!self.is_simple());
        GlyphEntry::new(self.value | FLAG_CHAR_IS_TAB)
    }

    #[inline(always)]
    fn set_char_is_newline(&self) -> GlyphEntry {
        assert!(!self.is_simple());
        GlyphEntry::new(self.value | FLAG_CHAR_IS_NEWLINE)
    }

    #[inline(always)]
    fn set_can_break_before(&self, e: BreakType) -> GlyphEntry {
        let flag = (break_enum_to_flag(e) as u32) << FLAG_CAN_BREAK_SHIFT;
        GlyphEntry::new(self.value | flag)
    }

    // helper methods

    fn glyph_count(&self) -> u16 {
        assert!(!self.is_simple());
        ((self.value & GLYPH_COUNT_MASK) >> GLYPH_COUNT_SHIFT) as u16
    }

    #[inline(always)]
    fn is_simple(&self) -> bool {
        self.has_flag(FLAG_IS_SIMPLE_GLYPH)
    }

    #[inline(always)]
    fn has_flag(&self, flag: u32) -> bool {
        (self.value & flag) != 0
    }

    #[inline(always)]
    fn adapt_character_flags_of_entry(&self, other: GlyphEntry) -> GlyphEntry {
        GlyphEntry { value: self.value | other.value }
    }
}

// Stores data for a detailed glyph, in the case that several glyphs
// correspond to one character, or the glyph's data couldn't be packed.
#[deriving(Clone)]
struct DetailedGlyph {
    index: GlyphIndex,
    // glyph's advance, in the text's direction (RTL or RTL)
    advance: Au,
    // glyph's offset from the font's em-box (from top-left)
    offset: Point2D<Au>
}

impl DetailedGlyph {
    fn new(index: GlyphIndex, advance: Au, offset: Point2D<Au>) -> DetailedGlyph {
        DetailedGlyph {
            index: index,
            advance: advance,
            offset: offset
        }
    }
}

#[deriving(Eq, Clone)]
struct DetailedGlyphRecord {
    // source string offset/GlyphEntry offset in the TextRun
    entry_offset: uint,
    // offset into the detailed glyphs buffer
    detail_offset: uint
}

impl Ord for DetailedGlyphRecord {
    fn lt(&self, other: &DetailedGlyphRecord) -> bool {
		self.entry_offset <  other.entry_offset
	}
    fn le(&self, other: &DetailedGlyphRecord) -> bool {
		self.entry_offset <= other.entry_offset
	}
    fn ge(&self, other: &DetailedGlyphRecord) -> bool {
		self.entry_offset >= other.entry_offset
	}
    fn gt(&self, other: &DetailedGlyphRecord) -> bool {
		self.entry_offset >  other.entry_offset
	}
}

// Manages the lookup table for detailed glyphs. Sorting is deferred
// until a lookup is actually performed; this matches the expected
// usage pattern of setting/appending all the detailed glyphs, and
// then querying without setting.
struct DetailedGlyphStore {
    // TODO(pcwalton): Allocation of this buffer is expensive. Consider a small-vector
    // optimization.
    detail_buffer: ~[DetailedGlyph],
    // TODO(pcwalton): Allocation of this buffer is expensive. Consider a small-vector
    // optimization.
    detail_lookup: ~[DetailedGlyphRecord],
    lookup_is_sorted: bool,
}

impl<'a> DetailedGlyphStore {
    fn new() -> DetailedGlyphStore {
        DetailedGlyphStore {
            detail_buffer: ~[], // TODO: default size?
            detail_lookup: ~[],
            lookup_is_sorted: false
        }
    }

    fn add_detailed_glyphs_for_entry(&mut self, entry_offset: uint, glyphs: &[DetailedGlyph]) {
        let entry = DetailedGlyphRecord {
            entry_offset: entry_offset,
            detail_offset: self.detail_buffer.len()
        };

        debug!("Adding entry[off={:u}] for detailed glyphs: {:?}", entry_offset, glyphs);

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

    fn get_detailed_glyphs_for_entry(&'a self, entry_offset: uint, count: u16)
                                  -> &'a [DetailedGlyph] {
        debug!("Requesting detailed glyphs[n={:u}] for entry[off={:u}]", count as uint, entry_offset);

        // FIXME: Is this right? --pcwalton
        // TODO: should fix this somewhere else
        if count == 0 {
            return self.detail_buffer.slice(0, 0);
        }

        assert!((count as uint) <= self.detail_buffer.len());
        assert!(self.lookup_is_sorted);

        let key = DetailedGlyphRecord {
            entry_offset: entry_offset,
            detail_offset: 0 // unused
        };

        // FIXME: This is a workaround for borrow of self.detail_lookup not getting inferred.
        let records : &[DetailedGlyphRecord] = self.detail_lookup;
        match records.binary_search_index(&key) {
            None => fail!(~"Invalid index not found in detailed glyph lookup table!"),
            Some(i) => {
                assert!(i + (count as uint) <= self.detail_buffer.len());
                // return a slice into the buffer
                self.detail_buffer.slice(i, i + count as uint)
            }
        }
    }

    fn get_detailed_glyph_with_index(&'a self,
                                     entry_offset: uint,
                                     detail_offset: u16)
            -> &'a DetailedGlyph {
        assert!((detail_offset as uint) <= self.detail_buffer.len());
        assert!(self.lookup_is_sorted);

        let key = DetailedGlyphRecord {
            entry_offset: entry_offset,
            detail_offset: 0 // unused
        };

        // FIXME: This is a workaround for borrow of self.detail_lookup not getting inferred.
        let records: &[DetailedGlyphRecord] = self.detail_lookup;
        match records.binary_search_index(&key) {
            None => fail!(~"Invalid index not found in detailed glyph lookup table!"),
            Some(i) => {
                assert!(i + (detail_offset as uint)  < self.detail_buffer.len());
                &self.detail_buffer[i+(detail_offset as uint)]
            }
        }
    }

    fn ensure_sorted(&mut self) {
        if self.lookup_is_sorted {
            return;
        }

        // Sorting a unique vector is surprisingly hard. The follwing
        // code is a good argument for using DVecs, but they require
        // immutable locations thus don't play well with freezing.

        // Thar be dragons here. You have been warned. (Tips accepted.)
        let mut unsorted_records: ~[DetailedGlyphRecord] = ~[];
        util::swap(&mut self.detail_lookup, &mut unsorted_records);
        let mut mut_records : ~[DetailedGlyphRecord] = unsorted_records;
        mut_records.sort_by(|a, b| {
            if a < b {
                Less
            } else {
                Greater
            }
        });
        let mut sorted_records = mut_records;
        util::swap(&mut self.detail_lookup, &mut sorted_records);

        self.lookup_is_sorted = true;
    }
}

// This struct is used by GlyphStore clients to provide new glyph data.
// It should be allocated on the stack and passed by reference to GlyphStore.
pub struct GlyphData {
    index: GlyphIndex,
    advance: Au,
    offset: Point2D<Au>,
    is_missing: bool,
    cluster_start: bool,
    ligature_start: bool,
}

impl GlyphData {
    pub fn new(index: GlyphIndex,
               advance: Au,
               offset: Option<Point2D<Au>>,
               is_missing: bool,
               cluster_start: bool,
               ligature_start: bool)
            -> GlyphData {
        let offset = match offset {
            None => geometry::zero_point(),
            Some(o) => o
        };

        GlyphData {
            index: index,
            advance: advance,
            offset: offset,
            is_missing: is_missing,
            cluster_start: cluster_start,
            ligature_start: ligature_start,
        }
    }
}

// This enum is a proxy that's provided to GlyphStore clients when iterating
// through glyphs (either for a particular TextRun offset, or all glyphs).
// Rather than eagerly assembling and copying glyph data, it only retrieves
// values as they are needed from the GlyphStore, using provided offsets.
enum GlyphInfo<'a> {
    SimpleGlyphInfo(&'a GlyphStore, uint),
    DetailGlyphInfo(&'a GlyphStore, uint, u16)
}

impl<'a> GlyphInfo<'a> {
    pub fn index(self) -> GlyphIndex {
        match self {
            SimpleGlyphInfo(store, entry_i) => store.entry_buffer[entry_i].index(),
            DetailGlyphInfo(store, entry_i, detail_j) => {
                store.detail_store.get_detailed_glyph_with_index(entry_i, detail_j).index
            }
        }
    }

    #[inline(always)]
    // FIXME: Resolution conflicts with IteratorUtil trait so adding trailing _
    pub fn advance(self) -> Au {
        match self {
            SimpleGlyphInfo(store, entry_i) => store.entry_buffer[entry_i].advance(),
            DetailGlyphInfo(store, entry_i, detail_j) => {
                store.detail_store.get_detailed_glyph_with_index(entry_i, detail_j).advance
            }
        }
    }

    pub fn offset(self) -> Option<Point2D<Au>> {
        match self {
            SimpleGlyphInfo(_, _) => None,
            DetailGlyphInfo(store, entry_i, detail_j) => {
                Some(store.detail_store.get_detailed_glyph_with_index(entry_i, detail_j).offset)
            }
        }
    }
}

// Public data structure and API for storing and retrieving glyph data
pub struct GlyphStore {
    // TODO(pcwalton): Allocation of this buffer is expensive. Consider a small-vector
    // optimization.
    entry_buffer: ~[GlyphEntry],

    detail_store: DetailedGlyphStore,

    is_whitespace: bool,
}

impl<'a> GlyphStore {
    // Initializes the glyph store, but doesn't actually shape anything.
    // Use the set_glyph, set_glyphs() methods to store glyph data.
    pub fn new(length: uint, is_whitespace: bool) -> GlyphStore {
        assert!(length > 0);

        GlyphStore {
            entry_buffer: vec::from_elem(length, GlyphEntry::initial()),
            detail_store: DetailedGlyphStore::new(),
            is_whitespace: is_whitespace,
        }
    }

    pub fn char_len(&self) -> uint {
        self.entry_buffer.len()
    }

    pub fn is_whitespace(&self) -> bool {
        self.is_whitespace
    }

    pub fn finalize_changes(&mut self) {
        self.detail_store.ensure_sorted();
    }

    pub fn add_glyph_for_char_index(&mut self, i: uint, data: &GlyphData) {
        fn glyph_is_compressible(data: &GlyphData) -> bool {
            is_simple_glyph_id(data.index)
                && is_simple_advance(data.advance)
                && data.offset == geometry::zero_point()
                && data.cluster_start  // others are stored in detail buffer
        }

        assert!(data.ligature_start); // can't compress ligature continuation glyphs.
        assert!(i < self.entry_buffer.len());

        let entry = match (data.is_missing, glyph_is_compressible(data)) {
            (true, _) => GlyphEntry::missing(1),
            (false, true) => GlyphEntry::simple(data.index, data.advance),
            (false, false) => {
                let glyph = [DetailedGlyph::new(data.index, data.advance, data.offset)];
                self.detail_store.add_detailed_glyphs_for_entry(i, glyph);
                GlyphEntry::complex(data.cluster_start, data.ligature_start, 1)
            }
        }.adapt_character_flags_of_entry(self.entry_buffer[i]);

        self.entry_buffer[i] = entry;
    }

    pub fn add_glyphs_for_char_index(&mut self, i: uint, data_for_glyphs: &[GlyphData]) {
        assert!(i < self.entry_buffer.len());
        assert!(data_for_glyphs.len() > 0);

        let glyph_count = data_for_glyphs.len();

        let first_glyph_data = data_for_glyphs[0];
        let entry = match first_glyph_data.is_missing {
            true  => GlyphEntry::missing(glyph_count),
            false => {
                let glyphs_vec = vec::from_fn(glyph_count, |i| {
                    DetailedGlyph::new(data_for_glyphs[i].index,
                                       data_for_glyphs[i].advance,
                                       data_for_glyphs[i].offset)
                });

                self.detail_store.add_detailed_glyphs_for_entry(i, glyphs_vec);
                GlyphEntry::complex(first_glyph_data.cluster_start,
                                    first_glyph_data.ligature_start,
                                    glyph_count)
            }
        }.adapt_character_flags_of_entry(self.entry_buffer[i]);

        debug!("Adding multiple glyphs[idx={:u}, count={:u}]: {:?}", i, glyph_count, entry);

        self.entry_buffer[i] = entry;
    }

    // used when a character index has no associated glyph---for example, a ligature continuation.
    pub fn add_nonglyph_for_char_index(&mut self, i: uint, cluster_start: bool, ligature_start: bool) {
        assert!(i < self.entry_buffer.len());

        let entry = GlyphEntry::complex(cluster_start, ligature_start, 0);
        debug!("adding spacer for chracter without associated glyph[idx={:u}]", i);

        self.entry_buffer[i] = entry;
    }

    pub fn iter_glyphs_for_char_index(&'a self, i: uint) -> GlyphIterator<'a> {
        self.iter_glyphs_for_char_range(&Range::new(i, 1))
    }

    #[inline]
    pub fn iter_glyphs_for_char_range(&'a self, rang: &Range) -> GlyphIterator<'a> {
        if rang.begin() >= self.entry_buffer.len() {
            fail!("iter_glyphs_for_range: range.begin beyond length!");
        }
        if rang.end() > self.entry_buffer.len() {
            fail!("iter_glyphs_for_range: range.end beyond length!");
        }

        GlyphIterator {
            store:       self,
            char_index:  rang.begin(),
            char_range:  rang.eachi(),
            glyph_range: None
        }
    }

    // getter methods
    pub fn char_is_space(&self, i: uint) -> bool {
        assert!(i < self.entry_buffer.len());
        self.entry_buffer[i].char_is_space()
    }

    pub fn char_is_tab(&self, i: uint) -> bool {
        assert!(i < self.entry_buffer.len());
        self.entry_buffer[i].char_is_tab()
    }

    pub fn char_is_newline(&self, i: uint) -> bool {
        assert!(i < self.entry_buffer.len());
        self.entry_buffer[i].char_is_newline()
    }

    pub fn is_ligature_start(&self, i: uint) -> bool {
        assert!(i < self.entry_buffer.len());
        self.entry_buffer[i].is_ligature_start()
    }

    pub fn is_cluster_start(&self, i: uint) -> bool {
        assert!(i < self.entry_buffer.len());
        self.entry_buffer[i].is_cluster_start()
    }

    pub fn can_break_before(&self, i: uint) -> BreakType {
        assert!(i < self.entry_buffer.len());
        self.entry_buffer[i].can_break_before()
    }

    // setter methods
    pub fn set_char_is_space(&mut self, i: uint) {
        assert!(i < self.entry_buffer.len());
        let entry = self.entry_buffer[i];
        self.entry_buffer[i] = entry.set_char_is_space();
    }

    pub fn set_char_is_tab(&mut self, i: uint) {
        assert!(i < self.entry_buffer.len());
        let entry = self.entry_buffer[i];
        self.entry_buffer[i] = entry.set_char_is_tab();
    }

    pub fn set_char_is_newline(&mut self, i: uint) {
        assert!(i < self.entry_buffer.len());
        let entry = self.entry_buffer[i];
        self.entry_buffer[i] = entry.set_char_is_newline();
    }

    pub fn set_can_break_before(&mut self, i: uint, t: BreakType) {
        assert!(i < self.entry_buffer.len());
        let entry = self.entry_buffer[i];
        self.entry_buffer[i] = entry.set_can_break_before(t);
    }
}

pub struct GlyphIterator<'a> {
    priv store:       &'a GlyphStore,
    priv char_index:  uint,
    priv char_range:  iter::Range<uint>,
    priv glyph_range: Option<iter::Range<uint>>,
}

impl<'a> GlyphIterator<'a> {
    // Slow path when there is a glyph range.
    #[inline(never)]
    fn next_glyph_range(&mut self) -> Option<(uint, GlyphInfo<'a>)> {
        match self.glyph_range.get_mut_ref().next() {
            Some(j) => Some((self.char_index,
                DetailGlyphInfo(self.store, self.char_index, j as u16))),
            None => {
                // No more glyphs for current character.  Try to get another.
                self.glyph_range = None;
                self.next()
            }
        }
    }

    // Slow path when there is a complex glyph.
    #[inline(never)]
    fn next_complex_glyph(&mut self, entry: &GlyphEntry, i: uint)
                          -> Option<(uint, GlyphInfo<'a>)> {
        let glyphs = self.store.detail_store.get_detailed_glyphs_for_entry(i, entry.glyph_count());
        self.glyph_range = Some(range(0, glyphs.len()));
        self.next()
    }
}

impl<'a> Iterator<(uint, GlyphInfo<'a>)> for GlyphIterator<'a> {
    // I tried to start with something simpler and apply FlatMap, but the
    // inability to store free variables in the FlatMap struct was problematic.
    //
    // This function consists of the fast path and is designed to be inlined into its caller. The
    // slow paths, which should not be inlined, are `next_glyph_range()` and
    // `next_complex_glyph()`.
    #[inline(always)]
    fn next(&mut self) -> Option<(uint, GlyphInfo<'a>)> {
        // Would use 'match' here but it borrows contents in a way that
        // interferes with mutation.
        if self.glyph_range.is_some() {
            self.next_glyph_range()
        } else {
            // No glyph range.  Look at next character.
            match self.char_range.next() {
                Some(i) => {
                    self.char_index = i;
                    assert!(i < self.store.entry_buffer.len());
                    let entry = &self.store.entry_buffer[i];
                    if entry.is_simple() {
                        Some((self.char_index, SimpleGlyphInfo(self.store, i)))
                    } else {
                        // Fall back to the slow path.
                        self.next_complex_glyph(entry, i)
                    }
                },
                None => None
            }
        }
    }
}
