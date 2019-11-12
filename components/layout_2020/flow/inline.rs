/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::context::LayoutContext;
use crate::flow::float::FloatBox;
use crate::flow::FlowChildren;
use crate::fragments::{AnonymousFragment, BoxFragment, CollapsedBlockMargins, Fragment};
use crate::geom::flow_relative::{Rect, Sides, Vec2};
use crate::positioned::{AbsolutelyPositionedBox, AbsolutelyPositionedFragment};
use crate::replaced::ReplacedContent;
use crate::style_ext::{ComputedValuesExt, Display, DisplayGeneratingBox, DisplayOutside};
use crate::{relative_adjustement, take, ContainingBlock};
use servo_arc::Arc;
use style::properties::ComputedValues;
use style::values::computed::Length;
use style::Zero;

#[derive(Debug, Default)]
pub(crate) struct InlineFormattingContext {
    pub(super) inline_level_boxes: Vec<Arc<InlineLevelBox>>,
}

#[derive(Debug)]
pub(crate) enum InlineLevelBox {
    InlineBox(InlineBox),
    TextRun(TextRun),
    OutOfFlowAbsolutelyPositionedBox(AbsolutelyPositionedBox),
    OutOfFlowFloatBox(FloatBox),
    Atomic {
        style: Arc<ComputedValues>,
        // FIXME: this should be IndependentFormattingContext:
        contents: ReplacedContent,
    },
}

#[derive(Debug)]
pub(crate) struct InlineBox {
    pub style: Arc<ComputedValues>,
    pub first_fragment: bool,
    pub last_fragment: bool,
    pub children: Vec<Arc<InlineLevelBox>>,
}

/// https://www.w3.org/TR/css-display-3/#css-text-run
#[derive(Debug)]
pub(crate) struct TextRun {
    pub parent_style: Arc<ComputedValues>,
    pub text: String,
}

struct InlineNestingLevelState<'box_tree> {
    remaining_boxes: std::slice::Iter<'box_tree, Arc<InlineLevelBox>>,
    fragments_so_far: Vec<Fragment>,
    inline_start: Length,
    max_block_size_of_fragments_so_far: Length,
}

struct PartialInlineBoxFragment<'box_tree> {
    style: Arc<ComputedValues>,
    start_corner: Vec2<Length>,
    padding: Sides<Length>,
    border: Sides<Length>,
    margin: Sides<Length>,
    last_box_tree_fragment: bool,
    parent_nesting_level: InlineNestingLevelState<'box_tree>,
}

struct InlineFormattingContextState<'box_tree, 'cb> {
    containing_block: &'cb ContainingBlock,
    line_boxes: LinesBoxes,
    inline_position: Length,
    partial_inline_boxes_stack: Vec<PartialInlineBoxFragment<'box_tree>>,
    current_nesting_level: InlineNestingLevelState<'box_tree>,
}

struct LinesBoxes {
    boxes: Vec<Fragment>,
    next_line_block_position: Length,
}

impl InlineFormattingContext {
    pub(super) fn layout<'a>(
        &'a self,
        layout_context: &LayoutContext,
        containing_block: &ContainingBlock,
        tree_rank: usize,
        absolutely_positioned_fragments: &mut Vec<AbsolutelyPositionedFragment<'a>>,
    ) -> FlowChildren {
        let mut ifc = InlineFormattingContextState {
            containing_block,
            partial_inline_boxes_stack: Vec::new(),
            line_boxes: LinesBoxes {
                boxes: Vec::new(),
                next_line_block_position: Length::zero(),
            },
            inline_position: Length::zero(),
            current_nesting_level: InlineNestingLevelState {
                remaining_boxes: self.inline_level_boxes.iter(),
                fragments_so_far: Vec::with_capacity(self.inline_level_boxes.len()),
                inline_start: Length::zero(),
                max_block_size_of_fragments_so_far: Length::zero(),
            },
        };
        loop {
            if let Some(child) = ifc.current_nesting_level.remaining_boxes.next() {
                match &**child {
                    InlineLevelBox::InlineBox(inline) => {
                        let partial = inline.start_layout(&mut ifc);
                        ifc.partial_inline_boxes_stack.push(partial)
                    },
                    InlineLevelBox::TextRun(run) => run.layout(layout_context, &mut ifc),
                    InlineLevelBox::Atomic { style: _, contents } => {
                        // FIXME
                        match *contents {}
                    },
                    InlineLevelBox::OutOfFlowAbsolutelyPositionedBox(box_) => {
                        let initial_start_corner =
                            match Display::from(box_.style.get_box().original_display) {
                                Display::GeneratingBox(DisplayGeneratingBox::OutsideInside {
                                    outside,
                                    inside: _,
                                }) => Vec2 {
                                    inline: match outside {
                                        DisplayOutside::Inline => ifc.inline_position,
                                        DisplayOutside::Block => Length::zero(),
                                    },
                                    block: ifc.line_boxes.next_line_block_position,
                                },
                                Display::Contents => {
                                    panic!("display:contents does not generate an abspos box")
                                },
                                Display::None => {
                                    panic!("display:none does not generate an abspos box")
                                },
                            };
                        absolutely_positioned_fragments
                            .push(box_.layout(initial_start_corner, tree_rank));
                    },
                    InlineLevelBox::OutOfFlowFloatBox(_box_) => {
                        // TODO
                        continue;
                    },
                }
            } else
            // Reached the end of ifc.remaining_boxes
            if let Some(mut partial) = ifc.partial_inline_boxes_stack.pop() {
                partial.finish_layout(
                    &mut ifc.current_nesting_level,
                    &mut ifc.inline_position,
                    false,
                );
                ifc.current_nesting_level = partial.parent_nesting_level
            } else {
                ifc.line_boxes
                    .finish_line(&mut ifc.current_nesting_level, containing_block);
                return FlowChildren {
                    fragments: ifc.line_boxes.boxes,
                    block_size: ifc.line_boxes.next_line_block_position,
                    collapsible_margins_in_children: CollapsedBlockMargins::zero(),
                };
            }
        }
    }
}

impl LinesBoxes {
    fn finish_line(
        &mut self,
        top_nesting_level: &mut InlineNestingLevelState,
        containing_block: &ContainingBlock,
    ) {
        let start_corner = Vec2 {
            inline: Length::zero(),
            block: self.next_line_block_position,
        };
        let size = Vec2 {
            inline: containing_block.inline_size,
            block: std::mem::replace(
                &mut top_nesting_level.max_block_size_of_fragments_so_far,
                Length::zero(),
            ),
        };
        self.next_line_block_position += size.block;
        self.boxes.push(Fragment::Anonymous(AnonymousFragment {
            children: take(&mut top_nesting_level.fragments_so_far),
            rect: Rect { start_corner, size },
            mode: containing_block.mode,
        }))
    }
}

impl InlineBox {
    fn start_layout<'box_tree>(
        &'box_tree self,
        ifc: &mut InlineFormattingContextState<'box_tree, '_>,
    ) -> PartialInlineBoxFragment<'box_tree> {
        let style = self.style.clone();
        let cbis = ifc.containing_block.inline_size;
        let mut padding = style.padding().percentages_relative_to(cbis);
        let mut border = style.border_width();
        let mut margin = style
            .margin()
            .percentages_relative_to(cbis)
            .auto_is(Length::zero);
        if self.first_fragment {
            ifc.inline_position += padding.inline_start + border.inline_start + margin.inline_start;
        } else {
            padding.inline_start = Length::zero();
            border.inline_start = Length::zero();
            margin.inline_start = Length::zero();
        }
        let mut start_corner = Vec2 {
            block: padding.block_start + border.block_start + margin.block_start,
            inline: ifc.inline_position - ifc.current_nesting_level.inline_start,
        };
        start_corner += &relative_adjustement(
            &style,
            ifc.containing_block.inline_size,
            ifc.containing_block.block_size,
        );
        PartialInlineBoxFragment {
            style,
            start_corner,
            padding,
            border,
            margin,
            last_box_tree_fragment: self.last_fragment,
            parent_nesting_level: std::mem::replace(
                &mut ifc.current_nesting_level,
                InlineNestingLevelState {
                    remaining_boxes: self.children.iter(),
                    fragments_so_far: Vec::with_capacity(self.children.len()),
                    inline_start: ifc.inline_position,
                    max_block_size_of_fragments_so_far: Length::zero(),
                },
            ),
        }
    }
}

impl<'box_tree> PartialInlineBoxFragment<'box_tree> {
    fn finish_layout(
        &mut self,
        nesting_level: &mut InlineNestingLevelState,
        inline_position: &mut Length,
        at_line_break: bool,
    ) {
        let mut fragment = BoxFragment {
            style: self.style.clone(),
            children: take(&mut nesting_level.fragments_so_far),
            content_rect: Rect {
                size: Vec2 {
                    inline: *inline_position - self.start_corner.inline,
                    block: nesting_level.max_block_size_of_fragments_so_far,
                },
                start_corner: self.start_corner.clone(),
            },
            padding: self.padding.clone(),
            border: self.border.clone(),
            margin: self.margin.clone(),
            block_margins_collapsed_with_children: CollapsedBlockMargins::zero(),
        };
        let last_fragment = self.last_box_tree_fragment && !at_line_break;
        if last_fragment {
            *inline_position += fragment.padding.inline_end +
                fragment.border.inline_end +
                fragment.margin.inline_end;
        } else {
            fragment.padding.inline_end = Length::zero();
            fragment.border.inline_end = Length::zero();
            fragment.margin.inline_end = Length::zero();
        }
        self.parent_nesting_level
            .max_block_size_of_fragments_so_far
            .max_assign(
                fragment.content_rect.size.block +
                    fragment.padding.block_sum() +
                    fragment.border.block_sum() +
                    fragment.margin.block_sum(),
            );
        self.parent_nesting_level
            .fragments_so_far
            .push(Fragment::Box(fragment));
    }
}

impl TextRun {
    fn layout(&self, _layout_context: &LayoutContext, _ifc: &mut InlineFormattingContextState) {
        // TODO
    }
}
