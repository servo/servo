/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::vec::IntoIter;

use app_units::Au;
use atomic_refcell::AtomicRef;
use gfx::font::FontMetrics;
use gfx::text::glyph::GlyphStore;
use servo_arc::Arc;
use style::properties::ComputedValues;
use style::values::computed::Length;
use style::values::generics::box_::{GenericVerticalAlign, VerticalAlignKeyword};
use style::values::generics::font::LineHeight;
use style::values::specified::box_::DisplayOutside;
use style::values::specified::text::TextDecorationLine;
use style::Zero;
use webrender_api::FontInstanceKey;

use crate::cell::ArcRefCell;
use crate::context::LayoutContext;
use crate::fragment_tree::{
    BaseFragmentInfo, BoxFragment, CollapsedBlockMargins, Fragment, HoistedSharedFragment,
    TextFragment,
};
use crate::geom::{LogicalRect, LogicalVec2};
use crate::positioned::{
    relative_adjustement, AbsolutelyPositionedBox, PositioningContext, PositioningContextLength,
};
use crate::style_ext::PaddingBorderMargin;
use crate::ContainingBlock;

pub(super) struct LineMetrics {
    /// The block offset of the line start in the containing
    /// [`crate::flow::InlineFormattingContext`].
    pub block_offset: Length,

    /// The block size of this line.
    pub block_size: Length,

    /// The block offset of this line's baseline from [`Self::block_offset`].
    pub baseline_block_offset: Au,
}

/// State used when laying out the [`LineItem`]s collected for the line currently being
/// laid out.
pub(super) struct LineItemLayoutState<'a> {
    pub inline_position: Length,

    /// The offset of the parent, relative to the start position of the line.
    pub parent_offset: LogicalVec2<Length>,

    /// The block offset of the parent's baseline relative to the block start of the line. This
    /// is often the same as [`Self::parent_offset`], but can be different for the root
    /// element.
    pub baseline_offset: Au,

    pub ifc_containing_block: &'a ContainingBlock<'a>,
    pub positioning_context: &'a mut PositioningContext,

    /// The amount of space to add to each justification opportunity in order to implement
    /// `text-align: justify`.
    pub justification_adjustment: Length,

    /// The metrics of this line, which should remain constant throughout the
    /// layout process.
    pub line_metrics: &'a LineMetrics,
}

pub(super) fn layout_line_items(
    iterator: &mut IntoIter<LineItem>,
    layout_context: &LayoutContext,
    state: &mut LineItemLayoutState,
    saw_end: &mut bool,
) -> Vec<Fragment> {
    let mut fragments = vec![];
    while let Some(item) = iterator.next() {
        match item {
            LineItem::TextRun(text_line_item) => {
                if let Some(fragment) = text_line_item.layout(state) {
                    fragments.push(Fragment::Text(fragment));
                }
            },
            LineItem::StartInlineBox(box_line_item) => {
                if let Some(fragment) = box_line_item.layout(iterator, layout_context, state) {
                    fragments.push(Fragment::Box(fragment))
                }
            },
            LineItem::EndInlineBox => {
                *saw_end = true;
                break;
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

pub(super) enum LineItem {
    TextRun(TextRunLineItem),
    StartInlineBox(InlineBoxLineItem),
    EndInlineBox,
    Atomic(AtomicLineItem),
    AbsolutelyPositioned(AbsolutelyPositionedLineItem),
    Float(FloatLineItem),
}

impl LineItem {
    pub(super) fn trim_whitespace_at_end(&mut self, whitespace_trimmed: &mut Length) -> bool {
        match self {
            LineItem::TextRun(ref mut item) => item.trim_whitespace_at_end(whitespace_trimmed),
            LineItem::StartInlineBox(_) => true,
            LineItem::EndInlineBox => true,
            LineItem::Atomic(_) => false,
            LineItem::AbsolutelyPositioned(_) => true,
            LineItem::Float(_) => true,
        }
    }

    pub(super) fn trim_whitespace_at_start(&mut self, whitespace_trimmed: &mut Length) -> bool {
        match self {
            LineItem::TextRun(ref mut item) => item.trim_whitespace_at_start(whitespace_trimmed),
            LineItem::StartInlineBox(_) => true,
            LineItem::EndInlineBox => true,
            LineItem::Atomic(_) => false,
            LineItem::AbsolutelyPositioned(_) => true,
            LineItem::Float(_) => true,
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

    fn trim_whitespace_at_start(&mut self, whitespace_trimmed: &mut Length) -> bool {
        if self
            .parent_style
            .get_inherited_text()
            .white_space
            .preserve_spaces()
        {
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
            .map(|glyph| Length::from(glyph.total_advance()))
            .sum();

        // Only keep going if we only encountered whitespace.
        self.text.is_empty()
    }

    fn layout(self, state: &mut LineItemLayoutState) -> Option<TextFragment> {
        if self.text.is_empty() {
            return None;
        }

        let mut number_of_justification_opportunities = 0;
        let mut inline_advance: Length = self
            .text
            .iter()
            .map(|glyph_store| {
                number_of_justification_opportunities += glyph_store.total_word_separators();
                Length::from(glyph_store.total_advance())
            })
            .sum();

        if !state.justification_adjustment.is_zero() {
            inline_advance +=
                state.justification_adjustment * number_of_justification_opportunities as f32;
        }

        // The block start of the TextRun is often zero (meaning it has the same font metrics as the
        // inline box's strut), but for children of the inline formatting context root or for
        // fallback fonts that use baseline relatve alignment, it might be different.
        let start_corner = &LogicalVec2 {
            inline: state.inline_position,
            block: (state.baseline_offset - self.font_metrics.ascent).into(),
        } - &state.parent_offset;

        let rect = LogicalRect {
            start_corner,
            size: LogicalVec2 {
                block: self.font_metrics.line_gap.into(),
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
            justification_adjustment: state.justification_adjustment,
        })
    }
}

#[derive(Clone)]
pub(super) struct InlineBoxLineItem {
    pub base_fragment_info: BaseFragmentInfo,
    pub style: Arc<ComputedValues>,
    pub pbm: PaddingBorderMargin,

    /// Whether this is the first fragment for this inline box. This means that it's the
    /// first potentially split box of a block-in-inline-split (or only if there's no
    /// split) and also the first appearance of this fragment on any line.
    pub is_first_fragment: bool,

    /// Whether this is the last fragment for this inline box. This means that it's the
    /// last potentially split box of a block-in-inline-split (or the only fragment if
    /// there's no split).
    pub is_last_fragment_of_ib_split: bool,

    /// The FontMetrics for the default font used in this inline box.
    pub font_metrics: FontMetrics,

    /// The block offset of this baseline relative to the baseline of the line. This will be
    /// zero for boxes with `vertical-align: top` and `vertical-align: bottom` since their
    /// baselines are calculated late in layout.
    pub baseline_offset: Au,
}

impl InlineBoxLineItem {
    fn layout(
        self,
        iterator: &mut IntoIter<LineItem>,
        layout_context: &LayoutContext,
        state: &mut LineItemLayoutState,
    ) -> Option<BoxFragment> {
        let style = self.style.clone();
        let mut padding = self.pbm.padding.clone();
        let mut border = self.pbm.border.clone();
        let mut margin = self.pbm.margin.auto_is(Au::zero);

        if !self.is_first_fragment {
            padding.inline_start = Au::zero();
            border.inline_start = Au::zero();
            margin.inline_start = Au::zero();
        }
        if !self.is_last_fragment_of_ib_split {
            padding.inline_end = Au::zero();
            border.inline_end = Au::zero();
            margin.inline_end = Au::zero();
        }
        let pbm_sums = &(&padding + &border) + &margin;
        state.inline_position += pbm_sums.inline_start.into();

        let space_above_baseline = self.calculate_space_above_baseline();
        let block_start_offset = self.calculate_block_start(state, space_above_baseline);

        let mut positioning_context = PositioningContext::new_for_style(&style);
        let nested_positioning_context = match positioning_context.as_mut() {
            Some(positioning_context) => positioning_context,
            None => &mut state.positioning_context,
        };
        let original_nested_positioning_context_length = nested_positioning_context.len();

        let mut nested_state = LineItemLayoutState {
            inline_position: state.inline_position,
            parent_offset: LogicalVec2 {
                inline: state.inline_position,
                block: block_start_offset.into(),
            },
            ifc_containing_block: state.ifc_containing_block,
            positioning_context: nested_positioning_context,
            justification_adjustment: state.justification_adjustment,
            line_metrics: state.line_metrics,
            baseline_offset: block_start_offset + space_above_baseline,
        };

        let mut saw_end = false;
        let fragments =
            layout_line_items(iterator, layout_context, &mut nested_state, &mut saw_end);

        // Only add ending padding, border, margin if this is the last fragment of a
        // potential block-in-inline split and this line included the actual end of this
        // fragment (it doesn't continue on the next line).
        if !self.is_last_fragment_of_ib_split || !saw_end {
            padding.inline_end = Au::zero();
            border.inline_end = Au::zero();
            margin.inline_end = Au::zero();
        }
        let pbm_sums = &(&padding + &border) + &margin.clone();

        // If the inline box didn't have any content at all, don't add a Fragment for it.
        let box_has_padding_border_or_margin = pbm_sums.inline_sum() > Au::zero();
        let box_had_absolutes =
            original_nested_positioning_context_length != nested_state.positioning_context.len();
        if !self.is_first_fragment &&
            fragments.is_empty() &&
            !box_has_padding_border_or_margin &&
            !box_had_absolutes
        {
            return None;
        }

        let mut content_rect = LogicalRect {
            start_corner: LogicalVec2 {
                inline: state.inline_position,
                block: block_start_offset.into(),
            },
            size: LogicalVec2 {
                inline: nested_state.inline_position - state.inline_position,
                block: self.font_metrics.line_gap.into(),
            },
        };

        // Make `content_rect` relative to the parent Fragment.
        content_rect.start_corner = &content_rect.start_corner - &state.parent_offset;

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
            None, /* clearance */
            CollapsedBlockMargins::zero(),
        );

        state.inline_position = nested_state.inline_position + pbm_sums.inline_end.into();

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

    /// Given our font metrics, calculate the space above the baseline we need for our content.
    /// Note that this space does not include space for any content in child inline boxes, as
    /// they are not included in our content rect.
    fn calculate_space_above_baseline(&self) -> Au {
        let (ascent, descent, line_gap) = (
            self.font_metrics.ascent,
            self.font_metrics.descent,
            self.font_metrics.line_gap,
        );
        let leading = line_gap - (ascent + descent);
        leading.scale_by(0.5) + ascent
    }

    /// Given the state for a line item layout and the space above the baseline for this inline
    /// box, find the block start position relative to the line block start position.
    fn calculate_block_start(&self, state: &LineItemLayoutState, space_above_baseline: Au) -> Au {
        let line_gap = self.font_metrics.line_gap;

        // The baseline offset that we have in `Self::baseline_offset` is relative to the line
        // baseline, so we need to make it relative to the line block start.
        match self.style.clone_vertical_align() {
            GenericVerticalAlign::Keyword(VerticalAlignKeyword::Top) => {
                let line_height: Au = line_height(&self.style, &self.font_metrics).into();
                (line_height - line_gap).scale_by(0.5)
            },
            GenericVerticalAlign::Keyword(VerticalAlignKeyword::Bottom) => {
                let line_height: Au = line_height(&self.style, &self.font_metrics).into();
                let half_leading = (line_height - line_gap).scale_by(0.5);
                Au::from(state.line_metrics.block_size) - line_height + half_leading
            },
            _ => {
                state.line_metrics.baseline_block_offset + self.baseline_offset -
                    space_above_baseline
            },
        }
    }
}

pub(super) struct AtomicLineItem {
    pub fragment: BoxFragment,
    pub size: LogicalVec2<Length>,
    pub positioning_context: Option<PositioningContext>,

    /// The block offset of this items' baseline relative to the baseline of the line.
    /// This will be zero for boxes with `vertical-align: top` and `vertical-align:
    /// bottom` since their baselines are calculated late in layout.
    pub baseline_offset_in_parent: Au,

    /// The offset of the baseline inside this item.
    pub baseline_offset_in_item: Au,
}

impl AtomicLineItem {
    fn layout(mut self, state: &mut LineItemLayoutState) -> BoxFragment {
        // The initial `start_corner` of the Fragment is only the PaddingBorderMargin sum start
        // offset, which is the sum of the start component of the padding, border, and margin.
        // This needs to be added to the calculated block and inline positions.
        self.fragment.content_rect.start_corner.inline += state.inline_position;
        self.fragment.content_rect.start_corner.block +=
            self.calculate_block_start(state.line_metrics);

        // Make the final result relative to the parent box.
        self.fragment.content_rect.start_corner =
            &self.fragment.content_rect.start_corner - &state.parent_offset;

        if self.fragment.style.clone_position().is_relative() {
            self.fragment.content_rect.start_corner +=
                &relative_adjustement(&self.fragment.style, state.ifc_containing_block);
        }

        state.inline_position += self.size.inline;

        if let Some(mut positioning_context) = self.positioning_context {
            positioning_context.adjust_static_position_of_hoisted_fragments_with_offset(
                &self.fragment.content_rect.start_corner,
                PositioningContextLength::zero(),
            );
            state.positioning_context.append(positioning_context);
        }

        self.fragment
    }

    /// Given the metrics for a line, our vertical alignment, and our block size, find a block start
    /// position relative to the top of the line.
    fn calculate_block_start(&self, line_metrics: &LineMetrics) -> Length {
        match self.fragment.style.clone_vertical_align() {
            GenericVerticalAlign::Keyword(VerticalAlignKeyword::Top) => Length::zero(),
            GenericVerticalAlign::Keyword(VerticalAlignKeyword::Bottom) => {
                line_metrics.block_size - self.size.block
            },

            // This covers all baseline-relative vertical alignment.
            _ => {
                let baseline = line_metrics.baseline_block_offset + self.baseline_offset_in_parent;
                Length::from(baseline - self.baseline_offset_in_item)
            },
        }
    }
}

pub(super) struct AbsolutelyPositionedLineItem {
    pub absolutely_positioned_box: ArcRefCell<AbsolutelyPositionedBox>,
}

impl AbsolutelyPositionedLineItem {
    fn layout(self, state: &mut LineItemLayoutState) -> ArcRefCell<HoistedSharedFragment> {
        let box_ = self.absolutely_positioned_box;
        let style = AtomicRef::map(box_.borrow(), |box_| box_.context.style());

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
                    inline: state.inline_position - state.parent_offset.inline,
                    block: -state.parent_offset.block,
                }
            } else {
                // After the bottom of the line at the start of the inline formatting context.
                LogicalVec2 {
                    inline: Length::zero(),
                    block: state.line_metrics.block_size - state.parent_offset.block,
                }
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

pub(super) struct FloatLineItem {
    pub fragment: BoxFragment,
    /// Whether or not this float Fragment has been placed yet. Fragments that
    /// do not fit on a line need to be placed after the hypothetical block start
    /// of the next line.
    pub needs_placement: bool,
}

impl FloatLineItem {
    fn layout(mut self, state: &mut LineItemLayoutState<'_>) -> BoxFragment {
        // The `BoxFragment` for this float is positioned relative to the IFC, so we need
        // to move it to be positioned relative to our parent InlineBox line item. Floats
        // fragments are children of these InlineBoxes and not children of the inline
        // formatting context, so that they are parented properly for StackingContext
        // properties such as opacity & filters.
        let distance_from_parent_to_ifc = LogicalVec2 {
            inline: state.parent_offset.inline,
            block: state.line_metrics.block_offset + state.parent_offset.block,
        };
        self.fragment.content_rect.start_corner =
            &self.fragment.content_rect.start_corner - &distance_from_parent_to_ifc;
        self.fragment
    }
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
