export FreeTypeNativeFont;

import vec_as_buf = vec::as_buf;
import result::{result, ok, err};
import ptr::{addr_of, null};
import glyph::GlyphIndex;
import azure::freetype;
import freetype::{ FT_Error, FT_Library, FT_Face, FT_Long };
import freetype::bindgen::{
    FT_Init_FreeType,
    FT_Done_FreeType,
    FT_New_Memory_Face,
    FT_Done_Face
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
            fail "FT_Done_Face failed";
        }
    }

    fn glyph_index(_codepoint: char) -> option<GlyphIndex> {
        fail;
    }

    // FIXME: What unit is this returning? Let's have a custom type
    fn glyph_h_advance(_glyph: GlyphIndex) -> int {
        fail;
    }
}

fn create(lib: FT_Library, buf: &[u8]) -> result<FreeTypeNativeFont, ()> {
    assert lib.is_not_null();
    let face: FT_Face = null();
    ret vec_as_buf(*buf) {|cbuf|
        if FT_New_Memory_Face(lib, cbuf, (*buf).len() as FT_Long,
                              0 as FT_Long, addr_of(face)).succeeded() {
            ok(FreeTypeNativeFont(face))
        } else {
            err(())
        }
    }
}

impl methods for FT_Error {
    fn succeeded() -> bool { self == 0 as FT_Error }
}

#[cfg(test)]
fn with_lib(f: fn@(FT_Library)) {
    let lib: FT_Library = null();
    assert FT_Init_FreeType(addr_of(lib)).succeeded();
    f(lib);
    FT_Done_FreeType(lib);
}

#[test]
fn create_should_return_err_if_buf_is_bogus() {
    with_lib { |lib|
        let buf = &[];
        assert create(lib, buf).is_err();
    }
}
