/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::ops::Range;

use icu_properties::props::{EnumeratedProperty, LineBreak};
use icu_segmenter::LineSegmenter;
use icu_segmenter::options::LineBreakOptions;
use servo_base::text::Utf8CodeUnits;

pub(crate) struct LineBreaker {
    linebreaks: Vec<Utf8CodeUnits>,
    current_linebreak_offset: usize,
    /// Whether or not the line breaker should include a break at this starting index.
    additional_break_at_start: Option<Utf8CodeUnits>,
}

impl LineBreaker {
    pub(crate) fn new(string: &str, options: LineBreakOptions<'_>) -> Self {
        let line_segmenter = LineSegmenter::new_auto(options);
        let mut linebreaks = line_segmenter
            .segment_str(string)
            // From https://docs.rs/icu_segmenter/1.5.0/icu_segmenter/struct.LineSegmenter.html
            // > For consistency with the grapheme, word, and sentence segmenters, there is always a
            // > breakpoint returned at index 0, but this breakpoint is not a meaningful line break
            // > opportunity.
            //
            // Skip this first line break opportunity, as it isn't interesting to us.
            .skip(1)
            .map(|offset| Utf8CodeUnits(offset as u32))
            .collect();
        add_break_opportunities_after_space(string, &mut linebreaks);
        Self {
            linebreaks,
            current_linebreak_offset: 0,
            additional_break_at_start: None,
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

    pub(crate) fn set_additional_break_at_start(&mut self, index: Utf8CodeUnits) {
        debug_assert!(self.additional_break_at_start.is_none());
        self.additional_break_at_start = Some(index);
    }

    pub(crate) fn take_additional_break_at_start(&mut self) -> Option<Utf8CodeUnits> {
        self.additional_break_at_start.take()
    }
}

/// Add soft wrap opportunities before punctuation that UAX #14 LB13 keeps
/// attached to the preceding word even after a space.
///
/// Chrome, Firefox and WebKit allow a break there and is flexible to UA:
/// <https://drafts.csswg.org/css-text-3/#line-break-details>
fn add_break_opportunities_after_space(string: &str, linebreaks: &mut Vec<Utf8CodeUnits>) {
    let mut extra_linebreaks = Vec::new();
    // U+0020 is the only character in the `Space` line breaking class which is one byte in
    // UTF-8, so it can be found without a character iterator. Only the character after each
    // space needs to be decoded and classified.
    for (space_offset, _) in string.match_indices(' ') {
        let offset = space_offset + 1;
        let Some(character) = string[offset..].chars().next() else {
            continue;
        };
        if is_punctuation_kept_after_spaces_by_lb13(LineBreak::for_char(character)) {
            extra_linebreaks.push(Utf8CodeUnits(offset as u32));
        }
    }

    if extra_linebreaks.is_empty() {
        return;
    }

    // Both lists are sorted, so merge them in one pass.
    let mut merged = Vec::with_capacity(linebreaks.len() + extra_linebreaks.len());
    let mut existing_linebreaks = linebreaks.iter().copied().peekable();
    for offset in extra_linebreaks {
        while existing_linebreaks
            .peek()
            .is_some_and(|next| *next < offset)
        {
            merged.push(existing_linebreaks.next().unwrap());
        }
        // skip duplicate.
        if existing_linebreaks.peek() != Some(&offset) {
            merged.push(offset);
        }
    }
    merged.extend(existing_linebreaks);
    *linebreaks = merged;
}

/// The line breaking classes for which UAX #14 rule LB13 prohibits a break even when the
/// character is preceded by a space: CL, CP, EX, IS and SY.
/// <https://www.unicode.org/reports/tr14/#LB13>
fn is_punctuation_kept_after_spaces_by_lb13(line_break: LineBreak) -> bool {
    matches!(
        line_break,
        LineBreak::ClosePunctuation |
            LineBreak::CloseParenthesis |
            LineBreak::Exclamation |
            LineBreak::InfixNumeric |
            LineBreak::BreakSymbols
    )
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
        assert_eq!(linebreaker.linebreaks, [Utf8CodeUnits(4), Utf8CodeUnits(7)]);
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
        assert_eq!(
            linebreaker.linebreaks,
            [Utf8CodeUnits(4), Utf8CodeUnits(6), Utf8CodeUnits(9)]
        );
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
        assert_eq!(
            linebreaker.linebreaks,
            [Utf8CodeUnits(4), Utf8CodeUnits(6), Utf8CodeUnits(9)]
        );
        assert!(
            advance_to_linebreaks_in_range(&mut linebreaker, 0..7) ==
                &[Utf8CodeUnits(4), Utf8CodeUnits(6)]
        );
        assert!(advance_to_linebreaks_in_range(&mut linebreaker, 8..9).is_empty());

        // We've already advanced, so a range from the beginning shouldn't affect things.
        assert!(advance_to_linebreaks_in_range(&mut linebreaker, 0..9).is_empty());

        linebreaker.current_linebreak_offset = 0;

        // Sending a value out of range shouldn't break things.
        assert!(
            advance_to_linebreaks_in_range(&mut linebreaker, 0..999) ==
                &[Utf8CodeUnits(4), Utf8CodeUnits(6)]
        );

        linebreaker.current_linebreak_offset = 0;

        std::panic::catch_unwind(|| {
            let mut linebreaker = LineBreaker::new("abc d def", LineBreakOptions::default());
            advance_to_linebreaks_in_range(&mut linebreaker, 2..0);
        })
        .expect_err("Reversed range should cause an assertion failure.");
    }
}
