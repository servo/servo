/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cmp::Ordering;
use std::ops::Range;
use std::ptr::NonNull;
use std::{fmt, ptr};

/// Implementation of Quartz (CoreGraphics) fonts.
use app_units::Au;
use byteorder::{BigEndian, ByteOrder};
use euclid::default::{Point2D, Rect, Size2D};
use fonts_traits::{FontIdentifier, LocalFontIdentifier};
use log::debug;
use objc2_core_foundation::{
    CFData, CFDictionary, CFIndex, CFNumber, CFRetained, CFString, CFType,
};
use objc2_core_graphics::CGGlyph;
use objc2_core_text::{
    CTFont, CTFontOrientation, CTFontSymbolicTraits, CTFontTableOptions, CTFontTableTag,
    kCTFontSlantTrait, kCTFontSymbolicTrait, kCTFontWeightTrait, kCTFontWidthTrait,
};
use skrifa::Tag;
use style::values::computed::font::{FontStretch, FontStyle, FontWeight};
use webrender_api::{FontInstanceFlags, FontVariation};

use super::core_text_font_cache::CoreTextFontCache;
use crate::{
    CBDT, COLR, FontData, FontMetrics, FontTableMethods, FontTemplateDescriptor, FractionalPixel,
    GlyphId, KERN, PlatformFontMethods, SBIX, map_platform_values_to_style_values,
};

const KERN_PAIR_LEN: usize = 6;

#[derive(Clone)]
pub struct FontTable {
    data: CFRetained<CFData>,
}

// assumes 72 points per inch, and 96 px per inch
fn pt_to_px(pt: f64) -> f64 {
    pt / 72. * 96.
}

impl FontTable {
    pub(crate) fn wrap(data: CFRetained<CFData>) -> FontTable {
        FontTable { data }
    }
}

impl FontTableMethods for FontTable {
    fn buffer(&self) -> &[u8] {
        unsafe { self.data.as_bytes_unchecked() }
    }
}

#[derive(Clone, Debug)]
pub struct PlatformFont {
    pub(crate) ctfont: CFRetained<CTFont>,
    variations: Vec<FontVariation>,
    h_kern_subtable: Option<CachedKernTable>,
    synthetic_bold: bool,
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
    pub(crate) fn new_with_ctfont(ctfont: CFRetained<CTFont>, synthetic_bold: bool) -> Self {
        Self::new_with_ctfont_and_variations(ctfont, vec![], synthetic_bold)
    }

    #[expect(unsafe_code)]
    pub(crate) fn new_with_ctfont_and_variations(
        ctfont: CFRetained<CTFont>,
        variations: Vec<FontVariation>,
        synthetic_bold: bool,
    ) -> PlatformFont {
        let ctfont_is_bold = unsafe {
            ctfont
                .symbolic_traits()
                .contains(CTFontSymbolicTraits::TraitBold)
        };
        let is_variable_font =
            unsafe { ctfont.variation_axes().is_some_and(|arr| !arr.is_empty()) };

        let synthetic_bold = !ctfont_is_bold && !is_variable_font && synthetic_bold;

        Self {
            ctfont,
            variations,
            h_kern_subtable: None,
            synthetic_bold,
        }
    }

    fn new(
        font_identifier: FontIdentifier,
        data: Option<&FontData>,
        requested_size: Option<Au>,
        variations: &[FontVariation],
        synthetic_bold: bool,
    ) -> Result<PlatformFont, &'static str> {
        let size = match requested_size {
            Some(s) => s.to_f64_px(),
            None => 0.0,
        };
        let Some(mut platform_font) = CoreTextFontCache::core_text_font(
            font_identifier,
            data,
            size,
            variations,
            synthetic_bold,
        ) else {
            return Err("Could not generate CTFont for FontTemplateData");
        };

        platform_font.load_h_kern_subtable();
        Ok(platform_font)
    }

    /// Cache all the data needed for basic horizontal kerning. This is used only as a fallback or
    /// fast path (when the GPOS table is missing or unnecessary) so it needn't handle every case.
    #[expect(unsafe_code)]
    fn load_h_kern_subtable(&mut self) {
        if self.h_kern_subtable.is_some() {
            return;
        }

        let Some(font_table) = self.table_for_tag(KERN) else {
            return;
        };

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
                return;
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
                        return;
                    }
                    // Read the subtable header.
                    let subtable_start = start + SUBTABLE_HEADER_LEN;
                    let n_pairs = BigEndian::read_u16(&table[subtable_start..]) as usize;
                    let pair_data_start = subtable_start + FORMAT_0_HEADER_LEN;

                    result.pair_data_range = pair_data_start..end;
                    if result.pair_data_range.len() != n_pairs * KERN_PAIR_LEN {
                        debug!("Bad data in kern header. Disable fast path.");
                        return;
                    }

                    let pt_per_font_unit =
                        unsafe { self.ctfont.size() / self.ctfont.units_per_em() as f64 };
                    result.px_per_font_unit = pt_to_px(pt_per_font_unit);
                }
                start = end;
            }
        }

        if !result.pair_data_range.is_empty() {
            self.h_kern_subtable = Some(result);
        }
    }

    // This is adapted from WebRender glyph rasterizer for macos.
    // <https://github.com/servo/webrender/blob/main/wr_glyph_rasterizer/src/rasterizer.rs#L1006>
    fn get_extra_strikes(&self, strike_scale: f64) -> usize {
        if self.synthetic_bold {
            let mut bold_offset = unsafe { self.ctfont.size() } / 48.0;
            if bold_offset < 1.0 {
                bold_offset = 0.25 + 0.75 * bold_offset;
            }
            (bold_offset * strike_scale).max(1.0).round() as usize
        } else {
            0
        }
    }
}

#[derive(Clone)]
struct CachedKernTable {
    font_table: FontTable,
    pair_data_range: Range<usize>,
    px_per_font_unit: f64,
}

impl CachedKernTable {
    /// Search for a glyph pair in the kern table and return the corresponding value.
    fn binary_search(&self, first_glyph: GlyphId, second_glyph: GlyphId) -> Option<i16> {
        let pairs = &self.font_table.buffer()[self.pair_data_range.clone()];

        let query = (first_glyph << 16) | second_glyph;
        let (mut start, mut end) = (0, pairs.len() / KERN_PAIR_LEN);
        while start < end {
            let i = (start + end) / 2;
            let key = BigEndian::read_u32(&pairs[i * KERN_PAIR_LEN..]);
            match key.cmp(&query) {
                Ordering::Less => start = i + 1,
                Ordering::Equal => {
                    return Some(BigEndian::read_i16(&pairs[i * KERN_PAIR_LEN + 4..]));
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
        data: &FontData,
        requested_size: Option<Au>,
        variations: &[FontVariation],
        synthetic_bold: bool,
    ) -> Result<PlatformFont, &'static str> {
        Self::new(
            font_identifier,
            Some(data),
            requested_size,
            variations,
            synthetic_bold,
        )
    }

    fn new_from_local_font_identifier(
        font_identifier: LocalFontIdentifier,
        requested_size: Option<Au>,
        variations: &[FontVariation],
        synthetic_bold: bool,
    ) -> Result<PlatformFont, &'static str> {
        Self::new(
            FontIdentifier::Local(font_identifier),
            None,
            requested_size,
            variations,
            synthetic_bold,
        )
    }

    fn descriptor(&self) -> FontTemplateDescriptor {
        font_template_descriptor_from_ctfont_attributes(unsafe { self.ctfont.traits() })
    }

    fn glyph_index(&self, codepoint: char) -> Option<GlyphId> {
        // CTFontGetGlyphsForCharacters takes UniChar, which are UTF-16 encoded characters. We are taking
        // a char here which is a 32bit Unicode character. This will encode into a maximum of two
        // UTF-16 code units and produce a maximum of 1 glyph. We could safely pass 2 as the length
        // of the buffer to CTFontGetGlyphsForCharacters, but passing the actual number of encoded
        // code units ensures that the resulting glyph is always placed in the first slot in the output
        // buffer.
        let mut characters: [u16; 2] = [0, 0];
        let encoded_characters = codepoint.encode_utf16(&mut characters);
        let mut glyphs: [CGGlyph; 2] = [0, 0];

        let result = unsafe {
            self.ctfont.glyphs_for_characters(
                NonNull::from(&mut encoded_characters[0]),
                NonNull::from(&mut glyphs[0]),
                encoded_characters.len() as CFIndex,
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

    fn glyph_h_advance(&self, glyph: GlyphId) -> Option<FractionalPixel> {
        let mut advance = unsafe {
            self.ctfont.advances_for_glyphs(
                CTFontOrientation::Default,
                NonNull::from(&mut (glyph as CGGlyph)),
                ptr::null_mut(),
                1,
            )
        };

        // Adjust advance if synthetic bold is applied. Adapted from webrender rasterizer
        // See <https://github.com/servo/webrender/blob/main/wr_glyph_rasterizer/src/platform/macos/font.rs#L595>
        //
        // TODO: x_scale and y_scale should be adjusted accordingly once affine transform
        //      is added to achieve synthetic italic. For now, x_scale and y_scale would be
        //      set to just 1.0 with no affine transformations.
        let x_scale = 1.0;
        let y_scale = 1.0;
        let is_bitmap_font = self.table_for_tag(COLR).is_some() ||
            self.table_for_tag(CBDT).is_some() ||
            self.table_for_tag(SBIX).is_some();

        let (strike_scale, pixel_step) = if is_bitmap_font {
            (y_scale, 1.0)
        } else {
            (x_scale, y_scale / x_scale)
        };
        let extra_strikes = self.get_extra_strikes(strike_scale);
        if advance > 0.0 {
            advance += extra_strikes as f64 * pixel_step;
        }

        Some(advance as FractionalPixel)
    }

    fn metrics(&self) -> FontMetrics {
        // TODO(mrobinson): Gecko first tries to get metrics from the SFNT tables via
        // HarfBuzz and only afterward falls back to platform APIs. We should do something
        // similar here. This will likely address issue #201 mentioned below.
        let ascent = unsafe { self.ctfont.ascent() };
        let descent = unsafe { self.ctfont.descent() };
        let leading = unsafe { self.ctfont.leading() };
        let x_height = unsafe { self.ctfont.x_height() };
        let underline_thickness = unsafe { self.ctfont.underline_thickness() };
        let line_gap = (ascent + descent + leading + 0.5).floor();

        let max_advance = Au::from_f64_px(unsafe { self.ctfont.bounding_box().size.width });
        let zero_horizontal_advance = self
            .glyph_index('0')
            .and_then(|idx| self.glyph_h_advance(idx))
            .map(Au::from_f64_px);
        let average_advance = zero_horizontal_advance.unwrap_or(max_advance);

        let ic_horizontal_advance = self
            .glyph_index('\u{6C34}')
            .and_then(|idx| self.glyph_h_advance(idx))
            .map(Au::from_f64_px);
        let space_advance = self
            .glyph_index(' ')
            .and_then(|index| self.glyph_h_advance(index))
            .map(Au::from_f64_px)
            .unwrap_or(average_advance);

        let pt_size = unsafe { self.ctfont.size() };
        let metrics = FontMetrics {
            underline_size: Au::from_f64_px(underline_thickness),
            // TODO(Issue #201): underline metrics are not reliable. Have to pull out of font table
            // directly.
            //
            // see also: https://bugs.webkit.org/show_bug.cgi?id=16768
            // see also: https://bugreports.qt-project.org/browse/QTBUG-13364
            underline_offset: Au::from_f64_px(unsafe { self.ctfont.underline_position() }),
            // There is no way to get these from CoreText or CoreGraphics APIs, so
            // derive them from the other font metrics. These should eventually be
            // found in the font tables directly when #201 is fixed.
            strikeout_size: Au::from_f64_px(underline_thickness),
            strikeout_offset: Au::from_f64_px((x_height + underline_thickness) / 2.0),
            leading: Au::from_f64_px(leading),
            x_height: Au::from_f64_px(x_height),
            em_size: Au::from_f64_px(pt_size),
            ascent: Au::from_f64_px(ascent),
            descent: Au::from_f64_px(descent),
            max_advance,
            average_advance,
            line_gap: Au::from_f64_px(line_gap),
            zero_horizontal_advance,
            ic_horizontal_advance,
            space_advance,
        };
        debug!("Font metrics (@{pt_size} pt): {metrics:?}");
        metrics
    }

    fn table_for_tag(&self, tag: Tag) -> Option<FontTable> {
        let tag_u32 = u32::from_be_bytes(tag.to_be_bytes());
        unsafe {
            self.ctfont
                .table(tag_u32 as CTFontTableTag, CTFontTableOptions::NoOptions)
        }
        .map(FontTable::wrap)
    }

    /// Get the necessary [`FontInstanceFlags`]` for this font.
    fn webrender_font_instance_flags(&self) -> FontInstanceFlags {
        let mut flags = {
            // TODO: Should this also validate these tables?
            if self.table_for_tag(COLR).is_some() ||
                self.table_for_tag(CBDT).is_some() ||
                self.table_for_tag(SBIX).is_some()
            {
                FontInstanceFlags::EMBEDDED_BITMAPS
            } else {
                FontInstanceFlags::empty()
            }
        };

        if self.synthetic_bold {
            flags |= FontInstanceFlags::SYNTHETIC_BOLD;
        }

        flags
    }

    fn typographic_bounds(&self, glyph_id: GlyphId) -> Rect<f32> {
        let rect = unsafe {
            self.ctfont.bounding_rects_for_glyphs(
                CTFontOrientation::Default,
                NonNull::from(&mut (glyph_id as CGGlyph)),
                std::ptr::null_mut(),
                1,
            )
        };

        Rect::new(
            Point2D::new(rect.origin.x as f32, rect.origin.y as f32),
            Size2D::new(rect.size.width as f32, rect.size.height as f32),
        )
    }

    fn variations(&self) -> &[FontVariation] {
        &self.variations
    }
}

pub(crate) fn font_template_descriptor_from_ctfont_attributes(
    traits: CFRetained<CFDictionary>,
) -> FontTemplateDescriptor {
    let traits = unsafe { traits.cast_unchecked::<CFString, CFType>() };

    let get_f64_trait = |key| {
        traits
            .get(key)
            .and_then(|value| value.downcast::<CFNumber>().ok()?.as_f64())
    };

    // From https://developer.apple.com/documentation/coretext/kctfontweighttrait?language=objc
    // > The value returned is a CFNumberRef representing a float value between -1.0 and
    // > 1.0 for normalized weight. The value of 0.0 corresponds to the regular or
    // > medium font weight.
    let mapping = [(-1., 0.), (0., 400.), (1., 1000.)];
    let font_weight = get_f64_trait(unsafe { kCTFontWeightTrait }).unwrap_or(400.);
    let weight =
        FontWeight::from_float(map_platform_values_to_style_values(&mapping, font_weight) as f32);

    let font_slant = get_f64_trait(unsafe { kCTFontSlantTrait }).unwrap_or(0.);
    let style = if font_slant == 0. {
        let symbolic_traits = traits
            .get(unsafe { kCTFontSymbolicTrait })
            .and_then(|value| value.downcast::<CFNumber>().ok()?.as_i64())
            .map(|value| CTFontSymbolicTraits::from_bits_retain(value as u32));
        match symbolic_traits {
            Some(symbolic_traits)
                if symbolic_traits.contains(CTFontSymbolicTraits::TraitItalic) =>
            {
                FontStyle::ITALIC
            },
            _ => FontStyle::NORMAL,
        }
    } else {
        // From https://developer.apple.com/documentation/coretext/kctfontslanttrait?language=objc
        // > The value returned is a CFNumberRef object representing a float value
        // > between -1.0 and 1.0 for normalized slant angle. The value of 0.0
        // > corresponds to 0 degrees clockwise rotation from the vertical and 1.0
        // > corresponds to 30 degrees clockwise rotation.
        let mapping = [(-1., -30.), (0., 0.), (1., 30.)];
        FontStyle::oblique(map_platform_values_to_style_values(&mapping, font_slant) as f32)
    };

    // From https://developer.apple.com/documentation/coretext/kctfontwidthtrait?language=objc
    // > This value corresponds to the relative interglyph spacing for a given font.
    // > The value returned is a CFNumberRef object representing a float between -1.0
    // > and 1.0. The value of 0.0 corresponds to regular glyph spacing, and negative
    // > values represent condensed glyph spacing.
    let font_stretch = get_f64_trait(unsafe { kCTFontWidthTrait }).unwrap_or(0.);
    let stretch = FontStretch::from_percentage(font_stretch as f32 + 1.0);

    FontTemplateDescriptor::new(weight, stretch, style)
}
