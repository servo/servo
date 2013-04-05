/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

/** Text layout. */

use layout::box::{TextBox, RenderBox, RenderBoxData, UnscannedTextBox};

use gfx::text::text_run::TextRun;
use gfx::util::range::Range;

pub struct TextBoxData {
    run: @TextRun,
    range: Range,
}

pub fn TextBoxData(run: @TextRun, range: &Range) -> TextBoxData {
    TextBoxData {
        run: run,
        range: copy *range,
    }
}

pub fn adapt_textbox_with_range(box_data: &mut RenderBoxData, run: @TextRun, 
                                range: &Range) -> @mut RenderBox {
    assert!(range.begin() < run.char_len());
    assert!(range.end() <= run.char_len());
    assert!(range.length() > 0);

    debug!("Creating textbox with span: (strlen=%u, off=%u, len=%u) of textrun: %s",
           run.char_len(), range.begin(), range.length(), run.text);
    let mut new_box_data = copy *box_data;
    let new_text_data = TextBoxData(run, range);
    let metrics = run.metrics_for_range(range);
    new_box_data.position.size = metrics.bounding_box.size;
    @mut TextBox(new_box_data, new_text_data)
}

pub trait UnscannedMethods {
    fn raw_text(&mut self) -> ~str;
}

impl UnscannedMethods for RenderBox {
    fn raw_text(&mut self) -> ~str {
        match self {
            &UnscannedTextBox(_, ref s) => copy *s,
            _ => fail!(~"unsupported operation: box.raw_text() on non-unscanned text box.")
        }
    }
}
