/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use layout_api::{AccessibilityDamage, LayoutElement, LayoutNode};
use script::layout_dom::ServoLayoutNode;
use style::selector_parser::RestyleDamage;

use crate::accessibility_tree::AccessibilityDamageMap;
use crate::context::LayoutContext;
use crate::dom::{LayoutBox, NodeExt};
use crate::flexbox::FlexLevelBox;
use crate::flow::BlockLevelBox;
use crate::flow::inline::InlineItem;
use crate::fragment_tree::Fragment;

/// A data structure to track a layout root.
///
/// Layout roots are places in the fragment tree where fragment tree layout can start.
/// These are fragments that isolate fragment damage from their ancestors sufficiently
/// that their ancestors can be preserved from one fragment tree layout to the next.
/// **Currently, this only includes absolutely-or-fixed-positioned fragments that do not
/// have any escaping fixed position fragments.**.
///
/// During damage propagation, upward flowing fragment tree `Relayout` damage can be
/// converted into `DescendantCollectedAsLayoutRoot` damage. When an ancestor of a layout
/// root has fragment tree layout damage, `DescendantCollectedAsLayoutRoot` damage is
/// converted back into `Relayout` and propagated up to the next layout root or the root
/// of the DOM.
///
/// This is only possible because the damage propagation traversal ensures that damage
/// above the absolute (which might shift its static position) is converted back into
/// `Relayout` damage. This means that the absolute should be placed with the fully
/// adjusted static positioning rectangle between two layouts.
///
/// It's possible that upon laying out a layout root again, a new fixed position element
/// has started to escape from the layout root. In that case, a full fragment tree layout
/// becomes necessary to rebuild the fragment properly.
pub(crate) struct LayoutRoot<'dom> {
    pub node: ServoLayoutNode<'dom>,
}

impl<'dom> TryFrom<ServoLayoutNode<'dom>> for LayoutRoot<'dom> {
    type Error = ();

    fn try_from(node: ServoLayoutNode<'dom>) -> Result<Self, Self::Error> {
        if !node.is_absolutely_positioned() {
            return Err(());
        }

        // Only accept previously known layout roots without fragmentation as compatible layout roots.
        let mut is_layout_root = false;
        node.with_layout_box_base(|base| {
            let fragments = base.fragments();
            is_layout_root =
                fragments.len() == 1 && matches!(fragments.first(), Some(Fragment::LayoutRoot(..)));
        });
        if !is_layout_root {
            return Err(());
        }

        Ok(Self { node })
    }
}

impl<'dom> LayoutRoot<'dom> {
    pub(crate) fn try_layout(
        &self,
        layout_context: &LayoutContext,
        mut accessibility_damage: Option<&mut AccessibilityDamageMap<'dom>>,
    ) -> bool {
        let Some(inner_layout_data) = self.node.inner_layout_data_mut() else {
            return false;
        };
        let layout_box = inner_layout_data.self_box.clone();

        let layout_box = layout_box.borrow();
        let Some(layout_box) = &*layout_box else {
            return false;
        };

        let positioned_box = match layout_box {
            LayoutBox::BlockLevel(block_level) => {
                let mut block_level = block_level.borrow_mut();
                match &mut *block_level {
                    BlockLevelBox::OutOfFlowAbsolutelyPositionedBox(positioned_box) => {
                        positioned_box.clone()
                    },
                    _ => return false,
                }
            },
            LayoutBox::InlineLevel(InlineItem::OutOfFlowAbsolutelyPositionedBox(
                positioned_box,
                _,
            )) => positioned_box.clone(),
            LayoutBox::FlexLevel(flex_level_box) => {
                let mut flex_level_box = flex_level_box.borrow_mut();
                match &mut *flex_level_box {
                    FlexLevelBox::OutOfFlowAbsolutelyPositionedBox(positioned_box) => {
                        positioned_box.clone()
                    },
                    _ => return false,
                }
            },
            _ => return false,
        };

        let positioned_box = positioned_box.borrow();
        let formatting_context = &positioned_box.context;
        let fragments = formatting_context.base.fragments();
        let Some(Fragment::LayoutRoot(layout_root_fragment)) = fragments.first() else {
            return false;
        };

        let layout_inputs = formatting_context.layout_root_layout_inputs.borrow();
        let Some(layout_inputs) = &*layout_inputs else {
            return false;
        };

        // If an `Err` is returned here, that means that this layout root is no longer
        // a viable layout root and a full fragment tree layout is necessary.
        let is_ok = layout_inputs
            .layout(
                layout_context,
                formatting_context,
                &layout_root_fragment.fragment,
            )
            .is_ok();

        if is_ok &&
            let Some(map) = accessibility_damage.as_mut() &&
            let Some(element) = self.node.as_element()
        {
            // Insert this element into the accessibility damage map with the damage stored on the
            // element. This allows the accessibility tree to use it as a starting point for damage
            // resolution for damage from layout. See
            // AccessibilityTree::apply_changes_from_dom_tree().
            let accessibility_damage =
                AccessibilityDamage::from_bits_retain(element.element_data().damage.bits());
            map.insert(self.node.opaque(), (self.node, accessibility_damage));
        }

        is_ok
    }

    pub(crate) fn handle_failed_layout_root_layout(
        &self,
        accessibility_damage: Option<&mut AccessibilityDamageMap<'dom>>,
    ) {
        self.node
            .clear_fragments_and_dirty_fragment_caches_recursively();

        if let Some(map) = accessibility_damage {
            self.propagate_accessibility_damage_to_root(map);
        }
    }

    /// If the accessibility tree can't use this node as a starting point for resolving damage from
    /// layout, remove it from the accessibility damage map, propagate the accessibility damage up
    /// to the root, and add the root element to the damage map instead.
    #[expect(unsafe_code)]
    fn propagate_accessibility_damage_to_root(&self, map: &mut AccessibilityDamageMap<'dom>) {
        let Some(element) = self.node.as_element() else {
            return;
        };
        map.remove(&self.node.opaque());

        let damage = RestyleDamage::from_bits_retain(
            AccessibilityDamage::DescendantHasDamageFromLayout.bits(),
        );
        let mut ancestor = unsafe { self.node.dangerous_flat_tree_parent() };
        let mut root_element = element;
        while let Some(node) = ancestor {
            let element = node
                .as_element()
                .expect("All ancestors of elements should be elements");
            let mut element_data = element.element_data_mut();
            element_data.damage |= damage;
            ancestor = unsafe { node.dangerous_flat_tree_parent() };
            if ancestor.is_none() {
                root_element = element;
                break;
            }
        }

        let root_node = root_element.as_node();
        map.insert(
            root_node.opaque(),
            (
                root_node,
                AccessibilityDamage::from_bits_retain(root_element.element_data().damage.bits()),
            ),
        );
    }
}
