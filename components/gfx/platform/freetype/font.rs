/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

extern crate freetype;

use font::{FontHandleMethods, FontMetrics, FontTableMethods};
use font::{FontTableTag, FractionalPixel};
use platform::font_context::FontContextHandle;
use platform::font_template::FontTemplateData;
use style::computed_values::{font_stretch, font_weight};
use text::glyph::GlyphId;
use text::util::{float_to_fixed, fixed_to_float};
use util::geometry::Au;
use util::str::c_str_to_string;

use freetype::freetype::{FTErrorMethods, FT_F26Dot6, FT_Face, FT_FaceRec};
use freetype::freetype::{FT_Get_Char_Index, FT_Get_Postscript_Name};
use freetype::freetype::{FT_Get_Kerning, FT_Get_Sfnt_Table, FT_Load_Sfnt_Table};
use freetype::freetype::{FT_GlyphSlot, FT_Library, FT_Long, FT_ULong};
use freetype::freetype::{FT_KERNING_DEFAULT, FT_STYLE_FLAG_ITALIC, FT_STYLE_FLAG_BOLD};
use freetype::freetype::{FT_Load_Glyph, FT_Set_Char_Size};
use freetype::freetype::{FT_New_Memory_Face, FT_Done_Face};
use freetype::freetype::{FT_SizeRec, FT_UInt, FT_Size_Metrics, struct_FT_Vector_};
use freetype::freetype::{ft_sfnt_os2};
use freetype::tt_os2::TT_OS2;

use libc::c_char;
use std::mem;
use std::ptr;
use std::sync::Arc;

fn float_to_fixed_ft(f: f64) -> i32 {
    float_to_fixed(6, f)
}

fn fixed_to_float_ft(f: i32) -> f64 {
    fixed_to_float(6, f)
}

pub struct FontTable {
    buffer: Vec<u8>,
}

impl FontTableMethods for FontTable {
    fn with_buffer<F>(&self, blk: F) where F: FnOnce(*const u8, usize) {
        blk(self.buffer.as_ptr(), self.buffer.len())
    }
}

pub struct FontHandle {
    // The font binary. This must stay valid for the lifetime of the font,
    // if the font is created using FT_Memory_Face.
    pub font_data: Arc<FontTemplateData>,
    pub face: FT_Face,
    pub handle: FontContextHandle
}

impl Drop for FontHandle {
    fn drop(&mut self) {
        assert!(!self.face.is_null());
        unsafe {
            if !FT_Done_Face(self.face).succeeded() {
                panic!("FT_Done_Face failed");
            }
        }
    }
}

impl FontHandleMethods for FontHandle {
    fn new_from_template(fctx: &FontContextHandle,
                       template: Arc<FontTemplateData>,
                       pt_size: Option<Au>)
                        -> Result<FontHandle, ()> {
        let ft_ctx: FT_Library = fctx.ctx.ctx;
        if ft_ctx.is_null() { return Err(()); }

        let face_result = create_face_from_buffer(ft_ctx, &template.bytes, pt_size);

        // TODO: this could be more simply written as result::chain
        // and moving buf into the struct ctor, but cant' move out of
        // captured binding.
        return match face_result {
            Ok(face) => {
              let handle = FontHandle {
                  face: face,
                  font_data: template.clone(),
                  handle: fctx.clone()
              };
              Ok(handle)
            }
            Err(()) => Err(())
        };

        fn create_face_from_buffer(lib: FT_Library, buffer: &[u8], pt_size: Option<Au>)
                                   -> Result<FT_Face, ()> {
            unsafe {
                let mut face: FT_Face = ptr::null_mut();
                let face_index = 0 as FT_Long;
                let result = FT_New_Memory_Face(lib, buffer.as_ptr(), buffer.len() as FT_Long,
                                                face_index, &mut face);

                if !result.succeeded() || face.is_null() {
                    return Err(());
                }
                match pt_size {
                    Some(s) => {
                        match FontHandle::set_char_size(face, s) {
                            Ok(_) => Ok(face),
                            Err(_) => Err(()),
                        }
                    }
                    None => Ok(face),
                }
            }
        }
    }
    fn template(&self) -> Arc<FontTemplateData> {
        self.font_data.clone()
    }
    fn family_name(&self) -> String {
        unsafe {
            c_str_to_string((*self.face).family_name as *const c_char)
        }
    }
    fn face_name(&self) -> String {
        unsafe {
            c_str_to_string(FT_Get_Postscript_Name(self.face) as *const c_char)
        }
    }
    fn is_italic(&self) -> bool {
        unsafe { (*self.face).style_flags & FT_STYLE_FLAG_ITALIC != 0 }
    }
    fn boldness(&self) -> font_weight::T {
        let default_weight = font_weight::T::Weight400;
        if unsafe { (*self.face).style_flags & FT_STYLE_FLAG_BOLD == 0 } {
            default_weight
        } else {
            unsafe {
                let os2 = FT_Get_Sfnt_Table(self.face, ft_sfnt_os2) as *mut TT_OS2;
                let valid = !os2.is_null() && (*os2).version != 0xffff;
                if valid {
                    let weight =(*os2).usWeightClass;
                    match weight {
                        1 | 100...199 => font_weight::T::Weight100,
                        2 | 200...299 => font_weight::T::Weight200,
                        3 | 300...399 => font_weight::T::Weight300,
                        4 | 400...499 => font_weight::T::Weight400,
                        5 | 500...599 => font_weight::T::Weight500,
                        6 | 600...699 => font_weight::T::Weight600,
                        7 | 700...799 => font_weight::T::Weight700,
                        8 | 800...899 => font_weight::T::Weight800,
                        9 | 900...999 => font_weight::T::Weight900,
                        _ => default_weight
                    }
                } else {
                    default_weight
                }
            }
        }
    }
    fn stretchiness(&self) -> font_stretch::T {
        // TODO(pcwalton): Implement this.
        font_stretch::T::normal
    }

    fn glyph_index(&self, codepoint: char) -> Option<GlyphId> {
        assert!(!self.face.is_null());
        unsafe {
            let idx = FT_Get_Char_Index(self.face, codepoint as FT_ULong);
            return if idx != 0 as FT_UInt {
                Some(idx as GlyphId)
            } else {
                debug!("Invalid codepoint: {}", codepoint);
                None
            };
        }
    }

    fn glyph_h_kerning(&self, first_glyph: GlyphId, second_glyph: GlyphId)
                       -> FractionalPixel {
        assert!(!self.face.is_null());
        let mut delta = struct_FT_Vector_ { x: 0, y: 0 };
        unsafe {
            FT_Get_Kerning(self.face, first_glyph, second_glyph, FT_KERNING_DEFAULT, &mut delta);
        }
        fixed_to_float_ft(delta.x as i32)
    }

    fn glyph_h_advance(&self, glyph: GlyphId) -> Option<FractionalPixel> {
        assert!(!self.face.is_null());
        unsafe {
            let res =  FT_Load_Glyph(self.face, glyph as FT_UInt, 0);
            if res.succeeded() {
                let void_glyph = (*self.face).glyph;
                let slot: FT_GlyphSlot = mem::transmute(void_glyph);
                assert!(!slot.is_null());
                let advance = (*slot).metrics.horiAdvance;
                debug!("h_advance for {} is {}", glyph, advance);
                let advance = advance as i32;
                return Some(fixed_to_float_ft(advance) as FractionalPixel);
            } else {
                debug!("Unable to load glyph {}. reason: {}", glyph, res);
                return None;
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
        unsafe {
            let os2 = FT_Get_Sfnt_Table(face, ft_sfnt_os2) as *mut TT_OS2;
            let valid = !os2.is_null() && (*os2).version != 0xffff;
            if valid {
               strikeout_size = self.font_units_to_au((*os2).yStrikeoutSize as f64);
               strikeout_offset = self.font_units_to_au((*os2).yStrikeoutPosition as f64);
               x_height = self.font_units_to_au((*os2).sxHeight as f64);
            }
        }

        let average_advance = self.glyph_index('0')
                                  .and_then(|idx| self.glyph_h_advance(idx))
                                  .map(|advance| self.font_units_to_au(advance))
                                  .unwrap_or(max_advance);

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
        return metrics;
    }

    fn table_for_tag(&self, tag: FontTableTag) -> Option<Box<FontTable>> {
        let tag = tag as FT_ULong;

        unsafe {
            // Get the length
            let mut len = 0;
            if !FT_Load_Sfnt_Table(self.face, tag, 0, ptr::null_mut(), &mut len).succeeded() {
                return None
            }
            // Get the bytes
            let mut buf = vec![0u8; len as usize];
            if !FT_Load_Sfnt_Table(self.face, tag, 0, buf.as_mut_ptr(), &mut len).succeeded() {
                return None
            }
            Some(box FontTable { buffer: buf })
        }
    }
}

impl<'a> FontHandle {
    fn set_char_size(face: FT_Face, pt_size: Au) -> Result<(), ()>{
        let char_width = float_to_fixed_ft((0.5f64 + pt_size.to_f64_px()).floor()) as FT_F26Dot6;

        unsafe {
            let result = FT_Set_Char_Size(face, char_width, 0, 0, 0);
            if result.succeeded() { Ok(()) } else { Err(()) }
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
        assert!(metrics.x_ppem == metrics.y_ppem);

        return Au::from_f64_px(value * x_scale);
    }
}
