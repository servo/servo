/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::ops::Range;

use icu_segmenter::LineSegmenter;

pub(crate) struct LineBreaker {
    linebreaks: Vec<usize>,
    current_offset: usize,
}

impl LineBreaker {
    pub(crate) fn new(string: &str) -> Self {
        let line_segmenter = LineSegmenter::new_auto();
        Self {
            // From https://docs.rs/icu_segmenter/1.5.0/icu_segmenter/struct.LineSegmenter.html
            // > For consistency with the grapheme, word, and sentence segmenters, there is always a
            // > breakpoint returned at index 0, but this breakpoint is not a meaningful line break
            // > opportunity.
            //
            // Skip this first line break opportunity, as it isn't interesting to us.
            linebreaks: line_segmenter.segment_str(string).skip(1).collect(),
            current_offset: 0,
        }
    }

    pub(crate) fn advance_to_linebreaks_in_range(&mut self, text_range: Range<usize>) -> &[usize] {
        let linebreaks_in_range = self.linebreaks_in_range_after_current_offset(text_range);
        self.current_offset = linebreaks_in_range.end;
        &self.linebreaks[linebreaks_in_range]
    }

    fn linebreaks_in_range_after_current_offset(&self, text_range: Range<usize>) -> Range<usize> {
        assert!(text_range.start <= text_range.end);

        let mut linebreaks_range = self.current_offset..self.linebreaks.len();

        while self.linebreaks[linebreaks_range.start] < text_range.start &&
            linebreaks_range.len() > 1
        {
            linebreaks_range.start += 1;
        }

        let mut ending_linebreak_index = linebreaks_range.start;
        while self.linebreaks[ending_linebreak_index] < text_range.end &&
            ending_linebreak_index < self.linebreaks.len() - 1
        {
            ending_linebreak_index += 1;
        }
        linebreaks_range.end = ending_linebreak_index;
        linebreaks_range
    }
}

#[test]
fn test_linebreaker_ranges() {
    let linebreaker = LineBreaker::new("abc def");
    assert_eq!(linebreaker.linebreaks, [4, 7]);
    assert_eq!(
        linebreaker.linebreaks_in_range_after_current_offset(0..5),
        0..1
    );
    // The last linebreak should not be included for the text range we are interested in.
    assert_eq!(
        linebreaker.linebreaks_in_range_after_current_offset(0..7),
        0..1
    );

    let linebreaker = LineBreaker::new("abc d def");
    assert_eq!(linebreaker.linebreaks, [4, 6, 9]);
    assert_eq!(
        linebreaker.linebreaks_in_range_after_current_offset(0..5),
        0..1
    );
    assert_eq!(
        linebreaker.linebreaks_in_range_after_current_offset(0..7),
        0..2
    );
    assert_eq!(
        linebreaker.linebreaks_in_range_after_current_offset(0..9),
        0..2
    );

    assert_eq!(
        linebreaker.linebreaks_in_range_after_current_offset(4..9),
        0..2
    );

    std::panic::catch_unwind(|| {
        let linebreaker = LineBreaker::new("abc def");
        linebreaker.linebreaks_in_range_after_current_offset(5..2);
    })
    .expect_err("Reversed range should cause an assertion failure.");
}

#[test]
fn test_linebreaker_stateful_advance() {
    let mut linebreaker = LineBreaker::new("abc d def");
    assert_eq!(linebreaker.linebreaks, [4, 6, 9]);
    assert!(linebreaker.advance_to_linebreaks_in_range(0..7) == &[4, 6]);
    assert!(linebreaker.advance_to_linebreaks_in_range(8..9).is_empty());

    // We've already advanced, so a range from the beginning shouldn't affect things.
    assert!(linebreaker.advance_to_linebreaks_in_range(0..9).is_empty());

    linebreaker.current_offset = 0;

    // Sending a value out of range shoudn't break things.
    assert!(linebreaker.advance_to_linebreaks_in_range(0..999) == &[4, 6]);

    linebreaker.current_offset = 0;

    std::panic::catch_unwind(|| {
        let mut linebreaker = LineBreaker::new("abc d def");
        linebreaker.advance_to_linebreaks_in_range(2..0);
    })
    .expect_err("Reversed range should cause an assertion failure.");
}
