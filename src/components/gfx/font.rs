/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use azure::{AzFloat, AzScaledFontRef};
use azure::azure_hl::{BackendType, ColorPattern};
use azure::scaled_font::ScaledFont;
use extra::arc::Arc;
use geom::{Point2D, Rect, Size2D};
use std::cast;
use std::ptr;
use std::str;
use std::rc::Rc;
use std::cell::RefCell;
use servo_util::cache::{Cache, HashCache};
use servo_util::range::Range;
use style::computed_values::{text_decoration, font_weight, font_style};
use color::Color;
use font_context::FontContext;
use servo_util::geometry::Au;
use platform::font_context::FontContextHandle;
use platform::font::{FontHandle, FontTable};
use render_context::RenderContext;
use text::glyph::{GlyphStore, GlyphIndex};
use text::shaping::ShaperMethods;
use text::{Shaper, TextRun};

// FontHandle encapsulates access to the platform's font API,
// e.g. quartz, FreeType. It provides access to metrics and tables
// needed by the text shaper as well as access to the underlying font
// resources needed by the graphics layer to draw glyphs.

pub trait FontHandleMethods {
    fn new_from_buffer(fctx: &FontContextHandle, buf: ~[u8], style: &SpecifiedFontStyle)
                    -> Result<Self,()>;

    // an identifier usable by FontContextHandle to recreate this FontHandle.
    fn face_identifier(&self) -> ~str;
    fn family_name(&self) -> ~str;
    fn face_name(&self) -> ~str;
    fn is_italic(&self) -> bool;
    fn boldness(&self) -> font_weight::T;

    fn clone_with_style(&self, fctx: &FontContextHandle, style: &UsedFontStyle)
                     -> Result<FontHandle, ()>;
    fn glyph_index(&self, codepoint: char) -> Option<GlyphIndex>;
    fn glyph_h_advance(&self, GlyphIndex) -> Option<FractionalPixel>;
    fn get_metrics(&self) -> FontMetrics;
    fn get_table_for_tag(&self, FontTableTag) -> Option<FontTable>;
}

// Used to abstract over the shaper's choice of fixed int representation.
pub type FractionalPixel = f64;

pub type FontTableTag = u32;

pub trait FontTableTagConversions {
    fn tag_to_str(&self) -> ~str;
}

impl FontTableTagConversions for FontTableTag {
    fn tag_to_str(&self) -> ~str {
        unsafe {
            let reversed = str::raw::from_buf_len(cast::transmute(self), 4);
            return str::from_chars([reversed.char_at(3),
                                    reversed.char_at(2),
                                    reversed.char_at(1),
                                    reversed.char_at(0)]);
        }
    }
}

pub trait FontTableMethods {
    fn with_buffer(&self, |*u8, uint|);
}

#[deriving(Clone)]
pub struct FontMetrics {
    underline_size:   Au,
    underline_offset: Au,
    strikeout_size:   Au,
    strikeout_offset: Au,
    leading:          Au,
    x_height:         Au,
    em_size:          Au,
    ascent:           Au,
    descent:          Au,
    max_advance:      Au
}

// TODO(Issue #179): eventually this will be split into the specified
// and used font styles.  specified contains uninterpreted CSS font
// property values, while 'used' is attached to gfx::Font to descript
// the instance's properties.
//
// For now, the cases are differentiated with a typedef
#[deriving(Clone, Eq)]
pub struct FontStyle {
    pt_size: f64,
    weight: font_weight::T,
    style: font_style::T,
    families: ~[~str],
    // TODO(Issue #198): font-stretch, text-decoration, font-variant, size-adjust
}

pub type SpecifiedFontStyle = FontStyle;
pub type UsedFontStyle = FontStyle;

// FontDescriptor serializes a specific font and used font style
// options, such as point size.

// It's used to swizzle/unswizzle gfx::Font instances when
// communicating across tasks, such as the display list between layout
// and render tasks.
#[deriving(Clone, Eq)]
pub struct FontDescriptor {
    style: UsedFontStyle,
    selector: FontSelector,
}

impl FontDescriptor {
    pub fn new(style: UsedFontStyle, selector: FontSelector) -> FontDescriptor {
        FontDescriptor {
            style: style,
            selector: selector,
        }
    }
}

// A FontSelector is a platform-specific strategy for serializing face names.
#[deriving(Clone, Eq)]
pub enum FontSelector {
    SelectorPlatformIdentifier(~str),
}

// This struct is the result of mapping a specified FontStyle into the
// available fonts on the system. It contains an ordered list of font
// instances to be used in case the prior font cannot be used for
// rendering the specified language.

// The ordering of font instances is mainly decided by the CSS
// 'font-family' property. The last font is a system fallback font.
pub struct FontGroup {
    families: ~[~str],
    // style of the first western font in group, which is
    // used for purposes of calculating text run metrics.
    style: UsedFontStyle,
    fonts: ~[Rc<RefCell<Font>>]
}

impl FontGroup {
    pub fn new(families: ~[~str], style: &UsedFontStyle, fonts: ~[Rc<RefCell<Font>>]) -> FontGroup {
        FontGroup {
            families: families,
            style: (*style).clone(),
            fonts: fonts,
        }
    }

    pub fn teardown(&mut self) {
        self.fonts = ~[];
    }

    pub fn create_textrun(&self, text: ~str, decoration: text_decoration::T) -> TextRun {
        assert!(self.fonts.len() > 0);

        // TODO(Issue #177): Actually fall back through the FontGroup when a font is unsuitable.
        self.fonts[0].borrow().with_mut(|font| {
            TextRun::new(font, text.clone(), decoration)
        })
    }
}

pub struct RunMetrics {
    // may be negative due to negative width (i.e., kerning of '.' in 'P.T.')
    advance_width: Au,
    ascent: Au, // nonzero
    descent: Au, // nonzero
    // this bounding box is relative to the left origin baseline.
    // so, bounding_box.position.y = -ascent
    bounding_box: Rect<Au>
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

/**
A font instance. Layout can use this to calculate glyph metrics
and the renderer can use it to render text.
*/
pub struct Font {
    priv handle: FontHandle,
    priv azure_font: Option<ScaledFont>,
    priv shaper: Option<Shaper>,
    style: UsedFontStyle,
    metrics: FontMetrics,
    backend: BackendType,
    shape_cache: HashCache<~str, Arc<GlyphStore>>,
    glyph_advance_cache: HashCache<u32, FractionalPixel>,
}

impl<'a> Font {
    pub fn new_from_buffer(ctx: &FontContext,
                       buffer: ~[u8],
                       style: &SpecifiedFontStyle,
                       backend: BackendType)
            -> Result<Rc<RefCell<Font>>, ()> {
        let handle = FontHandleMethods::new_from_buffer(&ctx.handle, buffer, style);
        let handle: FontHandle = if handle.is_ok() {
            handle.unwrap()
        } else {
            return Err(handle.unwrap_err());
        };

        let metrics = handle.get_metrics();
        // TODO(Issue #179): convert between specified and used font style here?

        return Ok(Rc::from_mut(RefCell::new(Font {
            handle: handle,
            azure_font: None,
            shaper: None,
            style: (*style).clone(),
            metrics: metrics,
            backend: backend,
            shape_cache: HashCache::new(),
            glyph_advance_cache: HashCache::new(),
        })));
    }

    pub fn new_from_adopted_handle(_fctx: &FontContext, handle: FontHandle,
                               style: &SpecifiedFontStyle, backend: BackendType)
                               -> Font {
        let metrics = handle.get_metrics();

        Font {
            handle: handle,
            azure_font: None,
            shaper: None,
            style: (*style).clone(),
            metrics: metrics,
            backend: backend,
            shape_cache: HashCache::new(),
            glyph_advance_cache: HashCache::new(),
        }
    }

    pub fn new_from_existing_handle(fctx: &FontContext, handle: &FontHandle,
                                style: &SpecifiedFontStyle, backend: BackendType)
                                -> Result<Rc<RefCell<Font>>,()> {

        // TODO(Issue #179): convert between specified and used font style here?
        let styled_handle = match handle.clone_with_style(&fctx.handle, style) {
            Ok(result) => result,
            Err(()) => return Err(())
        };

        return Ok(Rc::from_mut(RefCell::new(Font::new_from_adopted_handle(fctx, styled_handle, style, backend))));
    }

    fn make_shaper(&'a mut self) -> &'a Shaper {
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
        self.shaper.get_ref()
    }

    pub fn get_table_for_tag(&self, tag: FontTableTag) -> Option<FontTable> {
        let result = self.handle.get_table_for_tag(tag);
        let status = if result.is_some() { "Found" } else { "Didn't find" };

        debug!("{:s} font table[{:s}] with family={:s}, face={:s}",
               status, tag.tag_to_str(),
               self.handle.family_name(), self.handle.face_name());

        return result;
    }

    pub fn teardown(&mut self) {
        self.shaper = None;
        self.azure_font = None;
    }

    // TODO: this should return a borrowed pointer, but I can't figure
    // out why borrowck doesn't like my implementation.

    fn get_azure_font(&mut self) -> AzScaledFontRef {
        // fast path: we've already created the azure font resource
        match self.azure_font {
            Some(ref azfont) => return azfont.get_ref(),
            None => {}
        }

        let scaled_font = self.create_azure_font();
        self.azure_font = Some(scaled_font);
        // try again.
        return self.get_azure_font();
    }

    #[cfg(target_os="macos")]
    fn create_azure_font(&mut self) -> ScaledFont {
        let cg_font = self.handle.get_CGFont();
        let size = self.style.pt_size as AzFloat;
        ScaledFont::new(self.backend, &cg_font, size)
    }

    #[cfg(target_os="linux")]
    #[cfg(target_os="android")]
    fn create_azure_font(&self) -> ScaledFont {
        let freetype_font = self.handle.face;
        let size = self.style.pt_size as AzFloat;
        ScaledFont::new(self.backend, freetype_font, size)
    }
}


impl Font {
    pub fn draw_text_into_context(&mut self,
                              rctx: &RenderContext,
                              run: &~TextRun,
                              range: &Range,
                              baseline_origin: Point2D<Au>,
                              color: Color) {
        use std::libc::types::common::c99::{uint16_t, uint32_t};
        use azure::{struct__AzDrawOptions,
                    struct__AzGlyph,
                    struct__AzGlyphBuffer,
                    struct__AzPoint};
        use azure::azure::{AzDrawTargetFillGlyphs};

        let target = rctx.get_draw_target();
        let azfontref = self.get_azure_font();
        let pattern = ColorPattern(color);
        let azure_pattern = pattern.azure_color_pattern;
        assert!(azure_pattern.is_not_null());

        let options = struct__AzDrawOptions {
            mAlpha: 1f64 as AzFloat,
            fields: 0x0200 as uint16_t
        };

        let mut origin = baseline_origin.clone();
        let mut azglyphs = ~[];
        azglyphs.reserve(range.length());

        for (glyphs, _offset, slice_range) in run.iter_slices_for_range(range) {
            for (_i, glyph) in glyphs.iter_glyphs_for_char_range(&slice_range) {
                let glyph_advance = glyph.advance();
                let glyph_offset = glyph.offset().unwrap_or(Au::zero_point());

                let azglyph = struct__AzGlyph {
                    mIndex: glyph.index() as uint32_t,
                    mPosition: struct__AzPoint {
                        x: (origin.x + glyph_offset.x).to_nearest_px() as AzFloat,
                        y: (origin.y + glyph_offset.y).to_nearest_px() as AzFloat
                    }
                };
                origin = Point2D(origin.x + glyph_advance, origin.y);
                azglyphs.push(azglyph)
            };
        }

        let azglyph_buf_len = azglyphs.len();
        if azglyph_buf_len == 0 { return; } // Otherwise the Quartz backend will assert.

        let glyphbuf = struct__AzGlyphBuffer {
            mGlyphs: azglyphs.as_ptr(),
            mNumGlyphs: azglyph_buf_len as uint32_t            
        };

        unsafe {
            // TODO(Issue #64): this call needs to move into azure_hl.rs
            AzDrawTargetFillGlyphs(target.azure_draw_target,
                                   azfontref,
                                   ptr::to_unsafe_ptr(&glyphbuf),
                                   azure_pattern,
                                   ptr::to_unsafe_ptr(&options),
                                   ptr::null());
        }
    }

    pub fn measure_text(&self, run: &TextRun, range: &Range) -> RunMetrics {
        // TODO(Issue #199): alter advance direction for RTL
        // TODO(Issue #98): using inter-char and inter-word spacing settings  when measuring text
        let mut advance = Au(0);
        for (glyphs, _offset, slice_range) in run.iter_slices_for_range(range) {
            for (_i, glyph) in glyphs.iter_glyphs_for_char_range(&slice_range) {
                advance = advance + glyph.advance();
            }
        }
        RunMetrics::new(advance, self.metrics.ascent, self.metrics.descent)
    }

    pub fn measure_text_for_slice(&self,
                                  glyphs: &GlyphStore,
                                  slice_range: &Range)
                                  -> RunMetrics {
        let mut advance = Au(0);
        for (_i, glyph) in glyphs.iter_glyphs_for_char_range(slice_range) {
            advance = advance + glyph.advance();
        }
        RunMetrics::new(advance, self.metrics.ascent, self.metrics.descent)
    }

    pub fn shape_text(&mut self, text: ~str, is_whitespace: bool) -> Arc<GlyphStore> {

        //FIXME (ksh8281)
        self.make_shaper();
        self.shape_cache.find_or_create(&text, |txt| {
            let mut glyphs = GlyphStore::new(text.char_len(), is_whitespace);
            self.shaper.get_ref().shape_text(*txt, &mut glyphs);
            Arc::new(glyphs)
        })
    }

    pub fn get_descriptor(&self) -> FontDescriptor {
        FontDescriptor::new(self.style.clone(), SelectorPlatformIdentifier(self.handle.face_identifier()))
    }

    pub fn glyph_index(&self, codepoint: char) -> Option<GlyphIndex> {
        self.handle.glyph_index(codepoint)
    }

    pub fn glyph_h_advance(&mut self, glyph: GlyphIndex) -> FractionalPixel {
        self.glyph_advance_cache.find_or_create(&glyph, |glyph| {
            match self.handle.glyph_h_advance(*glyph) {
                Some(adv) => adv,
                None => /* FIXME: Need fallback strategy */ 10f64 as FractionalPixel
            }
        })
    }
}

/*fn should_destruct_on_fail_without_leaking() {
    #[test];
    #[should_fail];

    let fctx = @FontContext();
    let matcher = @FontMatcher(fctx);
    let _font = matcher.get_test_font();
    fail;
}

fn should_get_glyph_indexes() {
    #[test];

    let fctx = @FontContext();
    let matcher = @FontMatcher(fctx);
    let font = matcher.get_test_font();
    let glyph_idx = font.glyph_index('w');
    assert!(glyph_idx == Some(40u as GlyphIndex));
}

fn should_get_glyph_advance() {
    #[test];
    #[ignore];

    let fctx = @FontContext();
    let matcher = @FontMatcher(fctx);
    let font = matcher.get_test_font();
    let x = font.glyph_h_advance(40u as GlyphIndex);
    assert!(x == 15f || x == 16f);
}

// Testing thread safety
fn should_get_glyph_advance_stress() {
    #[test];
    #[ignore];

    let mut ports = ~[];

    for iter::repeat(100) {
        let (chan, port) = pipes::stream();
        ports += [@port];
        spawn_named("should_get_glyph_advance_stress") {
            let fctx = @FontContext();
            let matcher = @FontMatcher(fctx);
            let _font = matcher.get_test_font();
            let x = font.glyph_h_advance(40u as GlyphIndex);
            assert!(x == 15f || x == 16f);
            chan.send(());
        }
    }

    for ports.each |port| {
        port.recv();
    }
}

fn should_be_able_to_create_instances_in_multiple_threads() {
    #[test];

    for iter::repeat(10u) {
        do task::spawn {
            let fctx = @FontContext();
            let matcher = @FontMatcher(fctx);
            let _font = matcher.get_test_font();
        }
    }
}

*/
