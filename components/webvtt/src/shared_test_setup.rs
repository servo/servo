/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::RefCell;

use crate::{IncrementalWebVTTParser, WebVttCue, WebVttParserSink};

pub fn compute_result_in_seconds(hours: f64, minutes: f64, seconds: f64, thousands: f64) -> f64 {
    hours * 60. * 60. + minutes * 60. + seconds + thousands / 1000.
}

#[derive(Default)]
pub struct DummySink {
    pub collected_cues: RefCell<Vec<WebVttCue>>,
}

impl WebVttParserSink<()> for DummySink {
    fn consume_cue(&self, _: &mut (), cue: WebVttCue) {
        self.collected_cues.borrow_mut().push(cue);
    }
}

pub fn parser_with_dummy_sink() -> IncrementalWebVTTParser<(), DummySink> {
    IncrementalWebVTTParser::new(DummySink::default())
}
