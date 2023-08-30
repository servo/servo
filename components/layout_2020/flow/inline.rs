/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use super::float::PlacementAmongFloats;
use super::CollapsibleWithParentStartMargin;
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
use std::cell::OnceCell;
use style::computed_values::white_space::T as WhiteSpace;
use style::logical_geometry::WritingMode;
use style::properties::ComputedValues;
use style::values::computed::Length;
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
    line_items_so_far: Vec<LineItem>,
    /// Whether or not we have processed any content (an atomic element or text) for
    /// this inline box on the current line OR any previous line.
    has_content: bool,
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

/// Information about the current line under construction for a particular
/// [`InlineFormattingContextState`]. This tracks position and size information while
/// [`LineItem`]s are collected and is used as input when those [`LineItem`]s are
/// converted into [`Fragment`]s during the final phase of line layout. Note that this
/// does not store the [`LineItem`]s themselves, as they are stored as part of the
/// nesting state in the [`InlineFormattingContextState`].
struct LineUnderConstruction {
    /// The position where this line will start once it is laid out. This includes any
    /// offset from `text-indent`.
    start_position: Vec2<Length>,

    /// The current inline position in the line being laid out into [`LineItems`] in this
    /// [`InlineFormattingContext`] independent of the depth in the nesting level.
    inline_position: Length,

    /// If the current line ends with whitespace, this tracks the advance width of that
    /// whitespace. This is used to find the "real" width of a line if trailing whitespace
    /// is trimmed from the end.
    trailing_whitespace_advance: Length,

    /// The currently calculated block size of this line, taking into account all inline
    /// content already laid out into [`LineItem`]s. Later content may increase the block
    /// size.
    block_size: Length,

    /// Whether any active linebox has added a glyph, border, margin, or padding
    /// to this line, which indicates that the next run that exceeds the line length
    /// can cause a line break.
    has_content: bool,

    /// Whether or not there are floats that did not fit on the current line. Before
    /// the [`LineItems`] of this line are laid out, these floats will need to be
    /// placed directly below this line, but still as children of this line's Fragments.
    has_floats_waiting_to_be_placed: bool,

    /// A rectangular area (relative to the containing block / inline formatting
    /// context boundaries) where we can fit the line box without overlapping floats.
    /// Note that when this is not empty, its start corner takes precedence over
    /// [`LineUnderConstruction::start_position`].
    placement_among_floats: OnceCell<Rect<Length>>,
}

impl LineUnderConstruction {
    fn new(start_position: Vec2<Length>) -> Self {
        Self {
            inline_position: start_position.inline.clone(),
            trailing_whitespace_advance: Length::zero(),
            start_position: start_position,
            block_size: Length::zero(),
            has_content: false,
            has_floats_waiting_to_be_placed: false,
            placement_among_floats: OnceCell::new(),
        }
    }

    fn line_block_start_considering_placement_among_floats(&self) -> Length {
        match self.placement_among_floats.get() {
            Some(placement_among_floats) => placement_among_floats.start_corner.block,
            None => self.start_position.block,
        }
    }

    fn replace_placement_among_floats(&mut self, new_placement: Rect<Length>) {
        self.placement_among_floats.take();
        let _ = self.placement_among_floats.set(new_placement);
    }
}

struct InlineFormattingContextState<'box_tree, 'a, 'b> {
    positioning_context: &'a mut PositioningContext,
    containing_block: &'b ContainingBlock<'b>,
    sequential_layout_state: Option<&'a mut SequentialLayoutState>,

    /// A vector of fragment that are laid out. This includes one [`Fragment::Anonymous`]
    /// per line that is currently laid out plus fragments for all floats, which
    /// are currently laid out at the top-level of each [`InlineFormattingContext`].
    fragments: Vec<Fragment>,

    /// Information about the line currently being laid out into [`LineItems`]s. The
    /// [`LineItem`]s themselves are stored in the nesting state.
    current_line: LineUnderConstruction,

    /// The line breaking state for this inline formatting context.
    linebreaker: Option<LineBreakLeafIter>,

    /// The currently white-space setting of this line. This is stored on the
    /// [`InlineFormattingContextState`] because when a soft wrap opportunity is defined
    /// by the boundary between two characters, the white-space property of their nearest
    /// common ancestor is used.
    white_space: WhiteSpace,

    partial_inline_boxes_stack: Vec<PartialInlineBoxFragment<'box_tree>>,
    current_nesting_level: InlineNestingLevelState<'box_tree>,
}

impl<'box_tree, 'a, 'b> InlineFormattingContextState<'box_tree, 'a, 'b> {
    /// Push a completed [LineItem] to the current nesteding level of this
    /// [InlineFormattingContext].
    fn push_line_item(
        &mut self,
        inline_size: Length,
        line_item: LineItem,
        last_whitespace_advance: Length,
    ) {
        self.current_line.has_content = true;
        self.current_line.inline_position += inline_size;
        self.current_line.trailing_whitespace_advance = last_whitespace_advance;
        self.current_line
            .block_size
            .max_assign(line_item.block_size());

        self.current_nesting_level.line_items_so_far.push(line_item);
        self.current_nesting_level.has_content = true;
        self.propagate_current_nesting_level_white_space_style();
    }

    fn propagate_current_nesting_level_white_space_style(&mut self) {
        let style = match self.partial_inline_boxes_stack.last() {
            Some(partial) => &partial.style,
            None => self.containing_block.style,
        };
        self.white_space = style.get_inherited_text().white_space;
    }

    /// Finish layout of all the partial inline boxes in the current line,
    /// finish current line and start a new one.
    fn finish_line_and_reset(&mut self, layout_context: &LayoutContext) {
        let mut nesting_level = &mut self.current_nesting_level;
        for partial in self.partial_inline_boxes_stack.iter_mut().rev() {
            partial.finish_layout(
                nesting_level,
                &mut self.current_line.inline_position,
                false, /* at_end_of_inline_element */
            );
            nesting_level = &mut partial.parent_nesting_level;
        }

        let line_items = std::mem::take(&mut nesting_level.line_items_so_far);
        self.finish_current_line(layout_context, line_items, self.containing_block);
    }

    fn finish_current_line(
        &mut self,
        layout_context: &LayoutContext,
        mut line_items: Vec<LineItem>,
        containing_block: &ContainingBlock,
    ) {
        // From <https://www.w3.org/TR/css-text-3/#white-space-phase-2>:
        // > 3. A sequence of collapsible spaces at the end of a line is removed,
        // >    as well as any trailing U+1680   OGHAM SPACE MARK whose white-space
        // >    property is normal, nowrap, or pre-line.
        let mut whitespace_trimmed = Length::zero();
        for item in line_items.iter_mut().rev() {
            if !item.trim_whitespace_at_end(&mut whitespace_trimmed) {
                break;
            }
        }

        let inline_start_position =
            self.calculate_inline_start_for_current_line(containing_block, whitespace_trimmed);
        let block_start_position = self
            .current_line
            .line_block_start_considering_placement_among_floats();
        let block_end_position = block_start_position + self.current_line.block_size;

        if let Some(sequential_layout_state) = self.sequential_layout_state.as_mut() {
            // This amount includes both the block size of the line and any extra space
            // added to move the line down in order to avoid overlapping floats.
            let increment = block_end_position - self.current_line.start_position.block;
            sequential_layout_state.advance_block_position(increment);
        }

        if self.current_line.has_floats_waiting_to_be_placed {
            place_pending_floats(self, &mut line_items);
        }

        let mut state = LineItemLayoutState {
            inline_position: inline_start_position,
            max_block_size: Length::zero(),
            inline_start_of_parent: Length::zero(),
            ifc_containing_block: containing_block,
            positioning_context: &mut self.positioning_context,
            line_block_start: block_start_position,
        };

        let positioning_context_length = state.positioning_context.len();
        let fragments = layout_line_items(line_items, layout_context, &mut state);

        let size = Vec2 {
            inline: containing_block.inline_size,
            block: state.max_block_size,
        };

        // The inline part of this start offset was taken into account when determining
        // the inline start of the line in `calculate_inline_start_for_current_line` so
        // we do not need to include it in the `start_corner` of the line's main Fragment.
        let start_corner = Vec2 {
            inline: Length::zero(),
            block: block_start_position,
        };

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

        self.current_line = LineUnderConstruction::new(Vec2 {
            inline: Length::zero(),
            block: block_end_position,
        });
    }

    /// Given the amount of whitespace trimmed from the line and taking into consideration
    /// the `text-align` property, calculate where the line under construction starts in
    /// the inline axis.
    fn calculate_inline_start_for_current_line(
        &self,
        containing_block: &ContainingBlock,
        whitespace_trimmed: Length,
    ) -> Length {
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

        let (line_start, available_space) = match self.current_line.placement_among_floats.get() {
            Some(placement_among_floats) => (
                placement_among_floats.start_corner.inline,
                placement_among_floats.size.inline,
            ),
            None => (Length::zero(), self.containing_block.inline_size),
        };

        // Properly handling text-indent requires that we do not align the text
        // into the text-indent.
        // See <https://drafts.csswg.org/css-text/#text-indent-property>
        // "This property specifies the indentation applied to lines of inline content in
        // a block. The indent is treated as a margin applied to the start edge of the
        // line box."
        let text_indent = self.current_line.start_position.inline;
        let line_length = self.current_line.inline_position - whitespace_trimmed - text_indent;
        line_start +
            match text_align {
                TextAlign::Start => text_indent,
                TextAlign::End => (available_space - line_length).max(text_indent),
                TextAlign::Center => (available_space - line_length + text_indent) / 2.,
            }
    }

    fn place_float_fragment(&mut self, fragment: &mut BoxFragment) {
        let state = self
            .sequential_layout_state
            .as_mut()
            .expect("Tried to lay out a float with no sequential placement state!");

        let block_offset_from_containining_block_top = state
            .current_block_position_including_margins() -
            state.current_containing_block_offset();
        state.place_float_fragment(
            fragment,
            CollapsedMargin::zero(),
            block_offset_from_containining_block_top,
        );
    }

    /// Given a new potential line size for the current line, create a "placement" for that line.
    /// This tells us whether or not the new potential line will fit in the current block position
    /// or need to be moved. In addition, the placement rect determines the inline start and end
    /// of the line if it's used as the final placement among floats.
    fn place_line_among_floats(&self, potential_line_size: &Vec2<Length>) -> Rect<Length> {
        let sequential_layout_state = self
            .sequential_layout_state
            .as_ref()
            .expect("Should not have called this function without having floats.");

        let ifc_offset_in_float_container = Vec2 {
            inline: sequential_layout_state
                .floats
                .containing_block_info
                .inline_start,
            block: sequential_layout_state.current_containing_block_offset(),
        };

        let ceiling = self
            .current_line
            .line_block_start_considering_placement_among_floats();
        let mut placement = PlacementAmongFloats::new(
            &sequential_layout_state.floats,
            ceiling + ifc_offset_in_float_container.block,
            potential_line_size.clone(),
            &PaddingBorderMargin::zero(),
        );

        let mut placement_rect = placement.place();
        placement_rect.start_corner = &placement_rect.start_corner - &ifc_offset_in_float_container;
        placement_rect
    }

    /// Returns true if a new potential line size for the current line would require a line
    /// break. This takes into account floats and will also update the "placement among
    /// floats" for this line if the potential line size would not cause a line break.
    /// Thus, calling this method has side effects and should only be done while in the
    /// process of laying out line content that is always going to be committed to this
    /// line or the next.
    fn new_potential_line_size_causes_line_break(
        &mut self,
        potential_line_size: &Vec2<Length>,
    ) -> bool {
        let available_line_space = if self.sequential_layout_state.is_some() {
            self.current_line
                .placement_among_floats
                .get_or_init(|| self.place_line_among_floats(potential_line_size))
                .size
                .clone()
        } else {
            Vec2 {
                inline: self.containing_block.inline_size,
                block: Length::new(f32::INFINITY),
            }
        };

        let inline_would_overflow = potential_line_size.inline > available_line_space.inline;
        let block_would_overflow = potential_line_size.block > available_line_space.block;

        // The first content that is added to a line cannot trigger a line break and
        // the `white-space` propertly can also prevent all line breaking.
        let can_break = self.current_line.has_content && self.white_space.allow_wrap();

        // If this is the first content on the line and we already have a float placement,
        // that means that the placement was initialized by a leading float in the IFC.
        // This placement needs to be updated, because the first line content might push
        // the block start of the line downward. If there is no float placement, we want
        // to make one to properly set the block position of the line.
        if !can_break {
            // Even if we cannot break, adding content to this line might change its position.
            // In that case we need to redo our placement among floats.
            if self.sequential_layout_state.is_some() &&
                (inline_would_overflow || block_would_overflow)
            {
                let new_placement = self.place_line_among_floats(potential_line_size);
                self.current_line
                    .replace_placement_among_floats(new_placement);
            }

            return false;
        }

        // If the potential line is larger than the containing block we do not even need to consider
        // floats. We definitely have to do a linebreak.
        if potential_line_size.inline > self.containing_block.inline_size {
            return true;
        }

        // Not fitting in the block space means that our block size has changed and we had a
        // placement among floats that is no longer valid. This same placement might just
        // need to be expanded or perhaps we need to line break.
        if block_would_overflow {
            // If we have a limited block size then we are wedging this line between floats.
            assert!(self.sequential_layout_state.is_some());
            let new_placement = self.place_line_among_floats(potential_line_size);
            if new_placement.start_corner.block !=
                self.current_line
                    .line_block_start_considering_placement_among_floats()
            {
                return true;
            } else {
                self.current_line
                    .replace_placement_among_floats(new_placement);
                return false;
            }
        }

        // Otherwise the new potential line size will require a newline if it fits in the
        // inline space available for this line. This space may be smaller than the
        // containing block if floats shrink the available inline space.
        inline_would_overflow
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
                                        // For margins and paddings, a cyclic percentage is resolved against zero
                                        // for determining intrinsic size contributions.
                                        // https://drafts.csswg.org/css-sizing-3/#min-percentage-contribution
                                        let zero = Length::zero();
                                        let mut length = padding.$side.percentage_relative_to(zero) + border.$side;
                                        if let Some(lp) = margin.$side.non_auto() {
                                            length += lp.percentage_relative_to(zero)
                                        }
                                        self.add_length(length);
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
                            let outer = atomic.outer_inline_content_sizes(
                                self.layout_context,
                                self.containing_block_writing_mode,
                            );

                            self.current_line.min_content +=
                                self.pending_whitespace + outer.min_content;
                            self.current_line.max_content += outer.max_content;
                            self.pending_whitespace = Length::zero();
                            self.had_non_whitespace_content_yet = true;
                        },
                        InlineLevelBox::OutOfFlowFloatBox(_) |
                        InlineLevelBox::OutOfFlowAbsolutelyPositionedBox(_) => {},
                    }
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
            current_line: LineUnderConstruction::new(Vec2 {
                inline: first_line_inline_start,
                block: Length::zero(),
            }),
            white_space: containing_block.style.get_inherited_text().white_space,
            linebreaker: None,
            partial_inline_boxes_stack: Vec::new(),
            current_nesting_level: InlineNestingLevelState {
                remaining_boxes: InlineBoxChildIter::from_formatting_context(self),
                line_items_so_far: Vec::with_capacity(self.inline_level_boxes.len()),
                has_content: false,
                text_decoration_line: self.text_decoration_line,
            },
        };

        // FIXME(pcwalton): This assumes that margins never collapse through inline formatting
        // contexts (i.e. that inline formatting contexts are never empty). Is that right?
        // FIXME(mrobinson): This should not happen if the IFC collapses through.
        if let Some(ref mut sequential_layout_state) = ifc.sequential_layout_state {
            sequential_layout_state.collapse_margins();
            // FIXME(mrobinson): Collapse margins in the containing block offsets as well??
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
                        float_box.layout_into_line_items(layout_context, &mut ifc);
                    },
                }
            } else if let Some(mut partial) = ifc.partial_inline_boxes_stack.pop() {
                // We reached the end of the remaining boxes in this nesting level, so we finish it and
                // start working on the parent nesting level again.
                partial.finish_layout(
                    &mut ifc.current_nesting_level,
                    &mut ifc.current_line.inline_position,
                    true, /* at_end_of_inline_element */
                );

                let had_content = ifc.current_nesting_level.has_content;
                ifc.current_nesting_level = partial.parent_nesting_level;

                // If the inline box that we just finished had any content at all, we want to propagate
                // the `white-space` property of its parent to future inline children. This is because
                // when a soft wrap opportunity is defined by the boundary between two elements, the
                // `white-space` used is that of their nearest common ancestor.
                if had_content {
                    ifc.propagate_current_nesting_level_white_space_style();
                }
            } else {
                // We reached the end of the entire IFC.
                break;
            }
        }

        let line_items = std::mem::take(&mut ifc.current_nesting_level.line_items_so_far);
        ifc.finish_current_line(layout_context, line_items, containing_block);

        let mut collapsible_margins_in_children = CollapsedBlockMargins::zero();
        let content_block_size = ifc.current_line.start_position.block;
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
            ifc.current_line.inline_position += pbm.padding.inline_start +
                pbm.border.inline_start +
                pbm.margin.inline_start.auto_is(Length::zero)
        } else {
            pbm.padding.inline_start = Length::zero();
            pbm.border.inline_start = Length::zero();
            pbm.margin.inline_start = LengthOrAuto::zero();
        }

        let text_decoration_line =
            ifc.current_nesting_level.text_decoration_line | style.clone_text_decoration_line();
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
                    has_content: false,
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

        let size = &pbm_sums.sum() + &fragment.content_rect.size;
        let new_potential_line_size = Vec2 {
            inline: ifc.current_line.inline_position + size.inline,
            block: ifc.current_line.block_size.max(size.block),
        };

        if ifc.new_potential_line_size_causes_line_break(&new_potential_line_size) {
            ifc.finish_line_and_reset(layout_context);
        }

        ifc.push_line_item(
            size.inline,
            LineItem::Atomic(AtomicLineItem {
                fragment,
                size,
                positioning_context: child_positioning_context,
            }),
            Length::zero(),
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

        let white_space = self.parent_style.get_inherited_text().white_space;
        let add_glyphs_to_current_line =
            |ifc: &mut InlineFormattingContextState,
             glyphs: Vec<std::sync::Arc<GlyphStore>>,
             inline_advance,
             force_text_run_creation: bool| {
                if !force_text_run_creation && glyphs.is_empty() {
                    return;
                }

                let last_whitespace_advance = match (white_space.preserve_spaces(), glyphs.last()) {
                    (false, Some(last_glyph)) if last_glyph.is_whitespace() => {
                        last_glyph.total_advance()
                    },
                    _ => Au::zero(),
                };

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
                    Length::from(last_whitespace_advance),
                );
            };

        let line_height = line_height(&self.parent_style, &font_metrics);
        let new_max_height_of_line = ifc.current_line.block_size.max(line_height);

        let mut glyphs = vec![];
        let mut advance_from_text_run = Length::zero();
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
                        advance_from_text_run,
                        true,
                    );
                    ifc.finish_line_and_reset(layout_context);
                    advance_from_text_run = Length::zero();
                    continue;
                }
            }

            let new_advance_from_glyph_run = Length::from(run.glyph_store.total_advance());
            let new_total_advance = new_advance_from_glyph_run +
                advance_from_text_run +
                ifc.current_line.inline_position;

            let new_potential_line_size = Vec2 {
                inline: new_total_advance,
                block: new_max_height_of_line,
            };

            // If we cannot break at the start according to the text breaker and this is the first
            // unbreakable run of glyphs then we cannot break in any case.
            // TODO(mrobinson): If this doesn't fit on the current line and there is content we
            // need to line break, but this requires rewinding LineItems and adding them to the
            // next line.
            let can_break = break_at_start || run_index != 0;
            if ifc.new_potential_line_size_causes_line_break(&new_potential_line_size) && can_break
            {
                add_glyphs_to_current_line(
                    ifc,
                    glyphs.drain(..).collect(),
                    advance_from_text_run,
                    true,
                );
                ifc.finish_line_and_reset(layout_context);
                advance_from_text_run = Length::zero();
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
                !ifc.current_line.has_content
            {
                continue;
            }

            advance_from_text_run += Length::from(run.glyph_store.total_advance());
            glyphs.push(run.glyph_store.clone());
            ifc.current_line.has_content = true;
            ifc.propagate_current_nesting_level_white_space_style();
        }

        add_glyphs_to_current_line(
            ifc,
            glyphs.drain(..).collect(),
            advance_from_text_run,
            false,
        );
    }
}

impl FloatBox {
    fn layout_into_line_items(
        &mut self,
        layout_context: &LayoutContext,
        ifc: &mut InlineFormattingContextState,
    ) {
        let mut fragment = self.layout(
            layout_context,
            ifc.positioning_context,
            ifc.containing_block,
        );

        let margin_box = fragment.border_rect().inflate(&fragment.margin);
        let inline_size = margin_box.size.inline.max(Length::zero());

        let available_inline_size = match ifc.current_line.placement_among_floats.get() {
            Some(placement_among_floats) => placement_among_floats.size.inline,
            None => ifc.containing_block.inline_size,
        } - (ifc.current_line.inline_position -
            ifc.current_line.trailing_whitespace_advance);

        // If this float doesn't fit on the current line or a previous float didn't fit on
        // the current line, we need to place it starting at the next line BUT still as
        // children of this line's hierarchy of inline boxes (for the purposes of properly
        // parenting in their stacking contexts). Once all the line content is gathered we
        // will place them later.
        let fits_on_line = !ifc.current_line.has_content || inline_size <= available_inline_size;
        let needs_placement_later =
            ifc.current_line.has_floats_waiting_to_be_placed || !fits_on_line;

        if needs_placement_later {
            ifc.current_line.has_floats_waiting_to_be_placed = true;
        } else {
            ifc.place_float_fragment(&mut fragment);

            // We've added a new float to the IFC, but this may have actually changed the
            // position of the current line. In order to determine that we regenerate the
            // placement among floats for the current line, which may adjust its inline
            // start position.
            let new_placement = ifc.place_line_among_floats(&Vec2 {
                inline: ifc.current_line.inline_position,
                block: ifc.current_line.block_size,
            });
            ifc.current_line
                .replace_placement_among_floats(new_placement);
        }

        ifc.current_nesting_level
            .line_items_so_far
            .push(LineItem::Float(FloatLineItem {
                fragment,
                needs_placement: needs_placement_later,
            }));
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

    /// The inline start position of the parent (the inline box that established this state)
    /// relative to the edge of the containing block of this [`InlineFormattingCotnext`].
    inline_start_of_parent: Length,

    ifc_containing_block: &'a ContainingBlock<'a>,
    positioning_context: &'a mut PositioningContext,
    line_block_start: Length,
}

fn layout_line_items(
    line_items: Vec<LineItem>,
    layout_context: &LayoutContext,
    state: &mut LineItemLayoutState,
) -> Vec<Fragment> {
    let mut fragments = vec![];
    for item in line_items.into_iter() {
        match item {
            LineItem::TextRun(text_line_item) => {
                if let Some(fragment) = text_line_item.layout(state) {
                    fragments.push(Fragment::Text(fragment));
                }
            },
            LineItem::InlineBox(box_line_item) => {
                if let Some(fragment) = box_line_item.layout(layout_context, state) {
                    fragments.push(Fragment::Box(fragment))
                }
            },
            LineItem::Atomic(atomic_line_item) => {
                fragments.push(Fragment::Box(atomic_line_item.layout(state)));
            },
            LineItem::AbsolutelyPositioned(absolute_line_item) => {
                fragments.push(Fragment::AbsoluteOrFixedPositioned(
                    absolute_line_item.layout(state),
                ));
            },
            LineItem::Float(float_line_item) => {
                fragments.push(Fragment::Float(float_line_item.layout(state)));
            },
        }
    }
    fragments
}

fn place_pending_floats(ifc: &mut InlineFormattingContextState, line_items: &mut Vec<LineItem>) {
    for item in line_items.into_iter() {
        match item {
            LineItem::InlineBox(box_line_item) => {
                place_pending_floats(ifc, &mut box_line_item.children);
            },
            LineItem::Float(float_line_item) => {
                if float_line_item.needs_placement {
                    ifc.place_float_fragment(&mut float_line_item.fragment);
                }
            },
            _ => {},
        }
    }
}

enum LineItem {
    TextRun(TextRunLineItem),
    InlineBox(InlineBoxLineItem),
    Atomic(AtomicLineItem),
    AbsolutelyPositioned(AbsolutelyPositionedLineItem),
    Float(FloatLineItem),
}

impl LineItem {
    fn trim_whitespace_at_end(&mut self, whitespace_trimmed: &mut Length) -> bool {
        match self {
            LineItem::TextRun(ref mut item) => item.trim_whitespace_at_end(whitespace_trimmed),
            LineItem::InlineBox(b) => {
                for child in b.children.iter_mut().rev() {
                    if !child.trim_whitespace_at_end(whitespace_trimmed) {
                        return false;
                    }
                }
                true
            },
            LineItem::Atomic(_) => false,
            LineItem::AbsolutelyPositioned(_) => true,
            LineItem::Float(_) => true,
        }
    }

    fn block_size(&self) -> Length {
        match self {
            LineItem::TextRun(text_run) => text_run.line_height(),
            LineItem::InlineBox(_) => {
                // TODO(mrobinson): This should get the line height from the font.
                Length::zero()
            },
            LineItem::Atomic(atomic) => atomic.size.block,
            LineItem::AbsolutelyPositioned(_) => Length::zero(),
            LineItem::Float(_) => Length::zero(),
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

fn line_height(parent_style: &Arc<ComputedValues>, font_metrics: &FontMetrics) -> Length {
    let font_size = parent_style.get_font().font_size.size.0;
    match parent_style.get_inherited_text().line_height {
        LineHeight::Normal => font_metrics.line_gap,
        LineHeight::Number(n) => font_size * n.0,
        LineHeight::Length(l) => l.0,
    }
}

impl TextRunLineItem {
    fn trim_whitespace_at_end(&mut self, whitespace_trimmed: &mut Length) -> bool {
        if self
            .parent_style
            .get_inherited_text()
            .white_space
            .preserve_spaces()
        {
            return false;
        }

        let index_of_last_non_whitespace = self
            .text
            .iter()
            .rev()
            .position(|glyph| !glyph.is_whitespace())
            .map(|offset_from_end| self.text.len() - offset_from_end);

        let first_whitespace_index = index_of_last_non_whitespace.unwrap_or(0);
        *whitespace_trimmed += self
            .text
            .drain(first_whitespace_index..)
            .map(|glyph| Length::from(glyph.total_advance()))
            .sum();

        // Only keep going if we only encountered whitespace.
        index_of_last_non_whitespace.is_none()
    }

    fn line_height(&self) -> Length {
        line_height(&self.parent_style, &self.font_metrics)
    }

    fn layout(self, state: &mut LineItemLayoutState) -> Option<TextFragment> {
        state.max_block_size.max_assign(self.line_height());

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
                inline: state.inline_position - state.inline_start_of_parent,
            },
            size: Vec2 {
                block: self.line_height(),
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
            inline_start_of_parent: state.inline_position,
            ifc_containing_block: state.ifc_containing_block,
            positioning_context: nested_positioning_context,
            line_block_start: state.line_block_start,
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
                inline: state.inline_position - state.inline_start_of_parent,
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
            state.inline_position - state.inline_start_of_parent;

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
                            state.inline_position - state.inline_start_of_parent
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

struct FloatLineItem {
    fragment: BoxFragment,
    /// Whether or not this float Fragment has been placed yet. Fragments that
    /// do not fit on a line need to be placed after the hypothetical block start
    /// of the next line.
    needs_placement: bool,
}

impl FloatLineItem {
    fn layout(mut self, state: &mut LineItemLayoutState<'_>) -> BoxFragment {
        // The `BoxFragment` for this float is positioned relative to the IFC, so we need
        // to move it to be positioned relative to our parent InlineBox line item. Floats
        // fragments are children of these InlineBoxes and not children of the inline
        // formatting context, so that they are parented properly for StackingContext
        // properties such as opacity & filters.
        let distance_from_parent_to_ifc = Vec2 {
            inline: state.inline_start_of_parent,
            block: state.line_block_start,
        };
        self.fragment.content_rect.start_corner =
            &self.fragment.content_rect.start_corner - &distance_from_parent_to_ifc;
        self.fragment
    }
}
