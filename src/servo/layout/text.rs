#[doc="Text layout."]

import geom::size::Size2D;
import gfx::geometry::au;
import layout::base::*;     // FIXME: Can't get around import *; resolve bug.
import servo_text::text_run::text_run;

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

        let run = text_run(copy subbox.text);
        subbox.run = some(copy run);
        run.shape();

        self.bounds.size =
            Size2D(alt vec::last_opt(run.glyphs.get()) {
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

