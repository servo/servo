/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::fmt::Debug;
use std::usize;

use base::id::ScrollTreeNodeId;
use servo_arc::Arc as ServoArc;
use style::properties::ComputedValues;

/// Similar to clip tree and scroll tree, it is used to track the
/// effect css on the DOM tree. This is because blur and drop-shadow
/// can affect the size of the visual area of elements.
// #[derive(Debug)]
pub(crate) struct EffectNode {
    pub parent_id: EffectNodeId,
    /// Similar to a clip node, it is the currently associated scroll node.
    pub scroll_tree_node_id: ScrollTreeNodeId,
    pub style: ServoArc<ComputedValues>,
}

impl Debug for EffectNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(
            format!(
                "EffectNode [parent_id: {:?}, scroll_id: {:?}]",
                self.parent_id, self.scroll_tree_node_id
            )
            .as_str(),
        )
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct EffectNodeId(usize);

impl EffectNodeId {
    pub(crate) const INVALID: EffectNodeId = EffectNodeId(usize::MAX);
}

#[derive(Debug, Default)]
pub(crate) struct StackingContextTreeEffectStore(pub Vec<EffectNode>);

impl StackingContextTreeEffectStore {
    pub fn add_effect(
        &mut self,
        parent_id: EffectNodeId,
        scroll_tree_node_id: ScrollTreeNodeId,
        style: ServoArc<ComputedValues>,
    ) -> Option<EffectNodeId> {
        let effects = style.get_effects();
        if effects.opacity == 1.0 && effects.filter.0.is_empty() {
            return None;
        }

        let id = self.0.len();
        self.0.push(EffectNode {
            parent_id,
            scroll_tree_node_id,
            style,
        });

        Some(EffectNodeId(id))
    }

    pub fn get(&self, id: EffectNodeId) -> Option<&EffectNode> {
        self.0.get(id.0)
    }
}
