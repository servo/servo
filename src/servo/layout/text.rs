#[doc="Text layout."]

use geom::size::Size2D;
use gfx::geometry::{au, px_to_au};
use servo_text::text_run::TextRun;
use servo_text::font_library::FontLibrary;
use base::{Box, TextBoxKind};

struct TextBox {
    text: ~str,
    mut runs: ~[TextRun],
}

fn TextBox(text: ~str) -> TextBox {
    TextBox {
        text: text,
        runs: ~[],
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

        // Do line breaking.
        let mut current = TextRun(font, subbox.text);
        let mut lines = dvec::DVec();
        let mut width_left = px_to_au(800);
        let mut max_width = au(0);

        while current.size().width > width_left {
            let min_width = current.min_break_width();

            debug!("line %d, current width %d, width left %d, min width %d",
                   lines.len() as int,
                   *current.size().width as int,
                   *width_left as int,
                   *min_width as int);

            if min_width > width_left {
                // Too bad, we couldn't break. Overflow.
                break;
            }

            let (prev_line, next_line) = current.split(font, width_left);
            let prev_width = prev_line.size().width;
            if max_width < prev_width {
                max_width = prev_width;
            }

            lines.push(move prev_line);
            current = next_line;
        }

        let remaining_width = current.size().width;
        if max_width < remaining_width {
            max_width = remaining_width;
        }

        let line_count = 1 + (lines.len() as i32);
        let total_height = au(*current.size().height * line_count);
        lines.push(move current);

        self.bounds.size = Size2D(max_width, total_height);
        subbox.runs = dvec::unwrap(lines);
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
