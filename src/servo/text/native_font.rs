#[doc = "

NativeFont encapsulates access to the platform's font API,
e.g. quartz, FreeType. It provides access to metrics and tables
needed by the text shaper as well as access to the underlying
font resources needed by the graphics layer to draw glyphs.

"];

export NativeFont, create;

import result::result;
import font_library::native::NativeFontLibrary;

#[cfg(target_os = "macos")]
type NativeFont/& = quartz_native_font::QuartzNativeFont;

#[cfg(target_os = "linux")]
type NativeFont/& = ft_native_font::FreeTypeNativeFont;

#[cfg(target_os = "macos")]
fn create(_native_lib: &NativeFontLibrary, buf: &~[u8]) -> result<NativeFont, ()> {
    quartz_native_font::create(buf)
}

#[cfg(target_os = "linux")]
fn create(native_lib: &NativeFontLibrary, buf: &~[u8]) -> result<NativeFont, ()> {
    ft_native_font::create(*native_lib, buf)
}

#[cfg(target_os = "macos")]
fn with_test_native_font(f: fn@(nf: &NativeFont)) {
    quartz_native_font::with_test_native_font(f);
}

#[cfg(target_os = "linux")]
fn with_test_native_font(f: fn@(nf: &NativeFont)) {
    ft_native_font::with_test_native_font(f);
}

#[test]
#[ignore(cfg(target_os = "macos"))]
fn should_get_glyph_indexes() {
    with_test_native_font(|font| {
        let idx = font.glyph_index('w');
        assert idx == some(40u);
    })
}

#[test]
#[ignore(cfg(target_os = "macos"))]
fn should_return_none_glyph_index_for_bad_codepoints() {
    with_test_native_font(|font| {
        let idx = font.glyph_index(0 as char);
        assert idx == none;
    })
}

#[test]
#[ignore(cfg(target_os = "macos"))]
fn should_get_glyph_h_advance() {
    with_test_native_font(|font| {
        let adv = font.glyph_h_advance(40u);
        assert adv == some(15);
    })
}

#[test]
#[ignore(cfg(target_os = "macos"))]
fn should_return_none_glyph_h_advance_for_bad_codepoints() {
    with_test_native_font(|font| {
        let adv = font.glyph_h_advance(-1 as uint);
        assert adv == none;
    })
}
