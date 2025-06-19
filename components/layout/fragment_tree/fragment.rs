/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::sync::Arc;

use app_units::Au;
use base::id::PipelineId;
use base::print_tree::PrintTree;
use euclid::{Point2D, Rect, Size2D, UnknownUnit};
use fonts::{ByteIndex, FontMetrics, GlyphStore};
use malloc_size_of_derive::MallocSizeOf;
use range::Range as ServoRange;
use servo_arc::Arc as ServoArc;
use style::Zero;
use style::properties::ComputedValues;
use webrender_api::{FontInstanceKey, ImageKey};

use super::{
    BaseFragment, BoxFragment, ContainingBlockManager, HoistedSharedFragment, PositioningFragment,
    Tag,
};
use crate::cell::ArcRefCell;
use crate::flow::inline::SharedInlineStyles;
use crate::geom::{LogicalSides, PhysicalPoint, PhysicalRect};
use crate::style_ext::ComputedValuesExt;

#[derive(Clone, MallocSizeOf)]
pub(crate) enum Fragment {
    Box(ArcRefCell<BoxFragment>),
    /// Floating content. A floated fragment is very similar to a normal
    /// [BoxFragment] but it isn't positioned using normal in block flow
    /// positioning rules (margin collapse, etc). Instead, they are laid
    /// out by the [crate::flow::float::SequentialLayoutState] of their
    /// float containing block formatting context.
    Float(ArcRefCell<BoxFragment>),
    Positioning(ArcRefCell<PositioningFragment>),
    /// Absolute and fixed position fragments are hoisted up so that they
    /// are children of the BoxFragment that establishes their containing
    /// blocks, so that they can be laid out properly. When this happens
    /// an `AbsoluteOrFixedPositioned` fragment is left at the original tree
    /// position. This allows these hoisted fragments to be painted with
    /// regard to their original tree order during stacking context tree /
    /// display list construction.
    AbsoluteOrFixedPositioned(ArcRefCell<HoistedSharedFragment>),
    Text(ArcRefCell<TextFragment>),
    Image(ArcRefCell<ImageFragment>),
    IFrame(ArcRefCell<IFrameFragment>),
}

#[derive(Clone, MallocSizeOf)]
pub(crate) struct CollapsedBlockMargins {
    pub collapsed_through: bool,
    pub start: CollapsedMargin,
    pub end: CollapsedMargin,
}

#[derive(Clone, Copy, Debug, MallocSizeOf)]
pub(crate) struct CollapsedMargin {
    max_positive: Au,
    min_negative: Au,
}

#[derive(MallocSizeOf)]
pub(crate) struct TextFragment {
    pub base: BaseFragment,
    pub inline_styles: SharedInlineStyles,
    pub rect: PhysicalRect<Au>,
    pub font_metrics: FontMetrics,
    pub font_key: FontInstanceKey,
    #[conditional_malloc_size_of]
    pub glyphs: Vec<Arc<GlyphStore>>,

    /// Extra space to add for each justification opportunity.
    pub justification_adjustment: Au,
    pub selection_range: Option<ServoRange<ByteIndex>>,
}

#[derive(MallocSizeOf)]
pub(crate) struct ImageFragment {
    pub base: BaseFragment,
    pub style: ServoArc<ComputedValues>,
    pub rect: PhysicalRect<Au>,
    pub clip: PhysicalRect<Au>,
    pub image_key: Option<ImageKey>,
}

#[derive(MallocSizeOf)]
pub(crate) struct IFrameFragment {
    pub base: BaseFragment,
    pub pipeline_id: PipelineId,
    pub rect: PhysicalRect<Au>,
    pub style: ServoArc<ComputedValues>,
}

impl Fragment {
    pub fn base(&self) -> Option<BaseFragment> {
        Some(match self {
            Fragment::Box(fragment) => fragment.borrow().base.clone(),
            Fragment::Text(fragment) => fragment.borrow().base.clone(),
            Fragment::AbsoluteOrFixedPositioned(_) => return None,
            Fragment::Positioning(fragment) => fragment.borrow().base.clone(),
            Fragment::Image(fragment) => fragment.borrow().base.clone(),
            Fragment::IFrame(fragment) => fragment.borrow().base.clone(),
            Fragment::Float(fragment) => fragment.borrow().base.clone(),
        })
    }

    pub(crate) fn mutate_content_rect(&mut self, callback: impl FnOnce(&mut PhysicalRect<Au>)) {
        match self {
            Fragment::Box(box_fragment) | Fragment::Float(box_fragment) => {
                callback(&mut box_fragment.borrow_mut().content_rect)
            },
            Fragment::Positioning(_) | Fragment::AbsoluteOrFixedPositioned(_) => {},
            Fragment::Text(text_fragment) => callback(&mut text_fragment.borrow_mut().rect),
            Fragment::Image(image_fragment) => callback(&mut image_fragment.borrow_mut().rect),
            Fragment::IFrame(iframe_fragment) => callback(&mut iframe_fragment.borrow_mut().rect),
        }
    }

    pub(crate) fn set_containing_block(&self, containing_block: &PhysicalRect<Au>) {
        match self {
            Fragment::Box(box_fragment) => box_fragment
                .borrow_mut()
                .set_containing_block(containing_block),
            Fragment::Float(float_fragment) => float_fragment
                .borrow_mut()
                .set_containing_block(containing_block),
            Fragment::Positioning(positioning_fragment) => positioning_fragment
                .borrow_mut()
                .set_containing_block(containing_block),
            Fragment::AbsoluteOrFixedPositioned(hoisted_shared_fragment) => {
                if let Some(ref fragment) = hoisted_shared_fragment.borrow().fragment {
                    fragment.set_containing_block(containing_block);
                }
            },
            Fragment::Text(_) => {},
            Fragment::Image(_) => {},
            Fragment::IFrame(_) => {},
        }
    }

    pub fn tag(&self) -> Option<Tag> {
        self.base().and_then(|base| base.tag)
    }

    pub fn print(&self, tree: &mut PrintTree) {
        match self {
            Fragment::Box(fragment) => fragment.borrow().print(tree),
            Fragment::Float(fragment) => {
                tree.new_level("Float".to_string());
                fragment.borrow().print(tree);
                tree.end_level();
            },
            Fragment::AbsoluteOrFixedPositioned(_) => {
                tree.add_item("AbsoluteOrFixedPositioned".to_string());
            },
            Fragment::Positioning(fragment) => fragment.borrow().print(tree),
            Fragment::Text(fragment) => fragment.borrow().print(tree),
            Fragment::Image(fragment) => fragment.borrow().print(tree),
            Fragment::IFrame(fragment) => fragment.borrow().print(tree),
        }
    }

    pub(crate) fn scrolling_area(&self) -> PhysicalRect<Au> {
        match self {
            Fragment::Box(fragment) | Fragment::Float(fragment) => {
                let fragment = fragment.borrow();
                fragment.offset_by_containing_block(&fragment.scrollable_overflow())
            },
            _ => self.scrollable_overflow_for_parent(),
        }
    }

    pub(crate) fn scrollable_overflow_for_parent(&self) -> PhysicalRect<Au> {
        match self {
            Fragment::Box(fragment) | Fragment::Float(fragment) => {
                return fragment.borrow().scrollable_overflow_for_parent();
            },
            Fragment::AbsoluteOrFixedPositioned(_) => PhysicalRect::zero(),
            Fragment::Positioning(fragment) => fragment.borrow().scrollable_overflow_for_parent(),
            Fragment::Text(fragment) => fragment.borrow().rect,
            Fragment::Image(fragment) => fragment.borrow().rect,
            Fragment::IFrame(fragment) => fragment.borrow().rect,
        }
    }

    pub(crate) fn calculate_scrollable_overflow_for_parent(&self) -> PhysicalRect<Au> {
        self.calculate_scrollable_overflow();
        self.scrollable_overflow_for_parent()
    }

    pub(crate) fn calculate_scrollable_overflow(&self) {
        match self {
            Fragment::Box(fragment) | Fragment::Float(fragment) => {
                fragment.borrow_mut().calculate_scrollable_overflow()
            },
            Fragment::Positioning(fragment) => {
                fragment.borrow_mut().calculate_scrollable_overflow()
            },
            _ => {},
        }
    }

    pub(crate) fn cumulative_border_box_rect(&self) -> Option<PhysicalRect<Au>> {
        match self {
            Fragment::Box(fragment) | Fragment::Float(fragment) => {
                let fragment = fragment.borrow();
                Some(fragment.offset_by_containing_block(&fragment.border_rect()))
            },
            Fragment::Positioning(fragment) => {
                let fragment = fragment.borrow();
                Some(fragment.offset_by_containing_block(&fragment.rect))
            },
            Fragment::Text(_) |
            Fragment::AbsoluteOrFixedPositioned(_) |
            Fragment::Image(_) |
            Fragment::IFrame(_) => None,
        }
    }

    pub(crate) fn client_rect(&self) -> Rect<i32, UnknownUnit> {
        let rect = match self {
            Fragment::Box(fragment) | Fragment::Float(fragment) => {
                // https://drafts.csswg.org/cssom-view/#dom-element-clienttop
                // " If the element has no associated CSS layout box or if the
                //   CSS layout box is inline, return zero." For this check we
                // also explicitly ignore the list item portion of the display
                // style.
                let fragment = fragment.borrow();
                if fragment.is_inline_box() {
                    return Rect::zero();
                }

                if fragment.is_table_wrapper() {
                    // For tables the border actually belongs to the table grid box,
                    // so we need to include it in the dimension of the table wrapper box.
                    let mut rect = fragment.border_rect();
                    rect.origin = PhysicalPoint::zero();
                    rect
                } else {
                    let mut rect = fragment.padding_rect();
                    rect.origin = PhysicalPoint::new(fragment.border.left, fragment.border.top);
                    rect
                }
            },
            _ => return Rect::zero(),
        }
        .to_untyped();

        let rect = Rect::new(
            Point2D::new(rect.origin.x.to_f32_px(), rect.origin.y.to_f32_px()),
            Size2D::new(rect.size.width.to_f32_px(), rect.size.height.to_f32_px()),
        );
        rect.round().to_i32()
    }

    pub(crate) fn find<T>(
        &self,
        manager: &ContainingBlockManager<PhysicalRect<Au>>,
        level: usize,
        process_func: &mut impl FnMut(&Fragment, usize, &PhysicalRect<Au>) -> Option<T>,
    ) -> Option<T> {
        let containing_block = manager.get_containing_block_for_fragment(self);
        if let Some(result) = process_func(self, level, containing_block) {
            return Some(result);
        }

        match self {
            Fragment::Box(fragment) | Fragment::Float(fragment) => {
                let fragment = fragment.borrow();
                let content_rect = fragment
                    .content_rect
                    .translate(containing_block.origin.to_vector());
                let padding_rect = fragment
                    .padding_rect()
                    .translate(containing_block.origin.to_vector());
                let new_manager = if fragment
                    .style
                    .establishes_containing_block_for_all_descendants(fragment.base.flags)
                {
                    manager.new_for_absolute_and_fixed_descendants(&content_rect, &padding_rect)
                } else if fragment
                    .style
                    .establishes_containing_block_for_absolute_descendants(fragment.base.flags)
                {
                    manager.new_for_absolute_descendants(&content_rect, &padding_rect)
                } else {
                    manager.new_for_non_absolute_descendants(&content_rect)
                };

                fragment
                    .children
                    .iter()
                    .find_map(|child| child.find(&new_manager, level + 1, process_func))
            },
            Fragment::Positioning(fragment) => {
                let fragment = fragment.borrow();
                let content_rect = fragment.rect.translate(containing_block.origin.to_vector());
                let new_manager = manager.new_for_non_absolute_descendants(&content_rect);
                fragment
                    .children
                    .iter()
                    .find_map(|child| child.find(&new_manager, level + 1, process_func))
            },
            _ => None,
        }
    }

    pub(crate) fn repair_style(&self, style: &ServoArc<ComputedValues>) {
        match self {
            Fragment::Box(box_fragment) | Fragment::Float(box_fragment) => {
                box_fragment.borrow_mut().style = style.clone()
            },
            Fragment::Positioning(positioning_fragment) => {
                positioning_fragment.borrow_mut().style = style.clone();
            },
            Fragment::AbsoluteOrFixedPositioned(positioned_fragment) => {
                if let Some(ref fragment) = positioned_fragment.borrow().fragment {
                    fragment.repair_style(style);
                }
            },
            Fragment::Text(..) => unreachable!("Should never try to repair style of TextFragment"),
            Fragment::Image(image_fragment) => image_fragment.borrow_mut().style = style.clone(),
            Fragment::IFrame(iframe_fragment) => iframe_fragment.borrow_mut().style = style.clone(),
        }
    }
}

impl TextFragment {
    pub fn print(&self, tree: &mut PrintTree) {
        tree.add_item(format!(
            "Text num_glyphs={} box={:?}",
            self.glyphs
                .iter()
                .map(|glyph_store| glyph_store.len().0)
                .sum::<isize>(),
            self.rect,
        ));
    }

    pub fn has_selection(&self) -> bool {
        self.selection_range.is_some()
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
    pub fn from_margin(margin: &LogicalSides<Au>) -> Self {
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
            max_positive: Au::zero(),
            min_negative: Au::zero(),
        }
    }

    pub fn new(margin: Au) -> Self {
        Self {
            max_positive: margin.max(Au::zero()),
            min_negative: margin.min(Au::zero()),
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

    pub fn solve(&self) -> Au {
        self.max_positive + self.min_negative
    }
}
