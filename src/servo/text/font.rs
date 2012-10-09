pub use font_cache::FontCache;

use au = gfx::geometry;
use au::au;
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
    advance: au,
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
        // TODO: calculate actual bounding box as part of RunMetrics
        // TODO: using inter-char and inter-word spacing settings  when measuring text
        let mut advance = au(0);
        do run.glyphs.iter_glyphs_for_range(offset, length) |_i, glyph| {
            advance += glyph.advance()
        }

        RunMetrics { advance: advance }
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

// TODO: font should compute its own metrics using native_font.
// TODO: who should own fontbuf?
fn Font(lib: @FontCache, fontbuf: @~[u8], +native_font: NativeFont, +metrics: FontMetrics) -> Font {
    Font {
        lib: lib,
        fontbuf : fontbuf,
        native_font : move native_font,
        metrics: move metrics
    }
}

// TODO: what are the units of these metrics? CTFont metrics calls are
// font units for a specific point size, and these are normalized by
// font units-per-em. Are there some cases where these metrics depend
// on font size? If so, need to be careful about using same sizes on
// font creation, metrics gathering, and drawing.
struct FontMetrics {
    underline_size:   float,
    underline_offset: float,
    leading:          float,
    x_height:         float,

    // how many appunits an em is equivalent to (based on point-to-au)
    em_size:          au,
    em_height:        float,
    em_ascent:        float,
    em_descent:       float,
    max_advance:      float
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

    let lib = FontCache();
    let font = lib.get_test_font();
    let x = font.glyph_h_advance(40u as GlyphIndex);
    assert x == 15f || x == 16f;
}

// Testing thread safety
fn should_get_glyph_advance_stress() {
    #[test];

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

