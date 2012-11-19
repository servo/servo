use color::Color;
use font_context::FontContext;
use geometry::Au;
use render_context::RenderContext;
use util::range::Range;
use text::glyph::{GlyphStore, GlyphIndex};
use text::{Shaper, TextRun};

use azure::{AzFloat, AzScaledFontRef};
use azure::scaled_font::ScaledFont;
use azure::azure_hl::{BackendType, ColorPattern};
use core::dvec::DVec;
use geom::{Point2D, Rect, Size2D};

// FontHandle encapsulates access to the platform's font API,
// e.g. quartz, FreeType. It provides access to metrics and tables
// needed by the text shaper as well as access to the underlying font
// resources needed by the graphics layer to draw glyphs.

#[cfg(target_os = "macos")]
pub type FontHandle/& = quartz::font::QuartzFontHandle;

#[cfg(target_os = "linux")]
pub type FontHandle/& = freetype::font::FreeTypeFontHandle;

pub trait FontHandleMethods {
    // an identifier usable by FontContextHandle to recreate this FontHandle.
    pure fn face_identifier() -> ~str;
    pure fn family_name() -> ~str;
    pure fn face_name() -> ~str;
    pure fn is_italic() -> bool;
    pure fn boldness() -> CSSFontWeight;

    fn clone_with_style(fctx: &native::FontContextHandle, style: &UsedFontStyle) -> Result<FontHandle, ()>;
    fn glyph_index(codepoint: char) -> Option<GlyphIndex>;
    fn glyph_h_advance(GlyphIndex) -> Option<FractionalPixel>;
    fn get_metrics() -> FontMetrics;
    fn get_table_for_tag(FontTableTag) -> Option<FontTable>;
}

// TODO(Issue #163): this is a workaround for static methods and
// typedefs not working well together. It should be removed.
//
// `new` should be part of trait FontHandleMethods.

impl FontHandle {
    #[cfg(target_os = "macos")]
    static pub fn new_from_buffer(fctx: &native::FontContextHandle, buf: ~[u8], style: &SpecifiedFontStyle) -> Result<FontHandle, ()> {
        quartz::font::QuartzFontHandle::new_from_buffer(fctx, move buf, style)
    }

    #[cfg(target_os = "linux")]
    static pub fn new_from_buffer(fctx: &native::FontContextHandle, buf: ~[u8], style: &SpecifiedFontStyle) -> Result<FontHandle, ()> {
        freetype::font::FreeTypeFontHandle::new_from_buffer(fctx, move buf, style)
    }
}

// Used to abstract over the shaper's choice of fixed int representation.
type FractionalPixel = float;

pub type FontTableTag = u32;

trait FontTableTagConversions {
    pub pure fn tag_to_str() -> ~str;
}

impl FontTableTag : FontTableTagConversions {
    pub pure fn tag_to_str() -> ~str unsafe {
        let reversed = str::raw::from_buf_len(cast::transmute(&self), 4);
        return str::from_chars([reversed.char_at(3),
                                reversed.char_at(2),
                                reversed.char_at(1),
                                reversed.char_at(0)]);
    }
}

#[cfg(target_os = "macos")]
pub type FontTable/& = quartz::font::QuartzFontTable;

#[cfg(target_os = "linux")]
pub type FontTable/& = freetype::font::FreeTypeFontTable;

trait FontTableMethods {
    fn with_buffer(fn&(*u8, uint));
}

struct FontMetrics {
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
enum CSSFontWeight {
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
pub impl CSSFontWeight : cmp::Eq {
    pure fn eq(other: &CSSFontWeight) -> bool {
        (self as uint) == (*other as uint)
    }
    pure fn ne(other: &CSSFontWeight) -> bool { !self.eq(other) }
}

pub impl CSSFontWeight {
    pub pure fn is_bold() -> bool {
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
pub struct FontStyle {
    pt_size: float,
    weight: CSSFontWeight,
    italic: bool,
    oblique: bool,
    families: ~str,
    // TODO(Issue #198): font-stretch, text-decoration, font-variant, size-adjust
}

// TODO(Issue #181): use deriving for trivial cmp::Eq implementations
pub impl FontStyle : cmp::Eq {
    pure fn eq(other: &FontStyle) -> bool {
        use std::cmp::FuzzyEq;

        self.pt_size.fuzzy_eq(&other.pt_size) &&
            self.weight == other.weight &&
            self.italic == other.italic &&
            self.oblique == other.oblique &&
            self.families == other.families
    }
    pure fn ne(other: &FontStyle) -> bool { !self.eq(other) }
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
pub struct FontDescriptor {
    style: UsedFontStyle,
    selector: FontSelector,
}


// TODO(Issue #181): use deriving for trivial cmp::Eq implementations
pub impl FontDescriptor : cmp::Eq {
    pure fn eq(other: &FontDescriptor) -> bool {
        self.style == other.style &&
            self.selector == other.selector
    }
    pure fn ne(other: &FontDescriptor) -> bool { !self.eq(other) }
}

pub impl FontDescriptor {
    static pure fn new(style: UsedFontStyle, selector: FontSelector) -> FontDescriptor {
        FontDescriptor {
            style: move style,
            selector: move selector,
        }
    }
}

// A FontSelector is a platform-specific strategy for serializing face names.
pub enum FontSelector {
    SelectorPlatformIdentifier(~str),
    SelectorStubDummy, // aka, use Josephin Sans
}

// TODO(Issue #181): use deriving for trivial cmp::Eq implementations
pub impl FontSelector : cmp::Eq {
    pure fn eq(other: &FontSelector) -> bool {
        match (&self, other) {
            (&SelectorPlatformIdentifier(a), &SelectorPlatformIdentifier(b)) => a == b,
            (&SelectorStubDummy, &SelectorStubDummy) => true,
            _ => false
        }
    }
    pure fn ne(other: &FontSelector) -> bool { !self.eq(other) }
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
    fonts: ~[@Font],
}

pub impl FontGroup {
    static fn new(families: @str, style: &UsedFontStyle, fonts: ~[@Font]) -> FontGroup {
        FontGroup {
            families: families,
            style: copy *style,
            fonts: move fonts,
        }
    }

    fn create_textrun(text: ~str) -> TextRun {
        assert self.fonts.len() > 0;

        // TODO(Issue #177): Actually fall back through the FontGroup when a font is unsuitable.
        return TextRun::new(self.fonts[0], move text);
    }
}

struct RunMetrics {
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
    priv mut azure_font: Option<ScaledFont>,
    priv mut shaper: Option<@Shaper>,
    style: UsedFontStyle,
    metrics: FontMetrics,
    backend: BackendType,
}

impl Font {
    static fn new_from_buffer(ctx: &FontContext, buffer: ~[u8],
                              style: &SpecifiedFontStyle, backend: BackendType) -> Result<@Font, ()> {

        let handle = FontHandle::new_from_buffer(&ctx.handle, move buffer, style);
        let handle = if handle.is_ok() {
            result::unwrap(move handle)
        } else {
            return Err(handle.get_err());
        };
        
        let metrics = handle.get_metrics();
        // TODO(Issue #179): convert between specified and used font style here?

        return Ok(@Font {
            handle : move handle,
            azure_font: None,
            shaper: None,
            style: copy *style,
            metrics: move metrics,
            backend: backend,
        });
    }

    static fn new_from_adopted_handle(_fctx: &FontContext, handle: FontHandle,
                                      style: &SpecifiedFontStyle, backend: BackendType) -> @Font {
        let metrics = handle.get_metrics();

        @Font {
            handle : move handle,
            azure_font: None,
            shaper: None,
            style: copy *style,
            metrics: move metrics,
            backend: backend,
        }
    }

    static fn new_from_existing_handle(fctx: &FontContext, handle: &FontHandle,
                              style: &SpecifiedFontStyle, backend: BackendType) -> Result<@Font,()> {

        // TODO(Issue #179): convert between specified and used font style here?
        let styled_handle = match handle.clone_with_style(&fctx.handle, style) {
            Ok(move result) => move result,
            Err(()) => return Err(())
        };

        return Ok(Font::new_from_adopted_handle(fctx, move styled_handle, style, backend));
    }

    priv fn get_shaper(@self) -> @Shaper {
        // fast path: already created a shaper
        match self.shaper {
            Some(shaper) => { return shaper; },
            None => {}
        }

        let shaper = @Shaper::new(self);
        self.shaper = Some(shaper);
        shaper
    }

    fn get_table_for_tag(tag: FontTableTag) -> Option<FontTable> {
        let result = self.handle.get_table_for_tag(tag);
        let status = if result.is_some() { "Found" } else { "Didn't find" };

        debug!("%s font table[%s] with family=%s, face=%s",
               status, tag.tag_to_str(),
               self.handle.family_name(), self.handle.face_name());

        return move result;
    }

    // TODO: this should return a borrowed pointer, but I can't figure
    // out why borrowck doesn't like my implementation.

    priv fn get_azure_font(&self) -> AzScaledFontRef {
        // fast path: we've already created the azure font resource
        match self.azure_font {
            Some(ref azfont) => return azfont.get_ref(),
            None => {}
        }

        let mut scaled_font = self.create_azure_font();
        self.azure_font = Some(move scaled_font);
        // try again.
        return self.get_azure_font();
    }

    #[cfg(target_os="macos")]
    priv fn create_azure_font() -> ScaledFont {
        let cg_font = self.handle.get_CGFont();
        let size = self.style.pt_size as AzFloat;
        ScaledFont::new(self.backend, &cg_font, size)
    }

    #[cfg(target_os="linux")]
    priv fn create_azure_font() -> ScaledFont {
        let cairo_font = self.handle.face;
        let size = self.style.pt_size as AzFloat;
        ScaledFont::new(self.backend, cairo_font, size)
    }
}

// Public API
pub trait FontMethods {
    fn draw_text_into_context(rctx: &RenderContext,
                              run: &TextRun,
                              range: Range,
                              baseline_origin: Point2D<Au>,
                              color: Color);
    fn measure_text(&TextRun, Range) -> RunMetrics;
    fn shape_text(@self, &str) -> GlyphStore;
    fn get_descriptor() -> FontDescriptor;

    // these are used to get glyphs and advances in the case that the
    // shaper can't figure it out.
    fn glyph_index(char) -> Option<GlyphIndex>;
    fn glyph_h_advance(GlyphIndex) -> FractionalPixel;
}

pub impl Font : FontMethods {
    fn draw_text_into_context(rctx: &RenderContext,
                              run: &TextRun,
                              range: Range,
                              baseline_origin: Point2D<Au>,
                              color: Color) {
        use libc::types::common::c99::{uint16_t, uint32_t};
        use azure::{AzDrawOptions,
                    AzGlyph,
                    AzGlyphBuffer};
        use azure::bindgen::{AzCreateColorPattern,
                             AzDrawTargetFillGlyphs,
                             AzReleaseColorPattern};

        let target = rctx.get_draw_target();
        let azfontref = self.get_azure_font();
        let pattern = ColorPattern(color);
        let azure_pattern = pattern.azure_color_pattern;
        assert azure_pattern.is_not_null();

        let options: AzDrawOptions = {
            mAlpha: 1f as AzFloat,
            fields: 0x0200 as uint16_t
        };

        let mut origin = copy baseline_origin;
        let azglyphs = DVec();
        azglyphs.reserve(range.length());

        for run.glyphs.iter_glyphs_for_range(range) |_i, glyph| {
            let glyph_advance = glyph.advance();
            let glyph_offset = glyph.offset().get_default(Au::zero_point());

            let azglyph: AzGlyph = {
                mIndex: glyph.index() as uint32_t,
                mPosition: {
                    x: (origin.x + glyph_offset.x).to_px() as AzFloat,
                    y: (origin.y + glyph_offset.y).to_px() as AzFloat
                }
            };
            origin = Point2D(origin.x + glyph_advance, origin.y);
            azglyphs.push(move azglyph)
        };

        let azglyph_buf_len = azglyphs.len();
        let azglyph_buf = dvec::unwrap(move azglyphs);
        let glyphbuf: AzGlyphBuffer = unsafe {{
            mGlyphs: vec::raw::to_ptr(azglyph_buf),
            mNumGlyphs: azglyph_buf_len as uint32_t            
        }};

        // TODO(Issue #64): this call needs to move into azure_hl.rs
        AzDrawTargetFillGlyphs(target.azure_draw_target,
                               azfontref,
                               ptr::to_unsafe_ptr(&glyphbuf),
                               azure_pattern,
                               ptr::to_unsafe_ptr(&options),
                               ptr::null());
    }

    fn measure_text(run: &TextRun, range: Range) -> RunMetrics {
        assert range.is_valid_for_string(run.text);

        // TODO(Issue #199): alter advance direction for RTL
        // TODO(Issue #98): using inter-char and inter-word spacing settings  when measuring text
        let mut advance = Au(0);
        for run.glyphs.iter_glyphs_for_range(range) |_i, glyph| {
            advance += glyph.advance();
        }
        let mut bounds = Rect(Point2D(Au(0), -self.metrics.ascent),
                              Size2D(advance, self.metrics.ascent + self.metrics.descent));

        // TODO(Issue #125): support loose and tight bounding boxes; using the
        // ascent+descent and advance is sometimes too generous and
        // looking at actual glyph extents can yield a tighter box.

        RunMetrics { 
            advance_width: advance,
            bounding_box: move bounds,
            ascent: self.metrics.ascent,
            descent: self.metrics.descent,
        }
    }

    fn shape_text(@self, text: &str) -> GlyphStore {
        let store = GlyphStore(str::char_len(text));
        let shaper = self.get_shaper();
        shaper.shape_text(text, &store);
        return move store;
    }

    fn get_descriptor() -> FontDescriptor {
        FontDescriptor::new(copy self.style, SelectorPlatformIdentifier(self.handle.face_identifier()))
    }

    fn glyph_index(codepoint: char) -> Option<GlyphIndex> {
        self.handle.glyph_index(codepoint)
    }

    fn glyph_h_advance(glyph: GlyphIndex) -> FractionalPixel {
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
    assert glyph_idx == Some(40u as GlyphIndex);
}

fn should_get_glyph_advance() {
    #[test];
    #[ignore];

    let fctx = @FontContext();
    let matcher = @FontMatcher(fctx);
    let font = matcher.get_test_font();
    let x = font.glyph_h_advance(40u as GlyphIndex);
    assert x == 15f || x == 16f;
}

// Testing thread safety
fn should_get_glyph_advance_stress() {
    #[test];
    #[ignore];

    let mut ports = ~[];

    for iter::repeat(100) {
        let (chan, port) = pipes::stream();
        ports += [@move port];
        do task::spawn |move chan| {
            let fctx = @FontContext();
            let matcher = @FontMatcher(fctx);
            let _font = matcher.get_test_font();
            let x = font.glyph_h_advance(40u as GlyphIndex);
            assert x == 15f || x == 16f;
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
