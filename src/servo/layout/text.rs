#[doc="Text layout."]

use geom::size::Size2D;
use gfx::geometry::au;
use servo_text::text_run::TextRun;
use servo_text::font_library::FontLibrary;
use base::{Box, TextBoxKind};

struct TextBox {
    text: ~str,
    mut run: Option<TextRun>,
}

fn TextBox(text: ~str) -> TextBox {
    TextBox {
        text: text,
        run: None,
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
        let run = TextRun(font, subbox.text);
        self.bounds.size = run.size();
        subbox.run = Some(run);
    }
}

fn should_calculate_the_size_of_the_text_box() {
    #[test];
    #[ignore(cfg(target_os = "macos"))];

    use dom::rcu::{Scope};
    use dom::base::{Text, NodeScope};
    use util::tree;
    use gfx::geometry::px_to_au;

    let s = Scope();
    let n = s.new_node(Text(~"firecracker"));
    let b = n.construct_boxes().get();

    let subbox = match b.kind {
      TextBoxKind(subbox) => { subbox },
      _ => fail
    };
    b.reflow_text(subbox);
    let expected = Size2D(px_to_au(84), px_to_au(20));
    assert b.bounds.size == expected;
}
