/** Text layout. */

use servo_text::text_run::{TextRange, TextRun};
use layout::box::{TextBox, RenderBox, RenderBoxData, UnscannedTextBox};

pub struct TextBoxData {
    run: @TextRun,
    range: TextRange,
}

pub fn TextBoxData(run: @TextRun, range: TextRange) -> TextBoxData {
    TextBoxData {
        run: run,
        range: range,
    }
}

pub fn adapt_textbox_with_range(box_data: &RenderBoxData, run: @TextRun, 
                                range: TextRange) -> @RenderBox {
    debug!("Creating textbox with span: (strlen=%u, off=%u, len=%u) of textrun: %s",
           run.text.len(), range.begin(), range.length(), run.text);
    let new_box_data = copy *box_data;
    let new_text_data = TextBoxData(run, range);
    let metrics = run.metrics_for_range(range);
    new_box_data.position.size = metrics.bounding_box.size;
    @TextBox(move new_box_data, move new_text_data)
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
