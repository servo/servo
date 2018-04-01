/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use app_units::Au;
use font::{FontHandleMethods, FontMetrics, FontTableMethods};
use font::{FontTableTag, FractionalPixel, GPOS, GSUB, KERN};
use freetype::freetype::{FT_Done_Face, FT_New_Face, FT_New_Memory_Face};
use freetype::freetype::{FT_F26Dot6, FT_Face, FT_FaceRec};
use freetype::freetype::{FT_Get_Char_Index, FT_Get_Postscript_Name};
use freetype::freetype::{FT_Get_Kerning, FT_Get_Sfnt_Table, FT_Load_Sfnt_Table};
use freetype::freetype::{FT_GlyphSlot, FT_Library, FT_Long, FT_ULong};
use freetype::freetype::{FT_Int32, FT_Kerning_Mode, FT_STYLE_FLAG_ITALIC};
use freetype::freetype::{FT_Load_Glyph, FT_Set_Char_Size};
use freetype::freetype::{FT_SizeRec, FT_Size_Metrics, FT_UInt, FT_Vector};
use freetype::freetype::FT_Sfnt_Tag;
use freetype::succeeded;
use freetype::tt_os2::TT_OS2;
use platform::font_context::FontContextHandle;
use platform::font_template::FontTemplateData;
use servo_atoms::Atom;
use std::{mem, ptr};
use std::ffi::CString;
use std::os::raw::{c_char, c_long};
use std::sync::Arc;
use style::computed_values::font_stretch::T as FontStretch;
use style::computed_values::font_weight::T as FontWeight;
use style::values::computed::font::FontStyle;
use super::c_str_to_string;
use text::glyph::GlyphId;
use text::util::fixed_to_float;

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
/// See https://www.microsoft.com/typography/otspec/os2.htm
#[derive(Debug)]
struct OS2Table {
    us_weight_class: u16,
    us_width_class: u16,
    y_strikeout_size: i16,
    y_strikeout_position: i16,
    sx_height: i16,
}

#[derive(Debug)]
pub struct FontHandle {
    // The font binary. This must stay valid for the lifetime of the font,
    // if the font is created using FT_Memory_Face.
    font_data: Arc<FontTemplateData>,
    face: FT_Face,
    handle: FontContextHandle,
    can_do_fast_shaping: bool,
}

impl Drop for FontHandle {
    fn drop(&mut self) {
        assert!(!self.face.is_null());
        unsafe {
            if !succeeded(FT_Done_Face(self.face)) {
                panic!("FT_Done_Face failed");
            }
        }
    }
}

fn create_face(
    lib: FT_Library,
    template: &FontTemplateData,
    pt_size: Option<Au>,
) -> Result<FT_Face, ()>
{
    unsafe {
        let mut face: FT_Face = ptr::null_mut();
        let face_index = 0 as FT_Long;

        let result = if let Some(ref bytes) = template.bytes {
            FT_New_Memory_Face(lib, bytes.as_ptr(), bytes.len() as FT_Long, face_index, &mut face)
        } else {
            // This will trigger a synchronous file read in the layout thread, which we may want to
            // revisit at some point. See discussion here:
            //
            // https://github.com/servo/servo/pull/20506#issuecomment-378838800

            let filename = CString::new(&*template.identifier).expect("filename contains NUL byte!");
            FT_New_Face(lib, filename.as_ptr(), face_index, &mut face)
        };

        if !succeeded(result) || face.is_null() {
            return Err(());
        }

        if let Some(s) = pt_size {
            FontHandle::set_char_size(face, s).or(Err(()))?
        }

        Ok(face)
    }
}

impl FontHandleMethods for FontHandle {
    fn new_from_template(fctx: &FontContextHandle,
                       template: Arc<FontTemplateData>,
                       pt_size: Option<Au>)
                        -> Result<FontHandle, ()> {
        let ft_ctx: FT_Library = fctx.ctx.ctx;
        if ft_ctx.is_null() { return Err(()); }

        let face = create_face(ft_ctx, &template, pt_size)?;

        let mut handle = FontHandle {
              face: face,
              font_data: template.clone(),
              handle: fctx.clone(),
              can_do_fast_shaping: false,
        };
        // TODO (#11310): Implement basic support for GPOS and GSUB.
        handle.can_do_fast_shaping = handle.has_table(KERN) &&
                                     !handle.has_table(GPOS) &&
                                     !handle.has_table(GSUB);
        Ok(handle)
    }

    fn template(&self) -> Arc<FontTemplateData> {
        self.font_data.clone()
    }

    fn family_name(&self) -> String {
        unsafe {
            c_str_to_string((*self.face).family_name as *const c_char)
        }
    }

    fn face_name(&self) -> Option<String> {
        unsafe {
            let name = FT_Get_Postscript_Name(self.face) as *const c_char;

            if !name.is_null() {
                Some(c_str_to_string(name))
            } else {
                None
            }
        }
    }

    fn style(&self) -> FontStyle {
        use style::values::generics::font::FontStyle::*;
        if unsafe { (*self.face).style_flags & FT_STYLE_FLAG_ITALIC as c_long != 0 } {
            Italic
        } else {
            Normal
        }
    }

    fn boldness(&self) -> FontWeight {
        let os2 = match self.os2_table() {
            None => return FontWeight::normal(),
            Some(os2) => os2,
        };
        let weight = os2.us_weight_class as f32;
        FontWeight(weight.max(1.).min(1000.))
    }

    fn stretchiness(&self) -> FontStretch {
        use style::values::generics::NonNegative;
        use style::values::specified::font::FontStretchKeyword;
        let percentage = if let Some(os2) = self.os2_table() {
            match os2.us_width_class {
                1 => FontStretchKeyword::UltraCondensed,
                2 => FontStretchKeyword::ExtraCondensed,
                3 => FontStretchKeyword::Condensed,
                4 => FontStretchKeyword::SemiCondensed,
                5 => FontStretchKeyword::Normal,
                6 => FontStretchKeyword::SemiExpanded,
                7 => FontStretchKeyword::Expanded,
                8 => FontStretchKeyword::ExtraExpanded,
                9 => FontStretchKeyword::UltraExpanded,
                _ => FontStretchKeyword::Normal
            }
        } else {
            FontStretchKeyword::Normal
        }.compute();
        NonNegative(percentage)
    }

    fn glyph_index(&self, codepoint: char) -> Option<GlyphId> {
        assert!(!self.face.is_null());
        unsafe {
            let idx = FT_Get_Char_Index(self.face, codepoint as FT_ULong);
            if idx != 0 as FT_UInt {
                Some(idx as GlyphId)
            } else {
                debug!("Invalid codepoint: U+{:04X} ('{}')", codepoint as u32, codepoint);
                None
            }
        }
    }

    fn glyph_h_kerning(&self, first_glyph: GlyphId, second_glyph: GlyphId)
                       -> FractionalPixel {
        assert!(!self.face.is_null());
        let mut delta = FT_Vector { x: 0, y: 0 };
        unsafe {
            FT_Get_Kerning(self.face, first_glyph, second_glyph,
                           FT_Kerning_Mode::FT_KERNING_DEFAULT as FT_UInt,
                           &mut delta);
        }
        fixed_to_float_ft(delta.x as i32)
    }

    fn can_do_fast_shaping(&self) -> bool {
        self.can_do_fast_shaping
    }

    fn glyph_h_advance(&self, glyph: GlyphId) -> Option<FractionalPixel> {
        assert!(!self.face.is_null());
        unsafe {
            let res =  FT_Load_Glyph(self.face,
                                     glyph as FT_UInt,
                                     GLYPH_LOAD_FLAGS);
            if succeeded(res) {
                let void_glyph = (*self.face).glyph;
                let slot: FT_GlyphSlot = mem::transmute(void_glyph);
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
        /* TODO(Issue #76): complete me */
        let face = self.face_rec_mut();

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

        let average_advance = self.glyph_index('0')
                                  .and_then(|idx| self.glyph_h_advance(idx))
                                  .map_or(max_advance, |advance| self.font_units_to_au(advance));

        let metrics = FontMetrics {
            underline_size:   underline_size,
            underline_offset: underline_offset,
            strikeout_size:   strikeout_size,
            strikeout_offset: strikeout_offset,
            leading:          leading,
            x_height:         x_height,
            em_size:          em_size,
            ascent:           ascent,
            descent:          -descent, // linux font's seem to use the opposite sign from mac
            max_advance:      max_advance,
            average_advance:  average_advance,
            line_gap:         height,
        };

        debug!("Font metrics (@{}px): {:?}", em_size.to_f32_px(), metrics);
        metrics
    }

    fn table_for_tag(&self, tag: FontTableTag) -> Option<FontTable> {
        let tag = tag as FT_ULong;

        unsafe {
            // Get the length
            let mut len = 0;
            if !succeeded(FT_Load_Sfnt_Table(self.face, tag, 0, ptr::null_mut(), &mut len)) {
                return None
            }
            // Get the bytes
            let mut buf = vec![0u8; len as usize];
            if !succeeded(FT_Load_Sfnt_Table(self.face, tag, 0, buf.as_mut_ptr(), &mut len)) {
                return None
            }
            Some(FontTable { buffer: buf })
        }
    }

    fn identifier(&self) -> Atom {
        self.font_data.identifier.clone()
    }
}

impl<'a> FontHandle {
    fn set_char_size(face: FT_Face, pt_size: Au) -> Result<(), ()>{
        let char_size = pt_size.to_f64_px() * 64.0 + 0.5;

        unsafe {
            let result = FT_Set_Char_Size(face, char_size as FT_F26Dot6, 0, 0, 0);
            if succeeded(result) { Ok(()) } else { Err(()) }
        }
    }

    fn has_table(&self, tag: FontTableTag) -> bool {
        unsafe {
            succeeded(FT_Load_Sfnt_Table(self.face, tag as FT_ULong, 0, ptr::null_mut(), &mut 0))
        }
    }

    fn face_rec_mut(&'a self) -> &'a mut FT_FaceRec {
        unsafe {
            &mut (*self.face)
        }
    }

    fn font_units_to_au(&self, value: f64) -> Au {
        let face = self.face_rec_mut();

        // face.size is a *c_void in the bindings, presumably to avoid
        // recursive structural types
        let size: &FT_SizeRec = unsafe { mem::transmute(&(*face.size)) };
        let metrics: &FT_Size_Metrics = &(*size).metrics;

        let em_size = face.units_per_EM as f64;
        let x_scale = (metrics.x_ppem as f64) / em_size as f64;

        // If this isn't true then we're scaling one of the axes wrong
        assert_eq!(metrics.x_ppem, metrics.y_ppem);

        Au::from_f64_px(value * x_scale)
    }

    fn os2_table(&self) -> Option<OS2Table> {
        unsafe {
            let os2 = FT_Get_Sfnt_Table(self.face_rec_mut(), FT_Sfnt_Tag::FT_SFNT_OS2) as *mut TT_OS2;
            let valid = !os2.is_null() && (*os2).version != 0xffff;

            if !valid {
                return None
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
