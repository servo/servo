#[doc = "

NativeFont encapsulates access to the platform's font API,
e.g. quartz, FreeType. It provides access to metrics and tables
needed by the text shaper as well as access to the underlying
font resources needed by the graphics layer to draw glyphs.

"];

export NativeFont;

#[cfg(target_os = "macos")]
type NativeFont/& = quartz_native_font::QuartzNativeFont;

#[cfg(target_os = "linux")]
type NativeFont/& = ft_native_font::FreeTypeNativeFont;

fn with_test_native_font(f: fn@(NativeFont)) {
    fail
}

#[test]
#[ignore]
fn should_get_glyph_indexes() {
    with_test_native_font { |font|
        let idx = font.glyph_index('w');
        assert idx == some(40u);
    }
}

#[test]
#[ignore]
fn should_get_glyph_h_advance() {
    with_test_native_font { |font|
        let adv = font.glyph_h_advance(40u);
        assert adv == 15;
    }
}
