/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::fmt::{Debug, Formatter};

use app_units::Au;
use atomic_refcell::AtomicRefCell;
use malloc_size_of_derive::MallocSizeOf;
use servo_arc::Arc;
use style::properties::ComputedValues;

use crate::context::LayoutContext;
use crate::formatting_contexts::Baselines;
use crate::fragment_tree::{BaseFragmentInfo, CollapsedBlockMargins, Fragment, SpecificLayoutInfo};
use crate::geom::SizeConstraint;
use crate::positioned::PositioningContext;
use crate::sizing::{ComputeInlineContentSizes, InlineContentSizesResult};
use crate::{ConstraintSpace, ContainingBlockSize};

/// A box tree node that handles containing information about style and the original DOM
/// node or pseudo-element that it is based on. This also handles caching of layout values
/// such as the inline content sizes to avoid recalculating these values during layout
/// passes.
///
/// In the future, this will hold layout results to support incremental layout.
#[derive(MallocSizeOf)]
pub(crate) struct LayoutBoxBase {
    pub base_fragment_info: BaseFragmentInfo,
    pub style: Arc<ComputedValues>,
    pub cached_inline_content_size:
        AtomicRefCell<Option<Box<(SizeConstraint, InlineContentSizesResult)>>>,
    pub cached_layout_result: AtomicRefCell<Option<Box<CacheableLayoutResultAndInputs>>>,
    pub fragments: AtomicRefCell<Vec<Fragment>>,
}

impl LayoutBoxBase {
    pub(crate) fn new(base_fragment_info: BaseFragmentInfo, style: Arc<ComputedValues>) -> Self {
        Self {
            base_fragment_info,
            style,
            cached_inline_content_size: AtomicRefCell::default(),
            cached_layout_result: AtomicRefCell::default(),
            fragments: AtomicRefCell::default(),
        }
    }

    /// Get the inline content sizes of a box tree node that extends this [`LayoutBoxBase`], fetch
    /// the result from a cache when possible.
    pub(crate) fn inline_content_sizes(
        &self,
        layout_context: &LayoutContext,
        constraint_space: &ConstraintSpace,
        layout_box: &impl ComputeInlineContentSizes,
    ) -> InlineContentSizesResult {
        let mut cache = self.cached_inline_content_size.borrow_mut();
        if let Some(cached_inline_content_size) = cache.as_ref() {
            let (previous_cb_block_size, result) = **cached_inline_content_size;
            if !result.depends_on_block_constraints ||
                previous_cb_block_size == constraint_space.block_size
            {
                return result;
            }
            // TODO: Should we keep multiple caches for various block sizes?
        }

        let result =
            layout_box.compute_inline_content_sizes_with_fixup(layout_context, constraint_space);
        *cache = Some(Box::new((constraint_space.block_size, result)));
        result
    }

    /// Clear cached data accumulated during fragment tree layout, either fragments and
    /// the cached inline content size, or just fragments.
    pub(crate) fn clear_fragment_layout_cache(&self) {
        self.fragments.borrow_mut().clear();
        *self.cached_layout_result.borrow_mut() = None;
        *self.cached_inline_content_size.borrow_mut() = None;
    }

    pub(crate) fn fragments(&self) -> Vec<Fragment> {
        self.fragments.borrow().clone()
    }

    pub(crate) fn add_fragment(&self, fragment: Fragment) {
        self.fragments.borrow_mut().push(fragment);
    }

    pub(crate) fn set_fragment(&self, fragment: Fragment) {
        *self.fragments.borrow_mut() = vec![fragment];
    }

    pub(crate) fn clear_fragments(&self) {
        self.fragments.borrow_mut().clear();
    }

    pub(crate) fn repair_style(&mut self, new_style: &Arc<ComputedValues>) {
        self.style = new_style.clone();
        for fragment in self.fragments.borrow_mut().iter_mut() {
            fragment.repair_style(new_style);
        }
    }
}

impl Debug for LayoutBoxBase {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        f.debug_struct("LayoutBoxBase").finish()
    }
}

#[derive(Clone, MallocSizeOf)]
pub(crate) struct CacheableLayoutResult {
    pub fragments: Vec<Fragment>,

    /// <https://drafts.csswg.org/css2/visudet.html#root-height>
    pub content_block_size: Au,

    /// If this layout is for a block container, this tracks the collapsable size
    /// of start and end margins and whether or not the block container collapsed through.
    pub collapsible_margins_in_children: CollapsedBlockMargins,

    /// The contents of a table may force it to become wider than what we would expect
    /// from 'width' and 'min-width'. This is the resulting inline content size,
    /// or None for non-table layouts.
    pub content_inline_size_for_table: Option<Au>,

    /// The offset of the last inflow baseline of this layout in the content area, if
    /// there was one. This is used to propagate baselines to the ancestors of `display:
    /// inline-block`.
    pub baselines: Baselines,

    /// Whether or not this layout depends on the containing block size.
    pub depends_on_block_constraints: bool,

    /// Additional information of this layout that could be used by Javascripts and devtools.
    pub specific_layout_info: Option<SpecificLayoutInfo>,
}

/// A collection of layout inputs and a cached layout result for a [`LayoutBoxBase`].
#[derive(MallocSizeOf)]
pub(crate) struct CacheableLayoutResultAndInputs {
    /// The [`CacheableLayoutResult`] for this layout.
    pub result: CacheableLayoutResult,

    /// The [`ContainingBlockSize`] to use for this box's contents, but not
    /// for the box itself.
    pub containing_block_for_children_size: ContainingBlockSize,

    /// A [`PositioningContext`] holding absolutely-positioned descendants
    /// collected during the layout of this box.
    pub positioning_context: PositioningContext,
}
