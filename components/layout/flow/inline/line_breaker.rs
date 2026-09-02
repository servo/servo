/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::ops::Range;

use icu_segmenter::LineSegmenter;
use icu_segmenter::options::LineBreakOptions;
use servo_base::text::Utf8CodeUnits;

pub(crate) struct LineBreaker {
    linebreaks: Vec<Utf8CodeUnits>,
    current_linebreak_offset: usize,
}

impl LineBreaker {
    pub(crate) fn new(string: &str, options: LineBreakOptions<'_>) -> Self {
        let line_segmenter = LineSegmenter::new_auto(options);
        Self {
            // From https://docs.rs/icu_segmenter/1.5.0/icu_segmenter/struct.LineSegmenter.html
            // > For consistency with the grapheme, word, and sentence segmenters, there is always a
            // > breakpoint returned at index 0, but this breakpoint is not a meaningful line break
            // > opportunity.
            //
            // Skip this first line break opportunity, as it isn't interesting to us.
            linebreaks: line_segmenter
                .segment_str(string)
                .skip(1)
                .map(|offset| Utf8CodeUnits(offset as u32))
                .collect(),
            current_linebreak_offset: 0,
        }
    }

    pub(crate) fn advance_to_linebreaks_in_range(
        &mut self,
        text_range: Range<Utf8CodeUnits>,
    ) -> &[Utf8CodeUnits] {
        let linebreaks_in_range = self.linebreaks_in_range_after_current_offset(text_range);
        self.current_linebreak_offset = linebreaks_in_range.end;
        &self.linebreaks[linebreaks_in_range]
    }

    fn linebreaks_in_range_after_current_offset(
        &self,
        text_range: Range<Utf8CodeUnits>,
    ) -> Range<usize> {
        assert!(text_range.start <= text_range.end);

        let mut linebreaks_range = self.current_linebreak_offset..self.linebreaks.len();

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

#[cfg(test)]
mod test {
    use super::*;

    fn linebreaks_in_range_after_current_offset(
        linebreaker: &LineBreaker,
        range: Range<u32>,
    ) -> Range<usize> {
        linebreaker.linebreaks_in_range_after_current_offset(
            Utf8CodeUnits(range.start)..Utf8CodeUnits(range.end),
        )
    }

    #[test]
    fn test_linebreaker_ranges() {
        let linebreaker = LineBreaker::new("abc def", LineBreakOptions::default());
        assert_eq!(linebreaker.linebreaks, [4, 7]);
        assert_eq!(
            linebreaks_in_range_after_current_offset(&linebreaker, 0..5),
            0..1
        );
        // The last linebreak should not be included for the text range we are interested in.
        assert_eq!(
            linebreaks_in_range_after_current_offset(&linebreaker, 0..7),
            0..1
        );

        let linebreaker = LineBreaker::new("abc d def", LineBreakOptions::default());
        assert_eq!(linebreaker.linebreaks, [4, 6, 9]);
        assert_eq!(
            linebreaks_in_range_after_current_offset(&linebreaker, 0..5),
            0..1
        );
        assert_eq!(
            linebreaks_in_range_after_current_offset(&linebreaker, 0..7),
            0..2
        );
        assert_eq!(
            linebreaks_in_range_after_current_offset(&linebreaker, 0..9),
            0..2
        );

        assert_eq!(
            linebreaks_in_range_after_current_offset(&linebreaker, 4..9),
            0..2
        );

        std::panic::catch_unwind(|| {
            let linebreaker = LineBreaker::new("abc def", LineBreakOptions::default());
            linebreaks_in_range_after_current_offset(&linebreaker, 5..2);
        })
        .expect_err("Reversed range should cause an assertion failure.");
    }

    fn advance_to_linebreaks_in_range(
        linebreaker: &mut LineBreaker,
        range: Range<u32>,
    ) -> &[Utf8CodeUnits] {
        linebreaker
            .advance_to_linebreaks_in_range(Utf8CodeUnits(range.start)..Utf8CodeUnits(range.end))
    }

    #[test]
    fn test_linebreaker_stateful_advance() {
        let mut linebreaker = LineBreaker::new("abc d def", LineBreakOptions::default());
        assert_eq!(linebreaker.linebreaks, [4, 6, 9]);
        assert!(advance_to_linebreaks_in_range(&mut linebreaker, 0..7) == &[4, 6]);
        assert!(advance_to_linebreaks_in_range(&mut linebreaker, 8..9).is_empty());

        // We've already advanced, so a range from the beginning shouldn't affect things.
        assert!(advance_to_linebreaks_in_range(&mut linebreaker, 0..9).is_empty());

        linebreaker.current_linebreak_offset = 0;

        // Sending a value out of range shouldn't break things.
        assert!(advance_to_linebreaks_in_range(&mut linebreaker, 0..999) == &[4, 6]);

        linebreaker.current_linebreak_offset = 0;

        std::panic::catch_unwind(|| {
            let mut linebreaker = LineBreaker::new("abc d def", LineBreakOptions::default());
            advance_to_linebreaks_in_range(&mut linebreaker, 2..0);
        })
        .expect_err("Reversed range should cause an assertion failure.");
    }
}
