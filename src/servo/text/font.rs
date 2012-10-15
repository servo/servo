pub use font_cache::FontCache;

use au = gfx::geometry;
use au::au;
use geom::point::Point2D;
use geom::rect::Rect;
use geom::size::Size2D;
use glyph::GlyphIndex;
use libc::{ c_int, c_double, c_ulong };
use native_font::NativeFont;
use ptr::{null, addr_of};
use text::text_run::TextRun;
use vec_to_ptr = vec::raw::to_ptr;

// Used to abstract over the shaper's choice of fixed int representation.
type FractionalPixel = float;

/**
A font handle. Layout can use this to calculate glyph metrics
and the renderer can use it to render text.
*/
struct Font {
    // A back reference to keep the library alive
    lib: @FontCache,
    fontbuf: @~[u8],
    native_font: NativeFont,
    metrics: FontMetrics,
}

struct RunMetrics {
    // may be negative due to negative width
    advance_width: au,

    ascent: au, // nonzero
    descent: au, // nonzero

    // this bounding box is relative to the left origin baseline.
    // so, bounding_box.position.y = -ascent
    bounding_box: Rect<au>
}

// Public API
pub trait FontMethods {
    fn measure_text(run: &TextRun, offset: uint, length: uint) -> RunMetrics;

    fn buf(&self) -> @~[u8];
    // these are used to get glyphs and advances in the case that the
    // shaper can't figure it out.
    fn glyph_index(char) -> Option<GlyphIndex>;
    fn glyph_h_advance(GlyphIndex) -> FractionalPixel;
}

pub impl Font : FontMethods {
    fn measure_text(run: &TextRun, offset: uint, length: uint) -> RunMetrics {
        // TODO: alter advance direction for RTL
        // TODO(Issue #98): using inter-char and inter-word spacing settings  when measuring text
        let mut advance = au(0);
        let mut bounds = Rect(Point2D(au(0), -self.metrics.ascent),
                          Size2D(au(0), self.metrics.ascent + self.metrics.descent));
        do run.glyphs.iter_glyphs_for_range(offset, length) |_i, glyph| {
            advance += glyph.advance();
            bounds = bounds.translate(&Point2D(glyph.advance(), au(0)));
        }

        // TODO(Issue #125): support loose and tight bounding boxes; using the
        // ascent+descent and advance is sometimes too generous and
        // looking at actual glyph extents can yield a tighter box.

        let metrics = RunMetrics { advance_width: advance,
                                  bounding_box: bounds,
                                  ascent: self.metrics.ascent,
                                  descent: self.metrics.descent,
                                 };
        debug!("Measured text range '%s' with metrics:", run.text.substr(offset, length));
        debug!("%?", metrics);

        return metrics;
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

// TODO: who should own fontbuf?
fn Font(lib: @FontCache, fontbuf: @~[u8], native_font: NativeFont) -> Font {
    let metrics = native_font.get_metrics();

    Font {
        lib: lib,
        fontbuf : fontbuf,
        metrics: move metrics,
        native_font : move native_font,
    }
}

struct FontMetrics {
    underline_size:   au,
    underline_offset: au,
    leading:          au,
    x_height:         au,
    em_size:          au,
    ascent:           au,
    descent:          au,
    max_advance:      au
}

const TEST_FONT: [u8 * 33004] = #include_bin("JosefinSans-SemiBold.ttf");

fn test_font_bin() -> ~[u8] {
    return vec::from_fn(33004, |i| TEST_FONT[i]);
}

fn should_destruct_on_fail_without_leaking() {
    #[test];
    #[should_fail];

    let lib = FontCache();
    let _font = lib.get_test_font();
    fail;
}

fn should_get_glyph_indexes() {
    #[test];

    let lib = FontCache();
    let font = lib.get_test_font();
    let glyph_idx = font.glyph_index('w');
    assert glyph_idx == Some(40u as GlyphIndex);
}

fn should_get_glyph_advance() {
    #[test];
    #[ignore];

    let lib = FontCache();
    let font = lib.get_test_font();
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
        do task::spawn {
            let lib = FontCache();
            let font = lib.get_test_font();
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
            let lib = FontCache();
            let _font = lib.get_test_font();
        }
    }
}

