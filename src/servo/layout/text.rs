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
        self.bounds.size = run.size();
    }
}

fn should_calculate_the_size_of_the_text_box() {
    #[test];

    import dom::rcu::{Scope};
    import dom::base::{Text, NodeScope};
    import box_builder::box_builder_methods;
    import util::tree;
    import gfx::geometry::px_to_au;

    let s = Scope();
    let n = s.new_node(Text("firecracker"));
    let b = n.construct_boxes();

    let subbox = alt check b.kind { TextBox(subbox) { subbox } };
    b.reflow_text(px_to_au(800), subbox);
    let expected = Size2D(px_to_au(110), px_to_au(14));
    assert b.bounds.size == Size2D(px_to_au(110), px_to_au(14));
}
