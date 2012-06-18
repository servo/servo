import geom::point::Point2D;
import geom::size::Size2D;
import gfx::geometry::{au, px_to_au};
import libc::{c_void};
import font::{font, create_test_font};
import glyph::glyph;
import shaper::shape_text;

#[doc="A single, unbroken line of text."]
class text_run {
    let glyphs: [glyph];

    new(font: &font, text: str) {
        self.glyphs = shape_text(font, text);
    }

    fn size() -> Size2D<au> {
        let height = px_to_au(20);
        let pen_start_x = px_to_au(0);
        let pen_start_y = height;
        let pen_start = Point2D(pen_start_x, pen_start_y);
        let pen_end = self.glyphs.foldl(pen_start) { |cur, glyph|
            Point2D(cur.x.add(glyph.pos.offset.x).add(glyph.pos.advance.x),
                    cur.y.add(glyph.pos.offset.y).add(glyph.pos.advance.y))
        };
        ret Size2D(pen_end.x, pen_end.y);
    }
}

fn should_calculate_the_total_size() {
    #[test];
    #[ignore(reason = "random failures")];

    let font = create_test_font();
    let run = text_run(&font, "firecracker");
    let expected = Size2D(px_to_au(84), px_to_au(20));
    assert run.size() == expected;
}
