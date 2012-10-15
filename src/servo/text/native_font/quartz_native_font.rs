extern mod cocoa;

export QuartzNativeFont, with_test_native_font, create;

use font::{FontMetrics, FractionalPixel};

use au = gfx::geometry;
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
use cast::transmute;
use coretext::CTFontRef;
use coretext::coretext::CFRelease;

mod coretext {

    pub type CTFontRef = *u8;
    pub type UniChar = libc::c_ushort;
    pub type CGGlyph = libc::c_ushort;
    pub type CFIndex = libc::c_long;

    pub type CTFontOrientation = u32;
    pub const kCTFontDefaultOrientation: CTFontOrientation = 0;
    pub const kCTFontHorizontalOrientation: CTFontOrientation = 1;
    pub const kCTFontVerticalOrientation: CTFontOrientation = 2;

    // TODO: this is actually a libc::c_float on 32bit
    pub type CGFloat = libc::c_double;

    pub struct CGSize {
        width: CGFloat,
        height: CGFloat,
    }

    pub struct CGPoint {
        x: CGFloat,
        y: CGFloat,
    }

    pub struct CGRect {
        origin: CGPoint,
        size: CGSize
    }

    pub type CGAffineTransform = ();
    pub type CTFontDescriptorRef = *u8;

    #[nolink]
    #[link_args = "-framework ApplicationServices"]
    pub extern mod coretext {
        pub fn CTFontCreateWithGraphicsFont(graphicsFont: CGFontRef, size: CGFloat, matrix: *CGAffineTransform, attributes: CTFontDescriptorRef) -> CTFontRef;
        pub fn CTFontGetGlyphsForCharacters(font: CTFontRef, characters: *UniChar, glyphs: *CGGlyph, count: CFIndex) -> bool;
        pub fn CTFontGetAdvancesForGlyphs(font: CTFontRef, orientation: CTFontOrientation, glyphs: *CGGlyph, advances: *CGSize, count: CFIndex) -> libc::c_double;

        pub fn CTFontGetSize(font: CTFontRef) -> CGFloat;

        /* metrics API */
        pub fn CTFontGetAscent(font: CTFontRef) -> CGFloat;
        pub fn CTFontGetDescent(font: CTFontRef) -> CGFloat;
        pub fn CTFontGetLeading(font: CTFontRef) -> CGFloat;
        pub fn CTFontGetUnitsPerEm(font: CTFontRef) -> libc::c_uint;
        pub fn CTFontGetUnderlinePosition(font: CTFontRef) -> CGFloat;
        pub fn CTFontGetUnderlineThickness(font: CTFontRef) -> CGFloat;
        pub fn CTFontGetXHeight(font: CTFontRef) -> CGFloat;
        pub fn CTFontGetBoundingBox(font: CTFontRef) -> CGRect;
            
        pub fn CFRelease(font: CTFontRef);
    }
}

pub struct QuartzNativeFont {
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

    fn glyph_h_advance(glyph: GlyphIndex) -> Option<FractionalPixel> {
        use coretext::{CGGlyph, kCTFontDefaultOrientation};
        use coretext::coretext::{CTFontGetAdvancesForGlyphs};

        assert self.ctfont.is_not_null();
        let glyphs = ~[glyph as CGGlyph];
        let advance = do vec::as_imm_buf(glyphs) |glyph_buf, _l| {
            CTFontGetAdvancesForGlyphs(self.ctfont, kCTFontDefaultOrientation, glyph_buf, null(), 1)
        };

        return Some(advance as FractionalPixel);
    }

    fn get_metrics() -> FontMetrics {
        use coretext::CGRect;
        use coretext::coretext::*;

        let ctfont = self.ctfont;
        assert ctfont.is_not_null();

        let bounding_rect: CGRect = CTFontGetBoundingBox(ctfont);
        let ascent = au::from_pt(CTFontGetAscent(ctfont) as float);
        let descent = au::from_pt(CTFontGetDescent(ctfont) as float);

        let metrics =  FontMetrics {
            underline_size:   au::from_pt(CTFontGetUnderlineThickness(ctfont) as float),
            // TODO: underline metrics are not reliable. Have to pull out of font table directly.
            // see also: https://bugs.webkit.org/show_bug.cgi?id=16768
            // see also: https://bugreports.qt-project.org/browse/QTBUG-13364
            underline_offset: au::from_pt(CTFontGetUnderlinePosition(ctfont) as float),
            leading:          au::from_pt(CTFontGetLeading(ctfont) as float),
            x_height:         au::from_pt(CTFontGetXHeight(ctfont) as float),
            em_size:          ascent + descent,
            ascent:           ascent,
            descent:          descent,
            max_advance:      au::from_pt(bounding_rect.size.width as float)
        };

        debug!("Font metrics (@%f pt): %?", CTFontGetSize(ctfont) as float, metrics);
        return metrics;
    }
}

fn ctfont_from_cgfont(cgfont: CGFontRef) -> coretext::CTFontRef {
    use coretext::CGFloat;
    use coretext::coretext::CTFontCreateWithGraphicsFont;

    assert cgfont.is_not_null();
    // TODO: use actual font size here!
    CTFontCreateWithGraphicsFont(cgfont, 21f as CGFloat, null(), null())
}

pub fn create(buf: @~[u8]) -> Result<QuartzNativeFont, ()> {
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

pub fn with_test_native_font(f: fn@(nf: &NativeFont)) {
    use font::test_font_bin;
    use unwrap_result = result::unwrap;

    let buf = @test_font_bin();
    let res = create(buf);
    let font = unwrap_result(res);
    f(&font);
}
