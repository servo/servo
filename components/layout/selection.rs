/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::ops::Range;

use icu_segmenter::WordSegmenter;
use itertools::Either;
use layout_api::SegmentGranularity;
use script::layout_dom::ServoLayoutNode;
use servo_base::text::{AssumeUnder4GB, Utf32CodeUnits, Utf32CodeUnitsOrNodeOffset};
use style::dom::OpaqueNode;

use crate::ArcRefCell;
use crate::dom::{LayoutBox, NodeExt};
use crate::flow::inline::InlineFormattingContext;
use crate::flow::inline::text_run::TextRun;

/// An iterator that turns boundaries in UTF-8 code unit offsets into UTF-32 code unit ranges.
/// Empty ranges are skipped.
struct BoundaryIterator<'a, Boundaries> {
    boundaries: Boundaries,
    text: &'a str,
    previous_boundary: usize,
    previous_boundary_in_characters: Utf32CodeUnits,
}

impl<'a, Boundaries> BoundaryIterator<'a, Boundaries> {
    fn new(text: &'a str, boundaries: Boundaries) -> Self {
        Self {
            boundaries,
            text,
            previous_boundary: 0,
            previous_boundary_in_characters: Utf32CodeUnits(0),
        }
    }
}

impl<Boundaries: Iterator<Item = usize>> Iterator for BoundaryIterator<'_, Boundaries> {
    type Item = Range<Utf32CodeUnits>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut next_boundary = self.boundaries.next()?;
        while next_boundary == self.previous_boundary {
            next_boundary = self.boundaries.next()?;
        }

        let start = self.previous_boundary_in_characters;
        let end = start +
            Utf32CodeUnits::length_of(
                AssumeUnder4GB,
                &self.text[self.previous_boundary..next_boundary],
            );
        self.previous_boundary = next_boundary;
        self.previous_boundary_in_characters = end;
        Some(start..end)
    }
}

pub(crate) struct PositionInInlineFormattingContext {
    /// The [`LayoutBox`] of the containing inline formatting context box.
    container: LayoutBox,
    /// The offset of this position in the inline formatting context's transformed text.
    offset: Utf32CodeUnits,
    /// The [`TextRun`] that contains this position.
    text_run: ArcRefCell<TextRun>,
}

impl PositionInInlineFormattingContext {
    /// If the given text node and a DOM offset in that node are contained within an
    /// inline formatting context, create a [`PositionInInlineFormattingContext`] for
    /// them. If the node is not a text node or the containing inline formatting context
    /// cannot be found return `None`.
    pub(crate) fn from_dom_position_in_text_node(
        node: &ServoLayoutNode,
        offset: Utf32CodeUnitsOrNodeOffset,
    ) -> Option<Self> {
        let inner_layout_data = node.inner_layout_data();
        let self_box = inner_layout_data.as_ref()?.self_box.borrow();
        let layout_box = self_box.as_ref()?;

        let LayoutBox::Text(text_run) = layout_box else {
            return None;
        };
        let offset = text_run
            .borrow()
            .run_data
            .map_dom_offset_to_transformed_ifc_offset(Utf32CodeUnits(offset.0));
        let container = layout_box.layout_box_for_containing_inline_formatting_context()?;
        Some(Self {
            container,
            offset,
            text_run: text_run.clone(),
        })
    }

    /// Return the offsets of the segment with the given [`SegmentGranularity`] at this position.
    fn offsets(&self, granularity: SegmentGranularity) -> Range<Utf32CodeUnits> {
        let text_run = self.text_run.borrow();
        let run_data = &text_run.run_data;
        let text = run_data
            .text_content
            .get()
            .expect("There should always be text at this point");

        let iterator = || match granularity {
            SegmentGranularity::Word => Either::Left(BoundaryIterator::new(
                text,
                WordSegmenter::new_auto(Default::default()).segment_str(text),
            )),
            SegmentGranularity::HardLineBreak => Either::Right(BoundaryIterator::new(
                text,
                text.split_inclusive("\n").scan(0, |offset, chunk| {
                    *offset += chunk.len();
                    Some(*offset)
                }),
            )),
        };

        iterator()
            .find(|range| range.contains(&self.offset))
            .or_else(|| iterator().last())
            .unwrap_or_else(|| run_data.character_range_in_ifc_text.clone())
    }

    /// Find the segment boundaries of the segment with the given [`SegmentGranularity`]
    /// at this position.
    pub(crate) fn segment(
        &self,
        granularity: SegmentGranularity,
    ) -> Option<(
        (OpaqueNode, Utf32CodeUnitsOrNodeOffset),
        (OpaqueNode, Utf32CodeUnitsOrNodeOffset),
    )> {
        let range = self.offsets(granularity);
        self.container
            .with_inline_formatting_context(|inline_formatting_context| {
                let start = inline_formatting_context
                    .find_dom_position_of_offset(range.start, SearchAffinity::RangeStart)?;
                let end = inline_formatting_context
                    .find_dom_position_of_offset(range.end, SearchAffinity::RangeEnd)?;
                Some((start, end))
            })
            .flatten()
    }
}

/// Decides which way to bias a search for an offset in an inline formatting context.
enum SearchAffinity {
    /// Search for the first offset after the given offset, and fall back to the end of
    /// the last text run when searching for an offset.
    RangeStart,
    /// Search for the last offset before the given offset and fall back to the start of the
    /// first text run when searching for an offset.
    RangeEnd,
}

impl InlineFormattingContext {
    /// For the given UTF-32 offset in transformed inline formatting text, find the text
    /// node and DOM text offset that best corresponds to it.
    fn find_dom_position_of_offset(
        &self,
        offset: Utf32CodeUnits,
        search_direction: SearchAffinity,
    ) -> Option<(OpaqueNode, Utf32CodeUnitsOrNodeOffset)> {
        let mut text_runs = self.non_pseudo_text_runs();
        let text_run = match search_direction {
            SearchAffinity::RangeStart => text_runs
                .find(|text_run| offset < text_run.run_data.character_range_in_ifc_text.end)
                .or_else(|| self.non_pseudo_text_runs().next_back()),
            SearchAffinity::RangeEnd => text_runs
                .rev()
                .find(|text_run| offset > text_run.run_data.character_range_in_ifc_text.start)
                .or_else(|| self.non_pseudo_text_runs().next()),
        }?;

        let mapped_offset = text_run
            .run_data
            .map_transformed_ifc_offset_to_dom_offset(offset);
        Some((
            text_run.base_fragment_info.tag?.node,
            Utf32CodeUnitsOrNodeOffset(mapped_offset.0),
        ))
    }
}
