/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cmp::Ordering;
use std::ops::Range;
use std::sync::Arc;
use std::{fmt, ptr};

/// Implementation of Quartz (CoreGraphics) fonts.
use app_units::Au;
use byteorder::{BigEndian, ByteOrder};
use core_foundation::base::CFIndex;
use core_foundation::data::CFData;
use core_foundation::string::UniChar;
use core_graphics::font::CGGlyph;
use core_graphics::geometry::CGRect;
use core_text::font::CTFont;
use core_text::font_descriptor::{
    kCTFontDefaultOrientation, SymbolicTraitAccessors, TraitAccessors,
};
use log::debug;
use style::values::computed::font::{FontStretch, FontStyle, FontWeight};

use crate::font::{
    FontHandleMethods, FontMetrics, FontTableMethods, FontTableTag, FractionalPixel, GPOS, GSUB,
    KERN,
};
use crate::font_cache_thread::FontIdentifier;
use crate::platform::font_template::FontTemplateData;
use crate::platform::macos::font_context::FontContextHandle;
use crate::text::glyph::GlyphId;

const KERN_PAIR_LEN: usize = 6;

pub struct FontTable {
    data: CFData,
}

// assumes 72 points per inch, and 96 px per inch
fn px_to_pt(px: f64) -> f64 {
    px / 96. * 72.
}

// assumes 72 points per inch, and 96 px per inch
fn pt_to_px(pt: f64) -> f64 {
    pt / 72. * 96.
}

fn au_from_pt(pt: f64) -> Au {
    Au::from_f64_px(pt_to_px(pt))
}

impl FontTable {
    pub fn wrap(data: CFData) -> FontTable {
        FontTable { data }
    }
}

impl FontTableMethods for FontTable {
    fn buffer(&self) -> &[u8] {
        self.data.bytes()
    }
}

#[derive(Debug)]
pub struct FontHandle {
    font_data: Arc<FontTemplateData>,
    ctfont: CTFont,
    h_kern_subtable: Option<CachedKernTable>,
    can_do_fast_shaping: bool,
}

impl FontHandle {
    /// Cache all the data needed for basic horizontal kerning. This is used only as a fallback or
    /// fast path (when the GPOS table is missing or unnecessary) so it needn't handle every case.
    fn find_h_kern_subtable(&self) -> Option<CachedKernTable> {
        let font_table = self.table_for_tag(KERN)?;
        let mut result = CachedKernTable {
            font_table,
            pair_data_range: 0..0,
            px_per_font_unit: 0.0,
        };

        // Look for a subtable with horizontal kerning in format 0.
        // https://www.microsoft.com/typography/otspec/kern.htm
        const KERN_COVERAGE_HORIZONTAL_FORMAT_0: u16 = 1;
        const SUBTABLE_HEADER_LEN: usize = 6;
        const FORMAT_0_HEADER_LEN: usize = 8;
        {
            let table = result.font_table.buffer();
            let version = BigEndian::read_u16(table);
            if version != 0 {
                return None;
            }
            let num_subtables = BigEndian::read_u16(&table[2..]);
            let mut start = 4;
            for _ in 0..num_subtables {
                // TODO: Check the subtable version number?
                let len = BigEndian::read_u16(&table[start + 2..]) as usize;
                let cov = BigEndian::read_u16(&table[start + 4..]);
                let end = start + len;
                if cov == KERN_COVERAGE_HORIZONTAL_FORMAT_0 {
                    // Found a matching subtable.
                    if !result.pair_data_range.is_empty() {
                        debug!("Found multiple horizontal kern tables. Disable fast path.");
                        return None;
                    }
                    // Read the subtable header.
                    let subtable_start = start + SUBTABLE_HEADER_LEN;
                    let n_pairs = BigEndian::read_u16(&table[subtable_start..]) as usize;
                    let pair_data_start = subtable_start + FORMAT_0_HEADER_LEN;

                    result.pair_data_range = pair_data_start..end;
                    if result.pair_data_range.len() != n_pairs * KERN_PAIR_LEN {
                        debug!("Bad data in kern header. Disable fast path.");
                        return None;
                    }

                    let pt_per_font_unit =
                        self.ctfont.pt_size() / self.ctfont.units_per_em() as f64;
                    result.px_per_font_unit = pt_to_px(pt_per_font_unit);
                }
                start = end;
            }
        }
        if !result.pair_data_range.is_empty() {
            Some(result)
        } else {
            None
        }
    }
}

struct CachedKernTable {
    font_table: FontTable,
    pair_data_range: Range<usize>,
    px_per_font_unit: f64,
}

impl CachedKernTable {
    /// Search for a glyph pair in the kern table and return the corresponding value.
    fn binary_search(&self, first_glyph: GlyphId, second_glyph: GlyphId) -> Option<i16> {
        let pairs = &self.font_table.buffer()[self.pair_data_range.clone()];

        let query = first_glyph << 16 | second_glyph;
        let (mut start, mut end) = (0, pairs.len() / KERN_PAIR_LEN);
        while start < end {
            let i = (start + end) / 2;
            let key = BigEndian::read_u32(&pairs[i * KERN_PAIR_LEN..]);
            match key.cmp(&query) {
                Ordering::Less => start = i + 1,
                Ordering::Equal => {
                    return Some(BigEndian::read_i16(&pairs[i * KERN_PAIR_LEN + 4..]))
                },
                Ordering::Greater => end = i,
            }
        }
        None
    }
}

impl fmt::Debug for CachedKernTable {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "CachedKernTable")
    }
}

impl FontHandleMethods for FontHandle {
    fn new_from_template(
        _fctx: &FontContextHandle,
        template: Arc<FontTemplateData>,
        pt_size: Option<Au>,
    ) -> Result<FontHandle, &'static str> {
        let size = match pt_size {
            Some(s) => s.to_f64_px(),
            None => 0.0,
        };
        match template.ctfont(size) {
            Some(ref ctfont) => {
                let mut handle = FontHandle {
                    font_data: template.clone(),
                    ctfont: ctfont.clone_with_font_size(size),
                    h_kern_subtable: None,
                    can_do_fast_shaping: false,
                };
                handle.h_kern_subtable = handle.find_h_kern_subtable();
                // TODO (#11310): Implement basic support for GPOS and GSUB.
                handle.can_do_fast_shaping = handle.h_kern_subtable.is_some() &&
                    handle.table_for_tag(GPOS).is_none() &&
                    handle.table_for_tag(GSUB).is_none();
                Ok(handle)
            },
            None => Err("Could not generate CTFont for FontTemplateData"),
        }
    }

    fn template(&self) -> Arc<FontTemplateData> {
        self.font_data.clone()
    }

    fn family_name(&self) -> Option<String> {
        Some(self.ctfont.family_name())
    }

    fn face_name(&self) -> Option<String> {
        Some(self.ctfont.face_name())
    }

    fn style(&self) -> FontStyle {
        if self.ctfont.symbolic_traits().is_italic() {
            FontStyle::ITALIC
        } else {
            FontStyle::NORMAL
        }
    }

    fn boldness(&self) -> FontWeight {
        let normalized = self.ctfont.all_traits().normalized_weight(); // [-1.0, 1.0]

        // TODO(emilio): It may make sense to make this range [.01, 10.0], to
        // align with css-fonts-4's range of [1, 1000].
        let normalized = if normalized <= 0.0 {
            4.0 + normalized * 3.0 // [1.0, 4.0]
        } else {
            4.0 + normalized * 5.0 // [4.0, 9.0]
        }; // [1.0, 9.0], centered on 4.0
        FontWeight::from_float(normalized as f32 * 100.)
    }

    fn stretchiness(&self) -> FontStretch {
        let normalized = self.ctfont.all_traits().normalized_width(); // [-1.0, 1.0]
        FontStretch::from_percentage(normalized as f32 + 1.0)
    }

    fn glyph_index(&self, codepoint: char) -> Option<GlyphId> {
        let characters: [UniChar; 1] = [codepoint as UniChar];
        let mut glyphs: [CGGlyph; 1] = [0 as CGGlyph];
        let count: CFIndex = 1;

        let result = unsafe {
            self.ctfont
                .get_glyphs_for_characters(characters.as_ptr(), glyphs.as_mut_ptr(), count)
        };

        if !result || glyphs[0] == 0 {
            // No glyph for this character
            return None;
        }

        Some(glyphs[0] as GlyphId)
    }

    fn glyph_h_kerning(&self, first_glyph: GlyphId, second_glyph: GlyphId) -> FractionalPixel {
        if let Some(ref table) = self.h_kern_subtable {
            if let Some(font_units) = table.binary_search(first_glyph, second_glyph) {
                return font_units as f64 * table.px_per_font_unit;
            }
        }
        0.0
    }

    fn can_do_fast_shaping(&self) -> bool {
        self.can_do_fast_shaping
    }

    fn glyph_h_advance(&self, glyph: GlyphId) -> Option<FractionalPixel> {
        let glyphs = [glyph as CGGlyph];
        let advance = unsafe {
            self.ctfont.get_advances_for_glyphs(
                kCTFontDefaultOrientation,
                &glyphs[0],
                ptr::null_mut(),
                1,
            )
        };
        Some(advance as FractionalPixel)
    }

    fn metrics(&self) -> FontMetrics {
        let bounding_rect: CGRect = self.ctfont.bounding_box();
        let ascent = self.ctfont.ascent();
        let descent = self.ctfont.descent();
        let em_size = Au::from_f64_px(self.ctfont.pt_size());
        let leading = self.ctfont.leading();

        let scale = px_to_pt(self.ctfont.pt_size()) / (ascent + descent);
        let line_gap = (ascent + descent + leading + 0.5).floor();

        let max_advance = au_from_pt(bounding_rect.size.width);
        let average_advance = self
            .glyph_index('0')
            .and_then(|idx| self.glyph_h_advance(idx))
            .map(Au::from_f64_px)
            .unwrap_or(max_advance);

        let metrics = FontMetrics {
            underline_size: au_from_pt(self.ctfont.underline_thickness()),
            // TODO(Issue #201): underline metrics are not reliable. Have to pull out of font table
            // directly.
            //
            // see also: https://bugs.webkit.org/show_bug.cgi?id=16768
            // see also: https://bugreports.qt-project.org/browse/QTBUG-13364
            underline_offset: au_from_pt(self.ctfont.underline_position()),
            strikeout_size: Au(0),   // FIXME(Issue #942)
            strikeout_offset: Au(0), // FIXME(Issue #942)
            leading: au_from_pt(leading),
            x_height: au_from_pt(self.ctfont.x_height() * scale),
            em_size,
            ascent: au_from_pt(ascent * scale),
            descent: au_from_pt(descent * scale),
            max_advance,
            average_advance,
            line_gap: Au::from_f64_px(line_gap),
        };
        debug!(
            "Font metrics (@{} pt): {:?}",
            self.ctfont.pt_size(),
            metrics
        );
        metrics
    }

    fn table_for_tag(&self, tag: FontTableTag) -> Option<FontTable> {
        let result: Option<CFData> = self.ctfont.get_font_table(tag);
        result.map(FontTable::wrap)
    }

    fn identifier(&self) -> &FontIdentifier {
        &self.font_data.identifier
    }
}
