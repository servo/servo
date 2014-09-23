/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use geom::{Point2D, Rect, Size2D};
use std::mem;
use std::string;
use std::rc::Rc;
use std::cell::RefCell;
use servo_util::cache::{Cache, HashCache};
use style::computed_values::{font_weight, font_style, font_variant};
use sync::Arc;

use servo_util::geometry::Au;
use platform::font_context::FontContextHandle;
use platform::font::{FontHandle, FontTable};
use text::glyph::{GlyphStore, GlyphId};
use text::shaping::ShaperMethods;
use text::{Shaper, TextRun};
use font_template::FontTemplateDescriptor;
use platform::font_template::FontTemplateData;

// FontHandle encapsulates access to the platform's font API,
// e.g. quartz, FreeType. It provides access to metrics and tables
// needed by the text shaper as well as access to the underlying font
// resources needed by the graphics layer to draw glyphs.

pub trait FontHandleMethods {
    fn new_from_template(fctx: &FontContextHandle, template: Arc<FontTemplateData>, pt_size: Option<f64>)
                    -> Result<Self,()>;
    fn get_template(&self) -> Arc<FontTemplateData>;
    fn family_name(&self) -> String;
    fn face_name(&self) -> String;
    fn is_italic(&self) -> bool;
    fn boldness(&self) -> font_weight::T;

    fn glyph_index(&self, codepoint: char) -> Option<GlyphId>;
    fn glyph_h_advance(&self, GlyphId) -> Option<FractionalPixel>;
    fn glyph_h_kerning(&self, GlyphId, GlyphId) -> FractionalPixel;
    fn get_metrics(&self) -> FontMetrics;
    fn get_table_for_tag(&self, FontTableTag) -> Option<FontTable>;
}

// Used to abstract over the shaper's choice of fixed int representation.
pub type FractionalPixel = f64;

pub type FontTableTag = u32;

pub trait FontTableTagConversions {
    fn tag_to_str(&self) -> String;
}

impl FontTableTagConversions for FontTableTag {
    fn tag_to_str(&self) -> String {
        unsafe {
            let reversed = string::raw::from_buf_len(mem::transmute(self), 4);
            return String::from_chars([reversed.as_slice().char_at(3),
                                       reversed.as_slice().char_at(2),
                                       reversed.as_slice().char_at(1),
                                       reversed.as_slice().char_at(0)]);
        }
    }
}

pub trait FontTableMethods {
    fn with_buffer(&self, |*const u8, uint|);
}

#[deriving(Clone)]
pub struct FontMetrics {
    pub underline_size:   Au,
    pub underline_offset: Au,
    pub strikeout_size:   Au,
    pub strikeout_offset: Au,
    pub leading:          Au,
    pub x_height:         Au,
    pub em_size:          Au,
    pub ascent:           Au,
    pub descent:          Au,
    pub max_advance:      Au,
    pub line_gap:         Au,
}

// TODO(Issue #179): eventually this will be split into the specified
// and used font styles.  specified contains uninterpreted CSS font
// property values, while 'used' is attached to gfx::Font to descript
// the instance's properties.
//
// For now, the cases are differentiated with a typedef
#[deriving(Clone, PartialEq)]
pub struct FontStyle {
    pub pt_size: f64,
    pub weight: font_weight::T,
    pub style: font_style::T,
    pub families: Vec<String>,
    pub variant: font_variant::T,
    // TODO(Issue #198): font-stretch, text-decoration, size-adjust
}

pub type SpecifiedFontStyle = FontStyle;
pub type UsedFontStyle = FontStyle;

pub struct Font {
    pub handle: FontHandle,
    pub metrics: FontMetrics,
    pub variant: font_variant::T,
    pub descriptor: FontTemplateDescriptor,
    pub requested_pt_size: f64,
    pub actual_pt_size: f64,
    pub shaper: Option<Shaper>,
    pub shape_cache: HashCache<String, Arc<GlyphStore>>,
    pub glyph_advance_cache: HashCache<u32, FractionalPixel>,
}

impl Font {
    pub fn shape_text(&mut self, text: String, is_whitespace: bool) -> Arc<GlyphStore> {
        self.make_shaper();
        let shaper = &self.shaper;
        self.shape_cache.find_or_create(&text, |txt| {
            let mut glyphs = GlyphStore::new(text.as_slice().char_len() as int, is_whitespace);
            shaper.as_ref().unwrap().shape_text(txt.as_slice(), &mut glyphs);
            Arc::new(glyphs)
        })
    }

    fn make_shaper<'a>(&'a mut self) -> &'a Shaper {
        // fast path: already created a shaper
        match self.shaper {
            Some(ref shaper) => {
                let s: &'a Shaper = shaper;
                return s;
            },
            None => {}
        }

        let shaper = Shaper::new(self);
        self.shaper = Some(shaper);
        self.shaper.as_ref().unwrap()
    }

    pub fn get_table_for_tag(&self, tag: FontTableTag) -> Option<FontTable> {
        let result = self.handle.get_table_for_tag(tag);
        let status = if result.is_some() { "Found" } else { "Didn't find" };

        debug!("{:s} font table[{:s}] with family={}, face={}",
               status, tag.tag_to_str(),
               self.handle.family_name(), self.handle.face_name());

        return result;
    }

    pub fn glyph_index(&self, codepoint: char) -> Option<GlyphId> {
        let codepoint = match self.variant {
            font_variant::small_caps => codepoint.to_uppercase(),
            font_variant::normal => codepoint,
        };
        self.handle.glyph_index(codepoint)
    }

    pub fn glyph_h_kerning(&mut self, first_glyph: GlyphId, second_glyph: GlyphId) -> FractionalPixel {
        self.handle.glyph_h_kerning(first_glyph, second_glyph)
    }

    pub fn glyph_h_advance(&mut self, glyph: GlyphId) -> FractionalPixel {
        let handle = &self.handle;
        self.glyph_advance_cache.find_or_create(&glyph, |glyph| {
            match handle.glyph_h_advance(*glyph) {
                Some(adv) => adv,
                None => 10f64 as FractionalPixel // FIXME: Need fallback strategy
            }
        })
    }
}

pub struct FontGroup {
    pub fonts: Vec<Rc<RefCell<Font>>>,
}

impl FontGroup {
    pub fn new(fonts: Vec<Rc<RefCell<Font>>>) -> FontGroup {
        FontGroup {
            fonts: fonts
        }
    }

    pub fn create_textrun(&self, text: String) -> TextRun {
        assert!(self.fonts.len() > 0);

        // TODO(Issue #177): Actually fall back through the FontGroup when a font is unsuitable.
        TextRun::new(&mut *self.fonts[0].borrow_mut(), text.clone())
    }
}

pub struct RunMetrics {
    // may be negative due to negative width (i.e., kerning of '.' in 'P.T.')
    pub advance_width: Au,
    pub ascent: Au, // nonzero
    pub descent: Au, // nonzero
    // this bounding box is relative to the left origin baseline.
    // so, bounding_box.position.y = -ascent
    pub bounding_box: Rect<Au>
}

impl RunMetrics {
    pub fn new(advance: Au, ascent: Au, descent: Au) -> RunMetrics {
        let bounds = Rect(Point2D(Au(0), -ascent),
                          Size2D(advance, ascent + descent));

        // TODO(Issue #125): support loose and tight bounding boxes; using the
        // ascent+descent and advance is sometimes too generous and
        // looking at actual glyph extents can yield a tighter box.

        RunMetrics {
            advance_width: advance,
            bounding_box: bounds,
            ascent: ascent,
            descent: descent,
        }
    }
}
