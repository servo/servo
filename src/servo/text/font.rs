export Font, test_font_bin, create_test_font;

import glyph::GlyphIndex;
import vec_to_ptr = vec::unsafe::to_ptr;
import libc::{ c_int, c_double, c_ulong };
import ptr::{ null, addr_of };
import native_font::NativeFont;
import font_library::FontLibrary;

// FIXME (rust 2708): convert this to a class

#[doc = "
A font handle. Layout can use this to calculate glyph metrics
and the renderer can use it to render text.
"]
class Font {
    let fontbuf: @~[u8];
    let native_font: NativeFont;

    new(-fontbuf: ~[u8], -native_font: NativeFont) {
        self.fontbuf = @fontbuf;
        self.native_font = native_font;
    }

    fn buf() -> @~[u8] {
        self.fontbuf
    }

    fn glyph_index(codepoint: char) -> option<GlyphIndex> {
        self.native_font.glyph_index(codepoint)
    }

    fn glyph_h_advance(glyph: GlyphIndex) -> int {
        match self.native_font.glyph_h_advance(glyph) {
          some(adv) => adv,
          none => /* FIXME: Need fallback strategy */ 10
        }
    }
}

const TEST_FONT: [u8 * 33004] = #include_bin("JosefinSans-SemiBold.ttf");

fn test_font_bin() -> ~[u8] {
    return vec::from_fn(33004, |i| TEST_FONT[i]);
}

fn should_destruct_on_fail_without_leaking() {
    #[test];
    #[should_fail];

    let lib = FontLibrary();
    let _font = lib.get_test_font();
    fail;
}

fn should_get_glyph_indexes() {
    #[test];

    let lib = FontLibrary();
    let font = lib.get_test_font();
    let glyph_idx = font.glyph_index('w');
    assert glyph_idx == some(40u);
}

fn should_get_glyph_advance() {
    #[test];

    let lib = FontLibrary();
    let font = lib.get_test_font();
    let x = font.glyph_h_advance(40u);
    assert x == 15;
}

fn should_be_able_to_create_instances_in_multiple_threads() {
    #[test];

    for iter::repeat(10u) {
        do task::spawn {
            let lib = FontLibrary();
            let _font = lib.get_test_font();
        }
    }
}

