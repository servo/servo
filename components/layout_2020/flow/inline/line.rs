/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::rc::Rc;
use std::vec::IntoIter;

use app_units::Au;
use bitflags::bitflags;
use fonts::{FontMetrics, GlyphStore};
use servo_arc::Arc;
use style::computed_values::white_space_collapse::T as WhiteSpaceCollapse;
use style::properties::ComputedValues;
use style::values::computed::Length;
use style::values::generics::box_::{GenericVerticalAlign, VerticalAlignKeyword};
use style::values::generics::font::LineHeight;
use style::values::specified::box_::DisplayOutside;
use style::values::specified::text::TextDecorationLine;
use style::values::Either;
use style::Zero;
use webrender_api::FontInstanceKey;

use super::inline_box::{
    InlineBoxContainerState, InlineBoxIdentifier, InlineBoxTreePathToken, InlineBoxes,
};
use super::{InlineFormattingContextState, LineBlockSizes};
use crate::cell::ArcRefCell;
use crate::context::LayoutContext;
use crate::fragment_tree::{
    BaseFragmentInfo, BoxFragment, CollapsedBlockMargins, Fragment, TextFragment,
};
use crate::geom::{LogicalRect, LogicalVec2};
use crate::positioned::{
    relative_adjustement, AbsolutelyPositionedBox, PositioningContext, PositioningContextLength,
};
use crate::ContainingBlock;

pub(super) struct LineMetrics {
    /// The block offset of the line start in the containing
    /// [`crate::flow::InlineFormattingContext`].
    pub block_offset: Au,

    /// The block size of this line.
    pub block_size: Au,

    /// The block offset of this line's baseline from [`Self::block_offset`].
    pub baseline_block_offset: Au,
}

bitflags! {
    struct LineLayoutInlineContainerFlags: u8 {
        /// Whether or not any line items were processed for this inline box, this includes
        /// any child inline boxes.
        const HAD_ANY_LINE_ITEMS = 1 << 0;
        /// Whether or not the starting inline border, padding, or margin of the inline box
        /// was encountered.
        const HAD_START_PBM = 1 << 2;
        /// Whether or not the ending inline border, padding, or margin of the inline box
        /// was encountered.
        const HAD_END_PBM = 1 << 3;
        /// Whether or not any floats were encountered while laying out this inline box.
        const HAD_ANY_FLOATS = 1 << 4;
    }
}

/// The state used when laying out a collection of [`LineItem`]s into a line. This state is stored
/// per-inline container. For instance, when laying out the conents of a `<span>` a fresh
/// [`LineItemLayoutInlineContainerState`] is pushed onto [`LineItemLayout`]'s stack of states.
pub(super) struct LineItemLayoutInlineContainerState {
    /// If this inline container is not the root inline container, the identifier of the [`super::InlineBox`]
    /// that is currently being laid out.
    pub identifier: Option<InlineBoxIdentifier>,

    /// The fragments that are laid out into this inline container on a line.
    pub fragments: Vec<Fragment>,

    /// The current inline adavnce of the layout in the coordinates of this inline box.
    pub inline_advance: Au,

    /// Flags which track various features during layout.
    flags: LineLayoutInlineContainerFlags,

    /// The offset of the parent, relative to the start position of the line.
    pub parent_offset: LogicalVec2<Au>,

    /// The block offset of the parent's baseline relative to the block start of the line. This
    /// is often the same as [`Self::parent_offset`], but can be different for the root
    /// element.
    pub baseline_offset: Au,

    /// If this inline box establishes a containing block for positioned elements, this
    /// is a fresh positioning context to contain them. Otherwise, this holds the starting
    /// offset in the *parent* positioning context so that static positions can be updated
    /// at the end of layout.
    pub positioning_context_or_start_offset_in_parent:
        Either<PositioningContext, PositioningContextLength>,
}

impl LineItemLayoutInlineContainerState {
    fn new(
        identifier: Option<InlineBoxIdentifier>,
        parent_offset: LogicalVec2<Au>,
        baseline_offset: Au,
        positioning_context_or_start_offset_in_parent: Either<
            PositioningContext,
            PositioningContextLength,
        >,
    ) -> Self {
        Self {
            identifier,
            fragments: Vec::new(),
            inline_advance: Au::zero(),
            flags: LineLayoutInlineContainerFlags::empty(),
            parent_offset,
            baseline_offset,
            positioning_context_or_start_offset_in_parent,
        }
    }

    fn root(starting_inline_advance: Au, baseline_offset: Au) -> Self {
        let mut state = Self::new(
            None,
            LogicalVec2::zero(),
            baseline_offset,
            Either::Second(PositioningContextLength::zero()),
        );
        state.inline_advance = starting_inline_advance;
        state
    }
}

/// The second phase of [`super::InlineFormattingContext`] layout: once items are gathered
/// for a line, we must lay them out and create fragments for them, properly positioning them
/// according to their baselines and also handling absolutely positioned children.
pub(super) struct LineItemLayout<'a> {
    /// The set of [`super::InlineBox`]es for the [`super::InlineFormattingContext`]. This
    /// does *not* include any state from during phase one of layout.
    pub inline_boxes: &'a InlineBoxes,

    /// The set of [`super::InlineBoxContainerState`] from phase one of IFC layout. There is
    /// one of these for every inline box, *not* for the root inline container.
    pub inline_box_states: &'a [Rc<InlineBoxContainerState>],

    /// The set of [`super::LineItemLayoutInlineContainerState`] created while laying out items
    /// on this line. This does not include the current level of recursion.
    pub state_stack: Vec<LineItemLayoutInlineContainerState>,

    /// The current [`super::LineItemLayoutInlineContainerState`].
    pub state: LineItemLayoutInlineContainerState,

    /// The [`LayoutContext`] to use for laying out absolutely positioned line items.
    pub layout_context: &'a LayoutContext<'a>,

    /// The root positioning context for this layout.
    pub root_positioning_context: &'a mut PositioningContext,

    /// The [`ContainingBlock`] of the parent [`super::InlineFormattingContext`] of the line being
    /// laid out.
    pub ifc_containing_block: &'a ContainingBlock<'a>,

    /// The metrics of this line, which should remain constant throughout the
    /// layout process.
    pub line_metrics: LineMetrics,

    /// The amount of space to add to each justification opportunity in order to implement
    /// `text-align: justify`.
    pub justification_adjustment: Au,
}

impl<'a> LineItemLayout<'a> {
    pub(super) fn layout_line_items(
        state: &mut InlineFormattingContextState,
        iterator: &mut IntoIter<LineItem>,
        start_position: LogicalVec2<Au>,
        effective_block_advance: &LineBlockSizes,
        justification_adjustment: Au,
    ) -> Vec<Fragment> {
        let baseline_offset = effective_block_advance.find_baseline_offset();
        LineItemLayout {
            inline_boxes: state.inline_boxes,
            inline_box_states: &state.inline_box_states,
            state_stack: Vec::new(),
            root_positioning_context: state.positioning_context,
            layout_context: state.layout_context,
            state: LineItemLayoutInlineContainerState::root(start_position.inline, baseline_offset),
            ifc_containing_block: state.containing_block,
            line_metrics: LineMetrics {
                block_offset: start_position.block,
                block_size: effective_block_advance.resolve(),
                baseline_block_offset: baseline_offset,
            },
            justification_adjustment,
        }
        .layout(iterator)
    }

    /// Start and end inline boxes in tree order, so that it reflects the given inline box.
    fn prepare_layout_for_inline_box(&mut self, new_inline_box: Option<InlineBoxIdentifier>) {
        // Optimize the case where we are moving to the root of the inline box stack.
        let Some(new_inline_box) = new_inline_box else {
            while !self.state_stack.is_empty() {
                self.end_inline_box();
            }
            return;
        };

        // Otherwise, follow the path given to us by our collection of inline boxes, so we know which
        // inline boxes to start and end.
        let path = self
            .inline_boxes
            .get_path(self.state.identifier, new_inline_box);
        for token in path {
            match token {
                InlineBoxTreePathToken::Start(ref identifier) => self.start_inline_box(identifier),
                InlineBoxTreePathToken::End(_) => self.end_inline_box(),
            }
        }
    }

    pub(super) fn layout(&mut self, iterator: &mut IntoIter<LineItem>) -> Vec<Fragment> {
        for item in iterator.by_ref() {
            // When preparing to lay out a new line item, start and end inline boxes, so that the current
            // inline box state reflects the item's parent. Items in the line are not necessarily in tree
            // order due to BiDi and other reordering so the inline box of the item could potentially be
            // any in the inline formatting context.
            self.prepare_layout_for_inline_box(item.inline_box_identifier());

            self.state
                .flags
                .insert(LineLayoutInlineContainerFlags::HAD_ANY_LINE_ITEMS);
            match item {
                LineItem::StartInlineBoxPaddingBorderMargin(_) => {
                    self.state
                        .flags
                        .insert(LineLayoutInlineContainerFlags::HAD_START_PBM);
                },
                LineItem::EndInlineBoxPaddingBorderMargin(_) => {
                    self.state
                        .flags
                        .insert(LineLayoutInlineContainerFlags::HAD_END_PBM);
                },
                LineItem::TextRun(_, text_run) => self.layout_text_run(text_run),
                LineItem::Atomic(_, atomic) => self.layout_atomic(atomic),
                LineItem::AbsolutelyPositioned(_, absolute) => self.layout_absolute(absolute),
                LineItem::Float(_, float) => self.layout_float(float),
            }
        }

        // Move back to the root of the inline box tree, so that all boxes are ended.
        self.prepare_layout_for_inline_box(None);
        std::mem::take(&mut self.state.fragments)
    }

    fn current_positioning_context_mut(&mut self) -> &mut PositioningContext {
        if let Either::First(ref mut positioning_context) =
            self.state.positioning_context_or_start_offset_in_parent
        {
            return positioning_context;
        }
        self.state_stack
            .iter_mut()
            .rev()
            .find_map(
                |state| match state.positioning_context_or_start_offset_in_parent {
                    Either::First(ref mut positioning_context) => Some(positioning_context),
                    Either::Second(_) => None,
                },
            )
            .unwrap_or(self.root_positioning_context)
    }

    fn start_inline_box(&mut self, identifier: &InlineBoxIdentifier) {
        let inline_box_state = &*self.inline_box_states[identifier.index_in_inline_boxes as usize];
        let inline_box = self.inline_boxes.get(identifier);
        let inline_box = &*(inline_box.borrow());

        let style = &inline_box.style;
        let space_above_baseline = inline_box_state.calculate_space_above_baseline();
        let block_start_offset =
            self.calculate_inline_box_block_start(inline_box_state, space_above_baseline);

        let positioning_context_or_start_offset_in_parent =
            match PositioningContext::new_for_style(style) {
                Some(positioning_context) => Either::First(positioning_context),
                None => Either::Second(self.current_positioning_context_mut().len()),
            };

        let parent_offset = LogicalVec2 {
            inline: self.state.inline_advance + self.state.parent_offset.inline,
            block: block_start_offset,
        };

        let outer_state = std::mem::replace(
            &mut self.state,
            LineItemLayoutInlineContainerState::new(
                Some(*identifier),
                parent_offset,
                block_start_offset + space_above_baseline,
                positioning_context_or_start_offset_in_parent,
            ),
        );

        self.state_stack.push(outer_state);
    }

    fn end_inline_box(&mut self) {
        let outer_state = self.state_stack.pop().expect("Ended unknown inline box 11");
        let mut inner_state = std::mem::replace(&mut self.state, outer_state);

        let identifier = inner_state.identifier.expect("Ended unknown inline box 22");
        let inline_box_state = &*self.inline_box_states[identifier.index_in_inline_boxes as usize];
        let inline_box = self.inline_boxes.get(&identifier);
        let inline_box = &*(inline_box.borrow());

        let mut padding = inline_box_state.pbm.padding;
        let mut border = inline_box_state.pbm.border;
        let mut margin = inline_box_state.pbm.margin.auto_is(Au::zero);
        if !inner_state
            .flags
            .contains(LineLayoutInlineContainerFlags::HAD_START_PBM)
        {
            padding.inline_start = Au::zero();
            border.inline_start = Au::zero();
            margin.inline_start = Au::zero();
        }
        if !inner_state
            .flags
            .contains(LineLayoutInlineContainerFlags::HAD_END_PBM)
        {
            padding.inline_end = Au::zero();
            border.inline_end = Au::zero();
            margin.inline_end = Au::zero();
        }

        // If the inline box didn't have any content at all and it isn't the first fragment for
        // an element (needed for layout queries currently) and it didn't have any padding, border,
        // or margin do not make a fragment for it.
        //
        // Note: This is an optimization, but also has side effects. Any fragments on a line will
        // force the baseline to advance in the parent IFC.
        let pbm_sums = padding + border + margin;
        if inner_state.fragments.is_empty() &&
            !inner_state
                .flags
                .contains(LineLayoutInlineContainerFlags::HAD_START_PBM) &&
            pbm_sums.inline_sum().is_zero()
        {
            return;
        }

        // Make `content_rect` relative to the parent Fragment.
        let mut content_rect = LogicalRect {
            start_corner: LogicalVec2 {
                inline: self.state.inline_advance + pbm_sums.inline_start,
                block: inner_state.parent_offset.block - self.state.parent_offset.block,
            },
            size: LogicalVec2 {
                inline: inner_state.inline_advance,
                block: inline_box_state.base.font_metrics.line_gap,
            },
        };

        if inner_state
            .flags
            .contains(LineLayoutInlineContainerFlags::HAD_ANY_FLOATS)
        {
            for fragment in inner_state.fragments.iter_mut() {
                if let Fragment::Float(box_fragment) = fragment {
                    box_fragment.content_rect.start_corner -= pbm_sums.start_offset();
                }
            }
        }

        // Relative adjustment should not affect the rest of line layout, so we can
        // do it right before creating the Fragment.
        let style = &inline_box.style;
        if style.clone_position().is_relative() {
            content_rect.start_corner += relative_adjustement(style, self.ifc_containing_block);
        }

        let mut fragment = BoxFragment::new(
            inline_box.base_fragment_info,
            style.clone(),
            inner_state.fragments,
            content_rect,
            padding,
            border,
            margin,
            None, /* clearance */
            CollapsedBlockMargins::zero(),
        );

        match inner_state.positioning_context_or_start_offset_in_parent {
            Either::First(mut positioning_context) => {
                positioning_context.layout_collected_children(self.layout_context, &mut fragment);
                positioning_context.adjust_static_position_of_hoisted_fragments_with_offset(
                    &fragment.content_rect.start_corner,
                    PositioningContextLength::zero(),
                );
                self.current_positioning_context_mut()
                    .append(positioning_context);
            },
            Either::Second(start_offset) => {
                self.current_positioning_context_mut()
                    .adjust_static_position_of_hoisted_fragments_with_offset(
                        &fragment.content_rect.start_corner,
                        start_offset,
                    );
            },
        }

        self.state.inline_advance += inner_state.inline_advance + pbm_sums.inline_sum();
        self.state.fragments.push(Fragment::Box(fragment));
    }

    fn calculate_inline_box_block_start(
        &self,
        inline_box_state: &InlineBoxContainerState,
        space_above_baseline: Au,
    ) -> Au {
        let font_metrics = &inline_box_state.base.font_metrics;
        let style = &inline_box_state.base.style;
        let line_gap = font_metrics.line_gap;

        // The baseline offset that we have in `Self::baseline_offset` is relative to the line
        // baseline, so we need to make it relative to the line block start.
        match inline_box_state.base.style.clone_vertical_align() {
            GenericVerticalAlign::Keyword(VerticalAlignKeyword::Top) => {
                let line_height: Au = line_height(style, font_metrics).into();
                (line_height - line_gap).scale_by(0.5)
            },
            GenericVerticalAlign::Keyword(VerticalAlignKeyword::Bottom) => {
                let line_height: Au = line_height(style, font_metrics).into();
                let half_leading = (line_height - line_gap).scale_by(0.5);
                self.line_metrics.block_size - line_height + half_leading
            },
            _ => {
                self.line_metrics.baseline_block_offset + inline_box_state.base.baseline_offset -
                    space_above_baseline
            },
        }
    }

    fn layout_text_run(&mut self, text_item: TextRunLineItem) {
        if text_item.text.is_empty() {
            return;
        }

        let mut number_of_justification_opportunities = 0;
        let mut inline_advance = text_item
            .text
            .iter()
            .map(|glyph_store| {
                number_of_justification_opportunities += glyph_store.total_word_separators();
                glyph_store.total_advance()
            })
            .sum();

        if !self.justification_adjustment.is_zero() {
            inline_advance += self
                .justification_adjustment
                .scale_by(number_of_justification_opportunities as f32);
        }

        // The block start of the TextRun is often zero (meaning it has the same font metrics as the
        // inline box's strut), but for children of the inline formatting context root or for
        // fallback fonts that use baseline relative alignment, it might be different.
        let start_corner = LogicalVec2 {
            inline: self.state.inline_advance,
            block: self.state.baseline_offset -
                text_item.font_metrics.ascent -
                self.state.parent_offset.block,
        };

        let rect = LogicalRect {
            start_corner,
            size: LogicalVec2 {
                block: text_item.font_metrics.line_gap,
                inline: inline_advance,
            },
        };

        self.state.inline_advance += inline_advance;
        self.state.fragments.push(Fragment::Text(TextFragment {
            base: text_item.base_fragment_info.into(),
            parent_style: text_item.parent_style,
            rect,
            font_metrics: text_item.font_metrics,
            font_key: text_item.font_key,
            glyphs: text_item.text,
            text_decoration_line: text_item.text_decoration_line,
            justification_adjustment: self.justification_adjustment,
        }));
    }

    fn layout_atomic(&mut self, mut atomic: AtomicLineItem) {
        // The initial `start_corner` of the Fragment is only the PaddingBorderMargin sum start
        // offset, which is the sum of the start component of the padding, border, and margin.
        // This needs to be added to the calculated block and inline positions.
        // Make the final result relative to the parent box.
        atomic.fragment.content_rect.start_corner.inline += self.state.inline_advance;
        atomic.fragment.content_rect.start_corner.block +=
            atomic.calculate_block_start(&self.line_metrics) - self.state.parent_offset.block;

        if atomic.fragment.style.clone_position().is_relative() {
            atomic.fragment.content_rect.start_corner +=
                relative_adjustement(&atomic.fragment.style, self.ifc_containing_block);
        }

        if let Some(mut positioning_context) = atomic.positioning_context {
            positioning_context.adjust_static_position_of_hoisted_fragments_with_offset(
                &atomic.fragment.content_rect.start_corner,
                PositioningContextLength::zero(),
            );
            self.current_positioning_context_mut()
                .append(positioning_context);
        }

        self.state.inline_advance += atomic.size.inline;
        self.state.fragments.push(Fragment::Box(atomic.fragment));
    }

    fn layout_absolute(&mut self, absolute: AbsolutelyPositionedLineItem) {
        let absolutely_positioned_box = (*absolute.absolutely_positioned_box).borrow();
        let style = absolutely_positioned_box.context.style();

        // From https://drafts.csswg.org/css2/#abs-non-replaced-width
        // > The static-position containing block is the containing block of a
        // > hypothetical box that would have been the first box of the element if its
        // > specified position value had been static and its specified float had been
        // > none. (Note that due to the rules in section 9.7 this hypothetical
        // > calculation might require also assuming a different computed value for
        // > display.)
        //
        // This box is different based on the original `display` value of the
        // absolutely positioned element. If it's `inline` it would be placed inline
        // at the top of the line, but if it's block it would be placed in a new
        // block position after the linebox established by this line.
        let initial_start_corner =
            if style.get_box().original_display.outside() == DisplayOutside::Inline {
                // Top of the line at the current inline position.
                LogicalVec2 {
                    inline: self.state.inline_advance,
                    block: -self.state.parent_offset.block,
                }
            } else {
                // After the bottom of the line at the start of the inline formatting context.
                LogicalVec2 {
                    inline: -self.state.parent_offset.inline,
                    block: self.line_metrics.block_size - self.state.parent_offset.block,
                }
            };

        let hoisted_box = AbsolutelyPositionedBox::to_hoisted(
            absolute.absolutely_positioned_box.clone(),
            initial_start_corner.into(),
            self.ifc_containing_block,
        );
        let hoisted_fragment = hoisted_box.fragment.clone();
        self.current_positioning_context_mut().push(hoisted_box);
        self.state
            .fragments
            .push(Fragment::AbsoluteOrFixedPositioned(hoisted_fragment));
    }

    fn layout_float(&mut self, mut float: FloatLineItem) {
        self.state
            .flags
            .insert(LineLayoutInlineContainerFlags::HAD_ANY_FLOATS);

        // The `BoxFragment` for this float is positioned relative to the IFC, so we need
        // to move it to be positioned relative to our parent InlineBox line item. Float
        // fragments are children of these InlineBoxes and not children of the inline
        // formatting context, so that they are parented properly for StackingContext
        // properties such as opacity & filters.
        let distance_from_parent_to_ifc = LogicalVec2 {
            inline: self.state.parent_offset.inline,
            block: self.line_metrics.block_offset + self.state.parent_offset.block,
        };
        float.fragment.content_rect.start_corner -= distance_from_parent_to_ifc;
        self.state.fragments.push(Fragment::Float(float.fragment));
    }
}

pub(super) enum LineItem {
    StartInlineBoxPaddingBorderMargin(InlineBoxIdentifier),
    EndInlineBoxPaddingBorderMargin(InlineBoxIdentifier),
    TextRun(Option<InlineBoxIdentifier>, TextRunLineItem),
    Atomic(Option<InlineBoxIdentifier>, AtomicLineItem),
    AbsolutelyPositioned(Option<InlineBoxIdentifier>, AbsolutelyPositionedLineItem),
    Float(Option<InlineBoxIdentifier>, FloatLineItem),
}

impl LineItem {
    fn inline_box_identifier(&self) -> Option<InlineBoxIdentifier> {
        match self {
            LineItem::StartInlineBoxPaddingBorderMargin(identifier) => Some(*identifier),
            LineItem::EndInlineBoxPaddingBorderMargin(identifier) => Some(*identifier),
            LineItem::TextRun(identifier, _) => *identifier,
            LineItem::Atomic(identifier, _) => *identifier,
            LineItem::AbsolutelyPositioned(identifier, _) => *identifier,
            LineItem::Float(identifier, _) => *identifier,
        }
    }

    pub(super) fn trim_whitespace_at_end(&mut self, whitespace_trimmed: &mut Au) -> bool {
        match self {
            LineItem::StartInlineBoxPaddingBorderMargin(_) => true,
            LineItem::EndInlineBoxPaddingBorderMargin(_) => true,
            LineItem::TextRun(_, ref mut item) => item.trim_whitespace_at_end(whitespace_trimmed),
            LineItem::Atomic(..) => false,
            LineItem::AbsolutelyPositioned(..) => true,
            LineItem::Float(..) => true,
        }
    }

    pub(super) fn trim_whitespace_at_start(&mut self, whitespace_trimmed: &mut Au) -> bool {
        match self {
            LineItem::StartInlineBoxPaddingBorderMargin(_) => true,
            LineItem::EndInlineBoxPaddingBorderMargin(_) => true,
            LineItem::TextRun(_, ref mut item) => item.trim_whitespace_at_start(whitespace_trimmed),
            LineItem::Atomic(..) => false,
            LineItem::AbsolutelyPositioned(..) => true,
            LineItem::Float(..) => true,
        }
    }
}

pub(super) struct TextRunLineItem {
    pub base_fragment_info: BaseFragmentInfo,
    pub parent_style: Arc<ComputedValues>,
    pub text: Vec<std::sync::Arc<GlyphStore>>,
    pub font_metrics: FontMetrics,
    pub font_key: FontInstanceKey,
    pub text_decoration_line: TextDecorationLine,
}

impl TextRunLineItem {
    fn trim_whitespace_at_end(&mut self, whitespace_trimmed: &mut Au) -> bool {
        if matches!(
            self.parent_style.get_inherited_text().white_space_collapse,
            WhiteSpaceCollapse::Preserve | WhiteSpaceCollapse::BreakSpaces
        ) {
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
            .map(|glyph| glyph.total_advance())
            .sum();

        // Only keep going if we only encountered whitespace.
        index_of_last_non_whitespace.is_none()
    }

    fn trim_whitespace_at_start(&mut self, whitespace_trimmed: &mut Au) -> bool {
        if matches!(
            self.parent_style.get_inherited_text().white_space_collapse,
            WhiteSpaceCollapse::Preserve | WhiteSpaceCollapse::BreakSpaces
        ) {
            return false;
        }

        let index_of_first_non_whitespace = self
            .text
            .iter()
            .position(|glyph| !glyph.is_whitespace())
            .unwrap_or(self.text.len());

        *whitespace_trimmed += self
            .text
            .drain(0..index_of_first_non_whitespace)
            .map(|glyph| glyph.total_advance())
            .sum();

        // Only keep going if we only encountered whitespace.
        self.text.is_empty()
    }
}

pub(super) struct AtomicLineItem {
    pub fragment: BoxFragment,
    pub size: LogicalVec2<Au>,
    pub positioning_context: Option<PositioningContext>,

    /// The block offset of this items' baseline relative to the baseline of the line.
    /// This will be zero for boxes with `vertical-align: top` and `vertical-align:
    /// bottom` since their baselines are calculated late in layout.
    pub baseline_offset_in_parent: Au,

    /// The offset of the baseline inside this item.
    pub baseline_offset_in_item: Au,
}

impl AtomicLineItem {
    /// Given the metrics for a line, our vertical alignment, and our block size, find a block start
    /// position relative to the top of the line.
    fn calculate_block_start(&self, line_metrics: &LineMetrics) -> Au {
        match self.fragment.style.clone_vertical_align() {
            GenericVerticalAlign::Keyword(VerticalAlignKeyword::Top) => Au::zero(),
            GenericVerticalAlign::Keyword(VerticalAlignKeyword::Bottom) => {
                line_metrics.block_size - self.size.block
            },

            // This covers all baseline-relative vertical alignment.
            _ => {
                let baseline = line_metrics.baseline_block_offset + self.baseline_offset_in_parent;
                baseline - self.baseline_offset_in_item
            },
        }
    }
}

pub(super) struct AbsolutelyPositionedLineItem {
    pub absolutely_positioned_box: ArcRefCell<AbsolutelyPositionedBox>,
}

pub(super) struct FloatLineItem {
    pub fragment: BoxFragment,
    /// Whether or not this float Fragment has been placed yet. Fragments that
    /// do not fit on a line need to be placed after the hypothetical block start
    /// of the next line.
    pub needs_placement: bool,
}

fn line_height(parent_style: &ComputedValues, font_metrics: &FontMetrics) -> Length {
    let font = parent_style.get_font();
    let font_size = font.font_size.computed_size();
    match font.line_height {
        LineHeight::Normal => Length::from(font_metrics.line_gap),
        LineHeight::Number(number) => font_size * number.0,
        LineHeight::Length(length) => length.0,
    }
}
