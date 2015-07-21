/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use euclid::point::Point2D;

int_range_index! {
    #[derive(Deserialize, Serialize, RustcEncodable)]
    #[doc = "An index that refers to a character in a text run. This could \
             point to the middle of a glyph."]
    #[derive(HeapSizeOf)]
    struct CharIndex(isize)
}

/// GlyphEntry is a port of Gecko's CompressedGlyph scheme for storing glyph data compactly.
///
/// In the common case (reasonable glyph advances, no offsets from the font em-box, and one glyph
/// per character), we pack glyph advance, glyph id, and some flags into a single u32.
///
/// In the uncommon case (multiple glyphs per unicode character, large glyph index/advance, or
/// glyph offsets), we pack the glyph count into GlyphEntry, and store the other glyph information
/// in DetailedGlyphStore.
#[derive(Clone, Debug, Copy, Deserialize, Serialize)]
pub struct GlyphEntry {                     // made public
    value: u32,
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
}

// Manages the lookup table for detailed glyphs. Sorting is deferred
// until a lookup is actually performed; this matches the expected
// usage pattern of setting/appending all the detailed glyphs, and
// then querying without setting.
#[derive(Clone, Deserialize, Serialize)]
pub struct DetailedGlyphStore {             // made public
    // TODO(pcwalton): Allocation of this buffer is expensive. Consider a small-vector
    // optimization.
    detail_buffer: Vec<DetailedGlyph>,
    // TODO(pcwalton): Allocation of this buffer is expensive. Consider a small-vector
    // optimization.
    detail_lookup: Vec<DetailedGlyphRecord>,
    lookup_is_sorted: bool,
}

#[derive(PartialEq, Clone, Eq, Debug, Copy, Deserialize, Serialize)]
pub struct DetailedGlyphRecord {            // made public
    // source string offset/GlyphEntry offset in the TextRun
    entry_offset: CharIndex,
    // offset into the detailed glyphs buffer
    detail_offset: usize,
}

/// The id of a particular glyph within a font
pub type GlyphId = u32;

// Stores data for a detailed glyph, in the case that several glyphs
// correspond to one character, or the glyph's data couldn't be packed.
#[derive(Clone, Debug, Copy, Deserialize, Serialize)]
pub struct DetailedGlyph {                  // made public
    id: GlyphId,
    // glyph's advance, in the text's direction (LTR or RTL)
    advance: Au,
    // glyph's offset from the font's em-box (from top-left)
    offset: Point2D<Au>,
}
