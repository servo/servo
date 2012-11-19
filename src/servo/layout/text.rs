/** Text layout. */

use layout::box::{TextBox, RenderBox, RenderBoxData, UnscannedTextBox};

use gfx::text::text_run::TextRun;
use gfx::util::range::MutableRange;

pub struct TextBoxData {
    run: @TextRun,
    range: MutableRange,
}

pub fn TextBoxData(run: @TextRun, range: &const MutableRange) -> TextBoxData {
    TextBoxData {
        run: run,
        range: copy *range,
    }
}

pub fn adapt_textbox_with_range(box_data: &RenderBoxData, run: @TextRun, 
                                range: &const MutableRange) -> @RenderBox {
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
