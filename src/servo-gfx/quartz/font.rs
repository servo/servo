/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

/// Implementation of Quartz (CoreGraphics) fonts.

extern mod core_foundation;
extern mod core_graphics;
extern mod core_text;

use geometry::Au;
use gfx_font::{CSSFontWeight, FontHandleMethods, FontMetrics, FontTable, FontTableMethods};
use gfx_font::{FontTableTag, FontWeight100, FontWeight200, FontWeight300, FontWeight400};
use gfx_font::{FontWeight500, FontWeight600, FontWeight700, FontWeight800, FontWeight900};
use gfx_font::{FractionalPixel, SpecifiedFontStyle};
use quartz::font::core_foundation::base::{CFIndex, CFWrapper};
use quartz::font::core_foundation::data::CFData;
use quartz::font::core_foundation::string::UniChar;
use quartz::font::core_graphics::data_provider::CGDataProvider;
use quartz::font::core_graphics::font::{CGFont, CGGlyph};
use quartz::font::core_graphics::geometry::CGRect;
use quartz::font::core_text::font::{CTFont, CTFontMethods, CTFontMethodsPrivate};
use quartz::font::core_text::font_descriptor::{SymbolicTraitAccessors, TraitAccessors};
use quartz::font::core_text::font_descriptor::kCTFontDefaultOrientation;
use quartz::font_context::QuartzFontContextHandle;
use quartz;
use text::glyph::GlyphIndex;

struct QuartzFontTable {
    data: CFData,
}


// Noncopyable.
impl Drop for QuartzFontTable { fn finalize(&self) {} }

pub impl QuartzFontTable {
    fn wrap(data: CFData) -> QuartzFontTable {
        QuartzFontTable { data: data }
    }
}

impl FontTableMethods for QuartzFontTable {
    fn with_buffer(&self, blk: &fn(*u8, uint)) {
        blk(self.data.bytes(), self.data.len());
    }
}

pub struct QuartzFontHandle {
    priv cgfont: Option<CGFont>,
    ctfont: CTFont,
}

pub impl QuartzFontHandle {
    fn new_from_buffer(_fctx: &QuartzFontContextHandle, buf: ~[u8],
                       style: &SpecifiedFontStyle) -> Result<QuartzFontHandle, ()> {
        let fontprov : CGDataProvider = vec::as_imm_buf(buf, |cbuf, len| {
            quartz::font::core_graphics::data_provider::new_from_buffer(cbuf, len)
        });

        let cgfont = quartz::font::core_graphics::font::create_with_data_provider(&fontprov);
        let ctfont = quartz::font::core_text::font::new_from_CGFont(&cgfont, style.pt_size);

        let result = Ok(QuartzFontHandle {
            cgfont: Some(cgfont),
            ctfont: ctfont,
        });

        return result;
    }

    fn new_from_CTFont(_fctx: &QuartzFontContextHandle, ctfont: CTFont) -> Result<QuartzFontHandle, ()> {
        let result = Ok(QuartzFontHandle {
            mut cgfont: None,
            ctfont: ctfont,
        });
        
        return result;
    }

    fn get_CGFont(&mut self) -> CGFont {
        match self.cgfont {
            Some(ref font) => CFWrapper::wrap_shared(*(font.borrow_ref())),
            None => {
                let cgfont = self.ctfont.copy_to_CGFont();
                self.cgfont = Some(CFWrapper::clone(&cgfont));
                cgfont
            }
        }
    }
}

impl FontHandleMethods for QuartzFontHandle {
    fn family_name(&self) -> ~str {
        self.ctfont.family_name()
    }
    
    fn face_name(&self) -> ~str {
        self.ctfont.face_name()
    }

    fn is_italic(&self) -> bool {
        self.ctfont.symbolic_traits().is_italic()
    }

    fn boldness(&self) -> CSSFontWeight {
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
        return FontWeight900;
    }

    fn clone_with_style(&self, fctx: &QuartzFontContextHandle,
                        style: &SpecifiedFontStyle)
                     -> Result<QuartzFontHandle,()> {
        let new_font = self.ctfont.clone_with_font_size(style.pt_size);
        return QuartzFontHandle::new_from_CTFont(fctx, new_font);
    }

    fn glyph_index(&self, codepoint: char) -> Option<GlyphIndex> {
        let characters: [UniChar,  ..1] = [codepoint as UniChar];
        let glyphs: [CGGlyph, ..1] = [0 as CGGlyph];
        let count: CFIndex = 1;

        let result = self.ctfont.get_glyphs_for_characters(ptr::to_unsafe_ptr(&characters[0]),
                                                           ptr::to_unsafe_ptr(&glyphs[0]),
                                                           count);

        if !result {
            // No glyph for this character
            return None;
        }

        assert!(glyphs[0] != 0); // FIXME: error handling
        return Some(glyphs[0] as GlyphIndex);
    }

    fn glyph_h_advance(&self, glyph: GlyphIndex) -> Option<FractionalPixel> {
        let glyphs = [glyph as CGGlyph];
        unsafe {
            let advance = self.ctfont.get_advances_for_glyphs(kCTFontDefaultOrientation,
                                                              &glyphs[0],
                                                              ptr::null(),
                                                              1);
            return Some(advance as FractionalPixel);
        }
    }

    fn get_metrics(&self) -> FontMetrics {
        let bounding_rect: CGRect = self.ctfont.bounding_box();
        let ascent = Au::from_pt(self.ctfont.ascent() as float);
        let descent = Au::from_pt(self.ctfont.descent() as float);

        let metrics =  FontMetrics {
            underline_size:   Au::from_pt(self.ctfont.underline_thickness() as float),
            // TODO(Issue #201): underline metrics are not reliable. Have to pull out of font table
            // directly.
            //
            // see also: https://bugs.webkit.org/show_bug.cgi?id=16768
            // see also: https://bugreports.qt-project.org/browse/QTBUG-13364
            underline_offset: Au::from_pt(self.ctfont.underline_position() as float),
            leading:          Au::from_pt(self.ctfont.leading() as float),
            x_height:         Au::from_pt(self.ctfont.x_height() as float),
            em_size:          ascent + descent,
            ascent:           ascent,
            descent:          descent,
            max_advance:      Au::from_pt(bounding_rect.size.width as float)
        };

        debug!("Font metrics (@%f pt): %?", self.ctfont.pt_size() as float, metrics);
        return metrics;
    }

    fn get_table_for_tag(&self, tag: FontTableTag) -> Option<FontTable> {
        let result : Option<CFData> = self.ctfont.get_font_table(tag);
        result.chain(|data| {
            Some(QuartzFontTable::wrap(data))
        })
    }

    fn face_identifier(&self) -> ~str {
        self.ctfont.postscript_name()
    }
}

