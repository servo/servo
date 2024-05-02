/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::convert::TryInto;
use std::os::raw::c_long;
use std::sync::Arc;
use std::{mem, ptr};

use app_units::Au;
use freetype::freetype::{
    FT_Done_Face, FT_F26Dot6, FT_Face, FT_Get_Char_Index, FT_Get_Kerning, FT_Get_Sfnt_Table,
    FT_GlyphSlot, FT_Int32, FT_Kerning_Mode, FT_Load_Glyph, FT_Load_Sfnt_Table, FT_Long,
    FT_New_Memory_Face, FT_Set_Char_Size, FT_Sfnt_Tag, FT_SizeRec, FT_Size_Metrics, FT_UInt,
    FT_ULong, FT_Vector, FT_STYLE_FLAG_ITALIC,
};
use freetype::succeeded;
use freetype::tt_os2::TT_OS2;
use log::debug;
use parking_lot::ReentrantMutex;
use style::computed_values::font_stretch::T as FontStretch;
use style::computed_values::font_weight::T as FontWeight;
use style::values::computed::font::FontStyle;
use webrender_api::FontInstanceFlags;

use super::library_handle::FreeTypeLibraryHandle;
use crate::font::{
    FontMetrics, FontTableMethods, FontTableTag, FractionalPixel, PlatformFontMethods, GPOS, GSUB,
    KERN,
};
use crate::font_cache_thread::FontIdentifier;
use crate::font_template::FontTemplateDescriptor;
use crate::text::glyph::GlyphId;
use crate::text::util::fixed_to_float;

// This constant is not present in the freetype
// bindings due to bindgen not handling the way
// the macro is defined.
const FT_LOAD_TARGET_LIGHT: FT_Int32 = 1 << 16;

// Default to slight hinting, which is what most
// Linux distros use by default, and is a better
// default than no hinting.
// TODO(gw): Make this configurable.
const GLYPH_LOAD_FLAGS: FT_Int32 = FT_LOAD_TARGET_LIGHT;

fn fixed_to_float_ft(f: i32) -> f64 {
    fixed_to_float(6, f)
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
    us_weight_class: u16,
    us_width_class: u16,
    y_strikeout_size: i16,
    y_strikeout_position: i16,
    sx_height: i16,
}

#[derive(Debug)]
#[allow(unused)]
pub struct PlatformFont {
    /// The font data itself, which must stay valid for the lifetime of the
    /// platform [`FT_Face`].
    font_data: Arc<Vec<u8>>,
    face: ReentrantMutex<FT_Face>,
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
            if !succeeded(FT_Done_Face(*face)) {
                panic!("FT_Done_Face failed");
            }
        }
    }
}

fn create_face(
    data: Arc<Vec<u8>>,
    face_index: u32,
    pt_size: Option<Au>,
) -> Result<FT_Face, &'static str> {
    unsafe {
        let mut face: FT_Face = ptr::null_mut();
        let library = FreeTypeLibraryHandle::get().lock();

        // This is to support 32bit Android where FT_Long is defined as i32.
        let face_index = face_index.try_into().unwrap();
        let result = FT_New_Memory_Face(
            library.freetype_library,
            data.as_ptr(),
            data.len() as FT_Long,
            face_index,
            &mut face,
        );

        if !succeeded(result) || face.is_null() {
            return Err("Could not create FreeType face");
        }

        if let Some(s) = pt_size {
            PlatformFont::set_char_size(face, s)?
        }

        Ok(face)
    }
}

impl PlatformFontMethods for PlatformFont {
    fn new_from_data(
        _font_identifier: FontIdentifier,
        data: Arc<Vec<u8>>,
        face_index: u32,
        pt_size: Option<Au>,
    ) -> Result<PlatformFont, &'static str> {
        let face = create_face(data.clone(), face_index, pt_size)?;
        let mut handle = PlatformFont {
            face: ReentrantMutex::new(face),
            font_data: data,
            can_do_fast_shaping: false,
        };

        // TODO (#11310): Implement basic support for GPOS and GSUB.
        handle.can_do_fast_shaping =
            handle.has_table(KERN) && !handle.has_table(GPOS) && !handle.has_table(GSUB);

        Ok(handle)
    }

    fn descriptor(&self) -> FontTemplateDescriptor {
        let face = self.face.lock();
        let style = if unsafe { (**face).style_flags & FT_STYLE_FLAG_ITALIC as c_long != 0 } {
            FontStyle::ITALIC
        } else {
            FontStyle::NORMAL
        };

        let os2_table = self.os2_table();
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
                FT_Kerning_Mode::FT_KERNING_DEFAULT as FT_UInt,
                &mut delta,
            );
        }
        fixed_to_float_ft(delta.x as i32)
    }

    fn can_do_fast_shaping(&self) -> bool {
        self.can_do_fast_shaping
    }

    fn glyph_h_advance(&self, glyph: GlyphId) -> Option<FractionalPixel> {
        let face = self.face.lock();
        assert!(!face.is_null());

        unsafe {
            let res = FT_Load_Glyph(*face, glyph as FT_UInt, GLYPH_LOAD_FLAGS);
            if succeeded(res) {
                let void_glyph = (**face).glyph;
                let slot: FT_GlyphSlot = void_glyph;
                assert!(!slot.is_null());
                let advance = (*slot).metrics.horiAdvance;
                debug!("h_advance for {} is {}", glyph, advance);
                let advance = advance as i32;
                Some(fixed_to_float_ft(advance) as FractionalPixel)
            } else {
                debug!("Unable to load glyph {}. reason: {:?}", glyph, res);
                None
            }
        }
    }

    fn metrics(&self) -> FontMetrics {
        let face = self.face.lock();
        let face = unsafe { **face };

        let underline_size = self.font_units_to_au(face.underline_thickness as f64);
        let underline_offset = self.font_units_to_au(face.underline_position as f64);
        let em_size = self.font_units_to_au(face.units_per_EM as f64);
        let ascent = self.font_units_to_au(face.ascender as f64);
        let descent = self.font_units_to_au(face.descender as f64);
        let max_advance = self.font_units_to_au(face.max_advance_width as f64);

        // 'leading' is supposed to be the vertical distance between two baselines,
        // reflected by the height attribute in freetype.  On OS X (w/ CTFont),
        // leading represents the distance between the bottom of a line descent to
        // the top of the next line's ascent or: (line_height - ascent - descent),
        // see http://stackoverflow.com/a/5635981 for CTFont implementation.
        // Convert using a formula similar to what CTFont returns for consistency.
        let height = self.font_units_to_au(face.height as f64);
        let leading = height - (ascent + descent);

        let mut strikeout_size = Au(0);
        let mut strikeout_offset = Au(0);
        let mut x_height = Au(0);

        if let Some(os2) = self.os2_table() {
            strikeout_size = self.font_units_to_au(os2.y_strikeout_size as f64);
            strikeout_offset = self.font_units_to_au(os2.y_strikeout_position as f64);
            x_height = self.font_units_to_au(os2.sx_height as f64);
        }

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
            underline_size,
            underline_offset,
            strikeout_size,
            strikeout_offset,
            leading,
            x_height,
            em_size,
            ascent,
            descent: -descent, // linux font's seem to use the opposite sign from mac
            max_advance,
            average_advance,
            line_gap: height,
            zero_horizontal_advance,
            ic_horizontal_advance,
        };

        debug!("Font metrics (@{}px): {:?}", em_size.to_f32_px(), metrics);
        metrics
    }

    fn table_for_tag(&self, tag: FontTableTag) -> Option<FontTable> {
        let face = self.face.lock();
        let tag = tag as FT_ULong;

        unsafe {
            // Get the length
            let mut len = 0;
            if !succeeded(FT_Load_Sfnt_Table(*face, tag, 0, ptr::null_mut(), &mut len)) {
                return None;
            }
            // Get the bytes
            let mut buf = vec![0u8; len as usize];
            if !succeeded(FT_Load_Sfnt_Table(
                *face,
                tag,
                0,
                buf.as_mut_ptr(),
                &mut len,
            )) {
                return None;
            }
            Some(FontTable { buffer: buf })
        }
    }

    fn webrender_font_instance_flags(&self) -> FontInstanceFlags {
        FontInstanceFlags::empty()
    }
}

impl PlatformFont {
    fn set_char_size(face: FT_Face, pt_size: Au) -> Result<(), &'static str> {
        let char_size = pt_size.to_f64_px() * 64.0 + 0.5;

        unsafe {
            let result = FT_Set_Char_Size(face, char_size as FT_F26Dot6, 0, 0, 0);
            if succeeded(result) {
                Ok(())
            } else {
                Err("FT_Set_Char_Size failed")
            }
        }
    }

    fn has_table(&self, tag: FontTableTag) -> bool {
        let face = self.face.lock();
        unsafe {
            succeeded(FT_Load_Sfnt_Table(
                *face,
                tag as FT_ULong,
                0,
                ptr::null_mut(),
                &mut 0,
            ))
        }
    }

    fn font_units_to_au(&self, value: f64) -> Au {
        let face = self.face.lock();
        let face = unsafe { **face };

        // face.size is a *c_void in the bindings, presumably to avoid
        // recursive structural types
        let size: &FT_SizeRec = unsafe { mem::transmute(&(*face.size)) };
        let metrics: &FT_Size_Metrics = &(size).metrics;

        let em_size = face.units_per_EM as f64;
        let x_scale = (metrics.x_ppem as f64) / em_size;

        // If this isn't true then we're scaling one of the axes wrong
        assert_eq!(metrics.x_ppem, metrics.y_ppem);

        Au::from_f64_px(value * x_scale)
    }

    fn os2_table(&self) -> Option<OS2Table> {
        let face = self.face.lock();

        unsafe {
            let os2 = FT_Get_Sfnt_Table(*face, FT_Sfnt_Tag::FT_SFNT_OS2) as *mut TT_OS2;
            let valid = !os2.is_null() && (*os2).version != 0xffff;

            if !valid {
                return None;
            }

            Some(OS2Table {
                us_weight_class: (*os2).usWeightClass,
                us_width_class: (*os2).usWidthClass,
                y_strikeout_size: (*os2).yStrikeoutSize,
                y_strikeout_position: (*os2).yStrikeoutPosition,
                sx_height: (*os2).sxHeight,
            })
        }
    }
}
