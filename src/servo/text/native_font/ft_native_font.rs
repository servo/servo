export FreeTypeNativeFont, with_test_native_font, create;

use vec_as_buf = vec::as_imm_buf;
use ptr::{addr_of, null};
use unsafe::reinterpret_cast;
use glyph::GlyphIndex;
use azure::freetype;
use freetype::{ FT_Error, FT_Library, FT_Face, FT_Long, FT_ULong, FT_UInt, FT_GlyphSlot };
use freetype::bindgen::{
    FT_Init_FreeType,
    FT_Done_FreeType,
    FT_New_Memory_Face,
    FT_Done_Face,
    FT_Get_Char_Index,
    FT_Load_Glyph,
    FT_Set_Char_Size
};

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
            #warn("Invalid codepoint: %?", codepoint);
            None
        };
    }

    // FIXME: What unit is this returning? Let's have a custom type
    fn glyph_h_advance(glyph: GlyphIndex) -> Option<int> {
        assert self.face.is_not_null();
        let res =  FT_Load_Glyph(self.face, glyph as FT_UInt, 0);
        if res.succeeded() {
            unsafe {
                let void_glyph = (*self.face).glyph;
                let slot: FT_GlyphSlot = reinterpret_cast(&void_glyph);
                assert slot.is_not_null();
                let advance = (*slot).metrics.horiAdvance;
                #debug("h_advance for %? is %?", glyph, advance);
                // FIXME: Dividing by 64 converts to pixels, which
                // is not the unit we should be using
                return Some((advance / 64) as int);
            }
        } else {
            #warn("Unable to load glyph %?. reason: %?", glyph, res);
            return None;
        }
    }
}

fn create(lib: &FT_Library, buf: @~[u8]) -> Result<FreeTypeNativeFont, ()> {
    assert lib.is_not_null();
    let face: FT_Face = null();
    return vec_as_buf(*buf, |cbuf, _len| {
           if FT_New_Memory_Face(*lib, cbuf, (*buf).len() as FT_Long,
                                 0 as FT_Long, addr_of(face)).succeeded() {
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
    assert FT_Init_FreeType(addr_of(lib)).succeeded();
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
