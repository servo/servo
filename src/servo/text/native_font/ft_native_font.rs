export FreeTypeNativeFont, with_test_native_font;

import vec_as_buf = vec::as_buf;
import result::{result, ok, err};
import ptr::{addr_of, null};
import unsafe::reinterpret_cast;
import glyph::GlyphIndex;
import azure::freetype;
import freetype::{ FT_Error, FT_Library, FT_Face, FT_Long, FT_ULong, FT_UInt, FT_GlyphSlot };
import freetype::bindgen::{
    FT_Init_FreeType,
    FT_Done_FreeType,
    FT_New_Memory_Face,
    FT_Done_Face,
    FT_Get_Char_Index,
    FT_Load_Glyph,
    FT_Set_Char_Size
};

class FreeTypeNativeFont/& {
    let face: FT_Face;

    new(face: FT_Face) {
        assert face.is_not_null();
        self.face = face;
    }

    drop {
        assert self.face.is_not_null();
        if !FT_Done_Face(self.face).succeeded() {
            fail ~"FT_Done_Face failed";
        }
    }

    fn glyph_index(codepoint: char) -> option<GlyphIndex> {
        assert self.face.is_not_null();
        let idx = FT_Get_Char_Index(self.face, codepoint as FT_ULong);
        return if idx != 0 as FT_UInt {
            some(idx as GlyphIndex)
        } else {
            #warn("Invalid codepoint: %?", codepoint);
            none
        };
    }

    // FIXME: What unit is this returning? Let's have a custom type
    fn glyph_h_advance(glyph: GlyphIndex) -> option<int> {
        assert self.face.is_not_null();
        let res =  FT_Load_Glyph(self.face, glyph as FT_UInt, 0);
        if res.succeeded() {
            unsafe {
                let void_glyph = (*self.face).glyph;
                let slot: FT_GlyphSlot = reinterpret_cast(void_glyph);
                assert slot.is_not_null();
                let advance = (*slot).metrics.horiAdvance;
                #debug("h_advance for %? is %?", glyph, advance);
                // FIXME: Dividing by 64 converts to pixels, which
                // is not the unit we should be using
                return some((advance / 64) as int);
            }
        } else {
            #warn("Unable to load glyph %?. reason: %?", glyph, res);
            return none;
        }
    }
}

fn create(lib: FT_Library, buf: &~[u8]) -> result<FreeTypeNativeFont, ()> {
    assert lib.is_not_null();
    let face: FT_Face = null();
    return vec_as_buf(*buf, |cbuf, _len| {
           if FT_New_Memory_Face(lib, cbuf, (*buf).len() as FT_Long,
                                 0 as FT_Long, addr_of(face)).succeeded() {
               // FIXME: These values are placeholders
               let res = FT_Set_Char_Size(face, 0, 20*64, 0, 72);
               if !res.succeeded() { fail ~"unable to set font char size" }
               ok(FreeTypeNativeFont(face))
           } else {
               err(())
           }
    })
}

trait methods {
    fn succeeded() -> bool;
}

impl methods of methods for FT_Error {
    fn succeeded() -> bool { self == 0 as FT_Error }
}

fn with_test_native_font(f: fn@(nf: &NativeFont)) {
    import font::test_font_bin;
    import unwrap_result = result::unwrap;

    with_lib(|lib| {
        let buf = test_font_bin();
        let font = unwrap_result(create(lib, &buf));
        f(&font);
    })
}

fn with_lib(f: fn@(FT_Library)) {
    let lib: FT_Library = null();
    assert FT_Init_FreeType(addr_of(lib)).succeeded();
    f(lib);
    FT_Done_FreeType(lib);
}

#[test]
fn create_should_return_err_if_buf_is_bogus() {
    with_lib(|lib| {
        let buf = &~[];
        assert create(lib, buf).is_err();
    })
}
