/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use app_units::Au;
use base::print_tree::PrintTree;
use serde::Serialize;
use servo_arc::Arc as ServoArc;
use style::computed_values::overflow_x::T as ComputedOverflow;
use style::computed_values::position::T as ComputedPosition;
use style::properties::ComputedValues;
use style::values::computed::{CSSPixelLength, Length, LengthPercentage, LengthPercentageOrAuto};
use style::Zero;

use super::{BaseFragment, BaseFragmentInfo, CollapsedBlockMargins, Fragment};
use crate::cell::ArcRefCell;
use crate::formatting_contexts::Baselines;
use crate::geom::{
    AuOrAuto, LogicalRect, LogicalSides, PhysicalPoint, PhysicalRect, PhysicalSides, PhysicalSize,
};
use crate::style_ext::ComputedValuesExt;

/// Describes how a [`BoxFragment`] paints its background.
pub(crate) enum BackgroundMode {
    /// Draw the normal [`BoxFragment`] background as well as the extra backgrounds
    /// based on the style and positioning rectangles in this data structure.
    Extra(Vec<ExtraBackground>),
    /// Do not draw a background for this Fragment. This is used for elements like
    /// table tracks and table track groups, which rely on cells to paint their
    /// backgrounds.
    None,
    /// Draw the background normally, getting information from the Fragment style.
    Normal,
}

pub(crate) struct ExtraBackground {
    pub style: ServoArc<ComputedValues>,
    pub rect: LogicalRect<Au>,
}

#[derive(Serialize)]
pub(crate) struct BoxFragment {
    pub base: BaseFragment,

    #[serde(skip_serializing)]
    pub style: ServoArc<ComputedValues>,
    pub children: Vec<ArcRefCell<Fragment>>,

    /// From the containing block’s start corner…?
    /// This might be broken when the containing block is in a different writing mode:
    /// <https://drafts.csswg.org/css-writing-modes/#orthogonal-flows>
    pub content_rect: LogicalRect<Au>,

    pub padding: LogicalSides<Au>,
    pub border: LogicalSides<Au>,
    pub margin: LogicalSides<Au>,

    /// When the `clear` property is not set to `none`, it may introduce clearance.
    /// Clearance is some extra spacing that is added above the top margin,
    /// so that the element doesn't overlap earlier floats in the same BFC.
    /// The presence of clearance prevents the top margin from collapsing with
    /// earlier margins or with the bottom margin of the parent block.
    /// <https://drafts.csswg.org/css2/#clearance>
    pub clearance: Option<Au>,

    /// When this [`BoxFragment`] is for content that has a baseline, this tracks
    /// the first and last baselines of that content. This is used to propagate baselines
    /// to things such as tables and inline formatting contexts.
    pub baselines: Baselines,

    pub block_margins_collapsed_with_children: CollapsedBlockMargins,

    /// The scrollable overflow of this box fragment.
    pub scrollable_overflow_from_children: PhysicalRect<Au>,

    /// Whether or not this box was overconstrained in the given dimension.
    overconstrained: PhysicalSize<bool>,

    /// The resolved box insets if this box is `position: sticky`. These are calculated
    /// during stacking context tree construction because they rely on the size of the
    /// scroll container.
    pub(crate) resolved_sticky_insets: Option<PhysicalSides<AuOrAuto>>,

    #[serde(skip_serializing)]
    pub background_mode: BackgroundMode,
}

impl BoxFragment {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        base_fragment_info: BaseFragmentInfo,
        style: ServoArc<ComputedValues>,
        children: Vec<Fragment>,
        content_rect: LogicalRect<Au>,
        padding: LogicalSides<Au>,
        border: LogicalSides<Au>,
        margin: LogicalSides<Au>,
        clearance: Option<Au>,
        block_margins_collapsed_with_children: CollapsedBlockMargins,
    ) -> BoxFragment {
        let position = style.get_box().position;
        let insets = style.get_position();
        let width_overconstrained = position == ComputedPosition::Relative &&
            !insets.left.is_auto() &&
            !insets.right.is_auto();
        let height_overconstrained = position == ComputedPosition::Relative &&
            !insets.left.is_auto() &&
            !insets.bottom.is_auto();

        Self::new_with_overconstrained(
            base_fragment_info,
            style,
            children,
            content_rect,
            padding,
            border,
            margin,
            clearance,
            block_margins_collapsed_with_children,
            PhysicalSize::new(width_overconstrained, height_overconstrained),
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new_with_overconstrained(
        base_fragment_info: BaseFragmentInfo,
        style: ServoArc<ComputedValues>,
        children: Vec<Fragment>,
        content_rect: LogicalRect<Au>,
        padding: LogicalSides<Au>,
        border: LogicalSides<Au>,
        margin: LogicalSides<Au>,
        clearance: Option<Au>,
        block_margins_collapsed_with_children: CollapsedBlockMargins,
        overconstrained: PhysicalSize<bool>,
    ) -> BoxFragment {
        // FIXME(mrobinson, bug 25564): We should be using the containing block
        // here to properly convert scrollable overflow to physical geometry.
        let containing_block = PhysicalRect::zero();
        let scrollable_overflow_from_children =
            children.iter().fold(PhysicalRect::zero(), |acc, child| {
                acc.union(&child.scrollable_overflow(&containing_block))
            });

        // From the https://drafts.csswg.org/css-align-3/#baseline-export section on "block containers":
        // > However, for legacy reasons if its baseline-source is auto (the initial
        // > value) a block-level or inline-level block container that is a scroll container
        // > always has a last baseline set, whose baselines all correspond to its block-end
        // > margin edge.
        //
        // This applies even if there is no baseline set, so we unconditionally set the value here
        // and ignore anything that is set via [`Self::with_baselines`].
        let mut baselines = Baselines::default();
        if style.establishes_scroll_container() {
            baselines.last = Some(
                content_rect.size.block + padding.block_end + border.block_end + margin.block_end,
            )
        }

        BoxFragment {
            base: base_fragment_info.into(),
            style,
            children: children.into_iter().map(ArcRefCell::new).collect(),
            content_rect,
            padding,
            border,
            margin,
            clearance,
            baselines,
            block_margins_collapsed_with_children,
            scrollable_overflow_from_children,
            overconstrained,
            resolved_sticky_insets: None,
            background_mode: BackgroundMode::Normal,
        }
    }

    pub fn with_baselines(mut self, baselines: Baselines) -> Self {
        // From the https://drafts.csswg.org/css-align-3/#baseline-export section on "block containers":
        // > However, for legacy reasons if its baseline-source is auto (the initial
        // > value) a block-level or inline-level block container that is a scroll container
        // > always has a last baseline set, whose baselines all correspond to its block-end
        // > margin edge.
        if !self.style.establishes_scroll_container() {
            self.baselines.last = baselines.last;
        }
        self.baselines.first = baselines.first;
        self
    }

    pub fn add_extra_background(&mut self, extra_background: ExtraBackground) {
        match self.background_mode {
            BackgroundMode::Extra(ref mut backgrounds) => backgrounds.push(extra_background),
            _ => self.background_mode = BackgroundMode::Extra(vec![extra_background]),
        }
    }

    pub fn set_does_not_paint_background(&mut self) {
        self.background_mode = BackgroundMode::None;
    }

    pub fn scrollable_overflow(&self, containing_block: &PhysicalRect<Au>) -> PhysicalRect<Au> {
        let physical_padding_rect = self
            .padding_rect()
            .to_physical(self.style.writing_mode, containing_block);

        let content_origin = self
            .content_rect
            .start_corner
            .to_physical(self.style.writing_mode);
        physical_padding_rect.union(
            &self
                .scrollable_overflow_from_children
                .translate(content_origin.to_vector()),
        )
    }

    pub(crate) fn padding_rect(&self) -> LogicalRect<Au> {
        self.content_rect.inflate(&self.padding)
    }

    pub(crate) fn border_rect(&self) -> LogicalRect<Au> {
        self.padding_rect().inflate(&self.border)
    }

    pub(crate) fn margin_rect(&self) -> LogicalRect<Au> {
        self.border_rect().inflate(&self.margin)
    }

    pub fn print(&self, tree: &mut PrintTree) {
        tree.new_level(format!(
            "Box\
                \nbase={:?}\
                \ncontent={:?}\
                \npadding rect={:?}\
                \nborder rect={:?}\
                \nmargin={:?}\
                \nclearance={:?}\
                \nscrollable_overflow={:?}\
                \nbaselines={:?}\
                \noverflow={:?} / {:?}",
            self.base,
            self.content_rect,
            self.padding_rect(),
            self.border_rect(),
            self.margin,
            self.clearance,
            self.scrollable_overflow(&PhysicalRect::zero()),
            self.baselines,
            self.style.get_box().overflow_x,
            self.style.get_box().overflow_y,
        ));

        for child in &self.children {
            child.borrow().print(tree);
        }
        tree.end_level();
    }

    pub fn scrollable_overflow_for_parent(
        &self,
        containing_block: &PhysicalRect<Au>,
    ) -> PhysicalRect<Au> {
        let mut overflow = self
            .border_rect()
            .to_physical(self.style.writing_mode, containing_block);

        if self.style.establishes_scroll_container() {
            return overflow;
        }

        // https://www.w3.org/TR/css-overflow-3/#scrollable
        // Only include the scrollable overflow of a child box if it has overflow: visible.
        let scrollable_overflow = self.scrollable_overflow(containing_block);
        let bottom_right = PhysicalPoint::new(
            overflow.max_x().max(scrollable_overflow.max_x()),
            overflow.max_y().max(scrollable_overflow.max_y()),
        );

        if self.style.get_box().overflow_y == ComputedOverflow::Visible {
            overflow.origin.y = overflow.origin.y.min(scrollable_overflow.origin.y);
            overflow.size.height = bottom_right.y - overflow.origin.y;
        }

        if self.style.get_box().overflow_x == ComputedOverflow::Visible {
            overflow.origin.x = overflow.origin.x.min(scrollable_overflow.origin.x);
            overflow.size.width = bottom_right.x - overflow.origin.x;
        }

        overflow
    }

    pub(crate) fn calculate_resolved_insets_if_positioned(
        &self,
        containing_block: &PhysicalRect<Au>,
    ) -> PhysicalSides<AuOrAuto> {
        let position = self.style.get_box().position;
        debug_assert_ne!(
            position,
            ComputedPosition::Static,
            "Should not call this method on statically positioned box."
        );

        let (cb_width, cb_height) = (containing_block.width(), containing_block.height());
        let content_rect = self
            .content_rect
            .to_physical(self.style.writing_mode, containing_block);

        if let Some(resolved_sticky_insets) = self.resolved_sticky_insets {
            return resolved_sticky_insets;
        }

        let convert_to_length_or_auto = |sides: PhysicalSides<Au>| {
            PhysicalSides::new(
                AuOrAuto::LengthPercentage(sides.top),
                AuOrAuto::LengthPercentage(sides.right),
                AuOrAuto::LengthPercentage(sides.bottom),
                AuOrAuto::LengthPercentage(sides.left),
            )
        };

        // "A resolved value special case property like top defined in another
        // specification If the property applies to a positioned element and the
        // resolved value of the display property is not none or contents, and
        // the property is not over-constrained, then the resolved value is the
        // used value. Otherwise the resolved value is the computed value."
        // https://drafts.csswg.org/cssom/#resolved-values
        let insets = self.style.get_position();
        if position == ComputedPosition::Relative {
            let get_resolved_axis =
                |start: &LengthPercentageOrAuto,
                 end: &LengthPercentageOrAuto,
                 container_length: CSSPixelLength| {
                    let start = start.map(|v| v.percentage_relative_to(container_length));
                    let end = end.map(|v| v.percentage_relative_to(container_length));
                    match (start.non_auto(), end.non_auto()) {
                        (None, None) => (Length::zero(), Length::zero()),
                        (None, Some(end)) => (-end, end),
                        (Some(start), None) => (start, -start),
                        // This is the overconstrained case, for which the resolved insets will
                        // simply be the computed insets.
                        (Some(start), Some(end)) => (start, end),
                    }
                };
            let (left, right) = get_resolved_axis(&insets.left, &insets.right, cb_width.into());
            let (top, bottom) = get_resolved_axis(&insets.top, &insets.bottom, cb_height.into());
            return convert_to_length_or_auto(PhysicalSides::new(
                top.into(),
                right.into(),
                bottom.into(),
                left.into(),
            ));
        }

        debug_assert!(
            position == ComputedPosition::Fixed || position == ComputedPosition::Absolute
        );

        let resolve = |value: &LengthPercentageOrAuto, container_length: Au| -> Au {
            value
                .auto_is(LengthPercentage::zero)
                .percentage_relative_to(container_length.into())
                .into()
        };

        let (top, bottom) = if self.overconstrained.height {
            (
                resolve(&insets.top, cb_height),
                resolve(&insets.bottom, cb_height),
            )
        } else {
            (content_rect.origin.y, cb_height - content_rect.max_y())
        };
        let (left, right) = if self.overconstrained.width {
            (
                resolve(&insets.left, cb_width),
                resolve(&insets.right, cb_width),
            )
        } else {
            (content_rect.origin.x, cb_width - content_rect.max_x())
        };

        convert_to_length_or_auto(PhysicalSides::new(top, right, bottom, left))
    }
}
