#[legacy_exports];
export FreeTypeNativeFont, with_test_native_font, create;

use font::{FontMetrics, FractionalPixel};

use gfx::geometry;
use gfx::geometry::Au;
use util::*;
use vec_as_buf = vec::as_imm_buf;
use ptr::{addr_of, null};
use cast::reinterpret_cast;
use glyph::GlyphIndex;
use font::FontMetrics;
use azure::freetype;
use freetype::{ FT_Error, FT_Library, FT_Face, FT_Long, FT_ULong, FT_Size, FT_SizeRec,
               FT_UInt, FT_GlyphSlot, FT_Size_Metrics, FT_FaceRec };
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

struct FreeTypeNativeFont {
    /// The font binary. This must stay valid for the lifetime of the font
    buf: @~[u8],
    face: FT_Face,

    drop {
        assert self.face.is_not_null();
        if !FT_Done_Face(self.face).succeeded() {
            fail ~"FT_Done_Face failed";
        }
    }
}

fn FreeTypeNativeFont(face: FT_Face, buf: @~[u8]) -> FreeTypeNativeFont {
    assert face.is_not_null();
    FreeTypeNativeFont { buf: buf, face: face }
}

impl FreeTypeNativeFont {

    fn glyph_index(codepoint: char) -> Option<GlyphIndex> {
        assert self.face.is_not_null();
        let idx = FT_Get_Char_Index(self.face, codepoint as FT_ULong);
        return if idx != 0 as FT_UInt {
            Some(idx as GlyphIndex)
        } else {
            debug!("Invalid codepoint: %?", codepoint);
            None
        };
    }

    // FIXME: What unit is this returning? Let's have a custom type
    fn glyph_h_advance(glyph: GlyphIndex) -> Option<FractionalPixel> {
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

    fn get_metrics() -> FontMetrics {
        /* TODO: complete me (Issue #76) */

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

pub fn create(lib: &FT_Library, buf: @~[u8]) -> Result<FreeTypeNativeFont, ()> {
    assert lib.is_not_null();
    let face: FT_Face = null();
    return vec_as_buf(*buf, |cbuf, _len| {
           if FT_New_Memory_Face(*lib, cbuf, (*buf).len() as FT_Long,
                                 0 as FT_Long, addr_of(&face)).succeeded() {
               // FIXME: These values are placeholders
               let res = FT_Set_Char_Size(face, 0, 20*64, 0, 72);
               if !res.succeeded() { fail ~"unable to set font char size" }
               Ok(FreeTypeNativeFont(face, buf))
           } else {
               Err(())
           }
    })
}

trait FTErrorMethods {
    fn succeeded() -> bool;
}

impl FT_Error : FTErrorMethods {
    fn succeeded() -> bool { self == 0 as FT_Error }
}

fn with_test_native_font(f: fn@(nf: &NativeFont)) {
    use font::test_font_bin;
    use unwrap_result = result::unwrap;

    with_lib(|lib| {
        let buf = @test_font_bin();
        let font = unwrap_result(create(lib, move buf));
        f(&font);
    })
}

fn with_lib(f: fn@((&FT_Library))) {
    let lib: FT_Library = null();
    assert FT_Init_FreeType(addr_of(&lib)).succeeded();
    f(&lib);
    FT_Done_FreeType(lib);
}

#[test]
fn create_should_return_err_if_buf_is_bogus() {
    with_lib(|lib| {
        let buf = @~[];
        assert create(lib, buf).is_err();
    })
}
