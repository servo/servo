/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::sync::Arc;

use gfx::font::FontMetrics;
use gfx::text::glyph::GlyphStore;
use gfx_traits::print_tree::PrintTree;
use msg::constellation_msg::{BrowsingContextId, PipelineId};
use serde::Serialize;
use servo_arc::Arc as ServoArc;
use style::properties::ComputedValues;
use style::values::computed::Length;
use style::values::specified::text::TextDecorationLine;
use style::Zero;
use webrender_api::{FontInstanceKey, ImageKey};

use super::{
    BaseFragment, BoxFragment, ContainingBlockManager, HoistedSharedFragment, PositioningFragment,
    Tag,
};
use crate::cell::ArcRefCell;
use crate::geom::{LogicalRect, LogicalSides, PhysicalRect};
use crate::style_ext::ComputedValuesExt;

#[derive(Serialize)]
pub(crate) enum Fragment {
    Box(BoxFragment),
    /// Floating content. A floated fragment is very similar to a normal
    /// [BoxFragment] but it isn't positioned using normal in block flow
    /// positioning rules (margin collapse, etc). Instead, they are laid
    /// out by the [crate::flow::float::SequentialLayoutState] of their
    /// float containing block formatting context.
    Float(BoxFragment),
    Positioning(PositioningFragment),
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

#[derive(Serialize)]
pub(crate) struct TextFragment {
    pub base: BaseFragment,
    #[serde(skip_serializing)]
    pub parent_style: ServoArc<ComputedValues>,
    pub rect: LogicalRect<Length>,
    pub font_metrics: FontMetrics,
    #[serde(skip_serializing)]
    pub font_key: FontInstanceKey,
    pub glyphs: Vec<Arc<GlyphStore>>,

    /// A flag that represents the _used_ value of the text-decoration property.
    pub text_decoration_line: TextDecorationLine,

    /// Extra space to add for each justification opportunity.
    pub justification_adjustment: Length,
}

#[derive(Serialize)]
pub(crate) struct ImageFragment {
    pub base: BaseFragment,
    #[serde(skip_serializing)]
    pub style: ServoArc<ComputedValues>,
    pub rect: LogicalRect<Length>,
    #[serde(skip_serializing)]
    pub image_key: ImageKey,
}

#[derive(Serialize)]
pub(crate) struct IFrameFragment {
    pub base: BaseFragment,
    pub pipeline_id: PipelineId,
    pub browsing_context_id: BrowsingContextId,
    pub rect: LogicalRect<Length>,
    #[serde(skip_serializing)]
    pub style: ServoArc<ComputedValues>,
}

impl Fragment {
    pub fn base(&self) -> Option<&BaseFragment> {
        Some(match self {
            Fragment::Box(fragment) => &fragment.base,
            Fragment::Text(fragment) => &fragment.base,
            Fragment::AbsoluteOrFixedPositioned(_) => return None,
            Fragment::Positioning(fragment) => &fragment.base,
            Fragment::Image(fragment) => &fragment.base,
            Fragment::IFrame(fragment) => &fragment.base,
            Fragment::Float(fragment) => &fragment.base,
        })
    }

    pub fn tag(&self) -> Option<Tag> {
        self.base().and_then(|base| base.tag)
    }

    pub fn print(&self, tree: &mut PrintTree) {
        match self {
            Fragment::Box(fragment) => fragment.print(tree),
            Fragment::Float(fragment) => {
                tree.new_level("Float".to_string());
                fragment.print(tree);
                tree.end_level();
            },
            Fragment::AbsoluteOrFixedPositioned(_) => {
                tree.add_item("AbsoluteOrFixedPositioned".to_string());
            },
            Fragment::Positioning(fragment) => fragment.print(tree),
            Fragment::Text(fragment) => fragment.print(tree),
            Fragment::Image(fragment) => fragment.print(tree),
            Fragment::IFrame(fragment) => fragment.print(tree),
        }
    }

    pub fn scrolling_area(&self, containing_block: &PhysicalRect<Length>) -> PhysicalRect<Length> {
        match self {
            Fragment::Box(fragment) | Fragment::Float(fragment) => fragment
                .scrollable_overflow(containing_block)
                .translate(containing_block.origin.to_vector()),
            _ => self.scrollable_overflow(containing_block),
        }
    }

    pub fn scrollable_overflow(
        &self,
        containing_block: &PhysicalRect<Length>,
    ) -> PhysicalRect<Length> {
        match self {
            Fragment::Box(fragment) | Fragment::Float(fragment) => {
                fragment.scrollable_overflow_for_parent(containing_block)
            },
            Fragment::AbsoluteOrFixedPositioned(_) => PhysicalRect::zero(),
            Fragment::Positioning(fragment) => fragment.scrollable_overflow,
            Fragment::Text(fragment) => fragment
                .rect
                .to_physical(fragment.parent_style.writing_mode, containing_block),
            Fragment::Image(fragment) => fragment
                .rect
                .to_physical(fragment.style.writing_mode, containing_block),
            Fragment::IFrame(fragment) => fragment
                .rect
                .to_physical(fragment.style.writing_mode, containing_block),
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
            Fragment::Box(fragment) | Fragment::Float(fragment) => {
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
            Fragment::Positioning(fragment) => {
                let content_rect = fragment
                    .rect
                    .to_physical(fragment.writing_mode, containing_block)
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
    pub fn from_margin(margin: &LogicalSides<Length>) -> Self {
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
