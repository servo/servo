/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::ptr;
use std::ptr::NonNull;
use std::sync::Arc;

use app_units::Au;
use euclid::default::{Point2D, Rect, Size2D};
use fonts_traits::tag::{CBDT, COLR, SBIX};
use fonts_traits::{
    FontIdentifier, FontTemplateDescriptor, FontTemplateRef, LocalFontIdentifier, SkrifaFont,
};
use log::debug;
use objc2_core_foundation::{
    CFData, CFDictionary, CFIndex, CFNumber, CFRange, CFRetained, CFString, CFType,
};
use objc2_core_graphics::CGGlyph;
use objc2_core_text::{
    CTFont, CTFontOrientation, CTFontSymbolicTraits, CTFontTableOptions, CTFontTableTag,
    kCTFontSlantTrait, kCTFontSymbolicTrait, kCTFontWeightTrait, kCTFontWidthTrait,
};
use skrifa::Tag;
use skrifa::font::{Blob, TableFunction};
use style::font_face::{FontStyle, FontWidth};
use style::values::computed::FontWeight;
use webrender_api::{FontInstanceFlags, FontVariation};

use super::core_text_font_cache::CoreTextFontCache;
use crate::font::{FontTableMethods, FractionalPixel, map_platform_values_to_style_values};
use crate::font_context::SkrifaFontCache;
use crate::platform::font_list::font_template_for_local_font_descriptor;
use crate::{
    FallbackFontSelectionOptions, Font, FontData, FontMetrics, FontRef, GlyphId,
    PlatformFontMethods,
};

#[derive(Clone)]
pub struct FontTable {
    data: CFRetained<CFData>,
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
    pub(crate) synthetic_bold: bool,
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
    pub(crate) fn new_with_ctfont(ctfont: CFRetained<CTFont>) -> Self {
        Self::new_with_ctfont_and_variations(ctfont, vec![])
    }

    pub(crate) fn new_with_ctfont_and_variations(
        ctfont: CFRetained<CTFont>,
        variations: Vec<FontVariation>,
    ) -> PlatformFont {
        Self {
            ctfont,
            variations,
            synthetic_bold: false,
        }
    }

    fn new(
        font_identifier: FontIdentifier,
        data: Option<&FontData>,
        requested_size: Option<Au>,
    ) -> Result<PlatformFont, &'static str> {
        let size = match requested_size {
            Some(s) => s.to_f64_px(),
            None => 0.0,
        };
        CoreTextFontCache::core_text_font(font_identifier, data, size)
            .ok_or("Could not generate CTFont for FontTemplateData")
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

impl PlatformFontMethods for PlatformFont {
    fn new_from_data(
        font_identifier: FontIdentifier,
        data: &FontData,
        requested_size: Option<Au>,
    ) -> Result<PlatformFont, &'static str> {
        Self::new(font_identifier, Some(data), requested_size)
    }

    fn new_from_local_font_identifier(
        font_identifier: LocalFontIdentifier,
        requested_size: Option<Au>,
    ) -> Result<PlatformFont, &'static str> {
        Self::new(FontIdentifier::Local(font_identifier), None, requested_size)
    }

    fn copy_with_variations(
        self,
        font_identifier: &FontIdentifier,
        variations: &[FontVariation],
    ) -> Result<Self, &'static str> {
        Ok(CoreTextFontCache::add_variations_to_font(
            self,
            font_identifier,
            variations,
        ))
    }

    fn copy_with_synthetic_bold(mut self) -> Result<Self, &'static str> {
        self.synthetic_bold = true;
        Ok(self)
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

    fn create_skrifa_font(&self) -> Option<Arc<SkrifaFont>> {
        let font = ThreadSafeCTFont {
            font: self.ctfont.clone(),
        };
        let table_function = TableFunction::new(Arc::new(move |tag: Tag| -> Option<Blob> {
            font.copy_table(tag)
        }));
        SkrifaFont::new(
            table_function,
            0, /* This value is ignored, for a table function */
        )
    }
}

struct ThreadSafeCTFont {
    font: CFRetained<CTFont>,
}

impl ThreadSafeCTFont {
    fn copy_table(&self, tag: Tag) -> Option<Blob> {
        let tag_u32 = u32::from_be_bytes(tag.to_be_bytes());
        // SAFETY: This should always be safe because
        let data = unsafe {
            self.font
                .table(tag_u32 as CTFontTableTag, CTFontTableOptions::NoOptions)
        }?;
        Some(Blob::Shared(Arc::new(ThreadSafeCFData { data })))
    }
}

// From https://developer.apple.com/documentation/coretext:
// > All individual functions in CoreText are thread-safe. Font objects (CTFont,
// > CTFontDescriptor, and associated objects) can be used simultaneously by multiple
// > operations, work queues, or threads. However, the layout objects (CTTypesetter,
// > CTFramesetter, CTRun, CTLine, CTFrame, and associated objects) should be used in a
// > single operation, work queue, or thread.
unsafe impl Send for ThreadSafeCTFont {}
unsafe impl Sync for ThreadSafeCTFont {}

struct ThreadSafeCFData {
    data: CFRetained<CFData>,
}

// SAFETY: The slice returned from a CFData must not be mutated while the slice is alive.
// Data returned from CTFontCopyTable is immutable.
unsafe impl Send for ThreadSafeCFData {}
unsafe impl Sync for ThreadSafeCFData {}

impl AsRef<[u8]> for ThreadSafeCFData {
    fn as_ref(&self) -> &[u8] {
        // SAFETY: The slice returned from a CFData must not be mutated while the slice is alive.
        // Data returned from CTFontCopyTable is immutable.
        unsafe { self.data.as_bytes_unchecked() }
    }
}

impl Font {
    pub(crate) fn find_fallback_using_system_font_api(
        &self,
        options: &FallbackFontSelectionOptions,
        skrifa_font_cache: &SkrifaFontCache,
    ) -> Option<FontRef> {
        // TODO: The CoreText API is taking a UTS #35 string for the language value, and
        // the value stored in the HTML lang attribute is a BCP 47 language tag. These two
        // formats are generally compatible, but we may need to make refinements here in
        // the future.
        let language = if !options.language.is_unknown() {
            Some(&*CFString::from_str(options.language.as_str()))
        } else {
            None
        };
        let string = CFString::from_str(&options.character.to_string());
        let font = unsafe {
            self.handle.ctfont.for_string_with_language(
                &string,
                CFRange::new(0, string.length()),
                language,
            )
        };

        let template = unsafe { font_template_for_local_font_descriptor(font.font_descriptor())? };
        let template = FontTemplateRef::new(template);
        Some(FontRef(Arc::new(
            Font::new(
                template,
                self.descriptor.clone(),
                None,
                None,
                skrifa_font_cache,
            )
            .ok()?,
        )))
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
    let font_width = get_f64_trait(unsafe { kCTFontWidthTrait }).unwrap_or(0.);
    let width = FontWidth::from_percentage(font_width as f32 + 1.0);

    FontTemplateDescriptor::new(weight, width, style)
}
