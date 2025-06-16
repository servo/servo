/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::ffi::CString;
use std::fs::File;
use std::os::raw::c_long;
use std::{mem, ptr};

use app_units::Au;
use euclid::default::{Point2D, Rect, Size2D};
use freetype_sys::{
    FT_Byte, FT_Done_Face, FT_Error, FT_F26Dot6, FT_FACE_FLAG_COLOR, FT_FACE_FLAG_FIXED_SIZES,
    FT_FACE_FLAG_SCALABLE, FT_Face, FT_Get_Char_Index, FT_Get_Kerning, FT_GlyphSlot, FT_Int32,
    FT_KERNING_DEFAULT, FT_LOAD_COLOR, FT_LOAD_DEFAULT, FT_LOAD_NO_HINTING, FT_Load_Glyph, FT_Long,
    FT_New_Face, FT_New_Memory_Face, FT_Pos, FT_Select_Size, FT_Set_Char_Size, FT_Size_Metrics,
    FT_SizeRec, FT_UInt, FT_ULong, FT_Vector,
};
use log::debug;
use memmap2::Mmap;
use parking_lot::ReentrantMutex;
use read_fonts::tables::os2::SelectionFlags;
use read_fonts::{FontRef, ReadError, TableProvider};
use style::Zero;
use style::computed_values::font_stretch::T as FontStretch;
use style::computed_values::font_weight::T as FontWeight;
use style::values::computed::font::FontStyle;
use webrender_api::FontInstanceFlags;

use super::LocalFontIdentifier;
use super::library_handle::FreeTypeLibraryHandle;
use crate::FontData;
use crate::font::{
    FontMetrics, FontTableMethods, FontTableTag, FractionalPixel, PlatformFontMethods,
};
use crate::font_template::FontTemplateDescriptor;
use crate::glyph::GlyphId;
use crate::system_font_service::FontIdentifier;

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

#[derive(Debug)]
#[allow(unused)]
pub struct PlatformFont {
    face: ReentrantMutex<FT_Face>,
    requested_face_size: Au,
    actual_face_size: Au,

    /// A member that allows using `skrifa` to read values from this font.
    table_provider_data: FreeTypeFaceTableProviderData,
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
            return Err("Could not create FreeType face");
        }

        let (requested_face_size, actual_face_size) = match requested_size {
            Some(requested_size) => (requested_size, face.set_size(requested_size)?),
            None => (Au::zero(), Au::zero()),
        };

        Ok(PlatformFont {
            face: ReentrantMutex::new(face),
            requested_face_size,
            actual_face_size,
            table_provider_data: FreeTypeFaceTableProviderData::Web(font_data.clone()),
        })
    }

    fn new_from_local_font_identifier(
        font_identifier: LocalFontIdentifier,
        requested_size: Option<Au>,
    ) -> Result<PlatformFont, &'static str> {
        let mut face: FT_Face = ptr::null_mut();
        let library = FreeTypeLibraryHandle::get().lock();
        let filename = CString::new(&*font_identifier.path).expect("filename contains NUL byte!");

        let result = unsafe {
            FT_New_Face(
                library.freetype_library,
                filename.as_ptr(),
                font_identifier.index() as FT_Long,
                &mut face,
            )
        };

        if 0 != result || face.is_null() {
            return Err("Could not create FreeType face");
        }

        let (requested_face_size, actual_face_size) = match requested_size {
            Some(requested_size) => (requested_size, face.set_size(requested_size)?),
            None => (Au::zero(), Au::zero()),
        };

        let Ok(memory_mapped_font_data) =
            File::open(&*font_identifier.path).and_then(|file| unsafe { Mmap::map(&file) })
        else {
            return Err("Could not memory map");
        };

        Ok(PlatformFont {
            face: ReentrantMutex::new(face),
            requested_face_size,
            actual_face_size,
            table_provider_data: FreeTypeFaceTableProviderData::Local(
                memory_mapped_font_data,
                font_identifier.index(),
            ),
        })
    }

    fn descriptor(&self) -> FontTemplateDescriptor {
        let Ok(font_ref) = self.table_provider_data.font_ref() else {
            return FontTemplateDescriptor::default();
        };

        let Ok(os2) = font_ref.os2() else {
            return FontTemplateDescriptor::default();
        };

        let mut style = FontStyle::NORMAL;
        if os2.fs_selection().contains(SelectionFlags::ITALIC) {
            style = FontStyle::ITALIC;
        }

        let weight = FontWeight::from_float(os2.us_weight_class() as f32);
        let stretch = match os2.us_width_class() {
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
        };

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
        let font_ref = self.table_provider_data.font_ref();

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
            if let Ok(head) = font_ref.clone().and_then(|font_ref| font_ref.head()) {
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
                y_scale = em_height / head.units_per_em() as f64;
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

        if let Ok(os2) = font_ref.and_then(|font_ref| font_ref.os2()) {
            let y_strikeout_size = os2.y_strikeout_size();
            let y_strikeout_position = os2.y_strikeout_position();
            if !y_strikeout_size.is_zero() && !y_strikeout_position.is_zero() {
                strikeout_size = y_strikeout_size as f64 * y_scale;
                strikeout_offset = y_strikeout_position as f64 * y_scale;
            }

            let sx_height = os2.sx_height().unwrap_or(0);
            if !sx_height.is_zero() {
                x_height = sx_height as f64 * y_scale;
            }

            let x_average_char_width = os2.x_avg_char_width();
            if !x_average_char_width.is_zero() {
                average_advance = x_average_char_width as f64 * y_scale;
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

enum FreeTypeFaceTableProviderData {
    Web(FontData),
    Local(Mmap, u32),
}

impl FreeTypeFaceTableProviderData {
    fn font_ref(&self) -> Result<FontRef<'_>, ReadError> {
        match self {
            Self::Web(ipc_shared_memory) => FontRef::new(&ipc_shared_memory.0),
            Self::Local(mmap, index) => FontRef::from_index(mmap, *index),
        }
    }
}

impl std::fmt::Debug for FreeTypeFaceTableProviderData {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}
