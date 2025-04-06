/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::error::Error;
use std::ffi::{CString, NulError};
use std::ops::RangeInclusive;
use std::os::raw::c_long;
use std::{mem, ptr};

use app_units::Au;
use euclid::default::{Point2D, Rect, Size2D};
use freetype_sys::{
    FT_Byte, FT_Done_Face, FT_Err_Invalid_Argument, FT_Error, FT_F26Dot6, FT_FACE_FLAG_COLOR,
    FT_FACE_FLAG_FIXED_SIZES, FT_FACE_FLAG_MULTIPLE_MASTERS, FT_FACE_FLAG_SCALABLE, FT_Face,
    FT_Fixed, FT_Get_Char_Index, FT_Get_Kerning, FT_Get_MM_Var, FT_Get_Sfnt_Table, FT_GlyphSlot,
    FT_Int32, FT_KERNING_DEFAULT, FT_LOAD_COLOR, FT_LOAD_DEFAULT, FT_LOAD_NO_HINTING,
    FT_Load_Glyph, FT_Long, FT_MM_Var, FT_MulFix, FT_New_Face, FT_New_Memory_Face, FT_Pos,
    FT_STYLE_FLAG_ITALIC, FT_Select_Size, FT_Set_Char_Size, FT_Short, FT_Size_Metrics, FT_SizeRec,
    FT_UInt, FT_UInt32, FT_UInt64, FT_ULong, FT_UShort, FT_Var_Axis, FT_Vector, FTErrorMethods,
    TT_OS2, ft_sfnt_head, ft_sfnt_os2, FT_Get_Sfnt_Name, FT_SfntName, FT_Err_Ok, FT_Done_MM_Var,
    FT_LibraryRec, FT_Library
};
use icu_locid::LanguageIdentifier;
use icu_locid::subtags::Language;
use log::{debug, error, log_enabled, warn};
use parking_lot::ReentrantMutex;
use style::Zero;
use style::computed_values::font_stretch::T as FontStretch;
use style::computed_values::font_weight::T as FontWeight;
use style::values::computed::font::FontStyle;
use style::values::generics::Optional;
use unicode_language::{Match as LanguageMatch, detect};
use webrender_api::FontInstanceFlags;

use super::LocalFontIdentifier;
use super::library_handle::FreeTypeLibraryHandle;
use crate::font::{
    FontMetrics, FontTableMethods, FontTableTag, FractionalPixel, PlatformFontMethods,
};
use crate::font_template::FontTemplateDescriptor;
use crate::glyph::GlyphId;
use crate::platform::freetype::freetype_errors::CustomFtErrorMethods;
use crate::platform::freetype::freetype_truetype_unicode_ranges::convert_unicode_ranges;
use crate::system_font_service::FontIdentifier;
use crate::FontData;

// This constant is not present in the freetype
// bindings due to bindgen not handling the way
// the macro is defined.
const FT_LOAD_TARGET_LIGHT: FT_UInt = 1 << 16;

/// Convert FreeType-style 26.6 fixed point to an [`f64`].
fn fixed_26_dot_6_to_float(fixed: FT_F26Dot6) -> f64 {
    fixed as f64 / 64.0
}

#[derive(Debug)]
pub struct FontTable {
    buffer: Vec<u8>,
}

impl FontTableMethods for FontTable {
    fn buffer(&self) -> &[u8] {
        &self.buffer
    }
}

/// Data from the OS/2 table of an OpenType font.
/// See <https://www.microsoft.com/typography/otspec/os2.htm>
#[derive(Debug)]
struct OS2Table {
    version: FT_UShort,
    x_average_char_width: FT_Short,
    us_weight_class: FT_UShort,
    us_width_class: FT_UShort,
    y_strikeout_size: FT_Short,
    y_strikeout_position: FT_Short,
    // According to specs OS/2 unicode ranges should be FT_UInt32.
    // <https://learn.microsoft.com/en-us/typography/opentype/spec/os2>
    // <https://developer.apple.com/fonts/TrueType-Reference-Manual/RM06/Chap6OS2.html>
    // However freetype choses FT_UInt64. Understand why?
    // https://freetype.org/freetype2/docs/reference/ft2-truetype_tables.html#tt_os2
    ul_unicode_range1: FT_UInt64,
    ul_unicode_range2: FT_UInt64,
    ul_unicode_range3: FT_UInt64,
    ul_unicode_range4: FT_UInt64,
    sx_height: FT_Short,
}

#[derive(Debug)]
pub struct PlatformFontFaceHandle {
    face: FT_Face,
    variation_axes: *mut FT_MM_Var
}

// TODO(ddesyatkin): Should we store platform related information in CSS Au units?
// Maybe we should have better separation of abstractions layers...
#[derive(Debug)]
#[allow(unused)]
pub struct PlatformFont {
    face_handle: ReentrantMutex<PlatformFontFaceHandle>,
    requested_face_size: Au,
    actual_face_size: Au,
}

// FT_Face can be used in multiple threads, but from only one thread at a time.
// It's protected with a ReentrantMutex for PlatformFont.
// See https://freetype.org/freetype2/docs/reference/ft2-face_creation.html#ft_face.
unsafe impl Sync for PlatformFont {}
unsafe impl Send for PlatformFont {}

impl Drop for PlatformFont {
    fn drop(&mut self) {
        let face_handle = self.face_handle.lock();
        let face = face_handle.face;
        let var_axes = face_handle.variation_axes;
        assert!(!face.is_null());
        unsafe {
            // The FreeType documentation says that both `FT_New_Face` and `FT_Done_Face`
            // should be protected by a mutex.
            // See https://freetype.org/freetype2/docs/reference/ft2-library_setup.html.
            let ft_library_handle = FreeTypeLibraryHandle::get().lock();
            if !var_axes.is_null() {
                if FT_Done_MM_Var(ft_library_handle.freetype_library, var_axes) != 0 {
                    panic!("FT_Done_MM_Var failed");
                }
            }
            if FT_Done_Face(face) != 0 {
                panic!("FT_Done_Face failed");
            }
        }
    }
}

impl PlatformFontMethods for PlatformFont {
    fn new_from_data(
        _font_identifier: FontIdentifier,
        font_data: &FontData,
        requested_size: Option<Au>,
    ) -> Result<PlatformFont, &'static str> {
        let library = FreeTypeLibraryHandle::get().lock();
        let data: &[u8] = font_data.as_ref();
        let mut face: FT_Face = ptr::null_mut();
        let result = unsafe {
            FT_New_Memory_Face(
                library.freetype_library,
                data.as_ptr(),
                data.len() as FT_Long,
                0, /* face_index */
                &mut face,
            )
        };

        if 0 != result || face.is_null() {
            // We want to have more logs on OpenHarmony,
            // Linux and Android must not be affected;
            #[cfg(any(target_env = "ohos", ohos_mock))]
            {
                let ft_error_code: FT_Error = result as FT_Error;
                let error_string = ft_error_code.ft_get_error_message();
                log::error!("Could not create FreeType face. FT_error: {}", error_string);
            }
            return Err("Could not create FreeType face");
        }

        let mut variation_axes: *mut FT_MM_Var = ptr::null_mut();
        let result: FT_Error = unsafe { FT_Get_MM_Var(face, &mut variation_axes) };
        if !result.succeeded() || variation_axes.is_null() {
            let error_string = result.ft_get_error_message();
            log::error!(
                "We was not able to setup variation axis. FT_error: {}",
                error_string
            );
        }

        let face_handle = PlatformFontFaceHandle {
            face,
            variation_axes
        };

        let (requested_face_size, actual_face_size) = match requested_size {
            Some(requested_size) => (requested_size, face_handle.set_size(requested_size)?),
            None => (Au::zero(), Au::zero()),
        };

        if face_handle.has_axes() {
            log::warn!("ddesyatkin: Attempt to set font weight.");
            let result: FT_Error = face_handle.set_weight(app_units::Au(0));
            if !result.succeeded() {
                log::error!(
                    "Error on face variation axis setup. FT_error: {:?} \
                    Program will not be interrupted, but face will come with default variations",
                    result.ft_get_error_message()
                );
            }
        };

        Ok(PlatformFont {
            face_handle: ReentrantMutex::new(face_handle),
            requested_face_size,
            actual_face_size,
        })
    }

    fn new_from_local_font_identifier(
        font_identifier: LocalFontIdentifier,
        requested_size: Option<Au>,
    ) -> Result<PlatformFont, &'static str> {
        let mut face: FT_Face = ptr::null_mut();
        let library = FreeTypeLibraryHandle::get().lock();
        let filename = match CString::new(&*font_identifier.path) {
            Err(e) => {
                let e: NulError = e;
                // We want to have more logs on OpenHarmony,
                // Linux and Android must not be affected;
                #[cfg(any(target_env = "ohos", ohos_mock))]
                {
                    log::error!("{}\nCaused by: {}", e, e.source().unwrap());
                }
                return Err("filename contains NUL byte!");
            },
            Ok(data) => data,
        };

        let result: FT_Error = unsafe {
            FT_New_Face(
                library.freetype_library,
                filename.as_ptr(),
                font_identifier.index() as FT_Long,
                &mut face,
            )
        };

        if !result.succeeded() || face.is_null() {
            // We want to have more logs on OpenHarmony,
            // Linux and Android must not be affected;
            #[cfg(any(target_env = "ohos", ohos_mock))]
            {
                log::error!(
                    "Could not create FreeType face. FT_error: {}",
                    result.ft_get_error_message()
                );
            }
            return Err("Could not create FreeType face");
        }

        let mut variation_axes: *mut FT_MM_Var = ptr::null_mut();
        let result: FT_Error = unsafe { FT_Get_MM_Var(face, &mut variation_axes) };
        if !result.succeeded() || variation_axes.is_null() {
            let error_string = result.ft_get_error_message();
            log::error!(
                "We was not able to setup variation axis. FT_error: {}",
                error_string
            );
        }

        let face_handle = PlatformFontFaceHandle {
            face,
            variation_axes
        };

        let (requested_face_size, actual_face_size) = match requested_size {
            Some(requested_size) => (requested_size, face_handle.set_size(requested_size)?),
            None => (Au::zero(), Au::zero()),
        };

        if face_handle.has_axes() {
            log::warn!("ddesyatkin: Attempt to set font weight.");
            let result: FT_Error = face_handle.set_weight(app_units::Au(0));
            if !result.succeeded() {
                log::error!(
                    "Error on face variation axis setup. FT_error: {:?} \
                    Program will not be interrupted, but face will come with default variations",
                    result.ft_get_error_message()
                );
            }
        };

        Ok(PlatformFont {
            face_handle: ReentrantMutex::new(face_handle),
            requested_face_size,
            actual_face_size,
        })
    }

    fn descriptor(&self) -> FontTemplateDescriptor {
        let face_handle = self.face_handle.lock();
        let face = face_handle.face;
        let style = if unsafe { (*face).style_flags & FT_STYLE_FLAG_ITALIC as c_long != 0 } {
            FontStyle::ITALIC
        } else {
            FontStyle::NORMAL
        };

        let os2_table = face_handle.os2_table();
        let weight = os2_table
            .as_ref()
            .map(|os2| {
                log::warn!(
                    "ddesyatkin: Freetype us_weight_class {}",
                    os2.us_weight_class as f32
                );
                FontWeight::from_float(os2.us_weight_class as f32)
            })
            .unwrap_or_else(FontWeight::normal);
        let stretch = os2_table
            .as_ref()
            .map(|os2| match os2.us_width_class {
                1 => FontStretch::ULTRA_CONDENSED,
                2 => FontStretch::EXTRA_CONDENSED,
                3 => FontStretch::CONDENSED,
                4 => FontStretch::SEMI_CONDENSED,
                5 => FontStretch::NORMAL,
                6 => FontStretch::SEMI_EXPANDED,
                7 => FontStretch::EXPANDED,
                8 => FontStretch::EXTRA_EXPANDED,
                9 => FontStretch::ULTRA_EXPANDED,
                _ => FontStretch::NORMAL,
            })
            .unwrap_or(FontStretch::NORMAL);

        let unicode_ranges = os2_table
            .as_ref()
            .map(|os2| {
                Some(convert_unicode_ranges(
                    os2.ul_unicode_range1,
                    os2.ul_unicode_range2,
                    os2.ul_unicode_range3,
                    os2.ul_unicode_range4,
                ))
            })
            .unwrap_or(None);

        // Constant bellow decides the required overlap of ranges required by language
        // and ranges supported by font.
        // Check unicode_language crate
        // TODO(ddesyatkin): Choose correct constant here.
        const LANGUAGE_DETECTION_THRESHOLD: f64 = 0.8;
        let language_matches = unicode_language::detect(
            unicode_ranges
                .as_ref()
                .unwrap_or(&Vec::<RangeInclusive<u32>>::new())
                .iter()
                .map(|unicode_range| [unicode_range.start().clone(), unicode_range.end().clone()])
                .collect::<Vec<[u32; 2]>>(),
            LANGUAGE_DETECTION_THRESHOLD,
        );

        let languages: Vec<LanguageIdentifier> = language_matches
            .into_iter()
            .filter_map(|language_match: LanguageMatch| {
                Some(language_match.tag.parse::<Language>().ok()?)
            })
            .map(|language| LanguageIdentifier::from(language))
            .collect();

        FontTemplateDescriptor {
            weight: (weight, weight),
            stretch: (stretch, stretch),
            style: (style, style),
            languages: languages.into(),
            unicode_range: unicode_ranges,
        }
    }

    fn glyph_index(&self, codepoint: char) -> Option<GlyphId> {
        let face_handle = self.face_handle.lock();
        let face = face_handle.face;
        assert!(!face.is_null());

        unsafe {
            let idx = FT_Get_Char_Index(face, codepoint as FT_ULong);
            if idx != 0 as FT_UInt {
                Some(idx as GlyphId)
            } else {
                debug!(
                    "Invalid codepoint: U+{:04X} ('{}')",
                    codepoint as u32, codepoint
                );
                None
            }
        }
    }

    fn glyph_h_kerning(&self, first_glyph: GlyphId, second_glyph: GlyphId) -> FractionalPixel {
        let face_handle = self.face_handle.lock();
        let face = face_handle.face;
        assert!(!face.is_null());

        let mut delta = FT_Vector { x: 0, y: 0 };
        unsafe {
            FT_Get_Kerning(
                face,
                first_glyph,
                second_glyph,
                FT_KERNING_DEFAULT,
                &mut delta,
            );
        }
        fixed_26_dot_6_to_float(delta.x) * self.unscalable_font_metrics_scale()
    }

    fn glyph_h_advance(&self, glyph: GlyphId) -> Option<FractionalPixel> {
        let face_handle = self.face_handle.lock();
        let face = face_handle.face;
        assert!(!face.is_null());

        let load_flags = face_handle.glyph_load_flags();
        let result = unsafe { FT_Load_Glyph(face, glyph as FT_UInt, load_flags) };
        if 0 != result {
            debug!("Unable to load glyph {}. reason: {:?}", glyph, result);
            return None;
        }

        let void_glyph = unsafe { (*face).glyph };
        let slot: FT_GlyphSlot = void_glyph;
        assert!(!slot.is_null());

        let advance = unsafe { (*slot).metrics.horiAdvance };
        Some(fixed_26_dot_6_to_float(advance) * self.unscalable_font_metrics_scale())
    }

    fn metrics(&self) -> FontMetrics {
        let face_handle = self.face_handle.lock();
        let face_ptr = face_handle.face;
        let face = unsafe { &*face_ptr };

        // face.size is a *c_void in the bindings, presumably to avoid recursive structural types
        let freetype_size: &FT_SizeRec = unsafe { mem::transmute(&(*face.size)) };
        let freetype_metrics: &FT_Size_Metrics = &(freetype_size).metrics;

        let mut max_advance;
        let mut max_ascent;
        let mut max_descent;
        let mut line_height;
        let mut y_scale = 0.0;
        let mut em_height;
        if face_handle.scalable() {
            // Prefer FT_Size_Metrics::y_scale to y_ppem as y_ppem does not have subpixel accuracy.
            //
            // FT_Size_Metrics::y_scale is in 16.16 fixed point format.  Its (fractional) value is a
            // factor that converts vertical metrics from design units to units of 1/64 pixels, so
            // that the result may be interpreted as pixels in 26.6 fixed point format.
            //
            // This converts the value to a float without losing precision.
            y_scale = freetype_metrics.y_scale as f64 / 65535.0 / 64.0;

            max_advance = (face.max_advance_width as f64) * y_scale;
            max_ascent = (face.ascender as f64) * y_scale;
            max_descent = -(face.descender as f64) * y_scale;
            line_height = (face.height as f64) * y_scale;
            em_height = (face.units_per_EM as f64) * y_scale;
        } else {
            max_advance = fixed_26_dot_6_to_float(freetype_metrics.max_advance);
            max_ascent = fixed_26_dot_6_to_float(freetype_metrics.ascender);
            max_descent = -fixed_26_dot_6_to_float(freetype_metrics.descender);
            line_height = fixed_26_dot_6_to_float(freetype_metrics.height);

            em_height = freetype_metrics.y_ppem as f64;
            // FT_Face doc says units_per_EM and a bunch of following fields are "only relevant to
            // scalable outlines". If it's an sfnt, we can get units_per_EM from the 'head' table
            // instead; otherwise, we don't have a unitsPerEm value so we can't compute y_scale and
            // x_scale.
            let head = unsafe { FT_Get_Sfnt_Table(face_ptr, ft_sfnt_head) as *mut TtHeader };
            if !head.is_null() && unsafe { (*head).table_version != 0xffff } {
                // Bug 1267909 - Even if the font is not explicitly scalable, if the face has color
                // bitmaps, it should be treated as scalable and scaled to the desired size. Metrics
                // based on y_ppem need to be rescaled for the adjusted size.
                if face_handle.color() {
                    em_height = self.requested_face_size.to_f64_px();
                    let adjust_scale = em_height / (freetype_metrics.y_ppem as f64);
                    max_advance *= adjust_scale;
                    max_descent *= adjust_scale;
                    max_ascent *= adjust_scale;
                    line_height *= adjust_scale;
                }
                let units_per_em = unsafe { (*head).units_per_em } as f64;
                y_scale = em_height / units_per_em;
            }
        }

        // 'leading' is supposed to be the vertical distance between two baselines,
        // reflected by the height attribute in freetype. On OS X (w/ CTFont),
        // leading represents the distance between the bottom of a line descent to
        // the top of the next line's ascent or: (line_height - ascent - descent),
        // see http://stackoverflow.com/a/5635981 for CTFont implementation.
        // Convert using a formula similar to what CTFont returns for consistency.
        let leading = line_height - (max_ascent + max_descent);

        let underline_size = face.underline_thickness as f64 * y_scale;
        let underline_offset = face.underline_position as f64 * y_scale + 0.5;

        // The default values for strikeout size and offset. Use OpenType spec's suggested position
        // for Roman font as the default for offset.
        let mut strikeout_size = underline_size;
        let mut strikeout_offset = em_height * 409.0 / 2048.0 + 0.5 * strikeout_size;

        // CSS 2.1, section 4.3.2 Lengths: "In the cases where it is
        // impossible or impractical to determine the x-height, a value of
        // 0.5em should be used."
        let mut x_height = 0.5 * em_height;
        let mut average_advance = 0.0;
        if let Some(os2) = face_handle.os2_table() {
            if !os2.y_strikeout_size.is_zero() && !os2.y_strikeout_position.is_zero() {
                strikeout_size = os2.y_strikeout_size as f64 * y_scale;
                strikeout_offset = os2.y_strikeout_position as f64 * y_scale;
            }
            if !os2.sx_height.is_zero() {
                x_height = os2.sx_height as f64 * y_scale;
            }

            if !os2.x_average_char_width.is_zero() {
                average_advance = fixed_26_dot_6_to_float(unsafe {
                    FT_MulFix(
                        os2.x_average_char_width as FT_F26Dot6,
                        freetype_metrics.x_scale,
                    )
                });
            }
        }

        if average_advance.is_zero() {
            average_advance = self
                .glyph_index('0')
                .and_then(|idx| self.glyph_h_advance(idx))
                .map_or(max_advance, |advance| advance * y_scale);
        }

        let zero_horizontal_advance = self
            .glyph_index('0')
            .and_then(|idx| self.glyph_h_advance(idx))
            .map(Au::from_f64_px);
        let ic_horizontal_advance = self
            .glyph_index('\u{6C34}')
            .and_then(|idx| self.glyph_h_advance(idx))
            .map(Au::from_f64_px);
        let space_advance = self
            .glyph_index(' ')
            .and_then(|idx| self.glyph_h_advance(idx))
            .unwrap_or(average_advance);

        FontMetrics {
            underline_size: Au::from_f64_px(underline_size),
            underline_offset: Au::from_f64_px(underline_offset),
            strikeout_size: Au::from_f64_px(strikeout_size),
            strikeout_offset: Au::from_f64_px(strikeout_offset),
            leading: Au::from_f64_px(leading),
            x_height: Au::from_f64_px(x_height),
            em_size: Au::from_f64_px(em_height),
            ascent: Au::from_f64_px(max_ascent),
            descent: Au::from_f64_px(max_descent),
            max_advance: Au::from_f64_px(max_advance),
            average_advance: Au::from_f64_px(average_advance),
            line_gap: Au::from_f64_px(line_height),
            zero_horizontal_advance,
            ic_horizontal_advance,
            space_advance: Au::from_f64_px(space_advance),
        }
    }

    fn table_for_tag(&self, tag: FontTableTag) -> Option<FontTable> {
        let face_handle = self.face_handle.lock();
        face_handle.table_for_tag(tag)
    }

    fn typographic_bounds(&self, glyph_id: GlyphId) -> Rect<f32> {
        let face_handle = self.face_handle.lock();
        let face = face_handle.face;
        assert!(!face.is_null());

        let load_flags = FT_LOAD_DEFAULT | FT_LOAD_NO_HINTING;
        let result = unsafe { FT_Load_Glyph(face, glyph_id as FT_UInt, load_flags) };
        if 0 != result {
            debug!("Unable to load glyph {}. reason: {:?}", glyph_id, result);
            return Rect::default();
        }

        let metrics = unsafe { &(*(*face).glyph).metrics };

        Rect::new(
            Point2D::new(
                metrics.horiBearingX as f32,
                (metrics.horiBearingY - metrics.height) as f32,
            ),
            Size2D::new(metrics.width as f32, metrics.height as f32),
        ) * (1. / 64.)
    }

    fn webrender_font_instance_flags(&self) -> FontInstanceFlags {
        // On other platforms, we only pass this when we know that we are loading a font with
        // color characters, but not passing this flag simply *prevents* WebRender from
        // loading bitmaps. There's no harm to always passing it.
        FontInstanceFlags::EMBEDDED_BITMAPS
    }
}

impl PlatformFont {
    /// Find the scale to use for metrics of unscalable fonts. Unscalable fonts, those using bitmap
    /// glyphs, are scaled after glyph rasterization. In order for metrics to match the final scaled
    /// font, we need to scale them based on the final size and the actual font size.
    fn unscalable_font_metrics_scale(&self) -> f64 {
        self.requested_face_size.to_f64_px() / self.actual_face_size.to_f64_px()
    }
}

trait FreeTypeFaceHelpers {
    fn scalable(self) -> bool;
    fn color(self) -> bool;
    fn has_axes(self) -> bool;
    fn set_size(self, pt_size: Au) -> Result<Au, &'static str>;
    fn set_weight(self, weight: Au) -> FT_Error;
    fn glyph_load_flags(self) -> FT_Int32;
    fn os2_table(self) -> Option<OS2Table>;
    fn table_for_tag(self, tag: FontTableTag) -> Option<FontTable>;
}

impl FreeTypeFaceHelpers for &PlatformFontFaceHandle {
    fn scalable(self) -> bool {
        unsafe { (*self.face).face_flags & FT_FACE_FLAG_SCALABLE as c_long != 0 }
    }

    fn color(self) -> bool {
        unsafe { (*self.face).face_flags & FT_FACE_FLAG_COLOR as c_long != 0 }
    }

    fn has_axes(self) -> bool {
        unsafe { (*self.face).face_flags & FT_FACE_FLAG_MULTIPLE_MASTERS as c_long != 0 }
    }

    fn set_size(self, requested_size: Au) -> Result<Au, &'static str> {
        if self.scalable() {
            let size_in_fixed_point = (requested_size.to_f64_px() * 64.0 + 0.5) as FT_F26Dot6;
            let result = unsafe { FT_Set_Char_Size(self.face, size_in_fixed_point, 0, 72, 72) };
            if 0 != result {
                return Err("FT_Set_Char_Size failed");
            }
            return Ok(requested_size);
        }

        let requested_size = (requested_size.to_f64_px() * 64.0) as FT_Pos;
        let get_size_at_index = |index| unsafe {
            (
                (*(*self.face).available_sizes.offset(index as isize)).x_ppem,
                (*(*self.face).available_sizes.offset(index as isize)).y_ppem,
            )
        };

        let mut best_index = 0;
        let mut best_size = get_size_at_index(0);
        let mut best_dist = best_size.1 - requested_size;
        for strike_index in 1..unsafe { (*self.face).num_fixed_sizes } {
            let new_scale = get_size_at_index(strike_index);
            let new_distance = new_scale.1 - requested_size;

            // Distance is positive if strike is larger than desired size,
            // or negative if smaller. If previously a found smaller strike,
            // then prefer a larger strike. Otherwise, minimize distance.
            if (best_dist < 0 && new_distance >= best_dist) || new_distance.abs() <= best_dist {
                best_dist = new_distance;
                best_size = new_scale;
                best_index = strike_index;
            }
        }

        if 0 == unsafe { FT_Select_Size(self.face, best_index) } {
            Ok(Au::from_f64_px(best_size.1 as f64 / 64.0))
        } else {
            Err("FT_Select_Size failed")
        }
    }

    fn set_weight(self, weight: Au) -> FT_Error {

        // General way to setup and check axis information. Will work with all fonts
        let num_axis: u32 = unsafe { (*self.variation_axes).num_axis };
        if num_axis == 0 {
            log::warn!(
                "ddesyatkin: Freetype2 \
                variation axes count: \
                AllFonts {:?}",
                num_axis
            );
            return FT_Err_Ok;
        }
        log::warn!(
            "ddesyatkin: Freetype2 \
            variation axes count: \
            AllFonts {:?}",
            num_axis
        );
        let mut parsed_axes = Vec::<*mut FT_Var_Axis>::new();
        for i in 0..num_axis as usize {
            let elem_ptr: *mut FT_Var_Axis = unsafe{(*self.variation_axes).axis.wrapping_add(i)};
            if elem_ptr.is_null() {
                log::warn!("ddesyatkin: We was not able to get var axis ptr");
                continue;
            }
            parsed_axes.push(elem_ptr);
        }

        let mut name_record: * mut FT_SfntName = ptr::null_mut();
        for axis_ptr in parsed_axes {
            let axis = unsafe{*axis_ptr};
            let result: FT_Error = unsafe {FT_Get_Sfnt_Name(self.face, axis.strid, name_record)};
            if !result.succeeded() || name_record.is_null() {
                let error_string = result.ft_get_error_message();
                log::warn!(
                    "ddesyatkin: We was not able to read variation axis name. FT_error: {}",
                    error_string
                );
                log::warn!(
                    "ddesyatkin: Freetype2 \
                    variation {:?}",
                    axis
                );
                continue
            }
            log::warn!(
                "ddesyatkin: Freetype2 \
                variation {:?} \
                name record: {:?}",
                axis,
                unsafe{*name_record}
            );
        }

        // OpenType / TrueType specific way
        let num_namedstyles: u32 = unsafe { (*self.variation_axes).num_namedstyles };
        log::warn!(
            "ddesyatkin: Freetype2 \
            variation axes count: \
            OT/TT specific {:?}",
            num_namedstyles
        );

        // FT_Set_Var_Design_Coordinates

        FT_Err_Ok
    }

    fn glyph_load_flags(self) -> FT_Int32 {
        let mut load_flags = FT_LOAD_DEFAULT;

        // Default to slight hinting, which is what most
        // Linux distros use by default, and is a better
        // default than no hinting.
        // TODO(gw): Make this configurable.
        load_flags |= FT_LOAD_TARGET_LIGHT as i32;

        let face_flags = unsafe { (*self.face).face_flags };
        if (face_flags & (FT_FACE_FLAG_FIXED_SIZES as FT_Long)) != 0 {
            // We only set FT_LOAD_COLOR if there are bitmap strikes; COLR (color-layer) fonts
            // will be handled internally in Servo. In that case WebRender will just be asked to
            // paint individual layers.
            load_flags |= FT_LOAD_COLOR;
        }

        load_flags as FT_Int32
    }

    fn os2_table(self) -> Option<OS2Table> {
        unsafe {
            let os2 = FT_Get_Sfnt_Table(self.face, ft_sfnt_os2) as *mut TT_OS2;
            let valid = !os2.is_null() && (*os2).version != 0xffff;

            if !valid {
                return None;
            }

            Some(OS2Table {
                version: (*os2).version,
                x_average_char_width: (*os2).xAvgCharWidth,
                us_weight_class: (*os2).usWeightClass,
                us_width_class: (*os2).usWidthClass,
                y_strikeout_size: (*os2).yStrikeoutSize,
                y_strikeout_position: (*os2).yStrikeoutPosition,
                ul_unicode_range1: (*os2).ulUnicodeRange1,
                ul_unicode_range2: (*os2).ulUnicodeRange2,
                ul_unicode_range3: (*os2).ulUnicodeRange3,
                ul_unicode_range4: (*os2).ulUnicodeRange4,
                sx_height: (*os2).sxHeight,
            })
        }
    }

    fn table_for_tag(self, tag: FontTableTag) -> Option<FontTable> {
        let tag = tag as FT_ULong;
        // Zero value in tag check must be added.
        // Check wether Freetype checks for invalid tag...

        unsafe {
            // Get the length of the table
            let mut len = 0;
            if 0 != FT_Load_Sfnt_Table(self.face, tag, 0, ptr::null_mut(), &mut len) {
                return None;
            }
            // Get the data
            let mut buf = vec![0_u8; len as usize];
            // let Ok(font_table) = FontTable::new_four_alligned(len.try_into().unwrap()) else {
            //     return None;
            // };

            if 0 != FT_Load_Sfnt_Table(self.face, tag, 0, buf.as_mut_ptr(), &mut len) {
                return None;
            }
            return Some(FontTable{buffer: buf});
        }
    }
}

#[repr(C)]
struct TtHeader {
    table_version: FT_Fixed,
    font_revision: FT_Fixed,

    checksum_adjust: FT_Long,
    magic_number: FT_Long,

    flags: FT_UShort,
    units_per_em: FT_UShort,

    created: [FT_ULong; 2],
    modified: [FT_ULong; 2],

    x_min: FT_Short,
    y_min: FT_Short,
    x_max: FT_Short,
    y_max: FT_Short,

    mac_style: FT_UShort,
    lowest_rec_ppem: FT_UShort,

    font_direction: FT_Short,
    index_to_loc_format: FT_Short,
    glyph_data_format: FT_Short,
}

unsafe extern "C" {
    fn FT_Load_Sfnt_Table(
        face: FT_Face,
        tag: FT_ULong,
        offset: FT_Long,
        buffer: *mut FT_Byte,
        length: *mut FT_ULong,
    ) -> FT_Error;
}
