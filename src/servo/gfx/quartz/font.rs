extern mod core_foundation;
extern mod core_graphics;
extern mod core_text;

use font_context::QuartzFontContextHandle;
use gfx::au;
use gfx::font::{FontMetrics, FractionalPixel};
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
};

pub struct QuartzFontHandle {
    cgfont: CGFontRef,
    ctfont: CTFont,

    drop {
        assert self.cgfont.is_not_null();
        CGFontRelease(self.cgfont);
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
            cgfont : cgfont,
            ctfont : move ctfont,
        });

        return move result;
    }

    static fn new_from_CTFont(_fctx: &QuartzFontContextHandle, ctfont: CTFont) -> Result<QuartzFontHandle, ()> {
        let cgfont = ctfont.copy_to_CGFont();
        let result = Ok(QuartzFontHandle {
            cgfont: cgfont,
            ctfont: move ctfont,
        });
        
        return move result;
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

