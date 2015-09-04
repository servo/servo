/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use euclid::{Point2D, Rect, Size2D};
use smallvec::SmallVec;
use std::borrow::ToOwned;
use std::cell::RefCell;
use std::mem;
use std::rc::Rc;
use std::slice;
use std::sync::Arc;
use style::computed_values::{font_stretch, font_variant, font_weight};
use style::properties::style_structs::Font as FontStyle;
use util::cache::HashCache;

use font_template::FontTemplateDescriptor;
use platform::font::{FontHandle, FontTable};
use platform::font_context::FontContextHandle;
use platform::font_template::FontTemplateData;
use text::Shaper;
use text::glyph::{GlyphStore, GlyphId};
use text::shaping::ShaperMethods;
use util::geometry::Au;

// FontHandle encapsulates access to the platform's font API,
// e.g. quartz, FreeType. It provides access to metrics and tables
// needed by the text shaper as well as access to the underlying font
// resources needed by the graphics layer to draw glyphs.

pub trait FontHandleMethods: Sized {
    fn new_from_template(fctx: &FontContextHandle, template: Arc<FontTemplateData>, pt_size: Option<Au>)
                    -> Result<Self, ()>;
    fn template(&self) -> Arc<FontTemplateData>;
    fn family_name(&self) -> String;
    fn face_name(&self) -> String;
    fn is_italic(&self) -> bool;
    fn boldness(&self) -> font_weight::T;
    fn stretchiness(&self) -> font_stretch::T;

    fn glyph_index(&self, codepoint: char) -> Option<GlyphId>;
    fn glyph_h_advance(&self, GlyphId) -> Option<FractionalPixel>;
    fn glyph_h_kerning(&self, GlyphId, GlyphId) -> FractionalPixel;
    fn metrics(&self) -> FontMetrics;
    fn get_table_for_tag(&self, FontTableTag) -> Option<Box<FontTable>>;
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
            let pointer = mem::transmute::<&u32, *const u8>(self);
            let mut bytes = slice::from_raw_parts(pointer, 4).to_vec();
            bytes.reverse();
            String::from_utf8_unchecked(bytes)
        }
    }
}

pub trait FontTableMethods {
    fn with_buffer<F>(&self, F) where F: FnOnce(*const u8, usize);
}

#[derive(Clone, Debug, Deserialize, Serialize)]
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
    pub average_advance:  Au,
    pub line_gap:         Au,
}

pub type SpecifiedFontStyle = FontStyle;

pub struct Font {
    pub handle: FontHandle,
    pub metrics: FontMetrics,
    pub variant: font_variant::T,
    pub descriptor: FontTemplateDescriptor,
    pub requested_pt_size: Au,
    pub actual_pt_size: Au,
    pub shaper: Option<Shaper>,
    pub shape_cache: HashCache<ShapeCacheEntry, Arc<GlyphStore>>,
    pub glyph_advance_cache: HashCache<u32, FractionalPixel>,
}

bitflags! {
    flags ShapingFlags: u8 {
        #[doc = "Set if the text is entirely whitespace."]
        const IS_WHITESPACE_SHAPING_FLAG = 0x01,
        #[doc = "Set if we are to ignore ligatures."]
        const IGNORE_LIGATURES_SHAPING_FLAG = 0x02,
        #[doc = "Set if we are to disable kerning."]
        const DISABLE_KERNING_SHAPING_FLAG = 0x04,
        #[doc = "Text direction is right-to-left."]
        const RTL_FLAG = 0x08,
    }
}

/// Various options that control text shaping.
#[derive(Clone, Eq, PartialEq, Hash, Copy)]
pub struct ShapingOptions {
    /// Spacing to add between each letter. Corresponds to the CSS 2.1 `letter-spacing` property.
    /// NB: You will probably want to set the `IGNORE_LIGATURES_SHAPING_FLAG` if this is non-null.
    pub letter_spacing: Option<Au>,
    /// Spacing to add between each word. Corresponds to the CSS 2.1 `word-spacing` property.
    pub word_spacing: Au,
    /// Various flags.
    pub flags: ShapingFlags,
}

/// An entry in the shape cache.
#[derive(Clone, Eq, PartialEq, Hash)]
pub struct ShapeCacheEntry {
    text: String,
    options: ShapingOptions,
}

impl Font {
    pub fn shape_text(&mut self, text: &str, options: &ShapingOptions) -> Arc<GlyphStore> {
        self.make_shaper(options);

        //FIXME: find the equivalent of Equiv and the old ShapeCacheEntryRef
        let shaper = &self.shaper;
        let lookup_key = ShapeCacheEntry {
            text: text.to_owned(),
            options: options.clone(),
        };
        if let Some(glyphs) = self.shape_cache.find(&lookup_key) {
            return glyphs.clone();
        }

        let mut glyphs = GlyphStore::new(text.chars().count(),
                                         options.flags.contains(IS_WHITESPACE_SHAPING_FLAG),
                                         options.flags.contains(RTL_FLAG));
        shaper.as_ref().unwrap().shape_text(text, options, &mut glyphs);

        let glyphs = Arc::new(glyphs);
        self.shape_cache.insert(ShapeCacheEntry {
            text: text.to_owned(),
            options: *options,
        }, glyphs.clone());
        glyphs
    }

    fn make_shaper<'a>(&'a mut self, options: &ShapingOptions) -> &'a Shaper {
        // fast path: already created a shaper
        if let Some(ref mut shaper) = self.shaper {
            shaper.set_options(options);
            return shaper
        }

        let shaper = Shaper::new(self, options);
        self.shaper = Some(shaper);
        self.shaper.as_ref().unwrap()
    }

    pub fn get_table_for_tag(&self, tag: FontTableTag) -> Option<Box<FontTable>> {
        let result = self.handle.get_table_for_tag(tag);
        let status = if result.is_some() { "Found" } else { "Didn't find" };

        debug!("{} font table[{}] with family={}, face={}",
               status, tag.tag_to_str(),
               self.handle.family_name(), self.handle.face_name());

        result
    }

    #[inline]
    pub fn glyph_index(&self, codepoint: char) -> Option<GlyphId> {
        let codepoint = match self.variant {
            font_variant::T::small_caps => codepoint.to_uppercase().next().unwrap(), //FIXME: #5938
            font_variant::T::normal => codepoint,
        };
        self.handle.glyph_index(codepoint)
    }

    pub fn glyph_h_kerning(&mut self, first_glyph: GlyphId, second_glyph: GlyphId)
                           -> FractionalPixel {
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
    pub fonts: SmallVec<[Rc<RefCell<Font>>; 8]>,
}

impl FontGroup {
    pub fn new(fonts: SmallVec<[Rc<RefCell<Font>>; 8]>) -> FontGroup {
        FontGroup {
            fonts: fonts,
        }
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
        let bounds = Rect::new(Point2D::new(Au(0), -ascent),
                               Size2D::new(advance, ascent + descent));

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
