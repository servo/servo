/** Text layout. */

use au = gfx::geometry;
use au::au;
use geom::size::Size2D;
use servo_text::text_run::TextRun;
use servo_text::font_cache::FontCache;
use layout::box::{TextBox, RenderBox, RenderBoxData, UnscannedTextBox};
use layout::context::LayoutContext;

pub struct TextBoxData {
    run: @TextRun,
    offset: uint,
    length: uint
}

pub fn TextBoxData(run: @TextRun, offset: uint, length: uint) -> TextBoxData {
    TextBoxData {
        run: run,
        offset: offset,
        length: length
    }
}

pub fn adapt_textbox_with_range(box_data: &RenderBoxData, run: @TextRun, 
                                offset: uint, length: uint) -> @RenderBox {
    let new_box_data = copy *box_data;
    let new_text_data = TextBoxData(run, offset, length);
    @TextBox(move new_box_data, move new_text_data)
    // TODO: set position based on run metrics
    //new_box_data.position.size = { width: run.font
}

trait UnscannedMethods {
    pure fn raw_text() -> ~str;
}

impl RenderBox : UnscannedMethods {
    pure fn raw_text() -> ~str {
        match self {
            UnscannedTextBox(_, s) => copy s,
            _ => fail ~"unsupported operation: box.raw_text() on non-unscanned text box."
        }
    }
}

/* The main reflow routine for text layout. 
impl @RenderBox : TextLayout {
    fn reflow_text(ctx: &LayoutContext) {
        let d = match self {
            @TextBox(_,d) => { d }
            _ => { fail ~"expected text box in reflow_text!" }
        };

        // TODO: get font from textrun's TextStyle
        let font = ctx.font_cache.get_test_font();

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

        self.d().position.size = Size2D(max_width, total_height);
        d.runs = move dvec::unwrap(lines);
    }
}*/
