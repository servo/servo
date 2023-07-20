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
    FontMetrics, Fragment, HoistedSharedFragment, TextFragment,
};
use crate::geom::flow_relative::{Rect, Vec2};
use crate::geom::LengthOrAuto;
use crate::positioned::{
    relative_adjustement, AbsolutelyPositionedBox, PositioningContext, PositioningContextLength,
};
use crate::sizing::ContentSizes;
use crate::style_ext::{
    ComputedValuesExt, Display, DisplayGeneratingBox, DisplayOutside, PaddingBorderMargin,
};
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

use super::CollapsibleWithParentStartMargin;
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
    line_items_so_far: Vec<LineItem>,
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
    pbm: PaddingBorderMargin,

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
    sequential_layout_state: Option<&'a mut SequentialLayoutState>,

    /// A vector of fragment that are laid out. This includes one [`Fragment::Anonymous`]
    /// per line that is currently laid out plus fragments for all floats, which
    /// are currently laid out at the top-level of each [`InlineFormattingContext`].
    fragments: Vec<Fragment>,

    /// The position of where the next line will start.
    next_line_start_position: Vec2<Length>,

    /// The current inline position in the line being laid out into [`LineItems`] in this
    /// [`InlineFormattingContext`] independent of the depth in the nesting level.
    current_inline_position: Length,

    /// Whether any active line box has added a glyph, border, margin, or padding
    /// to this line, which indicates that the next run that exceeds the line length
    /// can cause a line break.
    line_had_any_content: bool,

    /// The line breaking state for this inline formatting context.
    linebreaker: Option<LineBreakLeafIter>,

    partial_inline_boxes_stack: Vec<PartialInlineBoxFragment<'box_tree>>,
    current_nesting_level: InlineNestingLevelState<'box_tree>,
}

impl<'box_tree, 'a, 'b> InlineFormattingContextState<'box_tree, 'a, 'b> {
    /// Push a completed [LineItem] to the current nesteding level of this
    /// [InlineFormattingContext].
    fn push_line_item(&mut self, inline_size: Length, line_item: LineItem) {
        self.current_nesting_level.line_items_so_far.push(line_item);
        self.line_had_any_content = true;
        self.current_inline_position += inline_size;
    }

    /// Finish layout of all the partial inline boxes in the current line,
    /// finish current line and start a new one.
    fn finish_line_and_reset(&mut self, layout_context: &LayoutContext) {
        let mut nesting_level = &mut self.current_nesting_level;
        for partial in self.partial_inline_boxes_stack.iter_mut().rev() {
            partial.finish_layout(
                nesting_level,
                &mut self.current_inline_position,
                false, /* at_end_of_inline_element */
            );
            nesting_level = &mut partial.parent_nesting_level;
        }

        let line_items = std::mem::take(&mut nesting_level.line_items_so_far);
        self.finish_current_line(layout_context, line_items, self.containing_block);

        self.current_inline_position = Length::zero();
        self.line_had_any_content = false;
    }

    fn finish_current_line(
        &mut self,
        layout_context: &LayoutContext,
        mut line_items: Vec<LineItem>,
        containing_block: &ContainingBlock,
    ) {
        let sequential_layout_state = self.sequential_layout_state.as_mut().map(|c| &mut **c);

        // From <https://www.w3.org/TR/css-text-3/#white-space-phase-2>:
        // > 3. A sequence of collapsible spaces at the end of a line is removed,
        // >    as well as any trailing U+1680   OGHAM SPACE MARK whose white-space
        // >    property is normal, nowrap, or pre-line.
        for item in line_items.iter_mut().rev() {
            if !item.trim_whitespace_at_end() {
                break;
            }
        }

        let mut state = LineItemLayoutState {
            inline_position: self.next_line_start_position.inline,
            max_block_size: Length::zero(),
            containing_block_inline_start: self.next_line_start_position.inline,
            ifc_containing_block: containing_block,
            positioning_context: &mut self.positioning_context,
        };

        let positioning_context_length = state.positioning_context.len();
        let mut fragments = layout_line_items(line_items, layout_context, &mut state);

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
            TextAlign::Center => (containing_block.inline_size - state.inline_position) / 2.,
            TextAlign::End => containing_block.inline_size - state.inline_position,
        };
        if move_by > Length::zero() {
            for fragment in &mut fragments {
                fragment.offset_inline(&move_by);
            }
        }

        let size = Vec2 {
            inline: containing_block.inline_size,
            block: state.max_block_size,
        };

        let start_corner = self.next_line_start_position.clone();
        self.next_line_start_position = Vec2 {
            inline: Length::zero(),
            block: self.next_line_start_position.block + size.block,
        };

        if let Some(sequential_layout_state) = sequential_layout_state {
            sequential_layout_state.advance_block_position(size.block);
        }

        let line_had_content =
            !fragments.is_empty() || state.positioning_context.len() != positioning_context_length;
        if line_had_content {
            state
                .positioning_context
                .adjust_static_position_of_hoisted_fragments_with_offset(
                    &start_corner,
                    positioning_context_length,
                );

            self.fragments
                .push(Fragment::Anonymous(AnonymousFragment::new(
                    Rect { start_corner, size },
                    fragments,
                    containing_block.style.writing_mode,
                )));
        }
    }
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
        collapsible_with_parent_start_margin: CollapsibleWithParentStartMargin,
    ) -> FlowLayout {
        let first_line_inline_start = if self.has_first_formatted_line {
            containing_block
                .style
                .get_inherited_text()
                .text_indent
                .to_used_value(containing_block.inline_size.into())
                .into()
        } else {
            Length::zero()
        };

        let mut ifc = InlineFormattingContextState {
            positioning_context,
            containing_block,
            sequential_layout_state,
            fragments: Vec::new(),
            next_line_start_position: Vec2 {
                inline: first_line_inline_start,
                block: Length::zero(),
            },
            current_inline_position: first_line_inline_start,
            line_had_any_content: false,
            linebreaker: None,
            partial_inline_boxes_stack: Vec::new(),
            current_nesting_level: InlineNestingLevelState {
                remaining_boxes: InlineBoxChildIter::from_formatting_context(self),
                line_items_so_far: Vec::with_capacity(self.inline_level_boxes.len()),
                white_space: containing_block.style.clone_inherited_text().white_space,
                text_decoration_line: self.text_decoration_line,
            },
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
                        let partial =
                            PartialInlineBoxFragment::new(inline, child.clone(), &mut ifc);
                        ifc.partial_inline_boxes_stack.push(partial);
                    },
                    InlineLevelBox::TextRun(run) => {
                        run.layout_into_line_items(layout_context, &mut ifc)
                    },
                    InlineLevelBox::Atomic(atomic_formatting_context) => {
                        atomic_formatting_context.layout_into_line_items(layout_context, &mut ifc);
                    },
                    InlineLevelBox::OutOfFlowAbsolutelyPositionedBox(box_) => {
                        ifc.current_nesting_level.line_items_so_far.push(
                            LineItem::AbsolutelyPositioned(AbsolutelyPositionedLineItem {
                                absolutely_positioned_box: box_.clone(),
                            }),
                        );
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
                        ifc.fragments.push(Fragment::Float(box_fragment));
                    },
                }
            } else if let Some(mut partial) = ifc.partial_inline_boxes_stack.pop() {
                // We reached the end of the remaining boxes in this nesting level, so we finish it and
                // start working on the parent nesting level again.
                partial.finish_layout(
                    &mut ifc.current_nesting_level,
                    &mut ifc.current_inline_position,
                    true, /* at_end_of_inline_element */
                );
                ifc.current_nesting_level = partial.parent_nesting_level
            } else {
                // We reached the end of the entire IFC.
                break;
            }
        }

        let line_items = std::mem::take(&mut ifc.current_nesting_level.line_items_so_far);
        ifc.finish_current_line(layout_context, line_items, containing_block);

        let mut collapsible_margins_in_children = CollapsedBlockMargins::zero();
        let content_block_size = ifc.next_line_start_position.block;
        collapsible_margins_in_children.collapsed_through =
            content_block_size == Length::zero() && collapsible_with_parent_start_margin.0;

        return FlowLayout {
            fragments: ifc.fragments,
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

impl<'box_tree> PartialInlineBoxFragment<'box_tree> {
    fn new(
        inline_box: &InlineBox,
        this_inline_level_box: ArcRefCell<InlineLevelBox>,
        ifc: &mut InlineFormattingContextState<'box_tree, '_, '_>,
    ) -> PartialInlineBoxFragment<'box_tree> {
        let style = inline_box.style.clone();
        let mut pbm = style.padding_border_margin(&ifc.containing_block);

        if inline_box.first_fragment {
            ifc.current_inline_position += pbm.padding.inline_start +
                pbm.border.inline_start +
                pbm.margin.inline_start.auto_is(Length::zero)
        } else {
            pbm.padding.inline_start = Length::zero();
            pbm.border.inline_start = Length::zero();
            pbm.margin.inline_start = LengthOrAuto::zero();
        }

        let text_decoration_line =
            ifc.current_nesting_level.text_decoration_line | style.clone_text_decoration_line();
        let white_space = style.clone_inherited_text().white_space;

        PartialInlineBoxFragment {
            base_fragment_info: inline_box.base_fragment_info,
            style,
            pbm,
            was_part_of_previous_line: false,
            parent_nesting_level: std::mem::replace(
                &mut ifc.current_nesting_level,
                InlineNestingLevelState {
                    remaining_boxes: InlineBoxChildIter::from_inline_level_box(
                        this_inline_level_box,
                    ),
                    line_items_so_far: Vec::with_capacity(inline_box.children.len()),
                    white_space,
                    text_decoration_line: text_decoration_line,
                },
            ),
        }
    }

    fn finish_layout(
        &mut self,
        nesting_level: &mut InlineNestingLevelState,
        inline_position: &mut Length,
        at_end_of_inline_element: bool,
    ) {
        // If we are finishing in order to fragment this InlineBox into multiple lines, do
        // not add end margins, borders, and padding.
        if !at_end_of_inline_element {
            self.pbm.padding.inline_end = Length::zero();
            self.pbm.border.inline_end = Length::zero();
            self.pbm.margin.inline_end = LengthOrAuto::zero();
        } else {
            *inline_position += self.pbm.padding.inline_end +
                self.pbm.border.inline_end +
                self.pbm.margin.inline_end.auto_is(Length::zero)
        }

        self.parent_nesting_level
            .line_items_so_far
            .push(LineItem::InlineBox(InlineBoxLineItem {
                base_fragment_info: self.base_fragment_info,
                style: self.style.clone(),
                pbm: self.pbm.clone(),
                children: std::mem::take(&mut nesting_level.line_items_so_far),
                always_make_fragment: !self.was_part_of_previous_line,
            }));

        // This InlineBox now has at least one Fragment that corresponds to it, so
        // if subsequent lines can ignore it if it is empty on those lines.
        self.was_part_of_previous_line = true;

        // If this partial / inline box appears on any subsequent lines, it should not
        // have any start margin, border, or padding.
        self.pbm.padding.inline_start = Length::zero();
        self.pbm.border.inline_start = Length::zero();
        self.pbm.margin.inline_start = LengthOrAuto::zero();
    }
}

impl IndependentFormattingContext {
    fn layout_into_line_items(
        &mut self,
        layout_context: &LayoutContext,
        ifc: &mut InlineFormattingContextState,
    ) {
        let style = self.style();
        let pbm = style.padding_border_margin(&ifc.containing_block);
        let margin = pbm.margin.auto_is(Length::zero);
        let pbm_sums = &(&pbm.padding + &pbm.border) + &margin;
        let mut child_positioning_context = None;

        // We need to know the inline size of the atomic before deciding whether to do the line break.
        let fragment = match self {
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
                    start_corner: pbm_sums.start_offset(),
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

                // This always collects for the nearest positioned ancestor even if the parent positioning
                // context doesn't. The thing is we haven't kept track up to this point and there isn't
                // any harm in keeping the hoisted boxes separate.
                child_positioning_context = Some(PositioningContext::new_for_subtree(
                    true, /* collects_for_nearest_positioned_ancestor */
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
                    start_corner: pbm_sums.start_offset(),
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
            ifc.containing_block.inline_size - ifc.current_inline_position &&
            ifc.current_nesting_level.white_space.allow_wrap() &&
            ifc.current_nesting_level.line_items_so_far.len() != 0
        {
            ifc.finish_line_and_reset(layout_context);
        }

        let size = &pbm_sums.sum() + &fragment.content_rect.size;
        ifc.push_line_item(
            size.inline,
            LineItem::Atomic(AtomicLineItem {
                fragment,
                size,
                positioning_context: child_positioning_context,
            }),
        );

        // After every atomic, we need to create a line breaking opportunity for the next TextRun.
        if let Some(linebreaker) = ifc.linebreaker.as_mut() {
            linebreaker.next(" ");
        }
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

    fn layout_into_line_items(
        &self,
        layout_context: &LayoutContext,
        ifc: &mut InlineFormattingContextState,
    ) {
        let BreakAndShapeResult {
            font_metrics,
            font_key,
            runs,
            break_at_start,
        } = self.break_and_shape(layout_context, &mut ifc.linebreaker);

        let add_glyphs_to_current_line =
            |ifc: &mut InlineFormattingContextState,
             glyphs: Vec<std::sync::Arc<GlyphStore>>,
             inline_advance,
             force_text_run_creation: bool| {
                if !force_text_run_creation && glyphs.is_empty() {
                    return;
                }

                ifc.push_line_item(
                    inline_advance,
                    LineItem::TextRun(TextRunLineItem {
                        text: glyphs,
                        base_fragment_info: self.base_fragment_info.into(),
                        parent_style: self.parent_style.clone(),
                        font_metrics,
                        font_key,
                        text_decoration_line: ifc.current_nesting_level.text_decoration_line,
                    }),
                );
            };

        let white_space = self.parent_style.get_inherited_text().white_space;
        let mut glyphs = vec![];
        let mut inline_advance = Length::zero();
        let mut iterator = runs.iter().enumerate();
        while let Some((run_index, run)) = iterator.next() {
            // If this whitespace forces a line break, finish the line and reset everything.
            if run.glyph_store.is_whitespace() && white_space.preserve_newlines() {
                let last_byte = self.text.as_bytes().get(run.range.end().to_usize() - 1);
                if last_byte == Some(&b'\n') {
                    // TODO: We shouldn't need to force the creation of a TextRun here, but only TextRuns are
                    // influencing line height calculation of lineboxes (and not all inline boxes on a line).
                    // Once that is fixed, we can avoid adding an empty TextRun here.
                    add_glyphs_to_current_line(
                        ifc,
                        glyphs.drain(..).collect(),
                        inline_advance,
                        true,
                    );
                    ifc.finish_line_and_reset(layout_context);
                    inline_advance = Length::zero();
                    continue;
                }
            }

            // We break the line if this new advance and any advances from pending
            // whitespace bring us past the inline end of the containing block.
            let new_advance = Length::from(run.glyph_store.total_advance());
            let will_advance_past_containing_block =
                (new_advance + inline_advance + ifc.current_inline_position) >
                    ifc.containing_block.inline_size;

            // We can only break the line, if this isn't the first actual content (non-whitespace or
            // preserved whitespace) on the line and this isn't the unbreakable run of this text run
            // (or we can break at the start according to the text breaker).
            let can_break = ifc.line_had_any_content && (break_at_start || run_index != 0);
            if will_advance_past_containing_block && can_break {
                add_glyphs_to_current_line(ifc, glyphs.drain(..).collect(), inline_advance, false);
                ifc.finish_line_and_reset(layout_context);
                inline_advance = Length::zero();
            }

            // From <https://www.w3.org/TR/css-text-3/#white-space-phase-2>:
            // "Then, the entire block is rendered. Inlines are laid out, taking bidi
            // reordering into account, and wrapping as specified by the text-wrap
            // property. As each line is laid out,
            //
            // > 1. A sequence of collapsible spaces at the beginning of a line is removed."
            //
            // This prevents whitespace from being added to the beginning of a line. We could
            // trim it later, but we don't want it to come into play when determining line
            // width.
            if run.glyph_store.is_whitespace() &&
                !white_space.preserve_spaces() &&
                !ifc.line_had_any_content
            {
                continue;
            }

            inline_advance += Length::from(run.glyph_store.total_advance());
            glyphs.push(run.glyph_store.clone());
            ifc.line_had_any_content = true;
        }

        add_glyphs_to_current_line(ifc, glyphs.drain(..).collect(), inline_advance, false);
    }
}

enum InlineBoxChildIter<'box_tree> {
    InlineFormattingContext(std::slice::Iter<'box_tree, ArcRefCell<InlineLevelBox>>),
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
            inline_formatting_context.inline_level_boxes.iter(),
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

/// State used when laying out the [`LineItem`]s collected for the line currently being
/// laid out.
struct LineItemLayoutState<'a> {
    inline_position: Length,
    max_block_size: Length,
    containing_block_inline_start: Length,
    ifc_containing_block: &'a ContainingBlock<'a>,
    positioning_context: &'a mut PositioningContext,
}

fn layout_line_items(
    line_items: Vec<LineItem>,
    layout_context: &LayoutContext,
    state: &mut LineItemLayoutState,
) -> Vec<Fragment> {
    let mut fragments = vec![];
    for item in line_items.into_iter() {
        match item {
            LineItem::TextRun(item) => {
                if let Some(fragment) = item.layout(state) {
                    fragments.push(Fragment::Text(fragment));
                }
            },
            LineItem::InlineBox(box_line_item) => {
                if let Some(fragment) = box_line_item.layout(layout_context, state) {
                    fragments.push(Fragment::Box(fragment))
                }
            },
            LineItem::Atomic(atomic) => {
                fragments.push(Fragment::Box(atomic.layout(state)));
            },
            LineItem::AbsolutelyPositioned(absolute_line_item) => {
                fragments.push(Fragment::AbsoluteOrFixedPositioned(
                    absolute_line_item.layout(state),
                ));
            },
        }
    }
    fragments
}

enum LineItem {
    TextRun(TextRunLineItem),
    InlineBox(InlineBoxLineItem),
    Atomic(AtomicLineItem),
    AbsolutelyPositioned(AbsolutelyPositionedLineItem),
}

impl LineItem {
    fn trim_whitespace_at_end(&mut self) -> bool {
        match self {
            LineItem::TextRun(ref mut item) => item.trim_whitespace_at_end(),
            LineItem::InlineBox(b) => {
                for child in b.children.iter_mut().rev() {
                    if !child.trim_whitespace_at_end() {
                        return false;
                    }
                }
                true
            },
            LineItem::Atomic(_) => false,
            LineItem::AbsolutelyPositioned(_) => true,
        }
    }
}

struct TextRunLineItem {
    base_fragment_info: BaseFragmentInfo,
    parent_style: Arc<ComputedValues>,
    text: Vec<std::sync::Arc<GlyphStore>>,
    font_metrics: FontMetrics,
    font_key: FontInstanceKey,
    text_decoration_line: TextDecorationLine,
}

impl TextRunLineItem {
    fn trim_whitespace_at_end(&mut self) -> bool {
        if self
            .parent_style
            .get_inherited_text()
            .white_space
            .preserve_spaces()
        {
            return false;
        }

        if let Some(pos) = self
            .text
            .iter()
            .rev()
            .position(|glyph| !glyph.is_whitespace())
        {
            self.text.truncate(self.text.len() - pos);
            return false;
        }

        self.text = Vec::new();
        true
    }

    fn layout(self, state: &mut LineItemLayoutState) -> Option<TextFragment> {
        let font_size = self.parent_style.get_font().font_size.size.0;
        let line_height = match self.parent_style.get_inherited_text().line_height {
            LineHeight::Normal => self.font_metrics.line_gap,
            LineHeight::Number(n) => font_size * n.0,
            LineHeight::Length(l) => l.0,
        };
        state.max_block_size.max_assign(line_height);

        // This happens after updating the `max_block_size`, because even trimmed newlines
        // should affect the height of the line.
        if self.text.is_empty() {
            return None;
        }

        let inline_advance: Length = self
            .text
            .iter()
            .map(|glyph_store| Length::from(glyph_store.total_advance()))
            .sum();
        let rect = Rect {
            start_corner: Vec2 {
                block: Length::zero(),
                inline: state.inline_position - state.containing_block_inline_start,
            },
            size: Vec2 {
                block: line_height,
                inline: inline_advance,
            },
        };

        state.inline_position += inline_advance;
        Some(TextFragment {
            base: self.base_fragment_info.into(),
            parent_style: self.parent_style,
            rect,
            font_metrics: self.font_metrics,
            font_key: self.font_key,
            glyphs: self.text,
            text_decoration_line: self.text_decoration_line,
        })
    }
}

struct InlineBoxLineItem {
    base_fragment_info: BaseFragmentInfo,
    style: Arc<ComputedValues>,
    pbm: PaddingBorderMargin,
    children: Vec<LineItem>,
    always_make_fragment: bool,
}

impl InlineBoxLineItem {
    fn layout(
        self,
        layout_context: &LayoutContext,
        state: &mut LineItemLayoutState,
    ) -> Option<BoxFragment> {
        let style = self.style.clone();

        let padding = self.pbm.padding.clone();
        let border = self.pbm.border.clone();
        let margin = self.pbm.margin.auto_is(Length::zero);
        let pbm_sums = &(&padding + &border) + &margin;

        state.inline_position += pbm_sums.inline_start;

        let mut positioning_context = PositioningContext::new_for_style(&style);
        let nested_positioning_context = match positioning_context.as_mut() {
            Some(positioning_context) => positioning_context,
            None => &mut state.positioning_context,
        };
        let original_nested_positioning_context_length = nested_positioning_context.len();

        let mut nested_state = LineItemLayoutState {
            inline_position: state.inline_position,
            max_block_size: Length::zero(),
            containing_block_inline_start: state.inline_position,
            ifc_containing_block: state.ifc_containing_block,
            positioning_context: nested_positioning_context,
        };
        let fragments = layout_line_items(self.children, layout_context, &mut nested_state);

        // If the inline box didn't have any content at all, don't add a Fragment for it.
        let box_has_padding_border_or_margin = pbm_sums.inline_sum() > Length::zero();
        let box_had_absolutes =
            original_nested_positioning_context_length != nested_state.positioning_context.len();
        if !self.always_make_fragment &&
            nested_state.max_block_size.is_zero() &&
            fragments.is_empty() &&
            !box_has_padding_border_or_margin &&
            !box_had_absolutes
        {
            return None;
        }

        let mut content_rect = Rect {
            start_corner: Vec2 {
                inline: state.inline_position - state.containing_block_inline_start,
                block: Length::zero(),
            },
            size: Vec2 {
                inline: nested_state.inline_position - state.inline_position,
                block: nested_state.max_block_size,
            },
        };

        state.inline_position = nested_state.inline_position + pbm_sums.inline_end;
        state.max_block_size.max_assign(content_rect.size.block);

        // Relative adjustment should not affect the rest of line layout, so we can
        // do it right before creating the Fragment.
        if style.clone_position().is_relative() {
            content_rect.start_corner += &relative_adjustement(&style, state.ifc_containing_block);
        }

        let mut fragment = BoxFragment::new(
            self.base_fragment_info,
            self.style.clone(),
            fragments,
            content_rect,
            padding,
            border,
            margin,
            None,
            CollapsedBlockMargins::zero(),
        );

        if let Some(mut positioning_context) = positioning_context.take() {
            assert!(original_nested_positioning_context_length == PositioningContextLength::zero());
            positioning_context.layout_collected_children(layout_context, &mut fragment);
            positioning_context.adjust_static_position_of_hoisted_fragments_with_offset(
                &fragment.content_rect.start_corner,
                PositioningContextLength::zero(),
            );
            state.positioning_context.append(positioning_context);
        } else {
            state
                .positioning_context
                .adjust_static_position_of_hoisted_fragments_with_offset(
                    &fragment.content_rect.start_corner,
                    original_nested_positioning_context_length,
                );
        }

        Some(fragment)
    }
}

struct AtomicLineItem {
    fragment: BoxFragment,
    size: Vec2<Length>,
    positioning_context: Option<PositioningContext>,
}

impl AtomicLineItem {
    fn layout(mut self, state: &mut LineItemLayoutState) -> BoxFragment {
        // The initial `start_corner` of the Fragment is the PaddingBorderMargin sum
        // start offset, which is the sum of the start component of the padding,
        // border, and margin. Offset that value by the inline start position of the
        // line layout.
        self.fragment.content_rect.start_corner.inline +=
            state.inline_position - state.containing_block_inline_start;

        if self.fragment.style.clone_position().is_relative() {
            self.fragment.content_rect.start_corner +=
                &relative_adjustement(&self.fragment.style, state.ifc_containing_block);
        }

        state.inline_position += self.size.inline;
        state.max_block_size.max_assign(self.size.block);

        if let Some(mut positioning_context) = self.positioning_context {
            positioning_context.adjust_static_position_of_hoisted_fragments_with_offset(
                &self.fragment.content_rect.start_corner,
                PositioningContextLength::zero(),
            );
            state.positioning_context.append(positioning_context);
        }

        self.fragment
    }
}

struct AbsolutelyPositionedLineItem {
    absolutely_positioned_box: ArcRefCell<AbsolutelyPositionedBox>,
}

impl AbsolutelyPositionedLineItem {
    fn layout(self, state: &mut LineItemLayoutState) -> ArcRefCell<HoistedSharedFragment> {
        let box_ = self.absolutely_positioned_box;
        let style = AtomicRef::map(box_.borrow(), |box_| box_.context.style());
        let initial_start_corner = match Display::from(style.get_box().original_display) {
            Display::GeneratingBox(DisplayGeneratingBox::OutsideInside { outside, inside: _ }) => {
                Vec2 {
                    inline: match outside {
                        DisplayOutside::Inline => {
                            state.inline_position - state.containing_block_inline_start
                        },
                        DisplayOutside::Block => Length::zero(),
                    },
                    block: Length::zero(),
                }
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
            state.ifc_containing_block,
        );
        let hoisted_fragment = hoisted_box.fragment.clone();
        state.positioning_context.push(hoisted_box);
        hoisted_fragment
    }
}
