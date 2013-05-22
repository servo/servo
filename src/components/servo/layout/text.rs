/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Text layout.

use layout::box::{RenderBox, RenderBoxBase, TextRenderBox, UnscannedTextRenderBoxClass};

use gfx::text::text_run::TextRun;
use servo_util::range::Range;

pub struct TextBoxData {
    run: @TextRun,
    range: Range,
}

impl TextBoxData {
    pub fn new(run: @TextRun, range: Range) -> TextBoxData {
        TextBoxData {
            run: run,
            range: range,
        }
    }

    pub fn teardown(&self) {
        self.run.teardown();
    }
}

pub fn adapt_textbox_with_range(mut base: RenderBoxBase, run: @TextRun, range: Range)
                                -> TextRenderBox {
    assert!(range.begin() < run.char_len());
    assert!(range.end() <= run.char_len());
    assert!(range.length() > 0);

    debug!("Creating textbox with span: (strlen=%u, off=%u, len=%u) of textrun: %s",
           run.char_len(),
           range.begin(),
           range.length(),
           run.text);
    let new_text_data = TextBoxData::new(run, range);
    let metrics = run.metrics_for_range(&range);

    base.position.size = metrics.bounding_box.size;

    TextRenderBox {
        base: base,
        text_data: new_text_data,
    }
}

pub trait UnscannedMethods {
    /// Copies out the text from an unscanned text box. Fails if this is not an unscanned text box.
    fn raw_text(&self) -> ~str;
}

impl UnscannedMethods for RenderBox {
    fn raw_text(&self) -> ~str {
        match *self {
            UnscannedTextRenderBoxClass(text_box) => copy text_box.text,
            _ => fail!(~"unsupported operation: box.raw_text() on non-unscanned text box."),
        }
    }
}
