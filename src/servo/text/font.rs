pub use font_cache::FontCache;

use au = gfx::geometry;
use au::Au;
use azure::{
    AzFloat,
    AzScaledFontRef,
};
use core::dvec::DVec;
use gfx::render_context::RenderContext;
use geom::point::Point2D;
use geom::rect::Rect;
use geom::size::Size2D;
use glyph::{GlyphStore, GlyphIndex};
use native_font::NativeFont;
use servo_util::range::Range;
use shaper::Shaper;
use text::text_run::TextRun;

// Used to abstract over the shaper's choice of fixed int representation.
type FractionalPixel = float;

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

// TODO: use enum from CSS bindings
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

struct FontStyle {
    pt_size: float,
    weight: CSSFontWeight,
    italic: bool,
    oblique: bool,
}

struct FontFaceProperties {
    family_name: @str,
    face_name: ~str,
    priv weight: u16,
    priv italic: bool,
}

impl FontFaceProperties {
    pure fn is_bold() -> bool { self.weight >= (500 as u16) }
    pure fn is_italic() -> bool { self.italic }
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
A font handle. Layout can use this to calculate glyph metrics
and the renderer can use it to render text.
*/
struct Font {
    priv fontbuf: @~[u8],
    priv native_font: NativeFont,
    priv mut azure_font: Option<AzScaledFontRef>,
    priv mut shaper: Option<@Shaper>,
    style: FontStyle,
    metrics: FontMetrics,

    drop {
        use azure::bindgen::AzReleaseScaledFont;
        do (copy self.azure_font).iter |fontref| { AzReleaseScaledFont(*fontref); }
    }
}

impl Font {
    // TODO: who should own fontbuf?
    static fn new(fontbuf: @~[u8], native_font: NativeFont, style: FontStyle) -> Font {
        let metrics = native_font.get_metrics();

        Font {
            fontbuf : fontbuf,
            native_font : move native_font,
            azure_font: None,
            shaper: None,
            style: move style,
            metrics: move metrics,
        }
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

    priv fn get_azure_font() -> AzScaledFontRef {
        use libc::{c_int, c_double};
        use azure::{
            AzNativeFont,
            AZ_NATIVE_FONT_CAIRO_FONT_FACE
        };
        use azure::bindgen::AzCreateScaledFontWithCairo;
        use cairo::{cairo_font_face_t, cairo_scaled_font_t};
        use cairo::bindgen::cairo_scaled_font_destroy;

        // fast path: we've already created the azure font resource
        match self.azure_font {
            Some(azfont) => { return azfont; },
            None => {}
        }
        
        let nfont: AzNativeFont = {
            mType: AZ_NATIVE_FONT_CAIRO_FONT_FACE,
            mFont: ptr::null()
        };

        // TODO(Issue #64): we should be able to remove cairo stepping
        // stones and manual memory management, and put them inside of
        // azure_hl.rs and elsewhere instead.
        let cfont = get_cairo_font(&self);
        // TODO: This should probably not even use cairo
        let azfont = AzCreateScaledFontWithCairo(ptr::to_unsafe_ptr(&nfont), 1f as AzFloat, cfont);
        assert azfont.is_not_null();
        cairo_scaled_font_destroy(cfont);

        self.azure_font = Some(azfont);
        return azfont;

        // TODO: these cairo-related things should be in rust-cairo.
        // creating a cairo font/face from a native font resource
        // should be part of the NativeFont API, not exposed here.
        #[cfg(target_os = "linux")]
        fn get_cairo_face(font: &Font) -> *cairo_font_face_t {
            use cairo::cairo_ft::bindgen::{cairo_ft_font_face_create_for_ft_face};

            let ftface = font.native_font.face;
            let cface = cairo_ft_font_face_create_for_ft_face(ftface, 0 as c_int);
            // FIXME: error handling
            return cface;
        }

        #[cfg(target_os = "macos")]
        fn get_cairo_face(font: &Font) -> *cairo_font_face_t {
            use cairo::cairo_quartz::bindgen::cairo_quartz_font_face_create_for_cgfont;

            let cgfont = font.native_font.cgfont;
            let face = cairo_quartz_font_face_create_for_cgfont(cgfont);
            // FIXME: error handling
            return face;
        }

        fn get_cairo_font(font: &Font) -> *cairo_scaled_font_t {
            use cairo::cairo_matrix_t;
            use cairo::bindgen::{cairo_matrix_init_identity,
                                 cairo_matrix_scale,
                                 cairo_font_options_create,
                                 cairo_scaled_font_create,
                                 cairo_font_options_destroy,
                                 cairo_font_face_destroy};

            // FIXME: error handling

            let face = get_cairo_face(font);

            let idmatrix: cairo_matrix_t = {
                xx: 0 as c_double,
                yx: 0 as c_double,
                xy: 0 as c_double,
                yy: 0 as c_double,
                x0: 0 as c_double,
                y0: 0 as c_double
            };
            cairo_matrix_init_identity(ptr::to_unsafe_ptr(&idmatrix));

            let fontmatrix = idmatrix;
            cairo_matrix_scale(ptr::to_unsafe_ptr(&fontmatrix),
                               font.style.pt_size as c_double, 
                               font.style.pt_size as c_double);
            let options = cairo_font_options_create();
            let cfont = cairo_scaled_font_create(face, 
                                                 ptr::to_unsafe_ptr(&fontmatrix),
                                                 ptr::to_unsafe_ptr(&idmatrix), 
                                                 options);
            cairo_font_options_destroy(options);
            cairo_font_face_destroy(face);

            return cfont;
        }
    }
}

// Public API
pub trait FontMethods {
    fn draw_text_into_context(rctx: &RenderContext, run: &TextRun, range: Range, baseline_origin: Point2D<Au>);
    fn measure_text(&TextRun, Range) -> RunMetrics;
    fn shape_text(@self, &str) -> GlyphStore;

    fn buf(&self) -> @~[u8];
    // these are used to get glyphs and advances in the case that the
    // shaper can't figure it out.
    fn glyph_index(char) -> Option<GlyphIndex>;
    fn glyph_h_advance(GlyphIndex) -> FractionalPixel;
}

pub impl Font : FontMethods {
    fn draw_text_into_context(rctx: &RenderContext, run: &TextRun, range: Range, baseline_origin: Point2D<Au>) {
        use libc::types::common::c99::{uint16_t, uint32_t};
        use azure::{AzDrawOptions,
                    AzGlyph,
                    AzGlyphBuffer};
        use azure::bindgen::{AzCreateColorPattern,
                             AzDrawTargetFillGlyphs,
                             AzReleaseColorPattern};

        let target = rctx.get_draw_target();
        let azfont = self.get_azure_font();
        let color = {
            r: 0f as AzFloat,
            g: 0f as AzFloat,
            b: 0f as AzFloat,
            a: 1f as AzFloat
        };
        let pattern = AzCreateColorPattern(ptr::to_unsafe_ptr(&color));
        assert pattern.is_not_null();

        let options: AzDrawOptions = {
            mAlpha: 1f as AzFloat,
            fields: 0 as uint16_t
        };

        let mut origin = copy baseline_origin;
        let azglyphs = DVec();
        azglyphs.reserve(range.length());

        do run.glyphs.iter_glyphs_for_range(range) |_i, glyph| {
            let glyph_advance = glyph.advance();
            let glyph_offset = glyph.offset().get_default(au::zero_point());

            let azglyph: AzGlyph = {
                mIndex: glyph.index() as uint32_t,
                mPosition: {
                    x: au::to_px(origin.x + glyph_offset.x) as AzFloat,
                    y: au::to_px(origin.y + glyph_offset.y) as AzFloat
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

        // TODO: this call needs to move into azure_hl.rs
        AzDrawTargetFillGlyphs(target.azure_draw_target, azfont,
                               ptr::to_unsafe_ptr(&glyphbuf), pattern, ptr::to_unsafe_ptr(&options), ptr::null());

        AzReleaseColorPattern(pattern);
    }

    fn measure_text(run: &TextRun, range: Range) -> RunMetrics {
        assert range.is_valid_for_string(run.text);

        // TODO: alter advance direction for RTL
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

        let metrics = RunMetrics { advance_width: advance,
                                  bounding_box: bounds,
                                  ascent: self.metrics.ascent,
                                  descent: self.metrics.descent,
                                 };
        debug!("Measured text range '%s' with metrics:", run.text.substr(range.begin(), range.length()));
        debug!("%?", metrics);

        return metrics;
    }

    fn shape_text(@self, text: &str) -> GlyphStore {
        let store = GlyphStore(text.len());
        let shaper = self.get_shaper();
        shaper.shape_text(text, &store);
        return move store;
    }

    fn buf(&self) -> @~[u8] {
        self.fontbuf
    }

    fn glyph_index(codepoint: char) -> Option<GlyphIndex> {
        self.native_font.glyph_index(codepoint)
    }

    fn glyph_h_advance(glyph: GlyphIndex) -> FractionalPixel {
        match self.native_font.glyph_h_advance(glyph) {
          Some(adv) => adv,
          None => /* FIXME: Need fallback strategy */ 10f as FractionalPixel
        }
    }
}

fn should_destruct_on_fail_without_leaking() {
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

