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
use core_foundation::data::CFData;
use core_foundation::string::UniChar;
use core_graphics::font::CGGlyph;
use core_text::font::CTFont;
use core_text::font_descriptor::{
    kCTFontDefaultOrientation, CTFontTraits, SymbolicTraitAccessors, TraitAccessors,
};
use euclid::default::{Point2D, Rect, Size2D};
use log::debug;
use style::values::computed::font::{FontStretch, FontStyle, FontWeight};
use webrender_api::FontInstanceFlags;

use super::core_text_font_cache::CoreTextFontCache;
use crate::{
    map_platform_values_to_style_values, FontIdentifier, FontMetrics, FontTableMethods,
    FontTableTag, FontTemplateDescriptor, FractionalPixel, GlyphId, PlatformFontMethods, CBDT,
    COLR, GPOS, GSUB, KERN, SBIX,
};

const KERN_PAIR_LEN: usize = 6;

pub struct FontTable {
    data: CFData,
}

// assumes 72 points per inch, and 96 px per inch
fn pt_to_px(pt: f64) -> f64 {
    pt / 72. * 96.
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
pub struct PlatformFont {
    ctfont: CTFont,
    /// A reference to this data used to create this [`PlatformFont`], ensuring the
    /// data stays alive of the lifetime of this struct.
    _data: Arc<Vec<u8>>,
    h_kern_subtable: Option<CachedKernTable>,
    can_do_fast_shaping: bool,
}

// From https://developer.apple.com/documentation/coretext:
// > All individual functions in Core Text are thread-safe. Font objects (CTFont,
// > CTFontDescriptor, and associated objects) can be used simultaneously by multiple
// > operations, work queues, or threads. However, the layout objects (CTTypesetter,
// > CTFramesetter, CTRun, CTLine, CTFrame, and associated objects) should be used in a
// > single operation, work queue, or thread.
//
// The other element is a read-only CachedKernTable which is stored in a CFData.
unsafe impl Sync for PlatformFont {}
unsafe impl Send for PlatformFont {}

impl PlatformFont {
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

impl PlatformFontMethods for PlatformFont {
    fn new_from_data(
        font_identifier: FontIdentifier,
        data: Arc<Vec<u8>>,
        _face_index: u32,
        pt_size: Option<Au>,
    ) -> Result<PlatformFont, &'static str> {
        let size = match pt_size {
            Some(s) => s.to_f64_px(),
            None => 0.0,
        };
        let Some(core_text_font) =
            CoreTextFontCache::core_text_font(font_identifier, data.clone(), size)
        else {
            return Err("Could not generate CTFont for FontTemplateData");
        };

        let mut handle = PlatformFont {
            _data: data,
            ctfont: core_text_font.clone_with_font_size(size),
            h_kern_subtable: None,
            can_do_fast_shaping: false,
        };
        handle.h_kern_subtable = handle.find_h_kern_subtable();
        handle.can_do_fast_shaping = handle.h_kern_subtable.is_some() &&
            handle.table_for_tag(GPOS).is_none() &&
            handle.table_for_tag(GSUB).is_none();
        Ok(handle)
    }

    fn descriptor(&self) -> FontTemplateDescriptor {
        let traits = self.ctfont.all_traits();
        FontTemplateDescriptor::new(traits.weight(), traits.stretch(), traits.style())
    }

    fn glyph_index(&self, codepoint: char) -> Option<GlyphId> {
        // CTFontGetGlyphsForCharacters takes UniChar, which are UTF-16 encoded characters. We are taking
        // a char here which is a 32bit Unicode character. This will encode into a maximum of two
        // UTF-16 code units and produce a maximum of 1 glyph. We could safely pass 2 as the length
        // of the buffer to CTFontGetGlyphsForCharacters, but passing the actual number of encoded
        // code units ensures that the resulting glyph is always placed in the first slot in the output
        // buffer.
        let mut characters: [UniChar; 2] = [0, 0];
        let encoded_characters = codepoint.encode_utf16(&mut characters);
        let mut glyphs: [CGGlyph; 2] = [0, 0];

        let result = unsafe {
            self.ctfont.get_glyphs_for_characters(
                encoded_characters.as_ptr(),
                glyphs.as_mut_ptr(),
                encoded_characters.len() as isize,
            )
        };

        // If the call failed or the glyph is the zero glyph no glyph was found for this character.
        if !result || glyphs[0] == 0 {
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
        // TODO(mrobinson): Gecko first tries to get metrics from the SFNT tables via
        // HarfBuzz and only afterward falls back to platform APIs. We should do something
        // similar here. This will likely address issue #201 mentioned below.
        let ascent = self.ctfont.ascent();
        let descent = self.ctfont.descent();
        let leading = self.ctfont.leading();
        let x_height = self.ctfont.x_height();
        let underline_thickness = self.ctfont.underline_thickness();
        let line_gap = (ascent + descent + leading + 0.5).floor();

        let max_advance = Au::from_f64_px(self.ctfont.bounding_box().size.width);
        let zero_horizontal_advance = self
            .glyph_index('0')
            .and_then(|idx| self.glyph_h_advance(idx))
            .map(Au::from_f64_px);
        let ic_horizontal_advance = self
            .glyph_index('\u{6C34}')
            .and_then(|idx| self.glyph_h_advance(idx))
            .map(Au::from_f64_px);
        let average_advance = zero_horizontal_advance.unwrap_or(max_advance);

        let metrics = FontMetrics {
            underline_size: Au::from_f64_au(underline_thickness),
            // TODO(Issue #201): underline metrics are not reliable. Have to pull out of font table
            // directly.
            //
            // see also: https://bugs.webkit.org/show_bug.cgi?id=16768
            // see also: https://bugreports.qt-project.org/browse/QTBUG-13364
            underline_offset: Au::from_f64_px(self.ctfont.underline_position()),
            // There is no way to get these from CoreText or CoreGraphics APIs, so
            // derive them from the other font metrics. These should eventually be
            // found in the font tables directly when #201 is fixed.
            strikeout_size: Au::from_f64_px(underline_thickness),
            strikeout_offset: Au::from_f64_px((x_height + underline_thickness) / 2.0),
            leading: Au::from_f64_px(leading),
            x_height: Au::from_f64_px(x_height),
            em_size: Au::from_f64_px(self.ctfont.pt_size()),
            ascent: Au::from_f64_px(ascent),
            descent: Au::from_f64_px(descent),
            max_advance,
            average_advance,
            line_gap: Au::from_f64_px(line_gap),
            zero_horizontal_advance,
            ic_horizontal_advance,
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

    /// Get the necessary [`FontInstanceFlags`]` for this font.
    fn webrender_font_instance_flags(&self) -> FontInstanceFlags {
        // TODO: Should this also validate these tables?
        if self.table_for_tag(COLR).is_some() ||
            self.table_for_tag(CBDT).is_some() ||
            self.table_for_tag(SBIX).is_some()
        {
            return FontInstanceFlags::EMBEDDED_BITMAPS;
        }
        FontInstanceFlags::empty()
    }

    fn typographic_bounds(&self, glyph_id: GlyphId) -> Rect<f32> {
        let rect = self
            .ctfont
            .get_bounding_rects_for_glyphs(kCTFontDefaultOrientation, &[glyph_id as u16]);
        Rect::new(
            Point2D::new(rect.origin.x as f32, rect.origin.y as f32),
            Size2D::new(rect.size.width as f32, rect.size.height as f32),
        )
    }
}

pub(super) trait CoreTextFontTraitsMapping {
    fn weight(&self) -> FontWeight;
    fn style(&self) -> FontStyle;
    fn stretch(&self) -> FontStretch;
}

impl CoreTextFontTraitsMapping for CTFontTraits {
    fn weight(&self) -> FontWeight {
        // From https://developer.apple.com/documentation/coretext/kctfontweighttrait?language=objc
        // > The value returned is a CFNumberRef representing a float value between -1.0 and
        // > 1.0 for normalized weight. The value of 0.0 corresponds to the regular or
        // > medium font weight.
        let mapping = [(-1., 0.), (0., 400.), (1., 1000.)];

        let mapped_weight = map_platform_values_to_style_values(&mapping, self.normalized_weight());
        FontWeight::from_float(mapped_weight as f32)
    }

    fn style(&self) -> FontStyle {
        let slant = self.normalized_slant();
        if slant == 0. && self.symbolic_traits().is_italic() {
            return FontStyle::ITALIC;
        }
        if slant == 0. {
            return FontStyle::NORMAL;
        }

        // From https://developer.apple.com/documentation/coretext/kctfontslanttrait?language=objc
        // > The value returned is a CFNumberRef object representing a float value
        // > between -1.0 and 1.0 for normalized slant angle. The value of 0.0
        // > corresponds to 0 degrees clockwise rotation from the vertical and 1.0
        // > corresponds to 30 degrees clockwise rotation.
        let mapping = [(-1., -30.), (0., 0.), (1., 30.)];
        let mapped_slant = map_platform_values_to_style_values(&mapping, slant);
        FontStyle::oblique(mapped_slant as f32)
    }

    fn stretch(&self) -> FontStretch {
        // From https://developer.apple.com/documentation/coretext/kctfontwidthtrait?language=objc
        // > This value corresponds to the relative interglyph spacing for a given font.
        // > The value returned is a CFNumberRef object representing a float between -1.0
        // > and 1.0. The value of 0.0 corresponds to regular glyph spacing, and negative
        // > values represent condensed glyph spacing.
        FontStretch::from_percentage(self.normalized_width() as f32 + 1.0)
    }
}
