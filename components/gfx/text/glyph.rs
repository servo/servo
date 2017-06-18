/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use app_units::Au;
use euclid::Point2D;
use range::{self, EachIndex, Range, RangeIndex};
#[cfg(any(target_feature = "sse2", target_feature = "neon"))]
use simd::u32x4;
use std::{fmt, mem, u16};
use std::cmp::{Ordering, PartialOrd};
use std::vec::Vec;

pub use gfx_traits::ByteIndex;

/// GlyphEntry is a port of Gecko's CompressedGlyph scheme for storing glyph data compactly.
///
/// In the common case (reasonable glyph advances, no offsets from the font em-box, and one glyph
/// per character), we pack glyph advance, glyph id, and some flags into a single u32.
///
/// In the uncommon case (multiple glyphs per unicode character, large glyph index/advance, or
/// glyph offsets), we pack the glyph count into GlyphEntry, and store the other glyph information
/// in DetailedGlyphStore.
#[derive(Clone, Debug, Copy, Deserialize, Serialize, PartialEq)]
pub struct GlyphEntry {
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

    fn is_initial(&self) -> bool {
        *self == GlyphEntry::initial()
    }
}

/// The id of a particular glyph within a font
pub type GlyphId = u32;

// TODO: make this more type-safe.

const FLAG_CHAR_IS_SPACE: u32       = 0x40000000;
#[cfg(any(target_feature = "sse2", target_feature = "neon"))]
const FLAG_CHAR_IS_SPACE_SHIFT: u32 = 30;
const FLAG_IS_SIMPLE_GLYPH: u32     = 0x80000000;

// glyph advance; in Au's.
const GLYPH_ADVANCE_MASK: u32       = 0x3FFF0000;
const GLYPH_ADVANCE_SHIFT: u32      = 16;
const GLYPH_ID_MASK: u32            = 0x0000FFFF;

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

pub type DetailedGlyphCount = u16;

// Getters and setters for GlyphEntry. Setter methods are functional,
// because GlyphEntry is immutable and only a u32 in size.
impl GlyphEntry {
    #[inline(always)]
    fn advance(&self) -> Au {
        Au::new(((self.value & GLYPH_ADVANCE_MASK) >> GLYPH_ADVANCE_SHIFT) as i32)
    }

    #[inline]
    fn id(&self) -> GlyphId {
        self.value & GLYPH_ID_MASK
    }

    /// True if original char was normal (U+0020) space. Other chars may
    /// map to space glyph, but this does not account for them.
    fn char_is_space(&self) -> bool {
        self.has_flag(FLAG_CHAR_IS_SPACE)
    }

    #[inline(always)]
    fn set_char_is_space(&mut self) {
        self.value |= FLAG_CHAR_IS_SPACE;
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
    entry_offset: ByteIndex,
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

    fn add_detailed_glyphs_for_entry(&mut self, entry_offset: ByteIndex, glyphs: &[DetailedGlyph]) {
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
        self.detail_buffer.extend_from_slice(glyphs);
        self.lookup_is_sorted = false;
    }

    fn detailed_glyphs_for_entry(&'a self, entry_offset: ByteIndex, count: u16)
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

        let i = self.detail_lookup.binary_search(&key)
            .expect("Invalid index not found in detailed glyph lookup table!");
        let main_detail_offset = self.detail_lookup[i].detail_offset;
        assert!(main_detail_offset + (count as usize) <= self.detail_buffer.len());
        // return a slice into the buffer
        &self.detail_buffer[main_detail_offset .. main_detail_offset + count as usize]
    }

    fn detailed_glyph_with_index(&'a self,
                                     entry_offset: ByteIndex,
                                     detail_offset: u16)
            -> &'a DetailedGlyph {
        assert!((detail_offset as usize) <= self.detail_buffer.len());
        assert!(self.lookup_is_sorted);

        let key = DetailedGlyphRecord {
            entry_offset: entry_offset,
            detail_offset: 0, // unused
        };

        let i = self.detail_lookup.binary_search(&key)
            .expect("Invalid index not found in detailed glyph lookup table!");
        let main_detail_offset = self.detail_lookup[i].detail_offset;
        assert!(main_detail_offset + (detail_offset as usize) < self.detail_buffer.len());
        &self.detail_buffer[main_detail_offset + (detail_offset as usize)]
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
    cluster_start: bool,
    ligature_start: bool,
}

impl GlyphData {
    /// Creates a new entry for one glyph.
    pub fn new(id: GlyphId,
               advance: Au,
               offset: Option<Point2D<Au>>,
               cluster_start: bool,
               ligature_start: bool)
            -> GlyphData {
        GlyphData {
            id: id,
            advance: advance,
            offset: offset.unwrap_or(Point2D::zero()),
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
    Simple(&'a GlyphStore, ByteIndex),
    Detail(&'a GlyphStore, ByteIndex, u16),
}

impl<'a> GlyphInfo<'a> {
    pub fn id(self) -> GlyphId {
        match self {
            GlyphInfo::Simple(store, entry_i) => store.entry_buffer[entry_i.to_usize()].id(),
            GlyphInfo::Detail(store, entry_i, detail_j) => {
                store.detail_store.detailed_glyph_with_index(entry_i, detail_j).id
            }
        }
    }

    #[inline(always)]
    // FIXME: Resolution conflicts with IteratorUtil trait so adding trailing _
    pub fn advance(self) -> Au {
        match self {
            GlyphInfo::Simple(store, entry_i) => store.entry_buffer[entry_i.to_usize()].advance(),
            GlyphInfo::Detail(store, entry_i, detail_j) => {
                store.detail_store.detailed_glyph_with_index(entry_i, detail_j).advance
            }
        }
    }

    #[inline]
    pub fn offset(self) -> Option<Point2D<Au>> {
        match self {
            GlyphInfo::Simple(_, _) => None,
            GlyphInfo::Detail(store, entry_i, detail_j) => {
                Some(store.detail_store.detailed_glyph_with_index(entry_i, detail_j).offset)
            }
        }
    }

    pub fn char_is_space(self) -> bool {
        let (store, entry_i) = match self {
            GlyphInfo::Simple(store, entry_i) => (store, entry_i),
            GlyphInfo::Detail(store, entry_i, _) => (store, entry_i),
        };

        store.char_is_space(entry_i)
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
    /// appear in the input text.
    /// Any changes will also need to be reflected in
    /// transmute_entry_buffer_to_u32_buffer().
    entry_buffer: Vec<GlyphEntry>,
    /// A store of the detailed glyph data. Detailed glyphs contained in the
    /// `entry_buffer` point to locations in this data structure.
    detail_store: DetailedGlyphStore,

    /// A cache of the advance of the entire glyph store.
    total_advance: Au,
    /// A cache of the number of spaces in the entire glyph store.
    total_spaces: i32,

    /// Used to check if fast path should be used in glyph iteration.
    has_detailed_glyphs: bool,
    is_whitespace: bool,
    is_rtl: bool,
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
            total_advance: Au(0),
            total_spaces: 0,
            has_detailed_glyphs: false,
            is_whitespace: is_whitespace,
            is_rtl: is_rtl,
        }
    }

    #[inline]
    pub fn len(&self) -> ByteIndex {
        ByteIndex(self.entry_buffer.len() as isize)
    }

    #[inline]
    pub fn is_whitespace(&self) -> bool {
        self.is_whitespace
    }

    pub fn finalize_changes(&mut self) {
        self.detail_store.ensure_sorted();
        self.cache_total_advance_and_spaces()
    }

    #[inline(never)]
    fn cache_total_advance_and_spaces(&mut self) {
        let mut total_advance = Au(0);
        let mut total_spaces = 0;
        for glyph in self.iter_glyphs_for_byte_range(&Range::new(ByteIndex(0), self.len())) {
            total_advance = total_advance + glyph.advance();
            if glyph.char_is_space() {
                total_spaces += 1;
            }
        }
        self.total_advance = total_advance;
        self.total_spaces = total_spaces;
    }

    /// Adds a single glyph.
    pub fn add_glyph_for_byte_index(&mut self,
                                    i: ByteIndex,
                                    character: char,
                                    data: &GlyphData) {
        let glyph_is_compressible = is_simple_glyph_id(data.id) &&
            is_simple_advance(data.advance) &&
                data.offset == Point2D::zero() &&
                data.cluster_start;  // others are stored in detail buffer

        debug_assert!(data.ligature_start); // can't compress ligature continuation glyphs.
        debug_assert!(i < self.len());

        let mut entry = if glyph_is_compressible {
            GlyphEntry::simple(data.id, data.advance)
        } else {
            let glyph = &[DetailedGlyph::new(data.id, data.advance, data.offset)];
            self.has_detailed_glyphs = true;
            self.detail_store.add_detailed_glyphs_for_entry(i, glyph);
            GlyphEntry::complex(data.cluster_start, data.ligature_start, 1)
        };

        if character == ' ' {
            entry.set_char_is_space()
        }

        self.entry_buffer[i.to_usize()] = entry;
    }

    pub fn add_glyphs_for_byte_index(&mut self, i: ByteIndex, data_for_glyphs: &[GlyphData]) {
        assert!(i < self.len());
        assert!(data_for_glyphs.len() > 0);

        let glyph_count = data_for_glyphs.len();

        let first_glyph_data = data_for_glyphs[0];
        let glyphs_vec: Vec<DetailedGlyph> = (0..glyph_count).map(|i| {
            DetailedGlyph::new(data_for_glyphs[i].id,
                               data_for_glyphs[i].advance,
                               data_for_glyphs[i].offset)
        }).collect();

        self.has_detailed_glyphs = true;
        self.detail_store.add_detailed_glyphs_for_entry(i, &glyphs_vec);

        let entry = GlyphEntry::complex(first_glyph_data.cluster_start,
                                        first_glyph_data.ligature_start,
                                        glyph_count);

        debug!("Adding multiple glyphs[idx={:?}, count={}]: {:?}", i, glyph_count, entry);

        self.entry_buffer[i.to_usize()] = entry;
    }

    #[inline]
    pub fn iter_glyphs_for_byte_range(&'a self, range: &Range<ByteIndex>) -> GlyphIterator<'a> {
        if range.begin() >= self.len() {
            panic!("iter_glyphs_for_range: range.begin beyond length!");
        }
        if range.end() > self.len() {
            panic!("iter_glyphs_for_range: range.end beyond length!");
        }

        GlyphIterator {
            store:       self,
            byte_index:  if self.is_rtl { range.end() } else { range.begin() - ByteIndex(1) },
            byte_range:  *range,
            glyph_range: None,
        }
    }

    // Scan the glyphs for a given range until we reach a given advance. Returns the index
    // and advance of the glyph in the range at the given advance, if reached. Otherwise, returns the
    // the number of glyphs and the advance for the given range.
    #[inline]
    pub fn range_index_of_advance(&self, range: &Range<ByteIndex>, advance: Au, extra_word_spacing: Au) -> (usize, Au) {
        let mut index = 0;
        let mut current_advance = Au(0);
        for glyph in self.iter_glyphs_for_byte_range(range) {
            if glyph.char_is_space() {
                current_advance += glyph.advance() + extra_word_spacing
            } else {
                current_advance += glyph.advance()
            }
            if current_advance > advance {
                break;
            }
            index += 1;
        }
        (index, current_advance)
    }

    #[inline]
    pub fn advance_for_byte_range(&self, range: &Range<ByteIndex>, extra_word_spacing: Au) -> Au {
        if range.begin() == ByteIndex(0) && range.end() == self.len() {
            self.total_advance + extra_word_spacing * self.total_spaces
        } else if !self.has_detailed_glyphs {
            self.advance_for_byte_range_simple_glyphs(range, extra_word_spacing)
        } else {
            self.advance_for_byte_range_slow_path(range, extra_word_spacing)
        }
    }

    #[inline]
    pub fn advance_for_byte_range_slow_path(&self, range: &Range<ByteIndex>, extra_word_spacing: Au) -> Au {
        self.iter_glyphs_for_byte_range(range)
            .fold(Au(0), |advance, glyph| {
                if glyph.char_is_space() {
                    advance + glyph.advance() + extra_word_spacing
                } else {
                    advance + glyph.advance()
                }
            })
    }

    #[inline]
    #[cfg(any(target_feature = "sse2", target_feature = "neon"))]
    fn advance_for_byte_range_simple_glyphs(&self, range: &Range<ByteIndex>, extra_word_spacing: Au) -> Au {
        let advance_mask = u32x4::splat(GLYPH_ADVANCE_MASK);
        let space_flag_mask = u32x4::splat(FLAG_CHAR_IS_SPACE);
        let mut simd_advance = u32x4::splat(0);
        let mut simd_spaces = u32x4::splat(0);
        let begin = range.begin().to_usize();
        let len = range.length().to_usize();
        let num_simd_iterations = len / 4;
        let leftover_entries = range.end().to_usize() - (len - num_simd_iterations * 4);
        let buf = self.transmute_entry_buffer_to_u32_buffer();

        for i in 0..num_simd_iterations {
            let v = u32x4::load(buf, begin + i * 4);
            let advance = (v & advance_mask) >> GLYPH_ADVANCE_SHIFT;
            let spaces = (v & space_flag_mask) >> FLAG_CHAR_IS_SPACE_SHIFT;
            simd_advance = simd_advance + advance;
            simd_spaces = simd_spaces + spaces;
        }

        let advance =
            (simd_advance.extract(0) +
             simd_advance.extract(1) +
             simd_advance.extract(2) +
             simd_advance.extract(3)) as i32;
        let spaces =
            (simd_spaces.extract(0) +
             simd_spaces.extract(1) +
             simd_spaces.extract(2) +
             simd_spaces.extract(3)) as i32;
        let mut leftover_advance = Au(0);
        let mut leftover_spaces = 0;
        for i in leftover_entries..range.end().to_usize() {
            leftover_advance = leftover_advance + self.entry_buffer[i].advance();
            if self.entry_buffer[i].char_is_space() {
                leftover_spaces += 1;
            }
        }
        Au::new(advance) + leftover_advance + extra_word_spacing * (spaces + leftover_spaces)
    }

    /// When SIMD isn't available, fallback to the slow path.
    #[inline]
    #[cfg(not(any(target_feature = "sse2", target_feature = "neon")))]
    fn advance_for_byte_range_simple_glyphs(&self, range: &Range<ByteIndex>, extra_word_spacing: Au) -> Au {
        self.advance_for_byte_range_slow_path(range, extra_word_spacing)
    }

    /// Used for SIMD.
    #[inline]
    #[cfg(any(target_feature = "sse2", target_feature = "neon"))]
    #[allow(unsafe_code)]
    fn transmute_entry_buffer_to_u32_buffer(&self) -> &[u32] {
        unsafe { mem::transmute(self.entry_buffer.as_slice()) }
    }

    pub fn char_is_space(&self, i: ByteIndex) -> bool {
        assert!(i < self.len());
        self.entry_buffer[i.to_usize()].char_is_space()
    }

    pub fn space_count_in_range(&self, range: &Range<ByteIndex>) -> u32 {
        let mut spaces = 0;
        for index in range.each_index() {
            if self.char_is_space(index) {
                spaces += 1
            }
        }
        spaces
    }
}

impl fmt::Debug for GlyphStore {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "GlyphStore:\n")?;
        let mut detailed_buffer = self.detail_store.detail_buffer.iter();
        for entry in self.entry_buffer.iter() {
            if entry.is_simple() {
                write!(formatter,
                            "  simple id={:?} advance={:?}\n",
                            entry.id(),
                            entry.advance())?;
                continue
            }
            if entry.is_initial() {
                continue
            }
            write!(formatter, "  complex...")?;
            if detailed_buffer.next().is_none() {
                continue
            }
            write!(formatter,
                        "  detailed id={:?} advance={:?}\n",
                        entry.id(),
                        entry.advance())?;
        }
        Ok(())
    }
}

/// An iterator over the glyphs in a byte range in a `GlyphStore`.
pub struct GlyphIterator<'a> {
    store: &'a GlyphStore,
    byte_index: ByteIndex,
    byte_range: Range<ByteIndex>,
    glyph_range: Option<EachIndex<isize, ByteIndex>>,
}

impl<'a> GlyphIterator<'a> {
    // Slow path when there is a glyph range.
    #[inline(never)]
    fn next_glyph_range(&mut self) -> Option<GlyphInfo<'a>> {
        match self.glyph_range.as_mut().unwrap().next() {
            Some(j) => {
                Some(GlyphInfo::Detail(self.store, self.byte_index, j.get() as u16 /* ??? */))
            }
            None => {
                // No more glyphs for current character.  Try to get another.
                self.glyph_range = None;
                self.next()
            }
        }
    }

    // Slow path when there is a complex glyph.
    #[inline(never)]
    fn next_complex_glyph(&mut self, entry: &GlyphEntry, i: ByteIndex) -> Option<GlyphInfo<'a>> {
        let glyphs = self.store.detail_store.detailed_glyphs_for_entry(i, entry.glyph_count());
        self.glyph_range = Some(range::each_index(ByteIndex(0), ByteIndex(glyphs.len() as isize)));
        self.next()
    }
}

impl<'a> Iterator for GlyphIterator<'a> {
    type Item  = GlyphInfo<'a>;

    // I tried to start with something simpler and apply FlatMap, but the
    // inability to store free variables in the FlatMap struct was problematic.
    //
    // This function consists of the fast path and is designed to be inlined into its caller. The
    // slow paths, which should not be inlined, are `next_glyph_range()` and
    // `next_complex_glyph()`.
    #[inline(always)]
    fn next(&mut self) -> Option<GlyphInfo<'a>> {
        // Would use 'match' here but it borrows contents in a way that interferes with mutation.
        if self.glyph_range.is_some() {
            return self.next_glyph_range()
        }

        // No glyph range. Look at next byte.
        self.byte_index = self.byte_index + if self.store.is_rtl {
            ByteIndex(-1)
        } else {
            ByteIndex(1)
        };
        let i = self.byte_index;
        if !self.byte_range.contains(i) {
            return None
        }
        debug_assert!(i < self.store.len());
        let entry = self.store.entry_buffer[i.to_usize()];
        if entry.is_simple() {
            Some(GlyphInfo::Simple(self.store, i))
        } else {
            // Fall back to the slow path.
            self.next_complex_glyph(&entry, i)
        }
    }
}
