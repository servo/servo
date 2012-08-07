#[doc="Text layout."]

import geom::size::Size2D;
import gfx::geometry::au;
import servo_text::text_run::TextRun;
import servo_text::font_library::FontLibrary;
import base::{Box, TextBoxKind};

struct TextBox {
    text: ~str;
    mut run: option<TextRun>;

    new(-text: ~str) {
        self.text = text;
        self.run = none;
    }
}

trait TextLayout {
    fn reflow_text(subbox: @TextBox);
}

#[doc="The main reflow routine for text layout."]
impl @Box : TextLayout {
    fn reflow_text(subbox: @TextBox) {
        match self.kind {
            TextBoxKind(*) => { /* ok */ }
            _ => { fail ~"expected text box in reflow_text!" }
        };

        // FIXME: The font library should not be initialized here
        let flib = FontLibrary();
        let font = flib.get_test_font();
        let run = TextRun(*font, subbox.text);
        self.bounds.size = run.size();
        subbox.run = some(run);
    }
}

fn should_calculate_the_size_of_the_text_box() {
    #[test];
    #[ignore];

    import dom::rcu::{Scope};
    import dom::base::{Text, NodeScope};
    import util::tree;
    import gfx::geometry::px_to_au;

    let s = Scope();
    let n = s.new_node(Text(~"firecracker"));
    let b = n.construct_boxes();

    let subbox = match check b.kind { TextBoxKind(subbox) => { subbox } };
    b.reflow_text(subbox);
    let expected = Size2D(px_to_au(84), px_to_au(20));
    assert b.bounds.size == expected;
}
