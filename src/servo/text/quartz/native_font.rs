extern mod core_foundation;
extern mod core_graphics;
extern mod core_text;

use font::{FontMetrics, FractionalPixel};
use font_cache::native::NativeFontCache;

use au = gfx::geometry;
use cast::transmute;
use glyph::GlyphIndex;
use libc::size_t;
use ptr::null;

use cf = core_foundation;
use cf::base::{
    CFIndex,
    CFRelease,
    CFTypeRef
};
use cf::string::UniChar;

use cg = core_graphics;
use cg::base::{CGFloat, CGAffineTransform};
use cg::data_provider::{
    CGDataProviderCreateWithData,
    CGDataProviderRef,
    CGDataProviderRelease,
};
use cg::font::{
    CGFontCreateWithDataProvider,
    CGFontRef,
    CGFontRelease,
    CGGlyph,
};
use cg::geometry::CGRect;

use ct = core_text;
use ct::font::{
    CTFontCreateWithGraphicsFont,
    CTFontRef,
    CTFontGetAdvancesForGlyphs,
    CTFontGetAscent,
    CTFontGetBoundingBox,
    CTFontGetDescent,
    CTFontGetGlyphsForCharacters,
    CTFontGetLeading,
    CTFontGetSize,
    CTFontGetUnderlinePosition,
    CTFontGetUnderlineThickness,
    CTFontGetXHeight,
    kCTFontDefaultOrientation,
};

pub struct QuartzNativeFont {
    fontprov: CGDataProviderRef,
    cgfont: CGFontRef,
    ctfont: CTFontRef,

    drop {
        assert self.ctfont.is_not_null();
        assert self.cgfont.is_not_null();
        assert self.fontprov.is_not_null();

        CFRelease(self.ctfont as CFTypeRef);
        CGFontRelease(self.cgfont);
        CGDataProviderRelease(self.fontprov);
    }
}

fn QuartzNativeFont(fontprov: CGDataProviderRef, cgfont: CGFontRef, pt_size: float) -> QuartzNativeFont {
    assert fontprov.is_not_null();
    assert cgfont.is_not_null();

    let ctfont = ctfont_from_cgfont(cgfont, pt_size);
    assert ctfont.is_not_null();

    QuartzNativeFont {
        fontprov : fontprov,
        cgfont : cgfont,
        ctfont : ctfont,
    }
}

impl QuartzNativeFont {
    fn glyph_index(codepoint: char) -> Option<GlyphIndex> {
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
        assert self.ctfont.is_not_null();

        let glyphs = ~[glyph as CGGlyph];
        let advance = do vec::as_imm_buf(glyphs) |glyph_buf, _l| {
            CTFontGetAdvancesForGlyphs(self.ctfont, kCTFontDefaultOrientation, glyph_buf, null(), 1)
        };

        return Some(advance as FractionalPixel);
    }

    fn get_metrics() -> FontMetrics {
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

fn ctfont_from_cgfont(cgfont: CGFontRef, pt_size: float) -> CTFontRef {
    assert cgfont.is_not_null();

    CTFontCreateWithGraphicsFont(cgfont, pt_size as CGFloat, null(), null())
}

pub fn create(_lib: &NativeFontCache, buf: @~[u8], pt_size: float) -> Result<QuartzNativeFont, ()> {
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
        true => Ok(QuartzNativeFont(fontprov, cgfont, pt_size)),
        false => Err(())
    }
    
}
