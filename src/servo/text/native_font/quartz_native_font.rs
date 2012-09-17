extern mod cocoa;

export QuartzNativeFont, with_test_native_font, create;

use font::FontMetrics;

use libc::size_t;
use ptr::null;
use glyph::GlyphIndex;
use cocoa::cg::{
    CGDataProviderRef,
    CGFontRef
};
use cocoa::cg::cg::{
    CGDataProviderCreateWithData,
    CGDataProviderRelease,
    CGFontCreateWithDataProvider,
    CGFontRelease
};
use unsafe::transmute;
use coretext::CTFontRef;
use coretext::coretext::CFRelease;

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
        width: CGFloat,
        height: CGFloat,
    }

    struct CGPoint {
        x: CGFloat,
        y: CGFloat,
    }

    struct CGRect {
        origin: CGPoint,
        size: CGSize
    }

    type CGAffineTransform = ();
    type CTFontDescriptorRef = *u8;

    #[nolink]
    #[link_args = "-framework ApplicationServices"]
    extern mod coretext {
        fn CTFontCreateWithGraphicsFont(graphicsFont: CGFontRef, size: CGFloat, matrix: *CGAffineTransform, attributes: CTFontDescriptorRef) -> CTFontRef;
        fn CTFontGetGlyphsForCharacters(font: CTFontRef, characters: *UniChar, glyphs: *CGGlyph, count: CFIndex) -> bool;
        fn CTFontGetAdvancesForGlyphs(font: CTFontRef, orientation: CTFontOrientation, glyphs: *CGGlyph, advances: *CGSize, count: CFIndex) -> libc::c_double;

        /* metrics API */
        fn CTFontGetAscent(font: CTFontRef) -> libc::c_float;
        fn CTFontGetDescent(font: CTFontRef) -> libc::c_float;
        fn CTFontGetLeading(font: CTFontRef) -> libc::c_float;
        fn CTFontGetUnitsPerEm(font: CTFontRef) -> libc::c_uint;
        fn CTFontGetUnderlinePosition(font: CTFontRef) -> libc::c_float;
        fn CTFontGetUnderlineThickness(font: CTFontRef) -> libc::c_float;
        fn CTFontGetXHeight(font: CTFontRef) -> libc::c_float;
        fn CTFontGetBoundingBox(font: CTFontRef) -> CGRect;
            
        fn CFRelease(font: CTFontRef);
    }
}

struct QuartzNativeFont {
    fontprov: CGDataProviderRef,
    cgfont: CGFontRef,
    ctfont: CTFontRef,

    drop {
        assert self.ctfont.is_not_null();
        assert self.cgfont.is_not_null();
        assert self.fontprov.is_not_null();

        CFRelease(self.ctfont);
        CGFontRelease(self.cgfont);
        CGDataProviderRelease(self.fontprov);
    }
}

fn QuartzNativeFont(fontprov: CGDataProviderRef, cgfont: CGFontRef) -> QuartzNativeFont {
    assert fontprov.is_not_null();
    assert cgfont.is_not_null();

    let ctfont = ctfont_from_cgfont(cgfont);
    assert ctfont.is_not_null();

    QuartzNativeFont {
        fontprov : fontprov,
        cgfont : cgfont,
        ctfont : ctfont,
    }
}

impl QuartzNativeFont {
    fn glyph_index(codepoint: char) -> Option<GlyphIndex> {

        use coretext::{UniChar, CGGlyph, CFIndex};
        use coretext::coretext::{CTFontGetGlyphsForCharacters};

        assert self.ctfont.is_not_null();
        let characters: ~[UniChar] = ~[codepoint as UniChar];
        let glyphs: ~[mut CGGlyph] = ~[mut 0 as CGGlyph];
        let count: CFIndex = 1;

        let result = do vec::as_imm_buf(characters) |character_buf, _l| {
            do vec::as_imm_buf(glyphs) |glyph_buf, _l| {
                CTFontGetGlyphsForCharacters(self.ctfont, character_buf, glyph_buf, count)
            }
        };

        if !result {
            // No glyph for this character
            return None;
        }

        assert glyphs[0] != 0; // FIXME: error handling
        return Some(glyphs[0] as GlyphIndex);
    }

    // FIXME: What unit is this returning? Let's have a custom type
    fn glyph_h_advance(glyph: GlyphIndex) -> Option<int> {
        use coretext::{CGGlyph, kCTFontDefaultOrientation};
        use coretext::coretext::{CTFontGetAdvancesForGlyphs};

        assert self.ctfont.is_not_null();
        let glyphs = ~[glyph as CGGlyph];
        let advance = do vec::as_imm_buf(glyphs) |glyph_buf, _l| {
            CTFontGetAdvancesForGlyphs(self.ctfont, kCTFontDefaultOrientation, glyph_buf, null(), 1)
        };

        return Some(advance as int);
    }

    fn get_metrics() -> FontMetrics {
        use coretext::CGRect;
        use coretext::coretext::*;

        let ctfont = self.ctfont;
        assert ctfont.is_not_null();

        let convFactor : float = 1.0 / (CTFontGetUnitsPerEm(ctfont) as float);
        let bounding_rect: CGRect = CTFontGetBoundingBox(ctfont);
        let em_ascent = CTFontGetAscent(ctfont) as float * convFactor;
        let em_descent = CTFontGetDescent(ctfont) as float * convFactor;

        return FontMetrics {
            underline_size:   CTFontGetUnderlineThickness(ctfont) as float * convFactor,
            underline_offset: CTFontGetUnderlinePosition(ctfont) as float * convFactor,
            leading:          CTFontGetLeading(ctfont) as float * convFactor,
            x_height:         CTFontGetXHeight(ctfont) as float * convFactor,
            em_ascent:        CTFontGetAscent(ctfont) as float * convFactor,
            em_descent:       CTFontGetDescent(ctfont) as float * convFactor,
            em_height:        em_ascent + em_descent,
            max_advance:      bounding_rect.size.width as float * convFactor,
        }
    }
}

fn ctfont_from_cgfont(cgfont: CGFontRef) -> coretext::CTFontRef {
    use coretext::CGFloat;
    use coretext::coretext::CTFontCreateWithGraphicsFont;

    assert cgfont.is_not_null();
    CTFontCreateWithGraphicsFont(cgfont, 21f as CGFloat, null(), null())
}

fn create(buf: @~[u8]) -> Result<QuartzNativeFont, ()> {
    let fontprov = vec::as_imm_buf(*buf, |cbuf, len| {
        CGDataProviderCreateWithData(
            null(),
            unsafe { transmute(copy cbuf) },
            len as size_t,
            null())
    });
    // FIXME: Error handling
    assert fontprov.is_not_null();
    let cgfont = CGFontCreateWithDataProvider(fontprov);

    match cgfont.is_not_null() {
        true => Ok(QuartzNativeFont(fontprov, cgfont)),
        false => Err(())
    }
    
}

fn with_test_native_font(f: fn@(nf: &NativeFont)) {
    use font::test_font_bin;
    use unwrap_result = result::unwrap;

    let buf = @test_font_bin();
    let res = create(buf);
    let font = unwrap_result(res);
    f(&font);
}
