use geom::point::Point2D;
use geom::size::Size2D;
use gfx::geometry::{au, px_to_au};
use libc::{c_void};
use font_library::FontLibrary;
use font::Font;
use glyph::Glyph;
use shaper::shape_text;

#[doc="A single, unbroken line of text."]
struct TextRun {
    priv glyphs: ~[Glyph],
    priv size_: Size2D<au>,
}

impl TextRun {
    fn size() -> Size2D<au> { self.size_ }
    fn preferred_width() -> au { self.size_.width }
}

fn TextRun(font: Font, text: ~str) -> TextRun {
    let glyphs = shape_text(&font, text);
    let size = glyph_run_size(glyphs);

    TextRun {
        size_: size,
        glyphs: shape_text(&font, text)
    }
}

fn glyph_run_size(glyphs: &[Glyph]) -> Size2D<au> {
    let height = px_to_au(20);
    let pen_start_x = px_to_au(0);
    let pen_start_y = height;
    let pen_start = Point2D(pen_start_x, pen_start_y);
    let pen_end = glyphs.foldl(pen_start, |cur, glyph| {
        Point2D(cur.x.add(glyph.pos.offset.x).add(glyph.pos.advance.x),
                cur.y.add(glyph.pos.offset.y).add(glyph.pos.advance.y))
    });
    return Size2D(pen_end.x, pen_end.y);
}

fn should_calculate_the_total_size() {
    #[test];
    #[ignore(cfg(target_os = "macos"))];

    let flib = FontLibrary();
    let font = flib.get_test_font();
    let run = TextRun(*font, ~"firecracker");
    let expected = Size2D(px_to_au(84), px_to_au(20));
    assert run.size() == expected;
}
