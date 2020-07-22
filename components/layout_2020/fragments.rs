/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::cell::ArcRefCell;
use crate::fragment_tree::{BaseFragment, BaseFragmentInfo, ContainingBlockManager, Tag};
use crate::geom::flow_relative::{Rect, Sides};
use crate::geom::{PhysicalPoint, PhysicalRect, PhysicalSides, PhysicalSize};
use crate::positioned::HoistedSharedFragment;
use crate::style_ext::ComputedValuesExt;
use gfx::font::FontMetrics as GfxFontMetrics;
use gfx::text::glyph::GlyphStore;
use gfx_traits::print_tree::PrintTree;
use msg::constellation_msg::{BrowsingContextId, PipelineId};
use servo_arc::Arc as ServoArc;
use std::sync::Arc;
use style::computed_values::overflow_x::T as ComputedOverflow;
use style::computed_values::position::T as ComputedPosition;
use style::logical_geometry::WritingMode;
use style::properties::ComputedValues;
use style::values::computed::{CSSPixelLength, Length, LengthPercentage, LengthPercentageOrAuto};
use style::values::specified::text::TextDecorationLine;
use style::Zero;
use webrender_api::{FontInstanceKey, ImageKey};

#[derive(Serialize)]
pub(crate) enum Fragment {
    Box(BoxFragment),
    // The original document position of a float in the document tree.
    Float,
    // A float hoisted up from its original position (where a placeholder `Fragment::Float` is) to
    // its containing block.
    HoistedFloat(HoistedFloatFragment),
    Anonymous(AnonymousFragment),
    /// Absolute and fixed position fragments are hoisted up so that they
    /// are children of the BoxFragment that establishes their containing
    /// blocks, so that they can be laid out properly. When this happens
    /// an `AbsoluteOrFixedPositioned` fragment is left at the original tree
    /// position. This allows these hoisted fragments to be painted with
    /// regard to their original tree order during stacking context tree /
    /// display list construction.
    AbsoluteOrFixedPositioned(ArcRefCell<HoistedSharedFragment>),
    Text(TextFragment),
    Image(ImageFragment),
    IFrame(IFrameFragment),
}

// A float hoisted up from its original position (where a placeholder `Fragment::Float` is) to its
// containing block.
#[derive(Serialize)]
pub(crate) struct HoistedFloatFragment {
    pub fragment: ArcRefCell<Fragment>,
}

#[derive(Serialize)]
pub(crate) struct BoxFragment {
    pub base: BaseFragment,

    #[serde(skip_serializing)]
    pub style: ServoArc<ComputedValues>,
    pub children: Vec<ArcRefCell<Fragment>>,

    /// From the containing block’s start corner…?
    /// This might be broken when the containing block is in a different writing mode:
    /// https://drafts.csswg.org/css-writing-modes/#orthogonal-flows
    pub content_rect: Rect<Length>,

    pub padding: Sides<Length>,
    pub border: Sides<Length>,
    pub margin: Sides<Length>,

    pub clearance: Length,

    pub block_margins_collapsed_with_children: CollapsedBlockMargins,

    /// The scrollable overflow of this box fragment.
    pub scrollable_overflow_from_children: PhysicalRect<Length>,

    /// Whether or not this box was overconstrained in the given dimension.
    overconstrained: PhysicalSize<bool>,
}

#[derive(Serialize)]
pub(crate) struct FloatFragment {
    pub box_fragment: BoxFragment,
}

#[derive(Serialize)]
pub(crate) struct CollapsedBlockMargins {
    pub collapsed_through: bool,
    pub start: CollapsedMargin,
    pub end: CollapsedMargin,
}

#[derive(Clone, Copy, Debug, Serialize)]
pub(crate) struct CollapsedMargin {
    max_positive: Length,
    min_negative: Length,
}

/// Can contain child fragments with relative coordinates, but does not contribute to painting itself.
#[derive(Serialize)]
pub(crate) struct AnonymousFragment {
    pub base: BaseFragment,
    pub rect: Rect<Length>,
    pub children: Vec<ArcRefCell<Fragment>>,
    pub mode: WritingMode,

    /// The scrollable overflow of this anonymous fragment's children.
    pub scrollable_overflow: PhysicalRect<Length>,
}

#[derive(Clone, Copy, Serialize)]
pub(crate) struct FontMetrics {
    pub ascent: Length,
    pub line_gap: Length,
    pub underline_offset: Length,
    pub underline_size: Length,
    pub strikeout_offset: Length,
    pub strikeout_size: Length,
}

impl From<&GfxFontMetrics> for FontMetrics {
    fn from(metrics: &GfxFontMetrics) -> FontMetrics {
        FontMetrics {
            ascent: metrics.ascent.into(),
            line_gap: metrics.line_gap.into(),
            underline_offset: metrics.underline_offset.into(),
            underline_size: metrics.underline_size.into(),
            strikeout_offset: metrics.strikeout_offset.into(),
            strikeout_size: metrics.strikeout_size.into(),
        }
    }
}

#[derive(Serialize)]
pub(crate) struct TextFragment {
    pub base: BaseFragment,
    #[serde(skip_serializing)]
    pub parent_style: ServoArc<ComputedValues>,
    pub rect: Rect<Length>,
    pub font_metrics: FontMetrics,
    #[serde(skip_serializing)]
    pub font_key: FontInstanceKey,
    pub glyphs: Vec<Arc<GlyphStore>>,
    /// A flag that represents the _used_ value of the text-decoration property.
    pub text_decoration_line: TextDecorationLine,
}

#[derive(Serialize)]
pub(crate) struct ImageFragment {
    pub base: BaseFragment,
    #[serde(skip_serializing)]
    pub style: ServoArc<ComputedValues>,
    pub rect: Rect<Length>,
    #[serde(skip_serializing)]
    pub image_key: ImageKey,
}

#[derive(Serialize)]
pub(crate) struct IFrameFragment {
    pub base: BaseFragment,
    pub pipeline_id: PipelineId,
    pub browsing_context_id: BrowsingContextId,
    pub rect: Rect<Length>,
    #[serde(skip_serializing)]
    pub style: ServoArc<ComputedValues>,
}

impl Fragment {
    pub fn offset_inline(&mut self, offset: &Length) {
        let position = match self {
            Fragment::Box(f) => &mut f.content_rect.start_corner,
            Fragment::HoistedFloat(_) |
            Fragment::Float |
            Fragment::AbsoluteOrFixedPositioned(_) => return,
            Fragment::Anonymous(f) => &mut f.rect.start_corner,
            Fragment::Text(f) => &mut f.rect.start_corner,
            Fragment::Image(f) => &mut f.rect.start_corner,
            Fragment::IFrame(f) => &mut f.rect.start_corner,
        };

        position.inline += *offset;
    }

    pub fn base(&self) -> Option<&BaseFragment> {
        Some(match self {
            Fragment::Box(fragment) => &fragment.base,
            Fragment::Text(fragment) => &fragment.base,
            Fragment::Float => return None,
            Fragment::AbsoluteOrFixedPositioned(_) => return None,
            Fragment::Anonymous(fragment) => &fragment.base,
            Fragment::Image(fragment) => &fragment.base,
            Fragment::IFrame(fragment) => &fragment.base,
            Fragment::HoistedFloat(_) => return None,
        })
    }

    pub fn tag(&self) -> Option<Tag> {
        self.base().and_then(|base| base.tag)
    }

    pub fn print(&self, tree: &mut PrintTree) {
        match self {
            Fragment::Box(fragment) => fragment.print(tree),
            Fragment::HoistedFloat(fragment) => fragment.print(tree),
            Fragment::Float => tree.add_item(format!("Float")),
            Fragment::AbsoluteOrFixedPositioned(_) => {
                tree.add_item("AbsoluteOrFixedPositioned".to_string());
            },
            Fragment::Anonymous(fragment) => fragment.print(tree),
            Fragment::Text(fragment) => fragment.print(tree),
            Fragment::Image(fragment) => fragment.print(tree),
            Fragment::IFrame(fragment) => fragment.print(tree),
        }
    }

    pub fn scrollable_overflow(
        &self,
        containing_block: &PhysicalRect<Length>,
    ) -> PhysicalRect<Length> {
        match self {
            Fragment::Box(fragment) => fragment.scrollable_overflow_for_parent(&containing_block),
            Fragment::HoistedFloat(fragment) => {
                (*fragment.fragment.borrow()).scrollable_overflow(&containing_block)
            },
            Fragment::Float | Fragment::AbsoluteOrFixedPositioned(_) => PhysicalRect::zero(),
            Fragment::Anonymous(fragment) => fragment.scrollable_overflow.clone(),
            Fragment::Text(fragment) => fragment
                .rect
                .to_physical(fragment.parent_style.writing_mode, &containing_block),
            Fragment::Image(fragment) => fragment
                .rect
                .to_physical(fragment.style.writing_mode, &containing_block),
            Fragment::IFrame(fragment) => fragment
                .rect
                .to_physical(fragment.style.writing_mode, &containing_block),
        }
    }

    pub(crate) fn find<T>(
        &self,
        manager: &ContainingBlockManager<PhysicalRect<Length>>,
        level: usize,
        process_func: &mut impl FnMut(&Fragment, usize, &PhysicalRect<Length>) -> Option<T>,
    ) -> Option<T> {
        let containing_block = manager.get_containing_block_for_fragment(self);
        if let Some(result) = process_func(self, level, containing_block) {
            return Some(result);
        }

        match self {
            Fragment::Box(fragment) => {
                let content_rect = fragment
                    .content_rect
                    .to_physical(fragment.style.writing_mode, containing_block)
                    .translate(containing_block.origin.to_vector());
                let padding_rect = fragment
                    .padding_rect()
                    .to_physical(fragment.style.writing_mode, containing_block)
                    .translate(containing_block.origin.to_vector());
                let new_manager = if fragment
                    .style
                    .establishes_containing_block_for_all_descendants()
                {
                    manager.new_for_absolute_and_fixed_descendants(&content_rect, &padding_rect)
                } else if fragment
                    .style
                    .establishes_containing_block_for_absolute_descendants()
                {
                    manager.new_for_absolute_descendants(&content_rect, &padding_rect)
                } else {
                    manager.new_for_non_absolute_descendants(&content_rect)
                };

                fragment
                    .children
                    .iter()
                    .find_map(|child| child.borrow().find(&new_manager, level + 1, process_func))
            },
            Fragment::Anonymous(fragment) => {
                let content_rect = fragment
                    .rect
                    .to_physical(fragment.mode, containing_block)
                    .translate(containing_block.origin.to_vector());
                let new_manager = manager.new_for_non_absolute_descendants(&content_rect);
                fragment
                    .children
                    .iter()
                    .find_map(|child| child.borrow().find(&new_manager, level + 1, process_func))
            },
            _ => None,
        }
    }
}

impl HoistedFloatFragment {
    #[allow(dead_code)]
    pub fn print(&self, tree: &mut PrintTree) {
        tree.new_level(format!("HoistedFloatFragment"));
        self.fragment.borrow().print(tree);
        tree.end_level();
    }
}

impl AnonymousFragment {
    pub fn new(rect: Rect<Length>, children: Vec<Fragment>, mode: WritingMode) -> Self {
        // FIXME(mrobinson, bug 25564): We should be using the containing block
        // here to properly convert scrollable overflow to physical geometry.
        let containing_block = PhysicalRect::zero();
        let content_origin = rect.start_corner.to_physical(mode);
        let scrollable_overflow = children.iter().fold(PhysicalRect::zero(), |acc, child| {
            acc.union(
                &child
                    .scrollable_overflow(&containing_block)
                    .translate(content_origin.to_vector()),
            )
        });
        AnonymousFragment {
            base: BaseFragment::anonymous(),
            rect,
            children: children
                .into_iter()
                .map(|fragment| ArcRefCell::new(fragment))
                .collect(),
            mode,
            scrollable_overflow,
        }
    }

    pub fn print(&self, tree: &mut PrintTree) {
        tree.new_level(format!(
            "Anonymous\
                \nrect={:?}\
                \nscrollable_overflow={:?}",
            self.rect, self.scrollable_overflow
        ));

        for child in &self.children {
            child.borrow().print(tree);
        }
        tree.end_level();
    }
}

impl BoxFragment {
    pub fn new(
        base_fragment_info: BaseFragmentInfo,
        style: ServoArc<ComputedValues>,
        children: Vec<Fragment>,
        content_rect: Rect<Length>,
        padding: Sides<Length>,
        border: Sides<Length>,
        margin: Sides<Length>,
        clearance: Length,
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

    pub fn new_with_overconstrained(
        base_fragment_info: BaseFragmentInfo,
        style: ServoArc<ComputedValues>,
        children: Vec<Fragment>,
        content_rect: Rect<Length>,
        padding: Sides<Length>,
        border: Sides<Length>,
        margin: Sides<Length>,
        clearance: Length,
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

        BoxFragment {
            base: base_fragment_info.into(),
            style,
            children: children
                .into_iter()
                .map(|fragment| ArcRefCell::new(fragment))
                .collect(),
            content_rect,
            padding,
            border,
            margin,
            clearance,
            block_margins_collapsed_with_children,
            scrollable_overflow_from_children,
            overconstrained,
        }
    }

    pub fn scrollable_overflow(
        &self,
        containing_block: &PhysicalRect<Length>,
    ) -> PhysicalRect<Length> {
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

    pub fn padding_rect(&self) -> Rect<Length> {
        self.content_rect.inflate(&self.padding)
    }

    pub fn border_rect(&self) -> Rect<Length> {
        self.padding_rect().inflate(&self.border)
    }

    pub fn print(&self, tree: &mut PrintTree) {
        tree.new_level(format!(
            "Box\
                \nbase={:?}\
                \ncontent={:?}\
                \npadding rect={:?}\
                \nborder rect={:?}\
                \nscrollable_overflow={:?}\
                \noverflow={:?} / {:?}\
                \noverconstrained={:?}
                \nstyle={:p}",
            self.base,
            self.content_rect,
            self.padding_rect(),
            self.border_rect(),
            self.scrollable_overflow(&PhysicalRect::zero()),
            self.style.get_box().overflow_x,
            self.style.get_box().overflow_y,
            self.overconstrained,
            self.style,
        ));

        for child in &self.children {
            child.borrow().print(tree);
        }
        tree.end_level();
    }

    pub fn scrollable_overflow_for_parent(
        &self,
        containing_block: &PhysicalRect<Length>,
    ) -> PhysicalRect<Length> {
        let mut overflow = self
            .border_rect()
            .to_physical(self.style.writing_mode, containing_block);

        if self.style.get_box().overflow_y != ComputedOverflow::Visible &&
            self.style.get_box().overflow_x != ComputedOverflow::Visible
        {
            return overflow;
        }

        // https://www.w3.org/TR/css-overflow-3/#scrollable
        // Only include the scrollable overflow of a child box if it has overflow: visible.
        let scrollable_overflow = self.scrollable_overflow(&containing_block);
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
        containing_block: &PhysicalRect<CSSPixelLength>,
    ) -> PhysicalSides<CSSPixelLength> {
        let position = self.style.get_box().position;
        debug_assert_ne!(
            position,
            ComputedPosition::Static,
            "Should not call this method on statically positioned box."
        );

        let (cb_width, cb_height) = (containing_block.width(), containing_block.height());
        let content_rect = self
            .content_rect
            .to_physical(self.style.writing_mode, &containing_block);

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
            let (left, right) = get_resolved_axis(&insets.left, &insets.right, cb_width);
            let (top, bottom) = get_resolved_axis(&insets.top, &insets.bottom, cb_height);
            return PhysicalSides::new(top, right, bottom, left);
        }

        debug_assert!(
            position == ComputedPosition::Fixed || position == ComputedPosition::Absolute,
            "Got unknown position."
        );

        let resolve = |value: &LengthPercentageOrAuto, container_length| {
            value
                .auto_is(LengthPercentage::zero)
                .percentage_relative_to(container_length)
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

        PhysicalSides::new(top, right, bottom, left)
    }
}

impl TextFragment {
    pub fn print(&self, tree: &mut PrintTree) {
        tree.add_item(format!(
            "Text num_glyphs={}",
            self.glyphs
                .iter()
                .map(|glyph_store| glyph_store.len().0)
                .sum::<isize>()
        ));
    }
}

impl ImageFragment {
    pub fn print(&self, tree: &mut PrintTree) {
        tree.add_item(format!(
            "Image\
                \nrect={:?}",
            self.rect
        ));
    }
}

impl IFrameFragment {
    pub fn print(&self, tree: &mut PrintTree) {
        tree.add_item(format!(
            "IFrame\
                \npipeline={:?} rect={:?}",
            self.pipeline_id, self.rect
        ));
    }
}

impl CollapsedBlockMargins {
    pub fn from_margin(margin: &Sides<Length>) -> Self {
        Self {
            collapsed_through: false,
            start: CollapsedMargin::new(margin.block_start),
            end: CollapsedMargin::new(margin.block_end),
        }
    }

    pub fn zero() -> Self {
        Self {
            collapsed_through: false,
            start: CollapsedMargin::zero(),
            end: CollapsedMargin::zero(),
        }
    }
}

impl CollapsedMargin {
    pub fn zero() -> Self {
        Self {
            max_positive: Length::zero(),
            min_negative: Length::zero(),
        }
    }

    pub fn new(margin: Length) -> Self {
        Self {
            max_positive: margin.max(Length::zero()),
            min_negative: margin.min(Length::zero()),
        }
    }

    pub fn adjoin(&self, other: &Self) -> Self {
        Self {
            max_positive: self.max_positive.max(other.max_positive),
            min_negative: self.min_negative.min(other.min_negative),
        }
    }

    pub fn adjoin_assign(&mut self, other: &Self) {
        *self = self.adjoin(other);
    }

    pub fn solve(&self) -> Length {
        self.max_positive + self.min_negative
    }
}
