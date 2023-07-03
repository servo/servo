/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::cell::ArcRefCell;
use crate::context::LayoutContext;
use crate::flow::float::{FloatBox, SequentialLayoutState};
use crate::flow::FlowLayout;
use crate::formatting_contexts::IndependentFormattingContext;
use crate::fragment_tree::{
    AnonymousFragment, BaseFragmentInfo, BoxFragment, CollapsedBlockMargins, CollapsedMargin,
    FontMetrics, Fragment, TextFragment,
};
use crate::geom::flow_relative::{Rect, Sides, Vec2};
use crate::positioned::{
    relative_adjustement, AbsolutelyPositionedBox, HoistedAbsolutelyPositionedBox,
    PositioningContext,
};
use crate::sizing::ContentSizes;
use crate::style_ext::{ComputedValuesExt, Display, DisplayGeneratingBox, DisplayOutside};
use crate::ContainingBlock;
use app_units::Au;
use atomic_refcell::AtomicRef;
use gfx::text::glyph::GlyphStore;
use gfx::text::text_run::GlyphRun;
use servo_arc::Arc;
use style::computed_values::white_space::T as WhiteSpace;
use style::logical_geometry::WritingMode;
use style::properties::ComputedValues;
use style::values::computed::{Length, LengthPercentage, Percentage};
use style::values::generics::text::LineHeight;
use style::values::specified::text::TextAlignKeyword;
use style::values::specified::text::TextDecorationLine;
use style::Zero;
use webrender_api::FontInstanceKey;
use xi_unicode::LineBreakLeafIter;
#[derive(Debug, Serialize)]
pub(crate) struct InlineFormattingContext {
    pub(super) inline_level_boxes: Vec<ArcRefCell<InlineLevelBox>>,
    pub(super) text_decoration_line: TextDecorationLine,
    // Whether this IFC contains the 1st formatted line of an element
    // https://www.w3.org/TR/css-pseudo-4/#first-formatted-line
    pub(super) has_first_formatted_line: bool,
    pub(super) contains_floats: bool,
    /// Whether this IFC being constructed currently ends with whitespace. This is used to
    /// implement rule 4 of <https://www.w3.org/TR/css-text-3/#collapse>:
    ///
    /// > Any collapsible space immediately following another collapsible space—even one
    /// > outside the boundary of the inline containing that space, provided both spaces are
    /// > within the same inline formatting context—is collapsed to have zero advance width.
    /// > (It is invisible, but retains its soft wrap opportunity, if any.)
    pub(super) ends_with_whitespace: bool,
}

#[derive(Debug, Serialize)]
pub(crate) enum InlineLevelBox {
    InlineBox(InlineBox),
    TextRun(TextRun),
    OutOfFlowAbsolutelyPositionedBox(ArcRefCell<AbsolutelyPositionedBox>),
    OutOfFlowFloatBox(FloatBox),
    Atomic(IndependentFormattingContext),
}

#[derive(Debug, Serialize)]
pub(crate) struct InlineBox {
    pub base_fragment_info: BaseFragmentInfo,
    #[serde(skip_serializing)]
    pub style: Arc<ComputedValues>,
    pub first_fragment: bool,
    pub last_fragment: bool,
    pub children: Vec<ArcRefCell<InlineLevelBox>>,
}

/// https://www.w3.org/TR/css-display-3/#css-text-run
#[derive(Debug, Serialize)]
pub(crate) struct TextRun {
    pub base_fragment_info: BaseFragmentInfo,
    #[serde(skip_serializing)]
    pub parent_style: Arc<ComputedValues>,
    pub text: String,
    pub has_uncollapsible_content: bool,
}

struct InlineNestingLevelState<'box_tree> {
    remaining_boxes: InlineBoxChildIter<'box_tree>,
    fragments_so_far: Vec<Fragment>,
    inline_start: Length,
    max_block_size_of_fragments_so_far: Length,
    positioning_context: Option<PositioningContext>,
    white_space: WhiteSpace,
    /// Indicates whether this nesting level have text decorations in effect.
    /// From https://drafts.csswg.org/css-text-decor/#line-decoration
    // "When specified on or propagated to a block container that establishes
    //  an IFC..."
    text_decoration_line: TextDecorationLine,
}

struct PartialInlineBoxFragment<'box_tree> {
    base_fragment_info: BaseFragmentInfo,
    style: Arc<ComputedValues>,
    start_corner: Vec2<Length>,
    padding: Sides<Length>,
    border: Sides<Length>,
    margin: Sides<Length>,

    /// Whether or not this inline box has already been part of a previous line.
    /// We need to create at least one Fragment for every inline box, but on following
    /// lines, if the inline box is totally empty (such as after a preserved line
    /// break), then we don't want to create empty Fragments for it.
    was_part_of_previous_line: bool,

    parent_nesting_level: InlineNestingLevelState<'box_tree>,
}

struct InlineFormattingContextState<'box_tree, 'a, 'b> {
    positioning_context: &'a mut PositioningContext,
    containing_block: &'b ContainingBlock<'b>,
    lines: Lines,

    /// The current inline position in this inline formatting context independent
    /// of the depth in the nesting level.
    inline_position: Length,

    /// Whether any active line box has added a glyph, border, margin, or padding
    /// to this line, which indicates that the next run that exceeds the line length
    /// can cause a line break.
    line_had_any_content: bool,

    // Whether or not this line had any absolutely positioned boxes.
    line_had_any_absolutes: bool,

    /// The line breaking state for this inline formatting context.
    linebreaker: Option<LineBreakLeafIter>,

    partial_inline_boxes_stack: Vec<PartialInlineBoxFragment<'box_tree>>,
    current_nesting_level: InlineNestingLevelState<'box_tree>,
    sequential_layout_state: Option<&'a mut SequentialLayoutState>,
}

impl<'box_tree, 'a, 'b> InlineFormattingContextState<'box_tree, 'a, 'b> {
    fn push_hoisted_box_to_positioning_context(
        &mut self,
        hoisted_box: HoistedAbsolutelyPositionedBox,
    ) {
        self.line_had_any_absolutes = true;

        if let Some(context) = self.current_nesting_level.positioning_context.as_mut() {
            context.push(hoisted_box);
            return;
        }

        for nesting_level in self.partial_inline_boxes_stack.iter_mut().rev() {
            if let Some(context) = nesting_level
                .parent_nesting_level
                .positioning_context
                .as_mut()
            {
                context.push(hoisted_box);
                return;
            }
        }

        self.positioning_context.push(hoisted_box);
    }

    /// Finish layout of all the partial inline boxes in the current line,
    /// finish current line and start a new one.
    fn finish_line_and_reset(&mut self, layout_context: &LayoutContext) {
        self.current_nesting_level.inline_start = Length::zero();
        let mut nesting_level = &mut self.current_nesting_level;
        for partial in self.partial_inline_boxes_stack.iter_mut().rev() {
            partial.finish_layout(
                layout_context,
                nesting_level,
                &mut self.inline_position,
                &mut self.line_had_any_content,
                self.line_had_any_absolutes,
                false, /* at_end_of_inline_element */
            );
            partial.start_corner.inline = Length::zero();
            partial.padding.inline_start = Length::zero();
            partial.border.inline_start = Length::zero();
            partial.margin.inline_start = Length::zero();
            partial.parent_nesting_level.inline_start = Length::zero();
            nesting_level = &mut partial.parent_nesting_level;
        }
        self.lines.finish_line(
            nesting_level,
            self.containing_block,
            self.sequential_layout_state.as_mut().map(|c| &mut **c),
            self.inline_position,
        );
        self.inline_position = Length::zero();
        self.line_had_any_content = false;
        self.line_had_any_absolutes = false;
    }

    /// Determine if we are in the final box of this inline formatting context.
    ///
    /// This is a big hack to trim the whitespace off the end of inline
    /// formatting contexts, that must stay in place until there is a
    /// better solution to use a temporary data structure to lay out
    /// lines.
    fn at_end_of_inline_formatting_context(&mut self) -> bool {
        let mut nesting_level = &mut self.current_nesting_level;
        if !nesting_level.remaining_boxes.at_end_of_iterator() {
            return false;
        }

        for partial in self.partial_inline_boxes_stack.iter_mut().rev() {
            nesting_level = &mut partial.parent_nesting_level;
            if !nesting_level.remaining_boxes.at_end_of_iterator() {
                return false;
            }
        }
        return true;
    }
}

struct Lines {
    // One anonymous fragment per line
    fragments: Vec<Fragment>,
    next_line_block_position: Length,
}

impl InlineFormattingContext {
    pub(super) fn new(
        text_decoration_line: TextDecorationLine,
        has_first_formatted_line: bool,
        ends_with_whitespace: bool,
    ) -> InlineFormattingContext {
        InlineFormattingContext {
            inline_level_boxes: Default::default(),
            text_decoration_line,
            has_first_formatted_line,
            contains_floats: false,
            ends_with_whitespace,
        }
    }

    // This works on an already-constructed `InlineFormattingContext`,
    // Which would have to change if/when
    // `BlockContainer::construct` parallelize their construction.
    pub(super) fn inline_content_sizes(
        &self,
        layout_context: &LayoutContext,
        containing_block_writing_mode: WritingMode,
    ) -> ContentSizes {
        struct Computation<'a> {
            layout_context: &'a LayoutContext<'a>,
            containing_block_writing_mode: WritingMode,
            paragraph: ContentSizes,
            current_line: ContentSizes,
            current_line_percentages: Percentage,
            /// Size for whitepsace pending to be added to this line.
            pending_whitespace: Length,
            /// Whether or not this IFC has seen any non-whitespace content.
            had_non_whitespace_content_yet: bool,
            /// The global linebreaking state.
            linebreaker: Option<LineBreakLeafIter>,
        }
        impl Computation<'_> {
            fn traverse(&mut self, inline_level_boxes: &[ArcRefCell<InlineLevelBox>]) {
                for inline_level_box in inline_level_boxes {
                    match &mut *inline_level_box.borrow_mut() {
                        InlineLevelBox::InlineBox(inline_box) => {
                            let padding =
                                inline_box.style.padding(self.containing_block_writing_mode);
                            let border = inline_box
                                .style
                                .border_width(self.containing_block_writing_mode);
                            let margin =
                                inline_box.style.margin(self.containing_block_writing_mode);
                            macro_rules! add {
                                ($condition: ident, $side: ident) => {
                                    if inline_box.$condition {
                                        self.add_lengthpercentage(padding.$side);
                                        self.add_length(border.$side);
                                        if let Some(lp) = margin.$side.non_auto() {
                                            self.add_lengthpercentage(lp)
                                        }
                                    }
                                };
                            }

                            add!(first_fragment, inline_start);
                            self.traverse(&inline_box.children);
                            add!(last_fragment, inline_end);
                        },
                        InlineLevelBox::TextRun(text_run) => {
                            let BreakAndShapeResult {
                                runs,
                                break_at_start,
                                ..
                            } = text_run
                                .break_and_shape(self.layout_context, &mut self.linebreaker);
                            if break_at_start {
                                self.line_break_opportunity()
                            }
                            for run in &runs {
                                let advance = Length::from(run.glyph_store.total_advance());

                                if !run.glyph_store.is_whitespace() {
                                    self.had_non_whitespace_content_yet = true;
                                    self.current_line.min_content += advance;
                                    self.current_line.max_content +=
                                        self.pending_whitespace + advance;
                                    self.pending_whitespace = Length::zero();
                                } else {
                                    // Discard any leading whitespace in the IFC. This will always be trimmed.
                                    if !self.had_non_whitespace_content_yet {
                                        continue;
                                    }

                                    // Wait to take into account other whitespace until we see more content.
                                    // Whitespace at the end of the IFC will always be trimmed.
                                    self.line_break_opportunity();
                                    self.pending_whitespace += advance;
                                }
                            }
                        },
                        InlineLevelBox::Atomic(atomic) => {
                            let (outer, pc) = atomic.outer_inline_content_sizes_and_percentages(
                                self.layout_context,
                                self.containing_block_writing_mode,
                            );

                            self.current_line.min_content +=
                                self.pending_whitespace + outer.min_content;
                            self.current_line.max_content += outer.max_content;
                            self.current_line_percentages += pc;
                            self.pending_whitespace = Length::zero();
                            self.had_non_whitespace_content_yet = true;
                        },
                        InlineLevelBox::OutOfFlowFloatBox(_) |
                        InlineLevelBox::OutOfFlowAbsolutelyPositionedBox(_) => {},
                    }
                }
            }

            fn add_lengthpercentage(&mut self, lp: &LengthPercentage) {
                if let Some(l) = lp.to_length() {
                    self.add_length(l);
                }
                if let Some(p) = lp.to_percentage() {
                    self.current_line_percentages += p;
                }
            }

            fn add_length(&mut self, l: Length) {
                self.current_line.min_content += l;
                self.current_line.max_content += l;
            }

            fn line_break_opportunity(&mut self) {
                self.paragraph
                    .min_content
                    .max_assign(take(&mut self.current_line.min_content));
            }

            fn forced_line_break(&mut self) {
                self.line_break_opportunity();
                self.current_line
                    .adjust_for_pbm_percentages(take(&mut self.current_line_percentages));
                self.paragraph
                    .max_content
                    .max_assign(take(&mut self.current_line.max_content));
            }
        }
        fn take<T: Zero>(x: &mut T) -> T {
            std::mem::replace(x, T::zero())
        }
        let mut computation = Computation {
            layout_context,
            containing_block_writing_mode,
            paragraph: ContentSizes::zero(),
            current_line: ContentSizes::zero(),
            current_line_percentages: Percentage::zero(),
            pending_whitespace: Length::zero(),
            had_non_whitespace_content_yet: false,
            linebreaker: None,
        };
        computation.traverse(&self.inline_level_boxes);
        computation.forced_line_break();
        computation.paragraph
    }

    pub(super) fn layout(
        &self,
        layout_context: &LayoutContext,
        positioning_context: &mut PositioningContext,
        containing_block: &ContainingBlock,
        sequential_layout_state: Option<&mut SequentialLayoutState>,
    ) -> FlowLayout {
        let mut ifc = InlineFormattingContextState {
            positioning_context,
            containing_block,
            partial_inline_boxes_stack: Vec::new(),
            lines: Lines {
                fragments: Vec::new(),
                next_line_block_position: Length::zero(),
            },
            inline_position: if self.has_first_formatted_line {
                containing_block
                    .style
                    .get_inherited_text()
                    .text_indent
                    .to_used_value(containing_block.inline_size.into())
                    .into()
            } else {
                Length::zero()
            },
            line_had_any_content: false,
            line_had_any_absolutes: false,
            linebreaker: None,
            current_nesting_level: InlineNestingLevelState {
                remaining_boxes: InlineBoxChildIter::from_formatting_context(self),
                fragments_so_far: Vec::with_capacity(self.inline_level_boxes.len()),
                inline_start: Length::zero(),
                max_block_size_of_fragments_so_far: Length::zero(),
                positioning_context: None,
                white_space: containing_block.style.clone_inherited_text().white_space,
                text_decoration_line: self.text_decoration_line,
            },
            sequential_layout_state,
        };

        // FIXME(pcwalton): This assumes that margins never collapse through inline formatting
        // contexts (i.e. that inline formatting contexts are never empty). Is that right?
        if let Some(ref mut sequential_layout_state) = ifc.sequential_layout_state {
            sequential_layout_state.collapse_margins();
        }

        loop {
            if let Some(child) = ifc.current_nesting_level.remaining_boxes.next() {
                match &mut *child.borrow_mut() {
                    InlineLevelBox::InlineBox(inline) => {
                        let partial = inline.start_layout(child.clone(), &mut ifc);
                        ifc.partial_inline_boxes_stack.push(partial)
                    },
                    InlineLevelBox::TextRun(run) => run.layout(layout_context, &mut ifc),
                    InlineLevelBox::Atomic(a) => layout_atomic(layout_context, &mut ifc, a),
                    InlineLevelBox::OutOfFlowAbsolutelyPositionedBox(box_) => {
                        let style = AtomicRef::map(box_.borrow(), |box_| box_.context.style());
                        let initial_start_corner =
                            match Display::from(style.get_box().original_display) {
                                Display::GeneratingBox(DisplayGeneratingBox::OutsideInside {
                                    outside,
                                    inside: _,
                                }) => Vec2 {
                                    inline: match outside {
                                        DisplayOutside::Inline => ifc.inline_position,
                                        DisplayOutside::Block => Length::zero(),
                                    },
                                    block: ifc.lines.next_line_block_position,
                                },
                                Display::Contents => {
                                    panic!("display:contents does not generate an abspos box")
                                },
                                Display::None => {
                                    panic!("display:none does not generate an abspos box")
                                },
                            };
                        let hoisted_box = AbsolutelyPositionedBox::to_hoisted(
                            box_.clone(),
                            initial_start_corner,
                            ifc.containing_block,
                        );
                        let hoisted_fragment = hoisted_box.fragment.clone();
                        ifc.push_hoisted_box_to_positioning_context(hoisted_box);
                        ifc.current_nesting_level
                            .fragments_so_far
                            .push(Fragment::AbsoluteOrFixedPositioned(hoisted_fragment));
                    },
                    InlineLevelBox::OutOfFlowFloatBox(float_box) => {
                        let mut box_fragment = float_box.layout(
                            layout_context,
                            ifc.positioning_context,
                            containing_block,
                        );

                        let state = ifc
                            .sequential_layout_state
                            .as_mut()
                            .expect("Tried to lay out a float with no sequential placement state!");

                        let block_offset_from_containining_block_top = state
                            .current_block_position_including_margins() -
                            state.current_containing_block_offset();
                        state.place_float_fragment(
                            &mut box_fragment,
                            CollapsedMargin::zero(),
                            block_offset_from_containining_block_top,
                        );

                        ifc.current_nesting_level
                            .fragments_so_far
                            .push(Fragment::Float(box_fragment));
                    },
                }
            } else if let Some(mut partial) = ifc.partial_inline_boxes_stack.pop() {
                // We reached the end of the remaining boxes in this nesting level, so we finish it and
                // start working on the parent nesting level again.
                partial.finish_layout(
                    layout_context,
                    &mut ifc.current_nesting_level,
                    &mut ifc.inline_position,
                    &mut ifc.line_had_any_content,
                    ifc.line_had_any_absolutes,
                    true, /* at_end_of_inline_element */
                );
                ifc.current_nesting_level = partial.parent_nesting_level
            } else {
                // We reached the end of the entire IFC.
                break;
            }
        }

        ifc.lines.finish_line(
            &mut ifc.current_nesting_level,
            containing_block,
            ifc.sequential_layout_state,
            ifc.inline_position,
        );

        let mut collapsible_margins_in_children = CollapsedBlockMargins::zero();
        let content_block_size = ifc.lines.next_line_block_position;
        if content_block_size == Length::zero() {
            collapsible_margins_in_children.collapsed_through = true;
        }

        return FlowLayout {
            fragments: ifc.lines.fragments,
            content_block_size,
            collapsible_margins_in_children,
        };
    }

    /// Return true if this [InlineFormattingContext] is empty for the purposes of ignoring
    /// during box tree construction. An IFC is empty if it only contains TextRuns with
    /// completely collapsible whitespace. When that happens it can be ignored completely.
    pub fn is_empty(&self) -> bool {
        fn inline_level_boxes_are_empty(boxes: &[ArcRefCell<InlineLevelBox>]) -> bool {
            boxes
                .iter()
                .all(|inline_level_box| inline_level_box_is_empty(&*inline_level_box.borrow()))
        }

        fn inline_level_box_is_empty(inline_level_box: &InlineLevelBox) -> bool {
            match inline_level_box {
                InlineLevelBox::InlineBox(_) => false,
                InlineLevelBox::TextRun(text_run) => !text_run.has_uncollapsible_content,
                InlineLevelBox::OutOfFlowAbsolutelyPositionedBox(_) => false,
                InlineLevelBox::OutOfFlowFloatBox(_) => false,
                InlineLevelBox::Atomic(_) => false,
            }
        }

        inline_level_boxes_are_empty(&self.inline_level_boxes)
    }
}

impl Lines {
    fn finish_line(
        &mut self,
        top_nesting_level: &mut InlineNestingLevelState,
        containing_block: &ContainingBlock,
        mut sequential_layout_state: Option<&mut SequentialLayoutState>,
        line_content_inline_size: Length,
    ) {
        let mut line_contents = std::mem::take(&mut top_nesting_level.fragments_so_far);
        let line_block_size = std::mem::replace(
            &mut top_nesting_level.max_block_size_of_fragments_so_far,
            Length::zero(),
        );
        enum TextAlign {
            Start,
            Center,
            End,
        }
        let line_left_is_inline_start = containing_block
            .style
            .writing_mode
            .line_left_is_inline_start();
        let text_align = match containing_block.style.clone_text_align() {
            TextAlignKeyword::Start => TextAlign::Start,
            TextAlignKeyword::Center => TextAlign::Center,
            TextAlignKeyword::End => TextAlign::End,
            TextAlignKeyword::Left => {
                if line_left_is_inline_start {
                    TextAlign::Start
                } else {
                    TextAlign::End
                }
            },
            TextAlignKeyword::Right => {
                if line_left_is_inline_start {
                    TextAlign::End
                } else {
                    TextAlign::Start
                }
            },
            TextAlignKeyword::Justify => {
                // TODO: Add support for justfied text.
                TextAlign::Start
            },
            TextAlignKeyword::ServoCenter |
            TextAlignKeyword::ServoLeft |
            TextAlignKeyword::ServoRight => {
                // TODO: Implement these modes which seem to be used by quirks mode.
                TextAlign::Start
            },
        };
        let move_by = match text_align {
            TextAlign::Start => Length::zero(),
            TextAlign::Center => (containing_block.inline_size - line_content_inline_size) / 2.,
            TextAlign::End => containing_block.inline_size - line_content_inline_size,
        };
        if move_by > Length::zero() {
            for fragment in &mut line_contents {
                fragment.offset_inline(&move_by);
            }
        }
        let start_corner = Vec2 {
            inline: Length::zero(),
            block: self.next_line_block_position,
        };
        let size = Vec2 {
            inline: containing_block.inline_size,
            block: line_block_size,
        };

        self.next_line_block_position += size.block;
        if let Some(ref mut sequential_layout_state) = sequential_layout_state {
            sequential_layout_state.advance_block_position(size.block);
        }

        if !line_contents.is_empty() {
            self.fragments
                .push(Fragment::Anonymous(AnonymousFragment::new(
                    Rect { start_corner, size },
                    line_contents,
                    containing_block.style.writing_mode,
                )));
        }
    }
}

impl InlineBox {
    fn start_layout<'box_tree>(
        &self,
        this_inline_level_box: ArcRefCell<InlineLevelBox>,
        ifc: &mut InlineFormattingContextState<'box_tree, '_, '_>,
    ) -> PartialInlineBoxFragment<'box_tree> {
        let style = self.style.clone();
        let pbm = style.padding_border_margin(&ifc.containing_block);
        let mut padding = pbm.padding;
        let mut border = pbm.border;
        let mut margin = pbm.margin.auto_is(Length::zero);

        if self.first_fragment {
            ifc.inline_position += padding.inline_start + border.inline_start + margin.inline_start;
        } else {
            padding.inline_start = Length::zero();
            border.inline_start = Length::zero();
            margin.inline_start = Length::zero();
        }

        let mut start_corner = Vec2 {
            block: Length::zero(),
            inline: ifc.inline_position - ifc.current_nesting_level.inline_start,
        };
        if style.clone_position().is_relative() {
            start_corner += &relative_adjustement(&style, ifc.containing_block)
        }
        let positioning_context = PositioningContext::new_for_style(&style);
        let white_space = style.clone_inherited_text().white_space;
        let text_decoration_line =
            ifc.current_nesting_level.text_decoration_line | style.clone_text_decoration_line();
        PartialInlineBoxFragment {
            base_fragment_info: self.base_fragment_info,
            style,
            start_corner,
            padding,
            border,
            margin,
            was_part_of_previous_line: false,
            parent_nesting_level: std::mem::replace(
                &mut ifc.current_nesting_level,
                InlineNestingLevelState {
                    remaining_boxes: InlineBoxChildIter::from_inline_level_box(
                        this_inline_level_box,
                    ),
                    fragments_so_far: Vec::with_capacity(self.children.len()),
                    inline_start: ifc.inline_position,
                    max_block_size_of_fragments_so_far: Length::zero(),
                    positioning_context,
                    white_space,
                    text_decoration_line: text_decoration_line,
                },
            ),
        }
    }
}

impl<'box_tree> PartialInlineBoxFragment<'box_tree> {
    fn finish_layout(
        &mut self,
        layout_context: &LayoutContext,
        nesting_level: &mut InlineNestingLevelState,
        inline_position: &mut Length,
        line_had_any_content: &mut bool,
        line_had_any_absolutes: bool,
        at_end_of_inline_element: bool,
    ) {
        let mut padding = self.padding.clone();
        let mut border = self.border.clone();
        let mut margin = self.margin.clone();

        if padding.inline_sum() > Length::zero() ||
            border.inline_sum() > Length::zero() ||
            margin.inline_sum() > Length::zero()
        {
            *line_had_any_content = true;
        }

        if !*line_had_any_content && !line_had_any_absolutes && self.was_part_of_previous_line {
            return;
        }
        *line_had_any_content = true;

        // If we are finishing in order to fragment this InlineBox into multiple lines, do
        // not add end margins, borders, and padding.
        if !at_end_of_inline_element {
            padding.inline_end = Length::zero();
            border.inline_end = Length::zero();
            margin.inline_end = Length::zero();
        }

        // TODO(mrobinson): `inline_position` is relative to the IFC, but `self.start_corner` is relative
        // to the containing block, which means that this size will be incorrect with multiple levels
        // of nesting.
        let content_rect = Rect {
            size: Vec2 {
                inline: *inline_position - self.start_corner.inline,
                block: nesting_level.max_block_size_of_fragments_so_far,
            },
            start_corner: self.start_corner.clone(),
        };

        self.parent_nesting_level
            .max_block_size_of_fragments_so_far
            .max_assign(content_rect.size.block);

        *inline_position += padding.inline_end + border.inline_end + margin.inline_end;

        let mut fragment = BoxFragment::new(
            self.base_fragment_info,
            self.style.clone(),
            std::mem::take(&mut nesting_level.fragments_so_far),
            content_rect,
            padding,
            border,
            margin,
            None,
            CollapsedBlockMargins::zero(),
        );
        if let Some(context) = nesting_level.positioning_context.as_mut() {
            context.layout_collected_children(layout_context, &mut fragment);
        }

        self.was_part_of_previous_line = true;
        self.parent_nesting_level
            .fragments_so_far
            .push(Fragment::Box(fragment));
    }
}

fn layout_atomic(
    layout_context: &LayoutContext,
    ifc: &mut InlineFormattingContextState,
    atomic: &mut IndependentFormattingContext,
) {
    let style = atomic.style();
    let pbm = style.padding_border_margin(&ifc.containing_block);
    let margin = pbm.margin.auto_is(Length::zero);
    let pbm_sums = &(&pbm.padding + &pbm.border) + &margin;
    let position = style.clone_position();

    let mut child_positioning_context = None;

    // We need to know the inline size of the atomic before deciding whether to do the line break.
    let mut fragment = match atomic {
        IndependentFormattingContext::Replaced(replaced) => {
            let size = replaced.contents.used_size_as_if_inline_element(
                ifc.containing_block,
                &replaced.style,
                None,
                &pbm,
            );
            let fragments = replaced
                .contents
                .make_fragments(&replaced.style, size.clone());
            let content_rect = Rect {
                start_corner: Vec2::zero(),
                size,
            };
            BoxFragment::new(
                replaced.base_fragment_info,
                replaced.style.clone(),
                fragments,
                content_rect,
                pbm.padding,
                pbm.border,
                margin,
                None,
                CollapsedBlockMargins::zero(),
            )
        },
        IndependentFormattingContext::NonReplaced(non_replaced) => {
            let box_size = non_replaced
                .style
                .content_box_size(&ifc.containing_block, &pbm);
            let max_box_size = non_replaced
                .style
                .content_max_box_size(&ifc.containing_block, &pbm);
            let min_box_size = non_replaced
                .style
                .content_min_box_size(&ifc.containing_block, &pbm)
                .auto_is(Length::zero);

            // https://drafts.csswg.org/css2/visudet.html#inlineblock-width
            let tentative_inline_size = box_size.inline.auto_is(|| {
                let available_size = ifc.containing_block.inline_size - pbm_sums.inline_sum();
                non_replaced
                    .inline_content_sizes(layout_context)
                    .shrink_to_fit(available_size)
            });

            // https://drafts.csswg.org/css2/visudet.html#min-max-widths
            // In this case “applying the rules above again” with a non-auto inline-size
            // always results in that size.
            let inline_size = tentative_inline_size
                .clamp_between_extremums(min_box_size.inline, max_box_size.inline);

            let containing_block_for_children = ContainingBlock {
                inline_size,
                block_size: box_size.block,
                style: &non_replaced.style,
            };
            assert_eq!(
                ifc.containing_block.style.writing_mode,
                containing_block_for_children.style.writing_mode,
                "Mixed writing modes are not supported yet"
            );

            let collects_for_nearest_positioned_ancestor = ifc
                .positioning_context
                .collects_for_nearest_positioned_ancestor();
            child_positioning_context = Some(PositioningContext::new_for_subtree(
                collects_for_nearest_positioned_ancestor,
            ));
            let independent_layout = non_replaced.layout(
                layout_context,
                child_positioning_context.as_mut().unwrap(),
                &containing_block_for_children,
            );

            // https://drafts.csswg.org/css2/visudet.html#block-root-margin
            let tentative_block_size = box_size
                .block
                .auto_is(|| independent_layout.content_block_size);

            // https://drafts.csswg.org/css2/visudet.html#min-max-heights
            // In this case “applying the rules above again” with a non-auto block-size
            // always results in that size.
            let block_size = tentative_block_size
                .clamp_between_extremums(min_box_size.block, max_box_size.block);

            let content_rect = Rect {
                start_corner: Vec2::zero(),
                size: Vec2 {
                    block: block_size,
                    inline: inline_size,
                },
            };

            BoxFragment::new(
                non_replaced.base_fragment_info,
                non_replaced.style.clone(),
                independent_layout.fragments,
                content_rect,
                pbm.padding,
                pbm.border,
                margin,
                None,
                CollapsedBlockMargins::zero(),
            )
        },
    };

    if fragment.content_rect.size.inline + pbm_sums.inline_sum() >
        ifc.containing_block.inline_size - ifc.inline_position &&
        ifc.current_nesting_level.white_space.allow_wrap() &&
        ifc.current_nesting_level.fragments_so_far.len() != 0
    {
        ifc.finish_line_and_reset(layout_context);
    }

    ifc.inline_position += pbm_sums.inline_start;
    let mut start_corner = Vec2 {
        block: pbm_sums.block_start,
        inline: ifc.inline_position - ifc.current_nesting_level.inline_start,
    };
    if position.is_relative() {
        start_corner += &relative_adjustement(atomic.style(), ifc.containing_block)
    }

    if let Some(mut child_positioning_context) = child_positioning_context.take() {
        child_positioning_context
            .adjust_static_position_of_hoisted_fragments_with_offset(&start_corner);
        ifc.positioning_context.append(child_positioning_context);
    }

    fragment.content_rect.start_corner = start_corner;

    ifc.line_had_any_content = true;
    ifc.inline_position += pbm_sums.inline_end + fragment.content_rect.size.inline;
    ifc.current_nesting_level
        .max_block_size_of_fragments_so_far
        .max_assign(pbm_sums.block_sum() + fragment.content_rect.size.block);
    ifc.current_nesting_level
        .fragments_so_far
        .push(Fragment::Box(fragment));

    // After every atomic, we need to create a line breaking opportunity for the next TextRun.
    if let Some(linebreaker) = ifc.linebreaker.as_mut() {
        linebreaker.next(" ");
    }
}

struct BreakAndShapeResult {
    font_metrics: FontMetrics,
    font_key: FontInstanceKey,
    runs: Vec<GlyphRun>,
    break_at_start: bool,
}

impl TextRun {
    fn break_and_shape(
        &self,
        layout_context: &LayoutContext,
        linebreaker: &mut Option<LineBreakLeafIter>,
    ) -> BreakAndShapeResult {
        use gfx::font::ShapingFlags;
        use style::computed_values::text_rendering::T as TextRendering;
        use style::computed_values::word_break::T as WordBreak;

        let font_style = self.parent_style.clone_font();
        let inherited_text_style = self.parent_style.get_inherited_text();
        let letter_spacing = if inherited_text_style.letter_spacing.0.px() != 0. {
            Some(app_units::Au::from(inherited_text_style.letter_spacing.0))
        } else {
            None
        };

        let mut flags = ShapingFlags::empty();
        if letter_spacing.is_some() {
            flags.insert(ShapingFlags::IGNORE_LIGATURES_SHAPING_FLAG);
        }
        if inherited_text_style.text_rendering == TextRendering::Optimizespeed {
            flags.insert(ShapingFlags::IGNORE_LIGATURES_SHAPING_FLAG);
            flags.insert(ShapingFlags::DISABLE_KERNING_SHAPING_FLAG)
        }
        if inherited_text_style.word_break == WordBreak::KeepAll {
            flags.insert(ShapingFlags::KEEP_ALL_FLAG);
        }

        crate::context::with_thread_local_font_context(layout_context, |font_context| {
            let font_group = font_context.font_group(font_style);
            let font = font_group
                .borrow_mut()
                .first(font_context)
                .expect("could not find font");
            let mut font = font.borrow_mut();

            let word_spacing = &inherited_text_style.word_spacing;
            let word_spacing = word_spacing
                .to_length()
                .map(|l| l.into())
                .unwrap_or_else(|| {
                    let space_width = font
                        .glyph_index(' ')
                        .map(|glyph_id| font.glyph_h_advance(glyph_id))
                        .unwrap_or(gfx::font::LAST_RESORT_GLYPH_ADVANCE);
                    word_spacing.to_used_value(Au::from_f64_px(space_width))
                });

            let shaping_options = gfx::font::ShapingOptions {
                letter_spacing,
                word_spacing,
                script: unicode_script::Script::Common,
                flags,
            };

            let (runs, break_at_start) = gfx::text::text_run::TextRun::break_and_shape(
                &mut font,
                &self.text,
                &shaping_options,
                linebreaker,
            );

            BreakAndShapeResult {
                font_metrics: (&font.metrics).into(),
                font_key: font.font_key,
                runs,
                break_at_start,
            }
        })
    }

    fn layout(&self, layout_context: &LayoutContext, ifc: &mut InlineFormattingContextState) {
        let white_space = self.parent_style.get_inherited_text().white_space;
        let preserving_newlines = white_space.preserve_newlines();
        let preserving_spaces = white_space.preserve_spaces();
        let last_box_in_ifc = ifc.at_end_of_inline_formatting_context();

        let BreakAndShapeResult {
            font_metrics,
            font_key,
            runs,
            break_at_start,
        } = self.break_and_shape(layout_context, &mut ifc.linebreaker);

        let mut glyphs = vec![];
        let mut inline_advance = Length::zero();
        let mut pending_whitespace = None;

        let mut iterator = runs.iter().enumerate().peekable();
        while let Some((run_index, run)) = iterator.next() {
            if run.glyph_store.is_whitespace() {
                // If this whitespace forces a line break, finish the line and reset everything.
                let last_byte = self.text.as_bytes().get(run.range.end().to_usize() - 1);
                if last_byte == Some(&b'\n') && preserving_newlines {
                    ifc.line_had_any_content = true;
                    self.add_fragment_for_glyphs(
                        ifc,
                        glyphs.drain(..).collect(),
                        inline_advance,
                        font_metrics,
                        font_key,
                    );
                    ifc.finish_line_and_reset(layout_context);
                    inline_advance = Length::zero();
                    continue;
                }

                if !preserving_spaces {
                    // From <https://www.w3.org/TR/css-text-3/#white-space-phase-2>:
                    // "Then, the entire block is rendered. Inlines are laid out, taking bidi
                    // reordering into account, and wrapping as specified by the text-wrap
                    // property. As each line is laid out,
                    //
                    // > 1. A sequence of collapsible spaces at the beginning of a line is removed.
                    if !ifc.line_had_any_content {
                        continue;
                    }

                    // > 3. A sequence of collapsible spaces at the end of a line is removed,
                    // >    as well as any trailing U+1680   OGHAM SPACE MARK whose white-space
                    // >    property is normal, nowrap, or pre-line.
                    // Try to trim whitespace at the end of lines. This is a hack. Ideally we
                    // would keep a temporary data structure for a line and lay it out once we
                    // know that we are going to make an entire one.
                    if iterator.peek().is_none() && last_box_in_ifc {
                        pending_whitespace = None;
                        continue;
                    }

                    // Don't push a space until we know we aren't going to line break in the
                    // next run.
                    pending_whitespace = Some(run);
                    continue;
                }
            }

            let advance_from_pending_whitespace = pending_whitespace
                .map_or_else(Length::zero, |run| {
                    Length::from(run.glyph_store.total_advance())
                });

            // We break the line if this new advance and any advances from pending
            // whitespace bring us past the inline end of the containing block.
            let new_advance =
                Length::from(run.glyph_store.total_advance()) + advance_from_pending_whitespace;
            let will_advance_past_containing_block =
                (new_advance + inline_advance + ifc.inline_position) >
                    ifc.containing_block.inline_size;

            // We can only break the line, if this isn't the first actual content (non-whitespace or
            // preserved whitespace) on the line and this isn't the unbreakable run of this text run
            // (or we can break at the start according to the text breaker).
            let can_break = ifc.line_had_any_content && (break_at_start || run_index != 0);
            if will_advance_past_containing_block && can_break {
                self.add_fragment_for_glyphs(
                    ifc,
                    glyphs.drain(..).collect(),
                    inline_advance,
                    font_metrics,
                    font_key,
                );

                pending_whitespace = None;
                ifc.finish_line_and_reset(layout_context);
                inline_advance = Length::zero();
            }

            if let Some(pending_whitespace) = pending_whitespace.take() {
                inline_advance += Length::from(pending_whitespace.glyph_store.total_advance());
                glyphs.push(pending_whitespace.glyph_store.clone());
            }

            inline_advance += Length::from(run.glyph_store.total_advance());
            glyphs.push(run.glyph_store.clone());
            ifc.line_had_any_content = true;
        }

        if let Some(pending_whitespace) = pending_whitespace.take() {
            inline_advance += Length::from(pending_whitespace.glyph_store.total_advance());
            glyphs.push(pending_whitespace.glyph_store.clone());
        }

        self.add_fragment_for_glyphs(
            ifc,
            glyphs.drain(..).collect(),
            inline_advance,
            font_metrics,
            font_key,
        );
    }

    fn add_fragment_for_glyphs(
        &self,
        ifc: &mut InlineFormattingContextState,
        glyphs: Vec<std::sync::Arc<GlyphStore>>,
        inline_advance: Length,
        font_metrics: FontMetrics,
        font_key: FontInstanceKey,
    ) {
        if glyphs.is_empty() {
            return;
        }

        let font_size = self.parent_style.get_font().font_size.size.0;
        let line_height = match self.parent_style.get_inherited_text().line_height {
            LineHeight::Normal => font_metrics.line_gap,
            LineHeight::Number(n) => font_size * n.0,
            LineHeight::Length(l) => l.0,
        };

        let rect = Rect {
            start_corner: Vec2 {
                block: Length::zero(),
                inline: ifc.inline_position - ifc.current_nesting_level.inline_start,
            },
            size: Vec2 {
                block: line_height,
                inline: inline_advance,
            },
        };

        ifc.inline_position += inline_advance;
        ifc.current_nesting_level
            .max_block_size_of_fragments_so_far
            .max_assign(line_height);
        ifc.current_nesting_level
            .fragments_so_far
            .push(Fragment::Text(TextFragment {
                base: self.base_fragment_info.into(),
                parent_style: self.parent_style.clone(),
                rect,
                font_metrics,
                font_key,
                glyphs,
                text_decoration_line: ifc.current_nesting_level.text_decoration_line,
            }));
    }
}

enum InlineBoxChildIter<'box_tree> {
    InlineFormattingContext(
        std::iter::Peekable<std::slice::Iter<'box_tree, ArcRefCell<InlineLevelBox>>>,
    ),
    InlineBox {
        inline_level_box: ArcRefCell<InlineLevelBox>,
        child_index: usize,
    },
}

impl<'box_tree> InlineBoxChildIter<'box_tree> {
    fn from_formatting_context(
        inline_formatting_context: &'box_tree InlineFormattingContext,
    ) -> InlineBoxChildIter<'box_tree> {
        InlineBoxChildIter::InlineFormattingContext(
            inline_formatting_context
                .inline_level_boxes
                .iter()
                .peekable(),
        )
    }

    fn from_inline_level_box(
        inline_level_box: ArcRefCell<InlineLevelBox>,
    ) -> InlineBoxChildIter<'box_tree> {
        InlineBoxChildIter::InlineBox {
            inline_level_box,
            child_index: 0,
        }
    }

    fn at_end_of_iterator(&mut self) -> bool {
        match *self {
            InlineBoxChildIter::InlineFormattingContext(ref mut iter) => iter.peek().is_none(),
            InlineBoxChildIter::InlineBox {
                ref inline_level_box,
                ref child_index,
            } => match *inline_level_box.borrow() {
                InlineLevelBox::InlineBox(ref inline_box) => {
                    *child_index >= inline_box.children.len()
                },
                _ => unreachable!(),
            },
        }
    }
}

impl<'box_tree> Iterator for InlineBoxChildIter<'box_tree> {
    type Item = ArcRefCell<InlineLevelBox>;
    fn next(&mut self) -> Option<ArcRefCell<InlineLevelBox>> {
        match *self {
            InlineBoxChildIter::InlineFormattingContext(ref mut iter) => iter.next().cloned(),
            InlineBoxChildIter::InlineBox {
                ref inline_level_box,
                ref mut child_index,
            } => match *inline_level_box.borrow() {
                InlineLevelBox::InlineBox(ref inline_box) => {
                    if *child_index >= inline_box.children.len() {
                        return None;
                    }

                    let kid = inline_box.children[*child_index].clone();
                    *child_index += 1;
                    Some(kid)
                },
                _ => unreachable!(),
            },
        }
    }
}
