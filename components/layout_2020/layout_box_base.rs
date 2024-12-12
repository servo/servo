/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use atomic_refcell::AtomicRefCell;
use serde::Serialize;
use servo_arc::Arc;
use style::properties::ComputedValues;

use crate::context::LayoutContext;
use crate::fragment_tree::BaseFragmentInfo;
use crate::geom::SizeConstraint;
use crate::sizing::{ComputeInlineContentSizes, InlineContentSizesResult};
use crate::ConstraintSpace;

/// A box tree node that handles containing information about style and the original DOM
/// node or pseudo-element that it is based on. This also handles caching of layout values
/// such as the inline content sizes to avoid recalculating these values during layout
/// passes.
///
/// In the future, this will hold layout results to support incremental layout.
#[derive(Debug, Serialize)]
pub(crate) struct LayoutBoxBase {
    pub base_fragment_info: BaseFragmentInfo,
    #[serde(skip_serializing)]
    pub style: Arc<ComputedValues>,
    #[serde(skip_serializing)]
    pub cached_inline_content_size:
        AtomicRefCell<Option<(SizeConstraint, InlineContentSizesResult)>>,
}

impl LayoutBoxBase {
    pub(crate) fn new(base_fragment_info: BaseFragmentInfo, style: Arc<ComputedValues>) -> Self {
        Self {
            base_fragment_info,
            style,
            cached_inline_content_size: AtomicRefCell::default(),
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
