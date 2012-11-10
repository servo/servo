extern mod core_foundation;
extern mod core_graphics;
extern mod core_text;

use font_context::QuartzFontContextHandle;
use gfx::au;
use gfx::font::{
    CSSFontWeight,
    FontHandleMethods,
    FontMetrics,
    FontWeight100,
    FontWeight200,
    FontWeight300,
    FontWeight400,
    FontWeight500,
    FontWeight600,
    FontWeight700,
    FontWeight800,
    FontWeight900,
    FractionalPixel
};
use text::glyph::GlyphIndex;

use libc::size_t;

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
    CGDataProviderRef, CGDataProvider
};
use cg::font::{
    CGFontCreateWithDataProvider,
    CGFontRef,
    CGFontRelease,
    CGGlyph,
};
use cg::geometry::CGRect;

use ct = core_text;
use ct::font::CTFont;
use ct::font_descriptor::{
    kCTFontDefaultOrientation,
    CTFontSymbolicTraits,
    SymbolicTraitAccessors,
};

pub struct QuartzFontHandle {
    priv mut cgfont: Option<CGFontRef>,
    ctfont: CTFont,

    drop {
        // TODO(Issue #152): use a wrapped CGFont.
        do (copy self.cgfont).iter |cgfont| {
            assert cgfont.is_not_null();
            CGFontRelease(*cgfont);
        }
    }
}

pub impl QuartzFontHandle {
    static fn new_from_buffer(_fctx: &QuartzFontContextHandle, buf: @~[u8], pt_size: float) -> Result<QuartzFontHandle, ()> {
        let fontprov = vec::as_imm_buf(*buf, |cbuf, len| {
            CGDataProvider::new_from_buffer(cbuf, len)
        });

        let cgfont = CGFontCreateWithDataProvider(fontprov.get_ref());
        if cgfont.is_null() { return Err(()); }

        let ctfont = CTFont::new_from_CGFont(cgfont, pt_size);

        let result = Ok(QuartzFontHandle {
            cgfont : Some(cgfont),
            ctfont : move ctfont,
        });

        return move result;
    }

    static fn new_from_CTFont(_fctx: &QuartzFontContextHandle, ctfont: CTFont) -> Result<QuartzFontHandle, ()> {
        let result = Ok(QuartzFontHandle {
            mut cgfont: None,
            ctfont: move ctfont,
        });
        
        return move result;
    }

    fn get_CGFont() -> CGFontRef {
        match self.cgfont {
            Some(cg) => cg,
            None => {
                let cgfont = self.ctfont.copy_to_CGFont();
                self.cgfont = Some(cgfont);
                cgfont
            }
        }
    }
}

pub impl QuartzFontHandle : FontHandleMethods {
    pure fn family_name() -> ~str {
        self.ctfont.family_name()
    }
    
    pure fn face_name() -> ~str {
        self.ctfont.face_name()
    }

    pure fn is_italic() -> bool {
        self.ctfont.symbolic_traits().is_italic()
    }

    pure fn boldness() -> CSSFontWeight {
        // -1.0 to 1.0
        let normalized = unsafe { self.ctfont.all_traits().normalized_weight() };
        // 0.0 to 9.0
        let normalized = (normalized + 1.0) / 2.0 * 9.0;
        if normalized < 1.0 { return FontWeight100; }
        if normalized < 2.0 { return FontWeight200; }
        if normalized < 3.0 { return FontWeight300; }
        if normalized < 4.0 { return FontWeight400; }
        if normalized < 5.0 { return FontWeight500; }
        if normalized < 6.0 { return FontWeight600; }
        if normalized < 7.0 { return FontWeight700; }
        if normalized < 8.0 { return FontWeight800; }
        else { return FontWeight900; }
    }


    fn glyph_index(codepoint: char) -> Option<GlyphIndex> {
        let characters: ~[UniChar] = ~[codepoint as UniChar];
        let glyphs: ~[mut CGGlyph] = ~[mut 0 as CGGlyph];
        let count: CFIndex = 1;

        let result = do vec::as_imm_buf(characters) |character_buf, _l| {
            do vec::as_imm_buf(glyphs) |glyph_buf, _l| {
                self.ctfont.get_glyphs_for_characters(character_buf, glyph_buf, count)
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
        let glyphs = ~[glyph as CGGlyph];
        let advance = do vec::as_imm_buf(glyphs) |glyph_buf, _l| {
            self.ctfont.get_advances_for_glyphs(kCTFontDefaultOrientation, glyph_buf, ptr::null(), 1)
        };

        return Some(advance as FractionalPixel);
    }

    fn get_metrics() -> FontMetrics {
        let bounding_rect: CGRect = self.ctfont.bounding_box();
        let ascent = au::from_pt(self.ctfont.ascent() as float);
        let descent = au::from_pt(self.ctfont.descent() as float);

        let metrics =  FontMetrics {
            underline_size:   au::from_pt(self.ctfont.underline_thickness() as float),
            // TODO: underline metrics are not reliable. Have to pull out of font table directly.
            // see also: https://bugs.webkit.org/show_bug.cgi?id=16768
            // see also: https://bugreports.qt-project.org/browse/QTBUG-13364
            underline_offset: au::from_pt(self.ctfont.underline_position() as float),
            leading:          au::from_pt(self.ctfont.leading() as float),
            x_height:         au::from_pt(self.ctfont.x_height() as float),
            em_size:          ascent + descent,
            ascent:           ascent,
            descent:          descent,
            max_advance:      au::from_pt(bounding_rect.size.width as float)
        };

        debug!("Font metrics (@%f pt): %?", self.ctfont.pt_size() as float, metrics);
        return metrics;
    }
}

