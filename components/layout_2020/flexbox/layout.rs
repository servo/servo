/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use super::FlexContainer;
use crate::context::LayoutContext;
use crate::formatting_contexts::IndependentLayout;
use crate::positioned::PositioningContext;
use crate::ContainingBlock;
use style::values::computed::Length;
use style::Zero;

// FIXME: `min-width: auto` is not zero: https://drafts.csswg.org/css-flexbox/#min-size-auto

impl FlexContainer {
    pub(crate) fn layout(
        &self,
        layout_context: &LayoutContext,
        positioning_context: &mut PositioningContext,
        containing_block: &ContainingBlock,
        tree_rank: usize,
    ) -> IndependentLayout {
        // FIXME
        let _ = layout_context;
        let _ = positioning_context;
        let _ = containing_block;
        let _ = tree_rank;
        IndependentLayout {
            fragments: Vec::new(),
            content_block_size: Length::zero(),
        }
    }
}
