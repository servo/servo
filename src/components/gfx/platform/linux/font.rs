/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

extern mod freetype;

use font::{FontHandleMethods, FontMetrics, FontTableMethods};
use font::{FontTableTag, FractionalPixel, SpecifiedFontStyle, UsedFontStyle};
use servo_util::geometry::Au;
use servo_util::geometry;
use platform::font_context::FontContextHandle;
use text::glyph::GlyphIndex;
use text::util::{float_to_fixed, fixed_to_float};
use style::computed_values::font_weight;

use freetype::freetype::{FT_Get_Char_Index, FT_Get_Postscript_Name};
use freetype::freetype::{FT_Load_Glyph, FT_Set_Char_Size};
use freetype::freetype::{FT_New_Face, FT_Get_Sfnt_Table};
use freetype::freetype::{FT_New_Memory_Face, FT_Done_Face};
use freetype::freetype::{FTErrorMethods, FT_F26Dot6, FT_Face, FT_FaceRec};
use freetype::freetype::{FT_GlyphSlot, FT_Library, FT_Long, FT_ULong};
use freetype::freetype::{FT_STYLE_FLAG_ITALIC, FT_STYLE_FLAG_BOLD};
use freetype::freetype::{FT_SizeRec, FT_UInt, FT_Size_Metrics};
use freetype::freetype::{ft_sfnt_os2};
use freetype::tt_os2::TT_OS2;

use std::cast;
use std::ptr;
use std::str;

fn float_to_fixed_ft(f: f64) -> i32 {
    float_to_fixed(6, f)
}

fn fixed_to_float_ft(f: i32) -> f64 {
    fixed_to_float(6, f)
}

pub struct FontTable {
    bogus: ()
}

impl FontTableMethods for FontTable {
    fn with_buffer(&self, _blk: |*u8, uint|) {
        fail!()
    }
}

enum FontSource {
    FontSourceMem(~[u8]),
    FontSourceFile(~str)
}

pub struct FontHandle {
    // The font binary. This must stay valid for the lifetime of the font,
    // if the font is created using FT_Memory_Face.
    source: FontSource,
    face: FT_Face,
    handle: FontContextHandle
}

#[unsafe_destructor]
impl Drop for FontHandle {
    fn drop(&mut self) {
        assert!(self.face.is_not_null());
        unsafe {
            if !FT_Done_Face(self.face).succeeded() {
                fail!(~"FT_Done_Face failed");
            }
        }
    }
}

impl FontHandleMethods for FontHandle {
    fn new_from_buffer(fctx: &FontContextHandle,
                           buf: ~[u8],
                           style: &SpecifiedFontStyle)
                        -> Result<FontHandle, ()> {
        let ft_ctx: FT_Library = fctx.ctx.borrow().ctx;
        if ft_ctx.is_null() { return Err(()); }

        let face_result = create_face_from_buffer(ft_ctx, buf.as_ptr(), buf.len(), style.pt_size);

        // TODO: this could be more simply written as result::chain
        // and moving buf into the struct ctor, but cant' move out of
        // captured binding.
        return match face_result {
            Ok(face) => {
              let handle = FontHandle {
                  face: face,
                  source: FontSourceMem(buf),
                  handle: fctx.clone()
              };
              Ok(handle)
            }
            Err(()) => Err(())
        };

         fn create_face_from_buffer(lib: FT_Library, cbuf: *u8, cbuflen: uint, pt_size: f64)
                                    -> Result<FT_Face, ()> {
             unsafe {
                 let mut face: FT_Face = ptr::null();
                 let face_index = 0 as FT_Long;
                 let result = FT_New_Memory_Face(lib, cbuf, cbuflen as FT_Long,
                                                 face_index, ptr::to_mut_unsafe_ptr(&mut face));
                 
                 if !result.succeeded() || face.is_null() {
                     return Err(());
                 }
                 if FontHandle::set_char_size(face, pt_size).is_ok() {
                     Ok(face)
                 } else {
                     Err(())
                 }
             }
         }
    }

    // an identifier usable by FontContextHandle to recreate this FontHandle.
    fn face_identifier(&self) -> ~str {
        /* FT_Get_Postscript_Name seems like a better choice here, but it
           doesn't give usable results for fontconfig when deserializing. */
        unsafe { str::raw::from_c_str((*self.face).family_name) }
    }
    fn family_name(&self) -> ~str {
        unsafe { str::raw::from_c_str((*self.face).family_name) }
    }
    fn face_name(&self) -> ~str {
        unsafe { str::raw::from_c_str(FT_Get_Postscript_Name(self.face)) }
    }
    fn is_italic(&self) -> bool {
        unsafe { (*self.face).style_flags & FT_STYLE_FLAG_ITALIC != 0 }
    }
    fn boldness(&self) -> font_weight::T {
        let default_weight = font_weight::Weight400;
        if unsafe { (*self.face).style_flags & FT_STYLE_FLAG_BOLD == 0 } {
            default_weight
        } else {
            unsafe {
                let os2 = FT_Get_Sfnt_Table(self.face, ft_sfnt_os2) as *TT_OS2;
                let valid = os2.is_not_null() && (*os2).version != 0xffff;
                if valid {
                    let weight =(*os2).usWeightClass;
                    match weight {
                        1 | 100..199 => font_weight::Weight100,
                        2 | 200..299 => font_weight::Weight200,
                        3 | 300..399 => font_weight::Weight300,
                        4 | 400..499 => font_weight::Weight400,
                        5 | 500..599 => font_weight::Weight500,
                        6 | 600..699 => font_weight::Weight600,
                        7 | 700..799 => font_weight::Weight700,
                        8 | 800..899 => font_weight::Weight800,
                        9 | 900..999 => font_weight::Weight900,
                        _ => default_weight
                    }
                } else {
                    default_weight
                }
            }
        }
    }

    fn clone_with_style(&self,
                        fctx: &FontContextHandle,
                        style: &UsedFontStyle) -> Result<FontHandle, ()> {
        match self.source {
            FontSourceMem(ref buf) => {
                FontHandleMethods::new_from_buffer(fctx, buf.clone(), style)
            }
            FontSourceFile(ref file) => {
                FontHandle::new_from_file(fctx, (*file).clone(), style)
            }
        }
    }

    fn glyph_index(&self,
                       codepoint: char) -> Option<GlyphIndex> {
        assert!(self.face.is_not_null());
        unsafe {
            let idx = FT_Get_Char_Index(self.face, codepoint as FT_ULong);
            return if idx != 0 as FT_UInt {
                Some(idx as GlyphIndex)
            } else {
                debug!("Invalid codepoint: {}", codepoint);
                None
            };
        }
    }

    fn glyph_h_advance(&self,
                           glyph: GlyphIndex) -> Option<FractionalPixel> {
        assert!(self.face.is_not_null());
        unsafe {
            let res =  FT_Load_Glyph(self.face, glyph as FT_UInt, 0);
            if res.succeeded() {
                let void_glyph = (*self.face).glyph;
                let slot: FT_GlyphSlot = cast::transmute(void_glyph);
                assert!(slot.is_not_null());
                debug!("metrics: {:?}", (*slot).metrics);
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

    fn get_metrics(&self) -> FontMetrics {
        /* TODO(Issue #76): complete me */
        let face = self.get_face_rec();

        let underline_size = self.font_units_to_au(face.underline_thickness as f64);
        let underline_offset = self.font_units_to_au(face.underline_position as f64);
        let em_size = self.font_units_to_au(face.units_per_EM as f64);
        let ascent = self.font_units_to_au(face.ascender as f64);
        let descent = self.font_units_to_au(face.descender as f64);
        let max_advance = self.font_units_to_au(face.max_advance_width as f64);

        // 'leading' is supposed to be the vertical distance between two baselines,
        // reflected by the height attibute in freetype.  On OS X (w/ CTFont),
        // leading represents the distance between the bottom of a line descent to
        // the top of the next line's ascent or: (line_height - ascent - descent),
        // see http://stackoverflow.com/a/5635981 for CTFont implementation.
        // Convert using a formular similar to what CTFont returns for consistency.
        let height = self.font_units_to_au(face.height as f64);
        let leading = height - (ascent + descent);

        let mut strikeout_size = geometry::from_pt(0.0);
        let mut strikeout_offset = geometry::from_pt(0.0);
        let mut x_height = geometry::from_pt(0.0);
        unsafe {
            let os2 = FT_Get_Sfnt_Table(face, ft_sfnt_os2) as *TT_OS2;
            let valid = os2.is_not_null() && (*os2).version != 0xffff;
            if valid {
               strikeout_size = self.font_units_to_au((*os2).yStrikeoutSize as f64);
               strikeout_offset = self.font_units_to_au((*os2).yStrikeoutPosition as f64);
               x_height = self.font_units_to_au((*os2).sxHeight as f64);
            }
        }

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
            max_advance:      max_advance
        };

        debug!("Font metrics (@{:f} pt): {:?}", geometry::to_pt(em_size), metrics);
        return metrics;
    }

    fn get_table_for_tag(&self, _: FontTableTag) -> Option<FontTable> {
        None
    }
}

impl<'a> FontHandle {
    fn set_char_size(face: FT_Face, pt_size: f64) -> Result<(), ()>{
        let char_width = float_to_fixed_ft(pt_size) as FT_F26Dot6;
        let char_height = float_to_fixed_ft(pt_size) as FT_F26Dot6;
        let h_dpi = 72;
        let v_dpi = 72;

        unsafe {
            let result = FT_Set_Char_Size(face, char_width, char_height, h_dpi, v_dpi);
            if result.succeeded() { Ok(()) } else { Err(()) }
        }
    }

    pub fn new_from_file(fctx: &FontContextHandle, file: &str,
                         style: &SpecifiedFontStyle) -> Result<FontHandle, ()> {
        unsafe {
            let ft_ctx: FT_Library = fctx.ctx.borrow().ctx;
            if ft_ctx.is_null() { return Err(()); }

            let mut face: FT_Face = ptr::null();
            let face_index = 0 as FT_Long;
            file.to_c_str().with_ref(|file_str| {
                FT_New_Face(ft_ctx, file_str,
                            face_index, ptr::to_mut_unsafe_ptr(&mut face));
            });
            if face.is_null() {
                return Err(());
            }
            if FontHandle::set_char_size(face, style.pt_size).is_ok() {
                Ok(FontHandle {
                    source: FontSourceFile(file.to_str()),
                    face: face,
                    handle: fctx.clone()
                })
            } else {
                Err(())
            }
        }
    }

    pub fn new_from_file_unstyled(fctx: &FontContextHandle, file: ~str)
                               -> Result<FontHandle, ()> {
        unsafe {
            let ft_ctx: FT_Library = fctx.ctx.borrow().ctx;
            if ft_ctx.is_null() { return Err(()); }

            let mut face: FT_Face = ptr::null();
            let face_index = 0 as FT_Long;
            file.to_c_str().with_ref(|file_str| {
                FT_New_Face(ft_ctx, file_str,
                            face_index, ptr::to_mut_unsafe_ptr(&mut face));
            });
            if face.is_null() {
                return Err(());
            }

            Ok(FontHandle {
                source: FontSourceFile(file),
                face: face,
                handle: fctx.clone()
            })
        }
    }

    fn get_face_rec(&'a self) -> &'a FT_FaceRec {
        unsafe {
            &(*self.face)
        }
    }

    fn font_units_to_au(&self, value: f64) -> Au {
        let face = self.get_face_rec();

        // face.size is a *c_void in the bindings, presumably to avoid
        // recursive structural types
        let size: &FT_SizeRec = unsafe { cast::transmute(&(*face.size)) };
        let metrics: &FT_Size_Metrics = &(*size).metrics;

        let em_size = face.units_per_EM as f64;
        let x_scale = (metrics.x_ppem as f64) / em_size as f64;

        // If this isn't true then we're scaling one of the axes wrong
        assert!(metrics.x_ppem == metrics.y_ppem);

        return geometry::from_frac_px(value * x_scale);
    }
}

