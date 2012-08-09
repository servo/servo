use cocoa;

export QuartzNativeFont, with_test_native_font;

import libc::size_t;
import ptr::null;
import unsafe::reinterpret_cast;
import result::{result, ok};
import glyph::GlyphIndex;
import cocoa::cg::{
    CGDataProviderRef,
    CGFontRef
};
import cocoa::cg::cg::{
    CGDataProviderCreateWithData,
    CGDataProviderRelease,
    CGFontCreateWithDataProvider,
    CGFontRelease
};

class QuartzNativeFont/& {
    let fontprov: CGDataProviderRef;
    let cgfont: CGFontRef;

    new (fontprov: CGDataProviderRef, cgfont: CGFontRef) {
        assert fontprov.is_not_null();
        assert cgfont.is_not_null();

        self.fontprov = fontprov;
        self.cgfont = cgfont;
    }

    drop {
        assert self.cgfont.is_not_null();
        assert self.fontprov.is_not_null();

        CGFontRelease(self.cgfont);
        CGDataProviderRelease(self.fontprov);
    }

    fn glyph_index(_codepoint: char) -> option<GlyphIndex> {
        // FIXME
        some(40u)
    }

    // FIXME: What unit is this returning? Let's have a custom type
    fn glyph_h_advance(_glyph: GlyphIndex) -> option<int> {
        // FIXME
        some(15)
    }
}

fn create(buf: ~[u8]) -> result<QuartzNativeFont, ()> {
    let fontprov = vec::as_buf(buf, |cbuf, len| {
        CGDataProviderCreateWithData(
            null(),
            unsafe { reinterpret_cast(cbuf) },
            len as size_t,
            null())
    });
    // FIXME: Error handling
    assert fontprov.is_not_null();
    let cgfont = CGFontCreateWithDataProvider(fontprov);
    // FIXME: Error handling
    assert cgfont.is_not_null();

    return ok(QuartzNativeFont(fontprov, cgfont));
}

fn with_test_native_font(f: fn@(nf: &NativeFont)) {
    import font::test_font_bin;
    import unwrap_result = result::unwrap;

    let buf = test_font_bin();
    let res = create(buf);
    let font = unwrap_result(res);
    f(&font);
}
