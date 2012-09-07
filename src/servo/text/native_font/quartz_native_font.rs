use cocoa;

export QuartzNativeFont, with_test_native_font, create;

import libc::size_t;
import ptr::null;
import unsafe::reinterpret_cast;
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

mod coretext {

    type CTFontRef = *u8;
    type UniChar = libc::c_ushort;
    type CGGlyph = libc::c_ushort;
    type CFIndex = libc::c_long;

    type CTFontOrientation = u32;
    const kCTFontDefaultOrientation: CTFontOrientation = 0;
    const kCTFontHorizontalOrientation: CTFontOrientation = 1;
    const kCTFontVerticalOrientation: CTFontOrientation = 2;

    type CGFloat = libc::c_double;

    struct CGSize {
        width: CGFloat;
        height: CGFloat;
    }

    type CGAffineTransform = ();
    type CTFontDescriptorRef = *u8;

    #[nolink]
    #[link_args = "-framework ApplicationServices"]
    extern mod coretext {
        fn CTFontCreateWithGraphicsFont(graphicsFont: CGFontRef, size: CGFloat, matrix: *CGAffineTransform, attributes: CTFontDescriptorRef) -> CTFontRef;
        fn CTFontGetGlyphsForCharacters(font: CTFontRef, characters: *UniChar, glyphs: *CGGlyph, count: CFIndex) -> bool;
        fn CTFontGetAdvancesForGlyphs(font: CTFontRef, orientation: CTFontOrientation, glyphs: *CGGlyph, advances: *CGSize, count: CFIndex) -> libc::c_double;
        fn CFRelease(font: CTFontRef);
    }
}

struct QuartzNativeFont/& {
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

    fn glyph_index(codepoint: char) -> Option<GlyphIndex> {

        import coretext::{UniChar, CGGlyph, CFIndex};
        import coretext::coretext::{CFRelease, CTFontGetGlyphsForCharacters};
         
        let ctfont = ctfont_from_cgfont(self.cgfont);
        assert ctfont.is_not_null();
        let characters: ~[UniChar] = ~[codepoint as UniChar];
        let glyphs: ~[mut CGGlyph] = ~[mut 0 as CGGlyph];
        let count: CFIndex = 1;

        let result = do vec::as_buf(characters) |character_buf, _l| {
            do vec::as_buf(glyphs) |glyph_buf, _l| {
                CTFontGetGlyphsForCharacters(ctfont, character_buf, glyph_buf, count)
            }
        };

        assert result != false; // FIXME: error handling

        CFRelease(ctfont);

        assert glyphs[0] != 0; // FIXME: error handling
        return Some(glyphs[0] as GlyphIndex);
    }

    // FIXME: What unit is this returning? Let's have a custom type
    fn glyph_h_advance(glyph: GlyphIndex) -> Option<int> {
        import coretext::{CGGlyph, kCTFontDefaultOrientation};
        import coretext::coretext::{CFRelease, CTFontGetAdvancesForGlyphs};

        let ctfont = ctfont_from_cgfont(self.cgfont);
        assert ctfont.is_not_null();
        let glyphs = ~[glyph as CGGlyph];
        let advance = do vec::as_buf(glyphs) |glyph_buf, _l| {
            CTFontGetAdvancesForGlyphs(ctfont, kCTFontDefaultOrientation, glyph_buf, null(), 1)
        };

        CFRelease(ctfont);

        return Some(advance as int);
    }
}

fn ctfont_from_cgfont(+cgfont: CGFontRef) -> coretext::CTFontRef unsafe {
    import coretext::CGFloat;
    import coretext::coretext::CTFontCreateWithGraphicsFont;

    assert cgfont.is_not_null();
    CTFontCreateWithGraphicsFont(cgfont, 21f as CGFloat, null(), null())
}

fn create(buf: &~[u8]) -> Result<QuartzNativeFont, ()> {
    let fontprov = vec::as_buf(*buf, |cbuf, len| {
        CGDataProviderCreateWithData(
            null(),
            unsafe { reinterpret_cast(&cbuf) },
            len as size_t,
            null())
    });
    // FIXME: Error handling
    assert fontprov.is_not_null();
    let cgfont = CGFontCreateWithDataProvider(fontprov);
    // FIXME: Error handling
    assert cgfont.is_not_null();

    return Ok(QuartzNativeFont(fontprov, cgfont));
}

fn with_test_native_font(f: fn@(nf: &NativeFont)) {
    import font::test_font_bin;
    import unwrap_result = result::unwrap;

    let buf = test_font_bin();
    let res = create(&buf);
    let font = unwrap_result(res);
    f(&font);
}
