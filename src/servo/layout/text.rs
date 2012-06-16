#[doc="Text layout."]

import geom::size::Size2D;
import gfx::geometry::au;
import layout::base::*;     // FIXME: Can't get around import *; resolve bug.
import servo_text::text_run::text_run;
import servo_text::font::create_test_font;

class text_box {
    let text: str;
    let mut run: option<text_run>;

    new(-text: str) {
        self.text = text;
        self.run = none;
    }
}

#[doc="The main reflow routine for text layout."]
impl text_layout_methods for @Box {
    fn reflow_text(_available_width: au, subbox: @text_box) {
        alt self.kind {
            TextBox(*) { /* ok */ }
            _ { fail "expected text box in reflow_text!" }
        };

        let font = create_test_font();
        let run = text_run(&font, subbox.text);
        subbox.run = some(run);

        self.bounds.size =
            Size2D(alt vec::last_opt(run.glyphs) {
                        some(glyph) {
                            au(*glyph.pos.offset.x + *glyph.pos.advance.x)
                        }
                        none {
                            au(0)
                        }
                   },
                   au(60 * 14));
    }
}

