use super::*;
use crate::fonts::BITSTREAM_VERA_SANS;
use crate::text::ShapedSegment;

#[derive(Debug, Default)]
pub(in crate::layout) struct InlineFormattingContext {
    pub(super) inline_level_boxes: Vec<Arc<InlineLevelBox>>,
}

#[derive(Debug)]
pub(in crate::layout) enum InlineLevelBox {
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
pub(in crate::layout) struct InlineBox {
    pub style: Arc<ComputedValues>,
    pub first_fragment: bool,
    pub last_fragment: bool,
    pub children: Vec<Arc<InlineLevelBox>>,
}

/// https://www.w3.org/TR/css-display-3/#css-text-run
#[derive(Debug)]
pub(in crate::layout) struct TextRun {
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
                    }
                    InlineLevelBox::TextRun(run) => run.layout(&mut ifc),
                    InlineLevelBox::Atomic { style: _, contents } => {
                        // FIXME
                        match *contents {}
                    }
                    InlineLevelBox::OutOfFlowAbsolutelyPositionedBox(box_) => {
                        let initial_start_corner = match box_.style.specified_display {
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
                            }
                            Display::None => panic!("display:none does not generate an abspos box"),
                        };
                        absolutely_positioned_fragments
                            .push(box_.layout(initial_start_corner, tree_rank));
                    }
                    InlineLevelBox::OutOfFlowFloatBox(_box_) => {
                        // TODO
                        continue;
                    }
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
        let mut border = style.border_width().percentages_relative_to(cbis);
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
            *inline_position += fragment.padding.inline_end
                + fragment.border.inline_end
                + fragment.margin.inline_end;
        } else {
            fragment.padding.inline_end = Length::zero();
            fragment.border.inline_end = Length::zero();
            fragment.margin.inline_end = Length::zero();
        }
        self.parent_nesting_level
            .max_block_size_of_fragments_so_far
            .max_assign(
                fragment.content_rect.size.block
                    + fragment.padding.block_sum()
                    + fragment.border.block_sum()
                    + fragment.margin.block_sum(),
            );
        self.parent_nesting_level
            .fragments_so_far
            .push(Fragment::Box(fragment));
    }
}

impl TextRun {
    fn layout(&self, ifc: &mut InlineFormattingContextState) {
        let available = ifc.containing_block.inline_size - ifc.inline_position;
        let mut chars = self.text.chars();
        loop {
            let mut shaped = ShapedSegment::new_with_naive_shaping(BITSTREAM_VERA_SANS.clone());
            let mut last_break_opportunity = None;
            loop {
                let next = chars.next();
                if matches!(next, Some(' ') | None) {
                    let inline_size = self.parent_style.font.font_size * shaped.advance_width;
                    if inline_size > available {
                        if let Some((state, iter)) = last_break_opportunity.take() {
                            shaped.restore(&state);
                            chars = iter;
                        }
                        break;
                    }
                }
                if let Some(ch) = next {
                    if ch == ' ' {
                        last_break_opportunity = Some((shaped.save(), chars.clone()))
                    }
                    shaped.append_char(ch).unwrap()
                } else {
                    break;
                }
            }
            let inline_size = self.parent_style.font.font_size * shaped.advance_width;
            // https://www.w3.org/TR/CSS2/visudet.html#propdef-line-height
            // 'normal':
            // “set the used value to a "reasonable" value based on the font of the element.”
            let line_height = self.parent_style.font.font_size.0 * 1.2;
            let content_rect = Rect {
                start_corner: Vec2 {
                    block: Length::zero(),
                    inline: ifc.inline_position - ifc.current_nesting_level.inline_start,
                },
                size: Vec2 {
                    block: line_height,
                    inline: inline_size,
                },
            };
            ifc.inline_position += inline_size;
            ifc.current_nesting_level
                .max_block_size_of_fragments_so_far
                .max_assign(line_height);
            ifc.current_nesting_level
                .fragments_so_far
                .push(Fragment::Text(TextFragment {
                    parent_style: self.parent_style.clone(),
                    content_rect,
                    text: shaped,
                }));
            if chars.as_str().is_empty() {
                break;
            } else {
                // New line
                ifc.current_nesting_level.inline_start = Length::zero();
                let mut nesting_level = &mut ifc.current_nesting_level;
                for partial in ifc.partial_inline_boxes_stack.iter_mut().rev() {
                    partial.finish_layout(nesting_level, &mut ifc.inline_position, true);
                    partial.start_corner.inline = Length::zero();
                    partial.padding.inline_start = Length::zero();
                    partial.border.inline_start = Length::zero();
                    partial.margin.inline_start = Length::zero();
                    partial.parent_nesting_level.inline_start = Length::zero();
                    nesting_level = &mut partial.parent_nesting_level;
                }
                ifc.line_boxes
                    .finish_line(nesting_level, ifc.containing_block);
                ifc.inline_position = Length::zero();
            }
        }
    }
}
