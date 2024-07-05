/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::os::raw::c_long;
use std::sync::Arc;
use std::{mem, ptr};

use app_units::Au;
use euclid::default::{Point2D, Rect, Size2D};
use freetype_sys::{
    ft_sfnt_head, ft_sfnt_os2, FT_Byte, FT_Done_Face, FT_Error, FT_F26Dot6, FT_Face, FT_Fixed,
    FT_Get_Char_Index, FT_Get_Kerning, FT_Get_Sfnt_Table, FT_GlyphSlot, FT_Int32, FT_Load_Glyph,
    FT_Long, FT_MulFix, FT_New_Memory_Face, FT_Pos, FT_Select_Size, FT_Set_Char_Size, FT_Short,
    FT_SizeRec, FT_Size_Metrics, FT_UInt, FT_ULong, FT_UShort, FT_Vector, FT_FACE_FLAG_COLOR,
    FT_FACE_FLAG_FIXED_SIZES, FT_FACE_FLAG_SCALABLE, FT_KERNING_DEFAULT, FT_LOAD_COLOR,
    FT_LOAD_DEFAULT, FT_LOAD_NO_HINTING, FT_STYLE_FLAG_ITALIC, TT_OS2,
};
use log::debug;
use parking_lot::ReentrantMutex;
use style::computed_values::font_stretch::T as FontStretch;
use style::computed_values::font_weight::T as FontWeight;
use style::values::computed::font::FontStyle;
use style::Zero;
use webrender_api::FontInstanceFlags;

use super::library_handle::FreeTypeLibraryHandle;
use crate::font::{
    FontMetrics, FontTableMethods, FontTableTag, FractionalPixel, PlatformFontMethods, GPOS, GSUB,
    KERN,
};
use crate::font_cache_thread::FontIdentifier;
use crate::font_template::FontTemplateDescriptor;
use crate::glyph::GlyphId;

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
    x_average_char_width: FT_Short,
    us_weight_class: FT_UShort,
    us_width_class: FT_UShort,
    y_strikeout_size: FT_Short,
    y_strikeout_position: FT_Short,
    sx_height: FT_Short,
}

#[derive(Debug)]
#[allow(unused)]
pub struct PlatformFont {
    /// The font data itself, which must stay valid for the lifetime of the
    /// platform [`FT_Face`].
    font_data: Arc<Vec<u8>>,
    face: ReentrantMutex<FT_Face>,
    requested_face_size: Au,
    actual_face_size: Au,
    can_do_fast_shaping: bool,
}

// FT_Face can be used in multiple threads, but from only one thread at a time.
// It's protected with a ReentrantMutex for PlatformFont.
// See https://freetype.org/freetype2/docs/reference/ft2-face_creation.html#ft_face.
unsafe impl Sync for PlatformFont {}
unsafe impl Send for PlatformFont {}

impl Drop for PlatformFont {
    fn drop(&mut self) {
        let face = self.face.lock();
        assert!(!face.is_null());
        unsafe {
            // The FreeType documentation says that both `FT_New_Face` and `FT_Done_Face`
            // should be protected by a mutex.
            // See https://freetype.org/freetype2/docs/reference/ft2-library_setup.html.
            let _guard = FreeTypeLibraryHandle::get().lock();
            if FT_Done_Face(*face) != 0 {
                panic!("FT_Done_Face failed");
            }
        }
    }
}

fn create_face(data: Arc<Vec<u8>>, face_index: u32) -> Result<FT_Face, &'static str> {
    unsafe {
        let mut face: FT_Face = ptr::null_mut();
        let library = FreeTypeLibraryHandle::get().lock();

        // This is to support 32bit Android where FT_Long is defined as i32.
        let result = FT_New_Memory_Face(
            library.freetype_library,
            data.as_ptr(),
            data.len() as FT_Long,
            face_index as FT_Long,
            &mut face,
        );

        if 0 != result || face.is_null() {
            return Err("Could not create FreeType face");
        }

        Ok(face)
    }
}

impl PlatformFontMethods for PlatformFont {
    fn new_from_data(
        _font_identifier: FontIdentifier,
        data: Arc<Vec<u8>>,
        face_index: u32,
        requested_size: Option<Au>,
    ) -> Result<PlatformFont, &'static str> {
        let face = create_face(data.clone(), face_index)?;

        let (requested_face_size, actual_face_size) = match requested_size {
            Some(requested_size) => (requested_size, face.set_size(requested_size)?),
            None => (Au::zero(), Au::zero()),
        };
        let can_do_fast_shaping =
            face.has_table(KERN) && !face.has_table(GPOS) && !face.has_table(GSUB);

        Ok(PlatformFont {
            face: ReentrantMutex::new(face),
            font_data: data,
            requested_face_size,
            actual_face_size,
            can_do_fast_shaping,
        })
    }

    fn descriptor(&self) -> FontTemplateDescriptor {
        let face = self.face.lock();
        let style = if unsafe { (**face).style_flags & FT_STYLE_FLAG_ITALIC as c_long != 0 } {
            FontStyle::ITALIC
        } else {
            FontStyle::NORMAL
        };

        let face = self.face.lock();
        let os2_table = face.os2_table();
        let weight = os2_table
            .as_ref()
            .map(|os2| FontWeight::from_float(os2.us_weight_class as f32))
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

        FontTemplateDescriptor::new(weight, stretch, style)
    }

    fn glyph_index(&self, codepoint: char) -> Option<GlyphId> {
        let face = self.face.lock();
        assert!(!face.is_null());

        unsafe {
            let idx = FT_Get_Char_Index(*face, codepoint as FT_ULong);
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
        let face = self.face.lock();
        assert!(!face.is_null());

        let mut delta = FT_Vector { x: 0, y: 0 };
        unsafe {
            FT_Get_Kerning(
                *face,
                first_glyph,
                second_glyph,
                FT_KERNING_DEFAULT,
                &mut delta,
            );
        }
        fixed_26_dot_6_to_float(delta.x) * self.unscalable_font_metrics_scale()
    }

    fn can_do_fast_shaping(&self) -> bool {
        self.can_do_fast_shaping
    }

    fn glyph_h_advance(&self, glyph: GlyphId) -> Option<FractionalPixel> {
        let face = self.face.lock();
        assert!(!face.is_null());

        let load_flags = face.glyph_load_flags();
        let result = unsafe { FT_Load_Glyph(*face, glyph as FT_UInt, load_flags) };
        if 0 != result {
            debug!("Unable to load glyph {}. reason: {:?}", glyph, result);
            return None;
        }

        let void_glyph = unsafe { (**face).glyph };
        let slot: FT_GlyphSlot = void_glyph;
        assert!(!slot.is_null());

        let advance = unsafe { (*slot).metrics.horiAdvance };
        Some(fixed_26_dot_6_to_float(advance) * self.unscalable_font_metrics_scale())
    }

    fn metrics(&self) -> FontMetrics {
        let face_ptr = *self.face.lock();
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
        if face_ptr.scalable() {
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
                if face_ptr.color() {
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
        if let Some(os2) = face_ptr.os2_table() {
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
        }
    }

    fn table_for_tag(&self, tag: FontTableTag) -> Option<FontTable> {
        let face = self.face.lock();
        let tag = tag as FT_ULong;

        unsafe {
            // Get the length
            let mut len = 0;
            if 0 != FT_Load_Sfnt_Table(*face, tag, 0, ptr::null_mut(), &mut len) {
                return None;
            }
            // Get the bytes
            let mut buf = vec![0u8; len as usize];
            if 0 != FT_Load_Sfnt_Table(*face, tag, 0, buf.as_mut_ptr(), &mut len) {
                return None;
            }
            Some(FontTable { buffer: buf })
        }
    }

    fn typographic_bounds(&self, glyph_id: GlyphId) -> Rect<f32> {
        let face = self.face.lock();
        assert!(!face.is_null());

        let load_flags = FT_LOAD_DEFAULT | FT_LOAD_NO_HINTING;
        let result = unsafe { FT_Load_Glyph(*face, glyph_id as FT_UInt, load_flags) };
        if 0 != result {
            debug!("Unable to load glyph {}. reason: {:?}", glyph_id, result);
            return Rect::default();
        }

        let metrics = unsafe { &(*(**face).glyph).metrics };

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
    fn set_size(self, pt_size: Au) -> Result<Au, &'static str>;
    fn glyph_load_flags(self) -> FT_Int32;
    fn has_table(self, tag: FontTableTag) -> bool;
    fn os2_table(self) -> Option<OS2Table>;
}

impl FreeTypeFaceHelpers for FT_Face {
    fn scalable(self) -> bool {
        unsafe { (*self).face_flags & FT_FACE_FLAG_SCALABLE as c_long != 0 }
    }

    fn color(self) -> bool {
        unsafe { (*self).face_flags & FT_FACE_FLAG_COLOR as c_long != 0 }
    }

    fn set_size(self, requested_size: Au) -> Result<Au, &'static str> {
        if self.scalable() {
            let size_in_fixed_point = (requested_size.to_f64_px() * 64.0 + 0.5) as FT_F26Dot6;
            let result = unsafe { FT_Set_Char_Size(self, size_in_fixed_point, 0, 72, 72) };
            if 0 != result {
                return Err("FT_Set_Char_Size failed");
            }
            return Ok(requested_size);
        }

        let requested_size = (requested_size.to_f64_px() * 64.0) as FT_Pos;
        let get_size_at_index = |index| unsafe {
            (
                (*(*self).available_sizes.offset(index as isize)).x_ppem,
                (*(*self).available_sizes.offset(index as isize)).y_ppem,
            )
        };

        let mut best_index = 0;
        let mut best_size = get_size_at_index(0);
        let mut best_dist = best_size.1 - requested_size;
        for strike_index in 1..unsafe { (*self).num_fixed_sizes } {
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

        if 0 == unsafe { FT_Select_Size(self, best_index) } {
            Ok(Au::from_f64_px(best_size.1 as f64 / 64.0))
        } else {
            Err("FT_Select_Size failed")
        }
    }

    fn glyph_load_flags(self) -> FT_Int32 {
        let mut load_flags = FT_LOAD_DEFAULT;

        // Default to slight hinting, which is what most
        // Linux distros use by default, and is a better
        // default than no hinting.
        // TODO(gw): Make this configurable.
        load_flags |= FT_LOAD_TARGET_LIGHT as i32;

        let face_flags = unsafe { (*self).face_flags };
        if (face_flags & (FT_FACE_FLAG_FIXED_SIZES as FT_Long)) != 0 {
            // We only set FT_LOAD_COLOR if there are bitmap strikes; COLR (color-layer) fonts
            // will be handled internally in Servo. In that case WebRender will just be asked to
            // paint individual layers.
            load_flags |= FT_LOAD_COLOR;
        }

        load_flags as FT_Int32
    }

    fn has_table(self, tag: FontTableTag) -> bool {
        unsafe { 0 == FT_Load_Sfnt_Table(self, tag as FT_ULong, 0, ptr::null_mut(), &mut 0) }
    }

    fn os2_table(self) -> Option<OS2Table> {
        unsafe {
            let os2 = FT_Get_Sfnt_Table(self, ft_sfnt_os2) as *mut TT_OS2;
            let valid = !os2.is_null() && (*os2).version != 0xffff;

            if !valid {
                return None;
            }

            Some(OS2Table {
                x_average_char_width: (*os2).xAvgCharWidth,
                us_weight_class: (*os2).usWeightClass,
                us_width_class: (*os2).usWidthClass,
                y_strikeout_size: (*os2).yStrikeoutSize,
                y_strikeout_position: (*os2).yStrikeoutPosition,
                sx_height: (*os2).sxHeight,
            })
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

extern "C" {
    fn FT_Load_Sfnt_Table(
        face: FT_Face,
        tag: FT_ULong,
        offset: FT_Long,
        buffer: *mut FT_Byte,
        length: *mut FT_ULong,
    ) -> FT_Error;
}
