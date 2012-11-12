extern mod freetype;

use font::{FontMetrics, FractionalPixel};
use font_context::FreeTypeFontContext;
use geometry::Au;
use text::glyph::GlyphIndex;
use text::util::{float_to_fixed, fixed_to_float};

use cast::reinterpret_cast;
use ptr::{addr_of, null};
use vec_as_buf = vec::as_imm_buf;

use freetype::{ FT_Error, FT_Library, FT_Face, FT_Long, FT_ULong, FT_Size, FT_SizeRec,
               FT_UInt, FT_GlyphSlot, FT_Size_Metrics, FT_FaceRec, FT_F26Dot6 };
use freetype::bindgen::{
    FT_Init_FreeType,
    FT_Done_FreeType,
    FT_New_Memory_Face,
    FT_Done_Face,
    FT_Get_Char_Index,
    FT_Load_Glyph,
    FT_Set_Char_Size
};

fn float_to_fixed_ft(f: float) -> i32 {
    float_to_fixed(6, f)
}

fn fixed_to_float_ft(f: i32) -> float {
    fixed_to_float(6, f)
}

pub struct FreeTypeFontHandle {
    // The font binary. This must stay valid for the lifetime of the font,
    // if the font is created using FT_Memory_Face.
    // TODO: support both FT_Memory_Face (from memory) and FT_Face (from file)
    buf: ~[u8],
    face: FT_Face,

    drop {
        assert self.face.is_not_null();
        if !FT_Done_Face(self.face).succeeded() {
            fail ~"FT_Done_Face failed";
        }
    }
}

pub impl FreeTypeFontHandle {
    static pub fn new(fctx: &FreeTypeFontContext,
                      buf: ~[u8], pt_size: float) -> Result<FreeTypeFontHandle, ()> {
        let ft_ctx = fctx.ctx;
        assert ft_ctx.is_not_null();
        let face_result: Result<FT_Face,()> = vec_as_buf(buf, |cbuf, _len| {
            if FT_New_Memory_Face(ft_ctx, cbuf, (*buf).len() as FT_Long,
                                  0 as FT_Long, addr_of(&face)).succeeded() {
                let res = FT_Set_Char_Size(face, // the face
                                           float_to_fixed_ft(pt_size) as FT_F26Dot6, // char width
                                           float_to_fixed_ft(pt_size) as FT_F26Dot6, // char height
                                           72, // horiz. DPI
                                           72); // vert. DPI
                if !res.succeeded() { fail ~"unable to set font char size" }
                Ok(face)
            } else {
                Err(())
            }
        });
        return do result::chain(face_result) |face| {
            Ok(FreeTypeFontHandle { face: face, buf: move buf })
        };
    }

    pub fn glyph_index(codepoint: char) -> Option<GlyphIndex> {
        assert self.face.is_not_null();
        let idx = FT_Get_Char_Index(self.face, codepoint as FT_ULong);
        return if idx != 0 as FT_UInt {
            Some(idx as GlyphIndex)
        } else {
            debug!("Invalid codepoint: %?", codepoint);
            None
        };
    }

    pub fn glyph_h_advance(glyph: GlyphIndex) -> Option<FractionalPixel> {
        assert self.face.is_not_null();
        let res =  FT_Load_Glyph(self.face, glyph as FT_UInt, 0);
        if res.succeeded() {
            unsafe {
                let void_glyph = (*self.face).glyph;
                let slot: FT_GlyphSlot = reinterpret_cast(&void_glyph);
                assert slot.is_not_null();
                debug!("metrics: %?", (*slot).metrics);
                let advance = (*slot).metrics.horiAdvance;
                debug!("h_advance for %? is %?", glyph, advance);
                let advance = advance as i32;
                return Some(fixed_to_float_ft(advance) as FractionalPixel);
            }
        } else {
            debug!("Unable to load glyph %?. reason: %?", glyph, res);
            return None;
        }
    }

    pub fn get_metrics() -> FontMetrics {
        /* TODO(Issue #76): complete me */
        let face = self.get_face_rec();

        let underline_size = self.font_units_to_au(face.underline_thickness as float);
        let underline_offset = self.font_units_to_au(face.underline_position as float);
        let em_size = self.font_units_to_au(face.units_per_EM as float);
        let ascent = self.font_units_to_au(face.ascender as float);
        let descent = self.font_units_to_au(face.descender as float);
        let max_advance = self.font_units_to_au(face.max_advance_width as float);

        return FontMetrics {
            underline_size:   underline_size,
            underline_offset: underline_offset,
            leading:          geometry::from_pt(0.0), //FIXME
            x_height:         geometry::from_pt(0.0), //FIXME
            em_size:          em_size,
            ascent:           ascent,
            descent:          descent,
            max_advance:      max_advance
        }
    }

    priv fn get_face_rec() -> &self/FT_FaceRec unsafe {
        &(*self.face)
    }

    priv fn font_units_to_au(value: float) -> Au {

        let face = self.get_face_rec();

        // face.size is a *c_void in the bindings, presumably to avoid
        // recursive structural types
        let size: &FT_SizeRec = unsafe { cast::transmute(&(*face.size)) };
        let metrics: &FT_Size_Metrics = unsafe { &((*size).metrics) };

        let em_size = face.units_per_EM as float;
        let x_scale = (metrics.x_ppem as float) / em_size as float;

        // If this isn't true then we're scaling one of the axes wrong
        assert metrics.x_ppem == metrics.y_ppem;

        return geometry::from_frac_px(value * x_scale);
    }
}

trait FTErrorMethods {
    fn succeeded() -> bool;
}

impl FT_Error : FTErrorMethods {
    fn succeeded() -> bool { self == 0 as FT_Error }
}
