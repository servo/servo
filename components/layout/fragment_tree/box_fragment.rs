/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use app_units::{Au, MAX_AU, MIN_AU};
use atomic_refcell::AtomicRefCell;
use base::print_tree::PrintTree;
use malloc_size_of_derive::MallocSizeOf;
use servo_arc::Arc as ServoArc;
use servo_geometry::f32_rect_to_au_rect;
use style::Zero;
use style::computed_values::border_collapse::T as BorderCollapse;
use style::computed_values::overflow_x::T as ComputedOverflow;
use style::computed_values::position::T as ComputedPosition;
use style::logical_geometry::WritingMode;
use style::properties::ComputedValues;
use style::values::specified::box_::DisplayOutside;

use super::{BaseFragment, BaseFragmentInfo, CollapsedBlockMargins, Fragment, FragmentFlags};
use crate::SharedStyle;
use crate::display_list::ToWebRender;
use crate::formatting_contexts::Baselines;
use crate::geom::{
    AuOrAuto, LengthPercentageOrAuto, PhysicalPoint, PhysicalRect, PhysicalSides, ToLogical,
};
use crate::style_ext::ComputedValuesExt;
use crate::table::SpecificTableGridInfo;
use crate::taffy::SpecificTaffyGridInfo;

/// Describes how a [`BoxFragment`] paints its background.
#[derive(MallocSizeOf)]
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
#[derive(MallocSizeOf)]
pub(crate) struct ExtraBackground {
    pub style: SharedStyle,
    pub rect: PhysicalRect<Au>,
}

#[derive(Clone, Debug, MallocSizeOf)]
pub(crate) enum SpecificLayoutInfo {
    Grid(Box<SpecificTaffyGridInfo>),
    TableCellWithCollapsedBorders,
    TableGridWithCollapsedBorders(Box<SpecificTableGridInfo>),
    TableWrapper,
}

#[derive(MallocSizeOf)]
pub(crate) struct BoxFragment {
    pub base: BaseFragment,

    pub style: ServoArc<ComputedValues>,
    pub children: Vec<Fragment>,

    /// The content rect of this fragment in the parent fragment's content rectangle. This
    /// does not include padding, border, or margin -- it only includes content.
    pub content_rect: PhysicalRect<Au>,

    /// This [`BoxFragment`]'s containing block rectangle in coordinates relative to
    /// the initial containing block, but not taking into account any transforms.
    pub cumulative_containing_block_rect: PhysicalRect<Au>,

    pub padding: PhysicalSides<Au>,
    pub border: PhysicalSides<Au>,
    pub margin: PhysicalSides<Au>,

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
    baselines: Baselines,

    block_margins_collapsed_with_children: Option<Box<CollapsedBlockMargins>>,

    /// The scrollable overflow of this box fragment.
    pub scrollable_overflow_from_children: PhysicalRect<Au>,

    /// The resolved box insets if this box is `position: sticky`. These are calculated
    /// during stacking context tree construction because they rely on the size of the
    /// scroll container.
    pub(crate) resolved_sticky_insets: AtomicRefCell<Option<PhysicalSides<AuOrAuto>>>,

    pub background_mode: BackgroundMode,

    /// Additional information of from layout that could be used by Javascripts and devtools.
    pub specific_layout_info: Option<SpecificLayoutInfo>,
}

impl BoxFragment {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        base_fragment_info: BaseFragmentInfo,
        style: ServoArc<ComputedValues>,
        children: Vec<Fragment>,
        content_rect: PhysicalRect<Au>,
        padding: PhysicalSides<Au>,
        border: PhysicalSides<Au>,
        margin: PhysicalSides<Au>,
        clearance: Option<Au>,
    ) -> BoxFragment {
        let scrollable_overflow_from_children =
            children.iter().fold(PhysicalRect::zero(), |acc, child| {
                acc.union(&child.scrollable_overflow_for_parent())
            });

        BoxFragment {
            base: base_fragment_info.into(),
            style,
            children,
            content_rect,
            cumulative_containing_block_rect: Default::default(),
            padding,
            border,
            margin,
            clearance,
            baselines: Baselines::default(),
            block_margins_collapsed_with_children: None,
            scrollable_overflow_from_children,
            resolved_sticky_insets: AtomicRefCell::default(),
            background_mode: BackgroundMode::Normal,
            specific_layout_info: None,
        }
    }

    pub fn with_baselines(mut self, baselines: Baselines) -> Self {
        self.baselines = baselines;
        self
    }

    /// Get the baselines for this [`BoxFragment`] if they are compatible with the given [`WritingMode`].
    /// If they are not compatible, [`Baselines::default()`] is returned.
    pub fn baselines(&self, writing_mode: WritingMode) -> Baselines {
        let mut baselines =
            if writing_mode.is_horizontal() == self.style.writing_mode.is_horizontal() {
                self.baselines
            } else {
                // If the writing mode of the container requesting baselines is not
                // compatible, ensure that the baselines established by this fragment are
                // not used.
                Baselines::default()
            };

        // From the https://drafts.csswg.org/css-align-3/#baseline-export section on "block containers":
        // > However, for legacy reasons if its baseline-source is auto (the initial
        // > value) a block-level or inline-level block container that is a scroll container
        // > always has a last baseline set, whose baselines all correspond to its block-end
        // > margin edge.
        //
        // This applies even if there is no baseline set, so we unconditionally set the value here
        // and ignore anything that is set via [`Self::with_baselines`].
        if self.style.establishes_scroll_container(self.base.flags) {
            let content_rect_size = self.content_rect.size.to_logical(writing_mode);
            let padding = self.padding.to_logical(writing_mode);
            let border = self.border.to_logical(writing_mode);
            let margin = self.margin.to_logical(writing_mode);
            baselines.last = Some(
                content_rect_size.block + padding.block_end + border.block_end + margin.block_end,
            )
        }
        baselines
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

    pub fn with_specific_layout_info(mut self, info: Option<SpecificLayoutInfo>) -> Self {
        self.specific_layout_info = info;
        self
    }

    pub fn with_block_margins_collapsed_with_children(
        mut self,
        collapsed_margins: CollapsedBlockMargins,
    ) -> Self {
        self.block_margins_collapsed_with_children = Some(collapsed_margins.into());
        self
    }

    /// Get the scrollable overflow for this [`BoxFragment`] relative to its
    /// containing block.
    pub fn scrollable_overflow(&self) -> PhysicalRect<Au> {
        let physical_padding_rect = self.padding_rect();
        let content_origin = self.content_rect.origin.to_vector();
        physical_padding_rect.union(
            &self
                .scrollable_overflow_from_children
                .translate(content_origin),
        )
    }

    pub(crate) fn set_containing_block(&mut self, containing_block: &PhysicalRect<Au>) {
        self.cumulative_containing_block_rect = *containing_block;
    }

    pub fn offset_by_containing_block(&self, rect: &PhysicalRect<Au>) -> PhysicalRect<Au> {
        rect.translate(self.cumulative_containing_block_rect.origin.to_vector())
    }

    pub(crate) fn padding_rect(&self) -> PhysicalRect<Au> {
        self.content_rect.outer_rect(self.padding)
    }

    pub(crate) fn border_rect(&self) -> PhysicalRect<Au> {
        self.padding_rect().outer_rect(self.border)
    }

    pub(crate) fn margin_rect(&self) -> PhysicalRect<Au> {
        self.border_rect().outer_rect(self.margin)
    }

    pub(crate) fn padding_border_margin(&self) -> PhysicalSides<Au> {
        self.margin + self.border + self.padding
    }

    pub(crate) fn is_root_element(&self) -> bool {
        self.base.flags.intersects(FragmentFlags::IS_ROOT_ELEMENT)
    }

    pub(crate) fn is_body_element_of_html_element_root(&self) -> bool {
        self.base
            .flags
            .intersects(FragmentFlags::IS_BODY_ELEMENT_OF_HTML_ELEMENT_ROOT)
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
                \noverflow={:?}",
            self.base,
            self.content_rect,
            self.padding_rect(),
            self.border_rect(),
            self.margin,
            self.clearance,
            self.scrollable_overflow(),
            self.baselines,
            self.style.effective_overflow(self.base.flags),
        ));

        for child in &self.children {
            child.print(tree);
        }
        tree.end_level();
    }

    pub fn scrollable_overflow_for_parent(&self) -> PhysicalRect<Au> {
        let mut overflow = self.border_rect();
        if !self.style.establishes_scroll_container(self.base.flags) {
            // https://www.w3.org/TR/css-overflow-3/#scrollable
            // Only include the scrollable overflow of a child box if it has overflow: visible.
            let scrollable_overflow = self.scrollable_overflow();
            let bottom_right = PhysicalPoint::new(
                overflow.max_x().max(scrollable_overflow.max_x()),
                overflow.max_y().max(scrollable_overflow.max_y()),
            );

            let overflow_style = self.style.effective_overflow(self.base.flags);
            if overflow_style.y == ComputedOverflow::Visible {
                overflow.origin.y = overflow.origin.y.min(scrollable_overflow.origin.y);
                overflow.size.height = bottom_right.y - overflow.origin.y;
            }

            if overflow_style.x == ComputedOverflow::Visible {
                overflow.origin.x = overflow.origin.x.min(scrollable_overflow.origin.x);
                overflow.size.width = bottom_right.x - overflow.origin.x;
            }
        }

        // <https://drafts.csswg.org/css-overflow-3/#scrollable-overflow-region>
        // > ...accounting for transforms by projecting each box onto the plane of
        // > the element that establishes its 3D rendering context. [CSS3-TRANSFORMS]
        // Both boxes and its scrollable overflow (if it is included) should be transformed accordingly.
        //
        // TODO(stevennovaryo): We are supposed to handle perspective transform and 3d context, but it is yet to happen.
        if self
            .style
            .has_effective_transform_or_perspective(self.base.flags)
        {
            if let Some(transform) =
                self.calculate_transform_matrix(&self.border_rect().to_untyped())
            {
                if let Some(transformed_overflow_box) =
                    transform.outer_transformed_rect(&overflow.to_webrender().to_rect())
                {
                    overflow =
                        f32_rect_to_au_rect(transformed_overflow_box.to_untyped()).cast_unit();
                }
            }
        }

        overflow
    }

    /// <https://drafts.csswg.org/css-overflow/#unreachable-scrollable-overflow-region>
    /// > area beyond the scroll origin in either axis is considered the unreachable scrollable overflow region
    ///
    /// Return the clipped the scrollable overflow based on its scroll origin, determined by overflow direction.
    /// For an element, the clip rect is the padding rect and for viewport, it is the initial containing block.
    pub fn clip_unreachable_scrollable_overflow_region(
        &self,
        scrollable_overflow: PhysicalRect<Au>,
        clipping_rect: PhysicalRect<Au>,
    ) -> PhysicalRect<Au> {
        let scrolling_direction = self.style.overflow_direction();
        let mut scrollable_overflow_box = scrollable_overflow.to_box2d();
        let mut clipping_box = clipping_rect.to_box2d();

        if scrolling_direction.rightward {
            clipping_box.max.x = MAX_AU;
        } else {
            clipping_box.min.x = MIN_AU;
        }

        if scrolling_direction.downward {
            clipping_box.max.y = MAX_AU;
        } else {
            clipping_box.min.y = MIN_AU;
        }

        scrollable_overflow_box = scrollable_overflow_box.intersection_unchecked(&clipping_box);

        match scrollable_overflow_box.is_negative() {
            true => PhysicalRect::zero(),
            false => scrollable_overflow_box.to_rect(),
        }
    }

    /// <https://drafts.csswg.org/css-overflow/#unreachable-scrollable-overflow-region>
    /// > area beyond the scroll origin in either axis is considered the unreachable scrollable overflow region
    ///
    /// Return the clipped the scrollable overflow based on its scroll origin, determined by overflow direction.
    /// This will coincides with the scrollport if the fragment is a scroll container.
    pub fn reachable_scrollable_overflow_region(&self) -> PhysicalRect<Au> {
        self.clip_unreachable_scrollable_overflow_region(
            self.scrollable_overflow(),
            self.padding_rect(),
        )
    }

    pub(crate) fn calculate_resolved_insets_if_positioned(&self) -> PhysicalSides<AuOrAuto> {
        let position = self.style.get_box().position;
        debug_assert_ne!(
            position,
            ComputedPosition::Static,
            "Should not call this method on statically positioned box."
        );

        if let Some(resolved_sticky_insets) = *self.resolved_sticky_insets.borrow() {
            return resolved_sticky_insets;
        }

        let convert_to_au_or_auto = |sides: PhysicalSides<Au>| {
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
        let insets = self.style.physical_box_offsets();
        let (cb_width, cb_height) = (
            self.cumulative_containing_block_rect.width(),
            self.cumulative_containing_block_rect.height(),
        );
        if position == ComputedPosition::Relative {
            let get_resolved_axis = |start: &LengthPercentageOrAuto,
                                     end: &LengthPercentageOrAuto,
                                     container_length: Au| {
                let start = start.map(|value| value.to_used_value(container_length));
                let end = end.map(|value| value.to_used_value(container_length));
                match (start.non_auto(), end.non_auto()) {
                    (None, None) => (Au::zero(), Au::zero()),
                    (None, Some(end)) => (-end, end),
                    (Some(start), None) => (start, -start),
                    // This is the overconstrained case, for which the resolved insets will
                    // simply be the computed insets.
                    (Some(start), Some(end)) => (start, end),
                }
            };
            let (left, right) = get_resolved_axis(&insets.left, &insets.right, cb_width);
            let (top, bottom) = get_resolved_axis(&insets.top, &insets.bottom, cb_height);
            return convert_to_au_or_auto(PhysicalSides::new(top, right, bottom, left));
        }

        debug_assert!(
            position == ComputedPosition::Fixed || position == ComputedPosition::Absolute
        );

        let margin_rect = self.margin_rect();
        let (top, bottom) = match (&insets.top, &insets.bottom) {
            (
                LengthPercentageOrAuto::LengthPercentage(top),
                LengthPercentageOrAuto::LengthPercentage(bottom),
            ) => (
                top.to_used_value(cb_height),
                bottom.to_used_value(cb_height),
            ),
            _ => (margin_rect.origin.y, cb_height - margin_rect.max_y()),
        };
        let (left, right) = match (&insets.left, &insets.right) {
            (
                LengthPercentageOrAuto::LengthPercentage(left),
                LengthPercentageOrAuto::LengthPercentage(right),
            ) => (left.to_used_value(cb_width), right.to_used_value(cb_width)),
            _ => (margin_rect.origin.x, cb_width - margin_rect.max_x()),
        };

        convert_to_au_or_auto(PhysicalSides::new(top, right, bottom, left))
    }

    /// Whether this is a non-replaced inline-level box whose inner display type is `flow`.
    /// <https://drafts.csswg.org/css-display-3/#inline-box>
    pub(crate) fn is_inline_box(&self) -> bool {
        self.style.is_inline_box(self.base.flags)
    }

    /// Whether this is an atomic inline-level box.
    /// <https://drafts.csswg.org/css-display-3/#atomic-inline>
    pub(crate) fn is_atomic_inline_level(&self) -> bool {
        self.style.get_box().display.outside() == DisplayOutside::Inline && !self.is_inline_box()
    }

    /// Whether this is a table wrapper box.
    /// <https://www.w3.org/TR/css-tables-3/#table-wrapper-box>
    pub(crate) fn is_table_wrapper(&self) -> bool {
        matches!(
            self.specific_layout_info,
            Some(SpecificLayoutInfo::TableWrapper)
        )
    }

    pub(crate) fn has_collapsed_borders(&self) -> bool {
        match &self.specific_layout_info {
            Some(SpecificLayoutInfo::TableCellWithCollapsedBorders) => true,
            Some(SpecificLayoutInfo::TableGridWithCollapsedBorders(_)) => true,
            Some(SpecificLayoutInfo::TableWrapper) => {
                self.style.get_inherited_table().border_collapse == BorderCollapse::Collapse
            },
            _ => false,
        }
    }

    pub(crate) fn block_margins_collapsed_with_children(&self) -> CollapsedBlockMargins {
        match self.block_margins_collapsed_with_children.as_ref() {
            Some(collapsed_block_margins) => *(collapsed_block_margins).clone(),
            _ => CollapsedBlockMargins::zero(),
        }
    }
}
