/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cmp;

use app_units::Au;
use base::text::is_bidi_control;
use euclid::default::Point2D;
use fonts_traits::ByteIndex;
use log::debug;
use num_traits::Zero as _;

use crate::{Font, GlyphData, GlyphId, GlyphStore, ShapingOptions, advance_for_shaped_glyph};

#[cfg(feature = "harfbuzz")]
mod harfbuzz;
#[cfg(feature = "harfbuzz")]
pub(crate) use harfbuzz::Shaper as HarfBuzzShaper;

#[cfg(feature = "harfrust")]
mod harfrust;
#[cfg(feature = "harfrust")]
pub(crate) use harfrust::Shaper as HarfRustShaper;

#[cfg(all(feature = "harfbuzz", feature = "harfrust"))]
mod both;
#[cfg(all(feature = "harfbuzz", feature = "harfrust"))]
pub(crate) use BothShaper as Shaper;
// Configure default shaper (actually used)
#[cfg(all(feature = "harfbuzz", not(feature = "harfrust")))]
pub(crate) use HarfBuzzShaper as Shaper;
#[cfg(all(not(feature = "harfbuzz"), feature = "harfrust"))]
pub(crate) use HarfRustShaper as Shaper;
#[cfg(all(feature = "harfbuzz", feature = "harfrust"))]
pub(crate) use both::Shaper as BothShaper;

const NO_GLYPH: i32 = -1;

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

#[derive(PartialEq)]
struct ShapedGlyphEntry {
    codepoint: GlyphId,
    advance: Au,
    offset: Option<Point2D<Au>>,
}

/// Holds the results of shaping. Abstracts over HarfBuzz and HarfRust which return data in very similar
/// form but with different types
trait HarfBuzzShapedGlyphData {
    /// The number of shaped glyphs
    fn len(&self) -> usize;
    /// The byte offset of the shaped glyph in the souce text
    fn byte_offset_of_glyph(&self, i: usize) -> u32;
    /// Returns shaped glyph data for one glyph, and updates the y-position of the pen.
    fn entry_for_glyph(&self, i: usize, y_pos: &mut Au) -> ShapedGlyphEntry;
}

/// Shape text using an `impl HarfBuzzShaper`
fn shape_text_harfbuzz<ShapedGlyphData: HarfBuzzShapedGlyphData>(
    glyph_data: &ShapedGlyphData,
    font: &Font,
    text: &str,
    options: &ShapingOptions,
    glyphs: &mut GlyphStore,
) {
    let glyph_count = glyph_data.len();
    let byte_max = text.len();

    debug!(
        "Shaped text[byte count={}], got back {} glyph info records.",
        byte_max, glyph_count
    );

    // make map of what chars have glyphs
    let mut byte_to_glyph = vec![NO_GLYPH; byte_max];

    debug!("(glyph idx) -> (text byte offset)");
    for i in 0..glyph_data.len() {
        let loc = glyph_data.byte_offset_of_glyph(i) as usize;
        if loc < byte_max {
            byte_to_glyph[loc] = i as i32;
        } else {
            debug!(
                "ERROR: tried to set out of range byte_to_glyph: idx={}, glyph idx={}",
                loc, i
            );
        }
        debug!("{} -> {}", i, loc);
    }

    debug!("text: {:?}", text);
    debug!("(char idx): char->(glyph index):");
    for (i, ch) in text.char_indices() {
        debug!("{}: {:?} --> {}", i, ch, byte_to_glyph[i]);
    }

    let mut glyph_span = 0..0;
    let mut byte_range = 0..0;

    let mut y_pos = Au::zero();

    // main loop over each glyph. each iteration usually processes 1 glyph and 1+ chars.
    // in cases with complex glyph-character associations, 2+ glyphs and 1+ chars can be
    // processed.
    while glyph_span.start < glyph_count {
        debug!("Processing glyph at idx={}", glyph_span.start);
        glyph_span.end = glyph_span.start;
        byte_range.end = glyph_data.byte_offset_of_glyph(glyph_span.start) as usize;

        while byte_range.end < byte_max {
            byte_range.end += 1;
            // Extend the byte range to include any following byte without its own glyph.
            while byte_range.end < byte_max && byte_to_glyph[byte_range.end] == NO_GLYPH {
                byte_range.end += 1;
            }

            // Extend the glyph range to include all glyphs covered by bytes processed so far.
            let mut max_glyph_idx = glyph_span.end;
            for glyph_idx in &byte_to_glyph[byte_range.clone()] {
                if *glyph_idx != NO_GLYPH {
                    max_glyph_idx = cmp::max(*glyph_idx as usize + 1, max_glyph_idx);
                }
            }
            if max_glyph_idx > glyph_span.end {
                glyph_span.end = max_glyph_idx;
                debug!("Extended glyph span to {:?}", glyph_span);
            }

            // if there's just one glyph, then we don't need further checks.
            if glyph_span.len() == 1 {
                break;
            }

            // if no glyphs were found yet, extend the char byte range more.
            if glyph_span.is_empty() {
                continue;
            }

            // If byte_range now includes all the byte offsets found in glyph_span, then we
            // have found a contiguous "cluster" and can stop extending it.
            let mut all_glyphs_are_within_cluster: bool = true;
            for j in glyph_span.clone() {
                let loc = glyph_data.byte_offset_of_glyph(j) as usize;
                if !(byte_range.start <= loc && loc < byte_range.end) {
                    all_glyphs_are_within_cluster = false;
                    break;
                }
            }
            if all_glyphs_are_within_cluster {
                break;
            }

            // Otherwise, the bytes we have seen so far correspond to a non-contiguous set of
            // glyphs.  Keep extending byte_range until we fill in all the holes in the glyph
            // span or reach the end of the text.
        }

        assert!(!byte_range.is_empty());
        assert!(!glyph_span.is_empty());

        // Now byte_range is the ligature clump formed by the glyphs in glyph_span.
        // We will save these glyphs to the glyph store at the index of the first byte.
        let byte_idx = ByteIndex(byte_range.start as isize);

        if glyph_span.len() == 1 {
            // Fast path: 1-to-1 mapping of byte offset to single glyph.
            //
            // TODO(Issue #214): cluster ranges need to be computed before
            // shaping, and then consulted here.
            // for now, just pretend that every character is a cluster start.
            // (i.e., pretend there are no combining character sequences).
            // 1-to-1 mapping of character to glyph also treated as ligature start.
            //
            // NB: When we acquire the ability to handle ligatures that cross word boundaries,
            // we'll need to do something special to handle `word-spacing` properly.
            let character = text[byte_range.clone()].chars().next().unwrap();
            if is_bidi_control(character) {
                // Don't add any glyphs for bidi control chars
            } else {
                let (glyph_id, advance, offset) = if character == '\t' {
                    // Treat tabs in pre-formatted text as a fixed number of spaces. The glyph id does
                    // not matter here as Servo doesn't render any glyphs for whitespace.
                    //
                    // TODO: Proper tab stops. This should happen in layout and be based on the
                    // size of the space character of the inline formatting context.
                    (
                        font.glyph_index(' ').unwrap_or(0),
                        font.metrics.space_advance * 8,
                        Default::default(),
                    )
                } else {
                    let shape = glyph_data.entry_for_glyph(glyph_span.start, &mut y_pos);
                    let advance = advance_for_shaped_glyph(shape.advance, character, options);
                    (shape.codepoint, advance, shape.offset)
                };

                let data = GlyphData::new(glyph_id, advance, offset, true, true);
                glyphs.add_glyph_for_byte_index(byte_idx, character, &data);
            }
        } else {
            // collect all glyphs to be assigned to the first character.
            let mut datas = vec![];

            for glyph_i in glyph_span.clone() {
                let shape = glyph_data.entry_for_glyph(glyph_i, &mut y_pos);
                datas.push(GlyphData::new(
                    shape.codepoint,
                    shape.advance,
                    shape.offset,
                    true, // treat as cluster start
                    glyph_i > glyph_span.start,
                ));
                // all but first are ligature continuations
            }
            // now add the detailed glyph entry.
            glyphs.add_glyphs_for_byte_index(byte_idx, &datas);
        }

        glyph_span.start = glyph_span.end;
        byte_range.start = byte_range.end;
    }

    // this must be called after adding all glyph data; it sorts the
    // lookup table for finding detailed glyphs by associated char index.
    glyphs.finalize_changes();
}
