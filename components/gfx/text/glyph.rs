/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use euclid::point::Point2D;
use std::cmp::{Ordering, PartialOrd};
use std::mem;
use std::u16;
use std::vec::Vec;
use util::geometry::Au;
use util::range::{self, Range, RangeIndex, EachIndex};
use util::vec::*;

/// GlyphEntry is a port of Gecko's CompressedGlyph scheme for storing glyph data compactly.
///
/// In the common case (reasonable glyph advances, no offsets from the font em-box, and one glyph
/// per character), we pack glyph advance, glyph id, and some flags into a single u32.
///
/// In the uncommon case (multiple glyphs per unicode character, large glyph index/advance, or
/// glyph offsets), we pack the glyph count into GlyphEntry, and store the other glyph information
/// in DetailedGlyphStore.
#[derive(Clone, Debug, Copy, Deserialize, Serialize)]
struct GlyphEntry {
    value: u32,
}

impl GlyphEntry {
    fn new(value: u32) -> GlyphEntry {
        GlyphEntry {
            value: value,
        }
    }

    fn initial() -> GlyphEntry {
        GlyphEntry::new(0)
    }

    // Creates a GlyphEntry for the common case
    fn simple(id: GlyphId, advance: Au) -> GlyphEntry {
        assert!(is_simple_glyph_id(id));
        assert!(is_simple_advance(advance));

        let id_mask = id as u32;
        let Au(advance) = advance;
        let advance_mask = (advance as u32) << GLYPH_ADVANCE_SHIFT;

        GlyphEntry::new(id_mask | advance_mask | FLAG_IS_SIMPLE_GLYPH)
    }

    // Create a GlyphEntry for uncommon case; should be accompanied by
    // initialization of the actual DetailedGlyph data in DetailedGlyphStore
    fn complex(starts_cluster: bool, starts_ligature: bool, glyph_count: usize) -> GlyphEntry {
        assert!(glyph_count <= u16::MAX as usize);

        debug!("creating complex glyph entry: starts_cluster={}, starts_ligature={}, \
                glyph_count={}",
               starts_cluster,
               starts_ligature,
               glyph_count);

        GlyphEntry::new(glyph_count as u32)
    }

    /// Create a GlyphEntry for the case where glyphs couldn't be found for the specified
    /// character.
    fn missing(glyph_count: usize) -> GlyphEntry {
        assert!(glyph_count <= u16::MAX as usize);

        GlyphEntry::new(glyph_count as u32)
    }
}

/// The id of a particular glyph within a font
pub type GlyphId = u32;

// TODO: make this more type-safe.

const FLAG_CHAR_IS_SPACE: u32      = 0x40000000;
const FLAG_IS_SIMPLE_GLYPH: u32    = 0x80000000;

// glyph advance; in Au's.
const GLYPH_ADVANCE_MASK: u32      = 0x3FFF0000;
const GLYPH_ADVANCE_SHIFT: u32     = 16;
const GLYPH_ID_MASK: u32           = 0x0000FFFF;

// Non-simple glyphs (more than one glyph per char; missing glyph,
// newline, tab, large advance, or nonzero x/y offsets) may have one
// or more detailed glyphs associated with them. They are stored in a
// side array so that there is a 1:1 mapping of GlyphEntry to
// unicode char.

// The number of detailed glyphs for this char.
const GLYPH_COUNT_MASK:              u32 = 0x0000FFFF;

fn is_simple_glyph_id(id: GlyphId) -> bool {
    ((id as u32) & GLYPH_ID_MASK) == id
}

fn is_simple_advance(advance: Au) -> bool {
    advance >= Au(0) && {
        let unsigned_au = advance.0 as u32;
        (unsigned_au & (GLYPH_ADVANCE_MASK >> GLYPH_ADVANCE_SHIFT)) == unsigned_au
    }
}

type DetailedGlyphCount = u16;

// Getters and setters for GlyphEntry. Setter methods are functional,
// because GlyphEntry is immutable and only a u32 in size.
impl GlyphEntry {
    #[inline(always)]
    fn advance(&self) -> Au {
        Au(((self.value & GLYPH_ADVANCE_MASK) >> GLYPH_ADVANCE_SHIFT) as i32)
    }

    fn id(&self) -> GlyphId {
        self.value & GLYPH_ID_MASK
    }

    /// True if original char was normal (U+0020) space. Other chars may
    /// map to space glyph, but this does not account for them.
    fn char_is_space(&self) -> bool {
        self.has_flag(FLAG_CHAR_IS_SPACE)
    }

    #[inline(always)]
    fn set_char_is_space(&self) -> GlyphEntry {
        GlyphEntry::new(self.value | FLAG_CHAR_IS_SPACE)
    }

    fn glyph_count(&self) -> u16 {
        assert!(!self.is_simple());
        (self.value & GLYPH_COUNT_MASK) as u16
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
#[derive(Clone, Debug, Copy, Deserialize, Serialize)]
struct DetailedGlyph {
    id: GlyphId,
    // glyph's advance, in the text's direction (LTR or RTL)
    advance: Au,
    // glyph's offset from the font's em-box (from top-left)
    offset: Point2D<Au>,
}

impl DetailedGlyph {
    fn new(id: GlyphId, advance: Au, offset: Point2D<Au>) -> DetailedGlyph {
        DetailedGlyph {
            id: id,
            advance: advance,
            offset: offset,
        }
    }
}

#[derive(PartialEq, Clone, Eq, Debug, Copy, Deserialize, Serialize)]
struct DetailedGlyphRecord {
    // source string offset/GlyphEntry offset in the TextRun
    entry_offset: CharIndex,
    // offset into the detailed glyphs buffer
    detail_offset: usize,
}

impl PartialOrd for DetailedGlyphRecord {
    fn partial_cmp(&self, other: &DetailedGlyphRecord) -> Option<Ordering> {
        self.entry_offset.partial_cmp(&other.entry_offset)
    }
}

impl Ord for DetailedGlyphRecord {
    fn cmp(&self, other: &DetailedGlyphRecord) -> Ordering {
        self.entry_offset.cmp(&other.entry_offset)
    }
}

// Manages the lookup table for detailed glyphs. Sorting is deferred
// until a lookup is actually performed; this matches the expected
// usage pattern of setting/appending all the detailed glyphs, and
// then querying without setting.
#[derive(Clone, Deserialize, Serialize)]
struct DetailedGlyphStore {
    // TODO(pcwalton): Allocation of this buffer is expensive. Consider a small-vector
    // optimization.
    detail_buffer: Vec<DetailedGlyph>,
    // TODO(pcwalton): Allocation of this buffer is expensive. Consider a small-vector
    // optimization.
    detail_lookup: Vec<DetailedGlyphRecord>,
    lookup_is_sorted: bool,
}

impl<'a> DetailedGlyphStore {
    fn new() -> DetailedGlyphStore {
        DetailedGlyphStore {
            detail_buffer: vec!(), // TODO: default size?
            detail_lookup: vec!(),
            lookup_is_sorted: false,
        }
    }

    fn add_detailed_glyphs_for_entry(&mut self, entry_offset: CharIndex, glyphs: &[DetailedGlyph]) {
        let entry = DetailedGlyphRecord {
            entry_offset: entry_offset,
            detail_offset: self.detail_buffer.len(),
        };

        debug!("Adding entry[off={:?}] for detailed glyphs: {:?}", entry_offset, glyphs);

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

    fn get_detailed_glyphs_for_entry(&'a self, entry_offset: CharIndex, count: u16)
                                  -> &'a [DetailedGlyph] {
        debug!("Requesting detailed glyphs[n={}] for entry[off={:?}]", count, entry_offset);

        // FIXME: Is this right? --pcwalton
        // TODO: should fix this somewhere else
        if count == 0 {
            return &self.detail_buffer[0..0];
        }

        assert!((count as usize) <= self.detail_buffer.len());
        assert!(self.lookup_is_sorted);

        let key = DetailedGlyphRecord {
            entry_offset: entry_offset,
            detail_offset: 0, // unused
        };

        let i = self.detail_lookup.binary_search_index(&key)
            .expect("Invalid index not found in detailed glyph lookup table!");

        assert!(i + (count as usize) <= self.detail_buffer.len());
        // return a slice into the buffer
        &self.detail_buffer[i .. i + count as usize]
    }

    fn get_detailed_glyph_with_index(&'a self,
                                     entry_offset: CharIndex,
                                     detail_offset: u16)
            -> &'a DetailedGlyph {
        assert!((detail_offset as usize) <= self.detail_buffer.len());
        assert!(self.lookup_is_sorted);

        let key = DetailedGlyphRecord {
            entry_offset: entry_offset,
            detail_offset: 0, // unused
        };

        let i = self.detail_lookup.binary_search_index(&key)
            .expect("Invalid index not found in detailed glyph lookup table!");

        assert!(i + (detail_offset as usize) < self.detail_buffer.len());
        &self.detail_buffer[i + (detail_offset as usize)]
    }

    fn ensure_sorted(&mut self) {
        if self.lookup_is_sorted {
            return;
        }

        // Sorting a unique vector is surprisingly hard. The following
        // code is a good argument for using DVecs, but they require
        // immutable locations thus don't play well with freezing.

        // Thar be dragons here. You have been warned. (Tips accepted.)
        let mut unsorted_records: Vec<DetailedGlyphRecord> = vec!();
        mem::swap(&mut self.detail_lookup, &mut unsorted_records);
        let mut mut_records: Vec<DetailedGlyphRecord> = unsorted_records;
        mut_records.sort_by(|a, b| {
            if a < b {
                Ordering::Less
            } else {
                Ordering::Greater
            }
        });
        let mut sorted_records = mut_records;
        mem::swap(&mut self.detail_lookup, &mut sorted_records);

        self.lookup_is_sorted = true;
    }
}

// This struct is used by GlyphStore clients to provide new glyph data.
// It should be allocated on the stack and passed by reference to GlyphStore.
#[derive(Copy, Clone)]
pub struct GlyphData {
    id: GlyphId,
    advance: Au,
    offset: Point2D<Au>,
    is_missing: bool,
    cluster_start: bool,
    ligature_start: bool,
}

impl GlyphData {
    /// Creates a new entry for one glyph.
    pub fn new(id: GlyphId,
               advance: Au,
               offset: Option<Point2D<Au>>,
               is_missing: bool,
               cluster_start: bool,
               ligature_start: bool)
            -> GlyphData {
        GlyphData {
            id: id,
            advance: advance,
            offset: offset.unwrap_or(Point2D::zero()),
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
#[derive(Copy, Clone)]
pub enum GlyphInfo<'a> {
    Simple(&'a GlyphStore, CharIndex),
    Detail(&'a GlyphStore, CharIndex, u16),
}

impl<'a> GlyphInfo<'a> {
    pub fn id(self) -> GlyphId {
        match self {
            GlyphInfo::Simple(store, entry_i) => store.entry_buffer[entry_i.to_usize()].id(),
            GlyphInfo::Detail(store, entry_i, detail_j) => {
                store.detail_store.get_detailed_glyph_with_index(entry_i, detail_j).id
            }
        }
    }

    #[inline(always)]
    // FIXME: Resolution conflicts with IteratorUtil trait so adding trailing _
    pub fn advance(self) -> Au {
        match self {
            GlyphInfo::Simple(store, entry_i) => store.entry_buffer[entry_i.to_usize()].advance(),
            GlyphInfo::Detail(store, entry_i, detail_j) => {
                store.detail_store.get_detailed_glyph_with_index(entry_i, detail_j).advance
            }
        }
    }

    pub fn offset(self) -> Option<Point2D<Au>> {
        match self {
            GlyphInfo::Simple(_, _) => None,
            GlyphInfo::Detail(store, entry_i, detail_j) => {
                Some(store.detail_store.get_detailed_glyph_with_index(entry_i, detail_j).offset)
            }
        }
    }
}

/// Stores the glyph data belonging to a text run.
///
/// Simple glyphs are stored inline in the `entry_buffer`, detailed glyphs are
/// stored as pointers into the `detail_store`.
///
/// ~~~ignore
/// +- GlyphStore --------------------------------+
/// |               +---+---+---+---+---+---+---+ |
/// | entry_buffer: |   | s |   | s |   | s | s | |  d = detailed
/// |               +-|-+---+-|-+---+-|-+---+---+ |  s = simple
/// |                 |       |       |           |
/// |                 |   +---+-------+           |
/// |                 |   |                       |
/// |               +-V-+-V-+                     |
/// | detail_store: | d | d |                     |
/// |               +---+---+                     |
/// +---------------------------------------------+
/// ~~~
#[derive(Clone, Deserialize, Serialize)]
pub struct GlyphStore {
    // TODO(pcwalton): Allocation of this buffer is expensive. Consider a small-vector
    // optimization.
    /// A buffer of glyphs within the text run, in the order in which they
    /// appear in the input text
    entry_buffer: Vec<GlyphEntry>,
    /// A store of the detailed glyph data. Detailed glyphs contained in the
    /// `entry_buffer` point to locations in this data structure.
    detail_store: DetailedGlyphStore,

    is_whitespace: bool,
    is_rtl: bool,
}

int_range_index! {
    #[derive(Deserialize, Serialize, RustcEncodable)]
    #[doc = "An index that refers to a character in a text run. This could \
             point to the middle of a glyph."]
    #[derive(HeapSizeOf)]
    struct CharIndex(isize)
}

impl<'a> GlyphStore {
    /// Initializes the glyph store, but doesn't actually shape anything.
    ///
    /// Use the `add_*` methods to store glyph data.
    pub fn new(length: usize, is_whitespace: bool, is_rtl: bool) -> GlyphStore {
        assert!(length > 0);

        GlyphStore {
            entry_buffer: vec![GlyphEntry::initial(); length],
            detail_store: DetailedGlyphStore::new(),
            is_whitespace: is_whitespace,
            is_rtl: is_rtl,
        }
    }

    pub fn char_len(&self) -> CharIndex {
        CharIndex(self.entry_buffer.len() as isize)
    }

    pub fn is_whitespace(&self) -> bool {
        self.is_whitespace
    }

    pub fn finalize_changes(&mut self) {
        self.detail_store.ensure_sorted();
    }

    /// Adds a single glyph. If `character` is present, this represents a single character;
    /// otherwise, this glyph represents multiple characters.
    pub fn add_glyph_for_char_index(&mut self,
                                    i: CharIndex,
                                    character: Option<char>,
                                    data: &GlyphData) {
        fn glyph_is_compressible(data: &GlyphData) -> bool {
            is_simple_glyph_id(data.id)
                && is_simple_advance(data.advance)
                && data.offset == Point2D::zero()
                && data.cluster_start  // others are stored in detail buffer
        }

        debug_assert!(data.ligature_start); // can't compress ligature continuation glyphs.
        debug_assert!(i < self.char_len());

        let mut entry = match (data.is_missing, glyph_is_compressible(data)) {
            (true, _) => GlyphEntry::missing(1),
            (false, true) => GlyphEntry::simple(data.id, data.advance),
            (false, false) => {
                let glyph = &[DetailedGlyph::new(data.id, data.advance, data.offset)];
                self.detail_store.add_detailed_glyphs_for_entry(i, glyph);
                GlyphEntry::complex(data.cluster_start, data.ligature_start, 1)
            }
        };

        if character == Some(' ') {
            entry = entry.set_char_is_space()
        }

        self.entry_buffer[i.to_usize()] = entry;
    }

    pub fn add_glyphs_for_char_index(&mut self, i: CharIndex, data_for_glyphs: &[GlyphData]) {
        assert!(i < self.char_len());
        assert!(data_for_glyphs.len() > 0);

        let glyph_count = data_for_glyphs.len();

        let first_glyph_data = data_for_glyphs[0];
        let entry = match first_glyph_data.is_missing {
            true  => GlyphEntry::missing(glyph_count),
            false => {
                let glyphs_vec: Vec<DetailedGlyph> = (0..glyph_count).map(|i| {
                    DetailedGlyph::new(data_for_glyphs[i].id,
                                       data_for_glyphs[i].advance,
                                       data_for_glyphs[i].offset)
                }).collect();

                self.detail_store.add_detailed_glyphs_for_entry(i, &glyphs_vec);
                GlyphEntry::complex(first_glyph_data.cluster_start,
                                    first_glyph_data.ligature_start,
                                    glyph_count)
            }
        }.adapt_character_flags_of_entry(self.entry_buffer[i.to_usize()]);

        debug!("Adding multiple glyphs[idx={:?}, count={}]: {:?}", i, glyph_count, entry);

        self.entry_buffer[i.to_usize()] = entry;
    }

    // used when a character index has no associated glyph---for example, a ligature continuation.
    pub fn add_nonglyph_for_char_index(&mut self, i: CharIndex, cluster_start: bool, ligature_start: bool) {
        assert!(i < self.char_len());

        let entry = GlyphEntry::complex(cluster_start, ligature_start, 0);
        debug!("adding spacer for chracter without associated glyph[idx={:?}]", i);

        self.entry_buffer[i.to_usize()] = entry;
    }

    #[inline]
    pub fn iter_glyphs_for_char_range(&'a self, rang: &Range<CharIndex>) -> GlyphIterator<'a> {
        if rang.begin() >= self.char_len() {
            panic!("iter_glyphs_for_range: range.begin beyond length!");
        }
        if rang.end() > self.char_len() {
            panic!("iter_glyphs_for_range: range.end beyond length!");
        }

        GlyphIterator {
            store:       self,
            char_index:  if self.is_rtl { rang.end() } else { rang.begin() - CharIndex(1) },
            char_range:  *rang,
            glyph_range: None,
        }
    }

    #[inline]
    pub fn advance_for_char_range(&self, rang: &Range<CharIndex>) -> Au {
        self.iter_glyphs_for_char_range(rang)
            .fold(Au(0), |advance, (_, glyph)| advance + glyph.advance())
    }

    pub fn char_is_space(&self, i: CharIndex) -> bool {
        assert!(i < self.char_len());
        self.entry_buffer[i.to_usize()].char_is_space()
    }

    pub fn space_count_in_range(&self, range: &Range<CharIndex>) -> u32 {
        let mut spaces = 0;
        for index in range.each_index() {
            if self.char_is_space(index) {
                spaces += 1
            }
        }
        spaces
    }

    pub fn distribute_extra_space_in_range(&mut self, range: &Range<CharIndex>, space: f64) {
        debug_assert!(space >= 0.0);
        if range.is_empty() {
            return
        }
        for index in range.each_index() {
            // TODO(pcwalton): Handle spaces that are detailed glyphs -- these are uncommon but
            // possible.
            let entry = &mut self.entry_buffer[index.to_usize()];
            if entry.is_simple() && entry.char_is_space() {
                // FIXME(pcwalton): This can overflow for very large font-sizes.
                let advance =
                    ((entry.value & GLYPH_ADVANCE_MASK) >> GLYPH_ADVANCE_SHIFT) +
                    Au::from_f64_px(space).0 as u32;
                entry.value = (entry.value & !GLYPH_ADVANCE_MASK) |
                    (advance << GLYPH_ADVANCE_SHIFT);
            }
        }
    }
}

/// An iterator over the glyphs in a character range in a `GlyphStore`.
pub struct GlyphIterator<'a> {
    store: &'a GlyphStore,
    char_index: CharIndex,
    char_range: Range<CharIndex>,
    glyph_range: Option<EachIndex<isize, CharIndex>>,
}

impl<'a> GlyphIterator<'a> {
    // Slow path when there is a glyph range.
    #[inline(never)]
    fn next_glyph_range(&mut self) -> Option<(CharIndex, GlyphInfo<'a>)> {
        match self.glyph_range.as_mut().unwrap().next() {
            Some(j) => Some((self.char_index,
                GlyphInfo::Detail(self.store, self.char_index, j.get() as u16 /* ??? */))),
            None => {
                // No more glyphs for current character.  Try to get another.
                self.glyph_range = None;
                self.next()
            }
        }
    }

    // Slow path when there is a complex glyph.
    #[inline(never)]
    fn next_complex_glyph(&mut self, entry: &GlyphEntry, i: CharIndex)
                          -> Option<(CharIndex, GlyphInfo<'a>)> {
        let glyphs = self.store.detail_store.get_detailed_glyphs_for_entry(i, entry.glyph_count());
        self.glyph_range = Some(range::each_index(CharIndex(0), CharIndex(glyphs.len() as isize)));
        self.next()
    }
}

impl<'a> Iterator for GlyphIterator<'a> {
    type Item  = (CharIndex, GlyphInfo<'a>);

    // I tried to start with something simpler and apply FlatMap, but the
    // inability to store free variables in the FlatMap struct was problematic.
    //
    // This function consists of the fast path and is designed to be inlined into its caller. The
    // slow paths, which should not be inlined, are `next_glyph_range()` and
    // `next_complex_glyph()`.
    #[inline(always)]
    fn next(&mut self) -> Option<(CharIndex, GlyphInfo<'a>)> {
        // Would use 'match' here but it borrows contents in a way that interferes with mutation.
        if self.glyph_range.is_some() {
            return self.next_glyph_range()
        }

        // No glyph range. Look at next character.
        self.char_index = self.char_index + if self.store.is_rtl {
            CharIndex(-1)
        } else {
            CharIndex(1)
        };
        let i = self.char_index;
        if !self.char_range.contains(i) {
            return None
        }
        debug_assert!(i < self.store.char_len());
        let entry = self.store.entry_buffer[i.to_usize()];
        if entry.is_simple() {
            Some((i, GlyphInfo::Simple(self.store, i)))
        } else {
            // Fall back to the slow path.
            self.next_complex_glyph(&entry, i)
        }
    }
}
