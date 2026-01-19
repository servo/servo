/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

mod harfbuzz;

use app_units::Au;
use euclid::default::Point2D;
pub(crate) use harfbuzz::Shaper;

use crate::GlyphId;

/// Utility function to convert a `unicode_script::Script` enum into the corresponding `c_uint` tag that
/// harfbuzz uses to represent unicode scipts.
fn unicode_script_to_iso15924_tag(script: unicode_script::Script) -> u32 {
    let bytes: [u8; 4] = match script {
        unicode_script::Script::Unknown => *b"Zzzz",
        _ => {
            let short_name = script.short_name();
            short_name.as_bytes().try_into().unwrap()
        },
    };

    u32::from_be_bytes(bytes)
}

pub(crate) struct ShapedGlyph {
    /// The actual glyph to render for this [`ShapedGlyph`].
    pub glyph_id: GlyphId,
    /// The original byte offset in the input buffer of the character that this
    /// glyph belongs to. More than one glyph can share the same character and
    /// one character can produce multiple glyphs.
    pub string_byte_offset: usize,
    /// The advance the direction of the writing mode that this glyph needs.
    pub advance: Au,
    /// An offset that should be applied when rendering this glyph.
    pub offset: Option<Point2D<Au>>,
}

/// Holds the results of shaping. Abstracts over HarfBuzz and HarfRust which return data in very similar
/// form but with different types
pub(crate) trait GlyphShapingResult {
    /// The number of shaped glyphs
    fn len(&self) -> usize;
    /// An iterator of the shaped glyphs of this data.
    fn iter(&self) -> impl Iterator<Item = ShapedGlyph>;
}
