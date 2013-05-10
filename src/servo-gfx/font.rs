/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use color::Color;
use font_context::FontContext;
use geometry::Au;
use platform::font_context::FontContextHandle;
use platform::font::{FontHandle, FontTable};
use render_context::RenderContext;
use servo_util::range::Range;
use text::glyph::{GlyphStore, GlyphIndex};
use text::shaping::ShaperMethods;
use text::{Shaper, TextRun};

use azure::{AzFloat, AzScaledFontRef};
use azure::scaled_font::ScaledFont;
use azure::azure_hl::{BackendType, ColorPattern};
use geom::{Point2D, Rect, Size2D};

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
    fn boldness(&self) -> CSSFontWeight;

    fn clone_with_style(&self, fctx: &FontContextHandle, style: &UsedFontStyle)
                     -> Result<FontHandle, ()>;
    fn glyph_index(&self, codepoint: char) -> Option<GlyphIndex>;
    fn glyph_h_advance(&self, GlyphIndex) -> Option<FractionalPixel>;
    fn get_metrics(&self) -> FontMetrics;
    fn get_table_for_tag(&self, FontTableTag) -> Option<FontTable>;
}

// Used to abstract over the shaper's choice of fixed int representation.
pub type FractionalPixel = float;

pub type FontTableTag = u32;

trait FontTableTagConversions {
    pub fn tag_to_str(&self) -> ~str;
}

impl FontTableTagConversions for FontTableTag {
    pub fn tag_to_str(&self) -> ~str {
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
    fn with_buffer(&self, &fn(*u8, uint));
}

pub struct FontMetrics {
    underline_size:   Au,
    underline_offset: Au,
    leading:          Au,
    x_height:         Au,
    em_size:          Au,
    ascent:           Au,
    descent:          Au,
    max_advance:      Au
}

// TODO(Issue #200): use enum from CSS bindings for 'font-weight'
#[deriving(Eq)]
pub enum CSSFontWeight {
    FontWeight100,
    FontWeight200,
    FontWeight300,
    FontWeight400,
    FontWeight500,
    FontWeight600,
    FontWeight700,
    FontWeight800,
    FontWeight900,
}

pub impl CSSFontWeight {
    pub fn is_bold(self) -> bool {
        match self {
            FontWeight900 | FontWeight800 | FontWeight700 | FontWeight600 => true,
            _ => false
        }
    }
}

// TODO(Issue #179): eventually this will be split into the specified
// and used font styles.  specified contains uninterpreted CSS font
// property values, while 'used' is attached to gfx::Font to descript
// the instance's properties.
//
// For now, the cases are differentiated with a typedef
#[deriving(Eq)]
pub struct FontStyle {
    pt_size: float,
    weight: CSSFontWeight,
    italic: bool,
    oblique: bool,
    families: ~str,
    // TODO(Issue #198): font-stretch, text-decoration, font-variant, size-adjust
}

pub type SpecifiedFontStyle = FontStyle;
pub type UsedFontStyle = FontStyle;

// FIXME: move me to layout
struct ResolvedFont {
    group: @FontGroup,
    style: SpecifiedFontStyle,
}

// FontDescriptor serializes a specific font and used font style
// options, such as point size.

// It's used to swizzle/unswizzle gfx::Font instances when
// communicating across tasks, such as the display list between layout
// and render tasks.
#[deriving(Eq)]
pub struct FontDescriptor {
    style: UsedFontStyle,
    selector: FontSelector,
}

pub impl FontDescriptor {
    fn new(style: UsedFontStyle, selector: FontSelector) -> FontDescriptor {
        FontDescriptor {
            style: style,
            selector: selector,
        }
    }
}

// A FontSelector is a platform-specific strategy for serializing face names.
#[deriving(Eq)]
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
    families: @str,
    // style of the first western font in group, which is
    // used for purposes of calculating text run metrics.
    style: UsedFontStyle,
    fonts: ~[@mut Font],
}

pub impl FontGroup {
    fn new(families: @str, style: &UsedFontStyle, fonts: ~[@mut Font]) -> FontGroup {
        FontGroup {
            families: families,
            style: copy *style,
            fonts: fonts,
        }
    }

    fn create_textrun(&self, text: ~str) -> TextRun {
        assert!(self.fonts.len() > 0);

        // TODO(Issue #177): Actually fall back through the FontGroup when a font is unsuitable.
        return TextRun::new(self.fonts[0], text);
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

/**
A font instance. Layout can use this to calculate glyph metrics
and the renderer can use it to render text.
*/
pub struct Font {
    priv handle: FontHandle,
    priv azure_font: Option<ScaledFont>,
    priv shaper: Option<@Shaper>,
    style: UsedFontStyle,
    metrics: FontMetrics,
    backend: BackendType,
}

pub impl Font {
    fn new_from_buffer(ctx: &FontContext,
                       buffer: ~[u8],
                       style: &SpecifiedFontStyle,
                       backend: BackendType)
            -> Result<@mut Font, ()> {
        let handle = FontHandleMethods::new_from_buffer(&ctx.handle, buffer, style);
        let handle: FontHandle = if handle.is_ok() {
            result::unwrap(handle)
        } else {
            return Err(handle.get_err());
        };
        
        let metrics = handle.get_metrics();
        // TODO(Issue #179): convert between specified and used font style here?

        return Ok(@mut Font {
            handle: handle,
            azure_font: None,
            shaper: None,
            style: copy *style,
            metrics: metrics,
            backend: backend,
        });
    }

    fn new_from_adopted_handle(_fctx: &FontContext, handle: FontHandle,
                               style: &SpecifiedFontStyle, backend: BackendType) -> @mut Font {
        let metrics = handle.get_metrics();

        @mut Font {
            handle: handle,
            azure_font: None,
            shaper: None,
            style: copy *style,
            metrics: metrics,
            backend: backend,
        }
    }

    fn new_from_existing_handle(fctx: &FontContext, handle: &FontHandle,
                                style: &SpecifiedFontStyle, backend: BackendType) -> Result<@mut Font,()> {

        // TODO(Issue #179): convert between specified and used font style here?
        let styled_handle = match handle.clone_with_style(&fctx.handle, style) {
            Ok(result) => result,
            Err(()) => return Err(())
        };

        return Ok(Font::new_from_adopted_handle(fctx, styled_handle, style, backend));
    }

    priv fn get_shaper(@mut self) -> @Shaper {
        // fast path: already created a shaper
        match self.shaper {
            Some(shaper) => { return shaper; },
            None => {}
        }

        let shaper = @Shaper::new(self);
        self.shaper = Some(shaper);
        shaper
    }

    fn get_table_for_tag(&self, tag: FontTableTag) -> Option<FontTable> {
        let result = self.handle.get_table_for_tag(tag);
        let status = if result.is_some() { "Found" } else { "Didn't find" };

        debug!("%s font table[%s] with family=%s, face=%s",
               status, tag.tag_to_str(),
               self.handle.family_name(), self.handle.face_name());

        return result;
    }

    // TODO: this should return a borrowed pointer, but I can't figure
    // out why borrowck doesn't like my implementation.

    priv fn get_azure_font(&mut self) -> AzScaledFontRef {
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
    priv fn create_azure_font(&mut self) -> ScaledFont {
        let cg_font = self.handle.get_CGFont();
        let size = self.style.pt_size as AzFloat;
        ScaledFont::new(self.backend, &cg_font, size)
    }

    #[cfg(target_os="linux")]
    priv fn create_azure_font(&self) -> ScaledFont {
        let freetype_font = self.handle.face;
        let size = self.style.pt_size as AzFloat;
        ScaledFont::new(self.backend, freetype_font, size)
    }
}


pub impl Font {
    fn draw_text_into_context(&mut self,
                              rctx: &RenderContext,
                              run: &TextRun,
                              range: &Range,
                              baseline_origin: Point2D<Au>,
                              color: Color) {
        use core::libc::types::common::c99::{uint16_t, uint32_t};
        use azure::{struct__AzDrawOptions,
                    struct__AzGlyph,
                    struct__AzGlyphBuffer,
                    struct__AzPoint};
        use azure::azure::bindgen::{AzDrawTargetFillGlyphs};

        let target = rctx.get_draw_target();
        let azfontref = self.get_azure_font();
        let pattern = ColorPattern(color);
        let azure_pattern = pattern.azure_color_pattern;
        assert!(azure_pattern.is_not_null());

        let options = struct__AzDrawOptions {
            mAlpha: 1f as AzFloat,
            fields: 0x0200 as uint16_t
        };

        let mut origin = copy baseline_origin;
        let mut azglyphs = ~[];
        vec::reserve(&mut azglyphs, range.length());

        for run.glyphs.iter_glyphs_for_char_range(range) |_i, glyph| {
            let glyph_advance = glyph.advance();
            let glyph_offset = glyph.offset().get_or_default(Au::zero_point());

            let azglyph = struct__AzGlyph {
                mIndex: glyph.index() as uint32_t,
                mPosition: struct__AzPoint {
                    x: (origin.x + glyph_offset.x).to_px() as AzFloat,
                    y: (origin.y + glyph_offset.y).to_px() as AzFloat
                }
            };
            origin = Point2D(origin.x + glyph_advance, origin.y);
            azglyphs.push(azglyph)
        };

        let azglyph_buf_len = azglyphs.len();
        if azglyph_buf_len == 0 { return; } // Otherwise the Quartz backend will assert.

        let glyphbuf = unsafe {
            struct__AzGlyphBuffer {
                mGlyphs: vec::raw::to_ptr(azglyphs),
                mNumGlyphs: azglyph_buf_len as uint32_t            
            }
        };

        // TODO(Issue #64): this call needs to move into azure_hl.rs
        AzDrawTargetFillGlyphs(target.azure_draw_target,
                               azfontref,
                               ptr::to_unsafe_ptr(&glyphbuf),
                               azure_pattern,
                               ptr::to_unsafe_ptr(&options),
                               ptr::null());
    }

    fn measure_text(&self, run: &TextRun, range: &Range) -> RunMetrics {
        // TODO(Issue #199): alter advance direction for RTL
        // TODO(Issue #98): using inter-char and inter-word spacing settings  when measuring text
        let mut advance = Au(0);
        for run.glyphs.iter_glyphs_for_char_range(range) |_i, glyph| {
            advance += glyph.advance();
        }
        let bounds = Rect(Point2D(Au(0), -self.metrics.ascent),
                          Size2D(advance, self.metrics.ascent + self.metrics.descent));

        // TODO(Issue #125): support loose and tight bounding boxes; using the
        // ascent+descent and advance is sometimes too generous and
        // looking at actual glyph extents can yield a tighter box.

        RunMetrics { 
            advance_width: advance,
            bounding_box: bounds,
            ascent: self.metrics.ascent,
            descent: self.metrics.descent,
        }
    }

    fn shape_text(@mut self, text: &str, store: &mut GlyphStore) {
        // TODO(Issue #229): use a more efficient strategy for repetitive shaping.
        // For example, Gecko uses a per-"word" hashtable of shaper results.
        let shaper = self.get_shaper();
        shaper.shape_text(text, store);
    }

    fn get_descriptor(&self) -> FontDescriptor {
        FontDescriptor::new(copy self.style, SelectorPlatformIdentifier(self.handle.face_identifier()))
    }

    fn glyph_index(&self, codepoint: char) -> Option<GlyphIndex> {
        self.handle.glyph_index(codepoint)
    }

    fn glyph_h_advance(&self, glyph: GlyphIndex) -> FractionalPixel {
        match self.handle.glyph_h_advance(glyph) {
          Some(adv) => adv,
          None => /* FIXME: Need fallback strategy */ 10f as FractionalPixel
        }
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
        do task::spawn {
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
