/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::fmt::{Debug, Formatter};

use app_units::Au;
use atomic_refcell::AtomicRefCell;
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
pub(crate) struct LayoutBoxBase {
    pub base_fragment_info: BaseFragmentInfo,
    pub style: Arc<ComputedValues>,
    pub cached_inline_content_size:
        AtomicRefCell<Option<(SizeConstraint, InlineContentSizesResult)>>,
    pub cached_layout_result: AtomicRefCell<Option<CacheableLayoutResultAndInputs>>,
}

impl LayoutBoxBase {
    pub(crate) fn new(base_fragment_info: BaseFragmentInfo, style: Arc<ComputedValues>) -> Self {
        Self {
            base_fragment_info,
            style,
            cached_inline_content_size: AtomicRefCell::default(),
            cached_layout_result: AtomicRefCell::default(),
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
        if let Some((previous_cb_block_size, result)) = *cache {
            if !result.depends_on_block_constraints ||
                previous_cb_block_size == constraint_space.block_size
            {
                return result;
            }
            // TODO: Should we keep multiple caches for various block sizes?
        }

        let result = layout_box.compute_inline_content_sizes(layout_context, constraint_space);
        *cache = Some((constraint_space.block_size, result));
        result
    }
}

impl Debug for LayoutBoxBase {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        f.debug_struct("LayoutBoxBase").finish()
    }
}

#[derive(Clone)]
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

pub(crate) struct CacheableLayoutResultAndInputs {
    pub result: CacheableLayoutResult,

    pub containing_block_for_children_size: ContainingBlockSize,

    pub positioning_context: PositioningContext,
}
