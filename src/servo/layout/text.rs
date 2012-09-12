#[doc="Text layout."]

use au = gfx::geometry;
use geom::size::Size2D;
use gfx::geometry::au;
use servo_text::text_run::TextRun;
use servo_text::font_library::FontLibrary;
use layout::base::{TextBox, Box};

struct TextBoxData {
    text: ~str,
    mut runs: ~[TextRun]
}

fn TextBoxData(text: ~str, runs: ~[TextRun]) -> TextBoxData {
    TextBoxData {
        text: text,
        runs: runs
    }
}

trait TextLayout {
    fn reflow_text();
}

#[doc="The main reflow routine for text layout."]
impl @Box : TextLayout {
    fn reflow_text() {
        let d = match self.kind {
            TextBox(d) => { d }
            _ => { fail ~"expected text box in reflow_text!" }
        };

        // FIXME: The font library should not be initialized here
        let flib = FontLibrary();
        let font = flib.get_test_font();

        // Do line breaking.
        let mut current = TextRun(font, d.text);
        let mut lines = dvec::DVec();
        let mut width_left = au::from_px(800);
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

        self.data.position.size = Size2D(max_width, total_height);
        d.runs = move dvec::unwrap(lines);
    }
}

fn should_calculate_the_size_of_the_text_box() {
    #[test];
    #[ignore(cfg(target_os = "macos"))];

    use au = gfx::geometry;
    use dom::rcu::{Scope};
    use dom::base::{Text, NodeScope};
    use util::tree;
    use layout::box_builder::LayoutTreeBuilder;

    let s = Scope();
    let n = s.new_node(Text(~"firecracker"));
    let builder = LayoutTreeBuilder();
    let b = builder.construct_trees(n).get();

    b.reflow_text();
    let expected = Size2D(au::from_px(84), au::from_px(20));
    assert b.data.position.size == expected;
}
