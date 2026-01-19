/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::fmt;
use std::vec::Vec;

use app_units::Au;
use euclid::default::Point2D;
use euclid::num::Zero;
use itertools::Either;
use malloc_size_of_derive::MallocSizeOf;
use serde::{Deserialize, Serialize};

use crate::{GlyphShapingResult, ShapedGlyph, ShapingFlags, ShapingOptions};

/// GlyphEntry is a port of Gecko's CompressedGlyph scheme for storing glyph data compactly.
///
/// In the common case (reasonable glyph advances, no offsets from the font em-box, and one glyph
/// per character), we pack glyph advance, glyph id, and some flags into a single u32.
///
/// In the uncommon case (multiple glyphs per unicode character, large glyph index/advance, or
/// glyph offsets), we pack the glyph count into GlyphEntry, and store the other glyph information
/// in DetailedGlyphStore.
#[derive(Clone, Copy, Debug, Deserialize, MallocSizeOf, PartialEq, Serialize)]
pub struct GlyphEntry {
    value: u32,
}

impl GlyphEntry {
    fn new(value: u32) -> GlyphEntry {
        GlyphEntry { value }
    }

    // Creates a GlyphEntry for the common case
    fn simple(id: GlyphId, advance: Au) -> GlyphEntry {
        assert!(is_simple_glyph_id(id));
        assert!(is_simple_advance(advance));

        let id_mask = id;
        let Au(advance) = advance;
        let advance_mask = (advance as u32) << GLYPH_ADVANCE_SHIFT;

        GlyphEntry::new(id_mask | advance_mask | FLAG_IS_SIMPLE_GLYPH)
    }

    fn complex(detailed_glyph_index: usize) -> GlyphEntry {
        assert!(detailed_glyph_index as u32 <= u32::MAX >> 1);
        GlyphEntry::new(detailed_glyph_index as u32)
    }
}

/// The id of a particular glyph within a font
pub(crate) type GlyphId = u32;

// TODO: make this more type-safe.

const FLAG_CHAR_IS_WORD_SEPARATOR: u32 = 0x40000000;
const FLAG_IS_SIMPLE_GLYPH: u32 = 0x80000000;

// glyph advance; in Au's.
const GLYPH_ADVANCE_MASK: u32 = 0x3FFF0000;
const GLYPH_ADVANCE_SHIFT: u32 = 16;
const GLYPH_ID_MASK: u32 = 0x0000FFFF;

// Non-simple glyphs (more than one glyph per char; missing glyph,
// newline, tab, large advance, or nonzero x/y offsets) may have one
// or more detailed glyphs associated with them. They are stored in a
// side array so that there is a 1:1 mapping of GlyphEntry to
// unicode char.

fn is_simple_glyph_id(id: GlyphId) -> bool {
    (id & GLYPH_ID_MASK) == id
}

fn is_simple_advance(advance: Au) -> bool {
    advance >= Au::zero() && {
        let unsigned_au = advance.0 as u32;
        (unsigned_au & (GLYPH_ADVANCE_MASK >> GLYPH_ADVANCE_SHIFT)) == unsigned_au
    }
}

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

    /// True if the original character was a word separator. These include spaces
    /// (U+0020), non-breaking spaces (U+00A0), and a few other characters
    /// non-exhaustively listed in the specification. Other characters may map to the same
    /// glyphs, but this function does not take mapping into account.
    ///
    /// See <https://drafts.csswg.org/css-text/#word-separator>.
    fn char_is_word_separator(&self) -> bool {
        self.has_flag(FLAG_CHAR_IS_WORD_SEPARATOR)
    }

    #[inline(always)]
    fn set_char_is_word_separator(&mut self) {
        self.value |= FLAG_CHAR_IS_WORD_SEPARATOR;
    }

    fn detailed_glyph_index(&self) -> usize {
        self.value as usize
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

#[derive(Clone, Deserialize, MallocSizeOf, Serialize)]
pub struct DetailedGlyphEntry {
    /// The id of the this glyph within the font.
    id: u32,
    /// The advance that this glyphs needs ie the distance between where this
    /// glyph is painted and the next is painted.
    advance: Au,
    /// The physical offset that this glyph should be painted with.
    offset: Option<Point2D<Au>>,
    /// The number of character this glyph corresponds to in the original string.
    /// This might be zero and this might be more than one.
    character_count: usize,
    /// Whether or not the originating character for this glyph was a word separator
    is_word_separator: bool,
}

// This enum is a proxy that's provided to GlyphStore clients when iterating
// through glyphs (either for a particular TextRun offset, or all glyphs).
// Rather than eagerly assembling and copying glyph data, it only retrieves
// values as they are needed from the GlyphStore, using provided offsets.
#[derive(Clone, Copy)]
pub enum GlyphInfo<'a> {
    Simple(&'a GlyphEntry),
    Detail(&'a DetailedGlyphEntry),
}

impl GlyphInfo<'_> {
    pub fn id(self) -> GlyphId {
        match self {
            GlyphInfo::Simple(entry) => entry.id(),
            GlyphInfo::Detail(entry) => entry.id,
        }
    }

    #[inline(always)]
    pub fn advance(self) -> Au {
        match self {
            GlyphInfo::Simple(entry) => entry.advance(),
            GlyphInfo::Detail(entry) => entry.advance,
        }
    }

    #[inline]
    pub fn offset(self) -> Option<Point2D<Au>> {
        match self {
            GlyphInfo::Simple(..) => None,
            GlyphInfo::Detail(entry) => entry.offset,
        }
    }

    #[inline]
    pub fn char_is_word_separator(self) -> bool {
        match self {
            GlyphInfo::Simple(entry) => entry.char_is_word_separator(),
            GlyphInfo::Detail(entry) => entry.is_word_separator,
        }
    }

    /// The number of characters that this glyph corresponds to. This may be more
    /// than one when a single glyph is produced for multiple characters. This may
    /// be zero when multiple glyphs are produced for a single character.
    #[inline]
    pub fn character_count(self) -> usize {
        match self {
            GlyphInfo::Simple(..) => 1,
            GlyphInfo::Detail(entry) => entry.character_count,
        }
    }
}

/// Stores the glyph data belonging to a text run.
///
/// Simple glyphs are stored inline in the `entry_buffer`, detailed glyphs are
/// stored as pointers into the `detail_store`.
///
/// ~~~ascii
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
#[derive(Clone, Deserialize, MallocSizeOf, Serialize)]
pub struct GlyphStore {
    // TODO(pcwalton): Allocation of this buffer is expensive. Consider a small-vector
    // optimization.
    /// A collection of [`GlyphEntry`]s within the [`GlyphStore`]. Each [`GlyphEntry`]
    /// maybe simple or detailed. When detailed, there will be a corresponding entry
    /// in [`Self::detailed_glyphs`].
    glyphs: Vec<GlyphEntry>,

    /// A vector of glyphs that cannot fit within a single [`GlyphEntry`] or that
    /// correspond to 0 or more than 1 character in the original string.
    detailed_glyphs: Vec<DetailedGlyphEntry>,

    /// A cache of the advance of the entire glyph store.
    total_advance: Au,

    /// The number of characters that correspond to the glyphs in this [`GlyphStore`]
    total_characters: usize,

    /// A cache of the number of word separators in the entire glyph store.
    /// See <https://drafts.csswg.org/css-text/#word-separator>.
    total_word_separators: usize,

    /// Whether or not this glyph store contains only glyphs for whitespace.
    is_whitespace: bool,

    /// Whether or not this glyph store ends with whitespace glyphs.
    /// Typically whitespace glyphs are placed in a separate store,
    /// but that may not be the case with `white-space: break-spaces`.
    ends_with_whitespace: bool,

    /// Whether or not this glyph store contains only a single glyph for a single
    /// preserved newline.
    is_single_preserved_newline: bool,

    /// Whether or not this [`GlyphStore`] has right-to-left text, which has implications
    /// about the order of the glyphs in the store.
    is_rtl: bool,
}

impl GlyphStore {
    /// Initializes the glyph store with the given capacity, but doesn't actually add any glyphs.
    ///
    /// Use the `add_*` methods to store glyph data.
    pub(crate) fn new(text: &str, length: usize, options: &ShapingOptions) -> Self {
        Self {
            glyphs: Vec::with_capacity(length),
            detailed_glyphs: Default::default(),
            total_advance: Au::zero(),
            total_characters: 0,
            total_word_separators: 0,
            is_whitespace: options
                .flags
                .contains(ShapingFlags::IS_WHITESPACE_SHAPING_FLAG),
            ends_with_whitespace: options
                .flags
                .contains(ShapingFlags::ENDS_WITH_WHITESPACE_SHAPING_FLAG),
            is_single_preserved_newline: text.len() == 1 && text.starts_with('\n'),
            is_rtl: options.flags.contains(ShapingFlags::RTL_FLAG),
        }
    }

    pub(crate) fn with_shaped_glyph_data(
        text: &str,
        options: &ShapingOptions,
        shaped_glyph_data: &impl GlyphShapingResult,
    ) -> Self {
        let mut characters = if !options.flags.contains(ShapingFlags::RTL_FLAG) {
            Either::Left(text.char_indices())
        } else {
            Either::Right(text.char_indices().rev())
        };

        let mut previous_character_offset = None;
        let mut glyph_store = GlyphStore::new(text, shaped_glyph_data.len(), options);
        for shaped_glyph in shaped_glyph_data.iter() {
            // The glyph "cluster" (HarfBuzz terminology) is the byte offset in the string that
            // this glyph corresponds to. More than one glyph can share a cluster.
            let glyph_cluster = shaped_glyph.string_byte_offset;

            if let Some(previous_character_offset) = previous_character_offset {
                if previous_character_offset == glyph_cluster {
                    glyph_store.add_glyph_for_current_character(&shaped_glyph)
                }
            }

            for (next_character_offset, next_character) in &mut characters {
                if next_character_offset == glyph_cluster {
                    previous_character_offset = Some(next_character_offset);
                    glyph_store.add_glyph(next_character, &shaped_glyph);
                    break;
                }
                glyph_store.extend_previous_glyph_by_character()
            }
        }

        // Consume any remaining characters that belong to the more-recently added glyph.
        for (_, _) in characters {
            glyph_store.extend_previous_glyph_by_character();
        }

        glyph_store
    }

    #[inline]
    pub fn total_advance(&self) -> Au {
        self.total_advance
    }

    /// Return the number of glyphs stored in this [`GlyphStore`].
    #[inline]
    pub fn len(&self) -> usize {
        self.glyphs.len()
    }

    /// Whether or not this [`GlyphStore`] has any glyphs.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.glyphs.is_empty()
    }

    /// The number of characters (`char`) from the original string that produced this
    /// [`GlyphStore`].
    #[inline]
    pub fn character_count(&self) -> usize {
        self.total_characters
    }

    /// Whether or not this [`GlyphStore`] is entirely whitepsace.
    #[inline]
    pub fn is_whitespace(&self) -> bool {
        self.is_whitespace
    }

    /// Whether or not this [`GlyphStore`] is a single preserved newline.
    #[inline]
    pub fn is_single_preserved_newline(&self) -> bool {
        self.is_single_preserved_newline
    }

    /// Whether or not this [`GlyphStore`] ends with whitespace.
    #[inline]
    pub fn ends_with_whitespace(&self) -> bool {
        self.ends_with_whitespace
    }

    /// The number of word separators in this [`GlyphStore`].
    #[inline]
    pub fn total_word_separators(&self) -> usize {
        self.total_word_separators
    }

    /// The number of characters that were consumed to produce this [`GlyphStore`]. Some
    /// characters correpond to more than one glyph and some glyphs correspond to more than
    /// one character.
    #[inline]
    pub fn total_characters(&self) -> usize {
        self.total_characters
    }

    /// Adds glyph that corresponds to a single character (as far we know) in the originating string.
    #[inline]
    pub(crate) fn add_glyph(&mut self, character: char, shaped_glyph_entry: &ShapedGlyph) {
        if !shaped_glyph_entry.can_be_simple_glyph() {
            self.add_detailed_glyph(shaped_glyph_entry, Some(character), 1);
            return;
        }

        let mut simple_glyph_entry =
            GlyphEntry::simple(shaped_glyph_entry.glyph_id, shaped_glyph_entry.advance);

        // This list is taken from the non-exhaustive list of word separator characters in
        // the CSS Text Module Level 3 Spec:
        // See https://drafts.csswg.org/css-text/#word-separator
        if character_is_word_separator(character) {
            self.total_word_separators += 1;
            simple_glyph_entry.set_char_is_word_separator();
        }

        self.total_characters += 1;
        self.total_advance += shaped_glyph_entry.advance;
        self.glyphs.push(simple_glyph_entry)
    }

    fn add_detailed_glyph(
        &mut self,
        shaped_glyph_entry: &ShapedGlyph,
        character: Option<char>,
        character_count: usize,
    ) {
        self.total_advance += shaped_glyph_entry.advance;

        let is_word_separator = character.is_some_and(character_is_word_separator);
        if is_word_separator {
            self.total_word_separators += 1;
        }

        self.total_characters += character_count;
        self.detailed_glyphs.push(DetailedGlyphEntry {
            id: shaped_glyph_entry.glyph_id,
            advance: shaped_glyph_entry.advance,
            offset: shaped_glyph_entry.offset,
            character_count,
            is_word_separator,
        });
    }

    fn extend_previous_glyph_by_character(&mut self) {
        let detailed_glyph_index = self.ensure_last_glyph_is_detailed();
        let detailed_glyph = self
            .detailed_glyphs
            .get_mut(detailed_glyph_index)
            .expect("GlyphEntry should have valid index to detailed glyph");
        detailed_glyph.character_count += 1;
        self.total_characters += 1;
    }

    fn add_glyph_for_current_character(&mut self, shaped_glyph_entry: &ShapedGlyph) {
        // Add a detailed glyph entry for this new glyph, but it corresponds to a character
        // we have already started processing. It should not contribute any character count.
        self.add_detailed_glyph(shaped_glyph_entry, None, 0);
    }

    /// If the last glyph added to this [`GlyphStore`] was a simple glyph, convert it to a
    /// detailed one. In either case, return the index into [`Self::detailed_glyphs`] for
    /// the most recently added glyph.
    fn ensure_last_glyph_is_detailed(&mut self) -> usize {
        let last_glyph = self
            .glyphs
            .last_mut()
            .expect("Should never call this before any glyphs have been added.");
        if !last_glyph.is_simple() {
            return last_glyph.detailed_glyph_index();
        }

        self.detailed_glyphs.push(DetailedGlyphEntry {
            id: last_glyph.id(),
            advance: last_glyph.advance(),
            offset: Default::default(),
            character_count: 1,
            is_word_separator: last_glyph.char_is_word_separator(),
        });

        let detailed_glyph_index = self.detailed_glyphs.len() - 1;
        *last_glyph = GlyphEntry::complex(detailed_glyph_index);
        detailed_glyph_index
    }

    pub fn glyphs(&self) -> impl Iterator<Item = GlyphInfo<'_>> + use<'_> {
        self.glyphs.iter().map(|entry| {
            if entry.is_simple() {
                GlyphInfo::Simple(entry)
            } else {
                GlyphInfo::Detail(&self.detailed_glyphs[entry.detailed_glyph_index()])
            }
        })
    }
}

impl ShapedGlyph {
    fn can_be_simple_glyph(&self) -> bool {
        is_simple_glyph_id(self.glyph_id) &&
            is_simple_advance(self.advance) &&
            self.offset
                .is_none_or(|offset| offset == Default::default())
    }
}

fn character_is_word_separator(character: char) -> bool {
    let is_word_separator = matches!(
        character,
        ' ' |
                '\u{00A0}' | // non-breaking space
                '\u{1361}' | // Ethiopic word space
                '\u{10100}' | // Aegean word separator
                '\u{10101}' | // Aegean word separator
                '\u{1039F}' | // Ugartic word divider
                '\u{1091F}' // Phoenician word separator
    );
    is_word_separator
}

impl fmt::Debug for GlyphStore {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        writeln!(formatter, "GlyphStore:")?;
        for entry in self.glyphs.iter() {
            if entry.is_simple() {
                writeln!(
                    formatter,
                    "  simple id={:?} advance={:?}",
                    entry.id(),
                    entry.advance()
                )?;
                continue;
            } else {
                let detailed_glyph = &self.detailed_glyphs[entry.detailed_glyph_index()];
                writeln!(
                    formatter,
                    "  detailed id={:?} advance={:?} characters={:?}",
                    detailed_glyph.id, detailed_glyph.advance, detailed_glyph.character_count,
                )?;
            }
        }
        Ok(())
    }
}
