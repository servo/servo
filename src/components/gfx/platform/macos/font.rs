/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

/// Implementation of Quartz (CoreGraphics) fonts.

extern mod core_foundation;
extern mod core_graphics;
extern mod core_text;

use font::{FontHandleMethods, FontMetrics, FontTableMethods};
use font::FontTableTag;
use font::{FractionalPixel, SpecifiedFontStyle};
use servo_util::geometry::{Au, px_to_pt};
use servo_util::geometry;
use platform::macos::font_context::FontContextHandle;
use text::glyph::GlyphIndex;
use style::computed_values::font_weight;

use core_foundation::base::CFIndex;
use core_foundation::data::CFData;
use core_foundation::string::UniChar;
use core_graphics::data_provider::CGDataProvider;
use core_graphics::font::{CGFont, CGGlyph};
use core_graphics::geometry::CGRect;
use core_text::font::CTFont;
use core_text::font_descriptor::{SymbolicTraitAccessors, TraitAccessors};
use core_text::font_descriptor::{kCTFontDefaultOrientation};
use core_text;

use std::ptr;

pub struct FontTable {
    data: CFData,
}

// Noncopyable.
impl Drop for FontTable {
    fn drop(&mut self) {}
}

impl FontTable {
    pub fn wrap(data: CFData) -> FontTable {
        FontTable { data: data }
    }
}

impl FontTableMethods for FontTable {
    fn with_buffer(&self, blk: |*u8, uint|) {
        blk(self.data.bytes().as_ptr(), self.data.len() as uint);
    }
}

pub struct FontHandle {
    priv cgfont: Option<CGFont>,
    ctfont: CTFont,
}

impl FontHandle {
    pub fn new_from_CTFont(_: &FontContextHandle, ctfont: CTFont) -> Result<FontHandle, ()> {
        Ok(FontHandle {
            cgfont: None,
            ctfont: ctfont,
        })
    }

    pub fn get_CGFont(&mut self) -> CGFont {
        match self.cgfont {
            Some(ref font) => font.clone(),
            None => {
                let cgfont = self.ctfont.copy_to_CGFont();
                self.cgfont = Some(cgfont.clone());
                cgfont
            }
        }
    }
}

impl FontHandleMethods for FontHandle {
    fn new_from_buffer(_: &FontContextHandle, buf: ~[u8], style: &SpecifiedFontStyle)
                    -> Result<FontHandle, ()> {
        let fontprov = CGDataProvider::from_buffer(buf);
        let cgfont = CGFont::from_data_provider(fontprov);
        let ctfont = core_text::font::new_from_CGFont(&cgfont, style.pt_size);

        let result = Ok(FontHandle {
            cgfont: Some(cgfont),
            ctfont: ctfont,
        });

        return result;
    }

    fn family_name(&self) -> ~str {
        self.ctfont.family_name()
    }
    
    fn face_name(&self) -> ~str {
        self.ctfont.face_name()
    }

    fn is_italic(&self) -> bool {
        self.ctfont.symbolic_traits().is_italic()
    }

    fn boldness(&self) -> font_weight::T {
        // -1.0 to 1.0
        let normalized = self.ctfont.all_traits().normalized_weight();
        // 0.0 to 9.0
        let normalized = (normalized + 1.0) / 2.0 * 9.0;
        if normalized < 1.0 { return font_weight::Weight100; }
        if normalized < 2.0 { return font_weight::Weight200; }
        if normalized < 3.0 { return font_weight::Weight300; }
        if normalized < 4.0 { return font_weight::Weight400; }
        if normalized < 5.0 { return font_weight::Weight500; }
        if normalized < 6.0 { return font_weight::Weight600; }
        if normalized < 7.0 { return font_weight::Weight700; }
        if normalized < 8.0 { return font_weight::Weight800; }
        return font_weight::Weight900;
    }

    fn clone_with_style(&self, fctx: &FontContextHandle, style: &SpecifiedFontStyle)
                     -> Result<FontHandle,()> {
        let new_font = self.ctfont.clone_with_font_size(style.pt_size);
        return FontHandle::new_from_CTFont(fctx, new_font);
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
        let advance = self.ctfont.get_advances_for_glyphs(kCTFontDefaultOrientation,
                                                          &glyphs[0],
                                                          ptr::null(),
                                                          1);
        Some(advance as FractionalPixel)
    }

    fn get_metrics(&self) -> FontMetrics {
        let bounding_rect: CGRect = self.ctfont.bounding_box();
        let ascent = Au::from_pt(self.ctfont.ascent() as f64);
        let descent = Au::from_pt(self.ctfont.descent() as f64);
        let em_size = Au::from_frac_px(self.ctfont.pt_size() as f64);

        let scale = px_to_pt(self.ctfont.pt_size() as f64) / (self.ctfont.ascent() as f64 + self.ctfont.descent() as f64);

        let metrics =  FontMetrics {
            underline_size:   Au::from_pt(self.ctfont.underline_thickness() as f64),
            // TODO(Issue #201): underline metrics are not reliable. Have to pull out of font table
            // directly.
            //
            // see also: https://bugs.webkit.org/show_bug.cgi?id=16768
            // see also: https://bugreports.qt-project.org/browse/QTBUG-13364
            underline_offset: Au::from_pt(self.ctfont.underline_position() as f64),
            strikeout_size:   geometry::from_pt(0.0), // FIXME(Issue #942)
            strikeout_offset: geometry::from_pt(0.0), // FIXME(Issue #942)
            leading:          Au::from_pt(self.ctfont.leading() as f64),
            x_height:         Au::from_pt(self.ctfont.x_height() as f64),
            em_size:          em_size,
            ascent:           ascent.scale_by(scale),
            descent:          descent.scale_by(scale),
            max_advance:      Au::from_pt(bounding_rect.size.width as f64)
        };

        debug!("Font metrics (@{:f} pt): {:?}", self.ctfont.pt_size() as f64, metrics);
        return metrics;
    }

    fn get_table_for_tag(&self, tag: FontTableTag) -> Option<FontTable> {
        let result: Option<CFData> = self.ctfont.get_font_table(tag);
        result.and_then(|data| {
            Some(FontTable::wrap(data))
        })
    }

    fn face_identifier(&self) -> ~str {
        self.ctfont.postscript_name()
    }
}

