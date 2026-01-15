/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::sync::atomic::Ordering;

use bitflags::Flags;
use layout_api::LayoutDamage;
use layout_api::wrapper_traits::{LayoutNode, ThreadSafeLayoutNode};
use script::layout_dom::ServoThreadSafeLayoutNode;
use style::context::{SharedStyleContext, StyleContext};
use style::data::ElementData;
use style::dom::{NodeInfo, TElement, TNode};
use style::selector_parser::RestyleDamage;
use style::traversal::{DomTraversal, PerLevelTraversalData, recalc_style_at};
use style::values::computed::Display;

use crate::context::LayoutContext;
use crate::dom::{DOMLayoutData, NodeExt};

pub struct RecalcStyle<'a> {
    context: &'a LayoutContext<'a>,
}

impl<'a> RecalcStyle<'a> {
    pub(crate) fn new(context: &'a LayoutContext<'a>) -> Self {
        RecalcStyle { context }
    }

    pub(crate) fn context(&self) -> &LayoutContext<'a> {
        self.context
    }
}

#[expect(unsafe_code)]
impl<'dom, E> DomTraversal<E> for RecalcStyle<'_>
where
    E: TElement,
    E::ConcreteNode: 'dom + LayoutNode<'dom>,
{
    fn process_preorder<F>(
        &self,
        traversal_data: &PerLevelTraversalData,
        context: &mut StyleContext<E>,
        node: E::ConcreteNode,
        note_child: F,
    ) where
        F: FnMut(E::ConcreteNode),
    {
        if node.is_text_node() {
            return;
        }

        let had_style_data = node.style_data().is_some();
        unsafe {
            node.initialize_style_and_layout_data::<DOMLayoutData>();
        }

        let element = node.as_element().unwrap();
        let mut element_data = element.mutate_data().unwrap();

        if !had_style_data {
            // *** TODO: accessibility damage should work analogously
            element_data.damage = RestyleDamage::reconstruct();
        }

        recalc_style_at(
            self,
            traversal_data,
            context,
            element,
            &mut element_data,
            note_child,
        );

        unsafe {
            element.unset_dirty_descendants();
        }
    }

    #[inline]
    fn needs_postorder_traversal() -> bool {
        false
    }

    fn process_postorder(&self, _style_context: &mut StyleContext<E>, _node: E::ConcreteNode) {
        panic!("this should never be called")
    }

    fn text_node_needs_traversal(node: E::ConcreteNode, parent_data: &ElementData) -> bool {
        node.layout_data().is_none() || !parent_data.damage.is_empty()
    }

    fn shared_context(&self) -> &SharedStyleContext<'_> {
        &self.context.style_context
    }
}

#[servo_tracing::instrument(skip_all)]
pub(crate) fn compute_damage_and_repair_style(
    context: &SharedStyleContext,
    node: ServoThreadSafeLayoutNode<'_>,
    damage_from_environment: RestyleDamage,
) -> RestyleDamage {
    compute_damage_and_repair_style_inner(context, node, damage_from_environment)
}

pub(crate) fn compute_damage_and_repair_style_inner(
    context: &SharedStyleContext,
    node: ServoThreadSafeLayoutNode<'_>,
    damage_from_parent: RestyleDamage,
) -> RestyleDamage {
    let mut element_damage;
    let original_element_damage;
    let element_data = &node
        .style_data()
        .expect("Should not run `compute_damage` before styling.")
        .element_data;

    {
        let mut element_data = element_data.borrow_mut();
        original_element_damage = element_data.damage;
        element_damage = original_element_damage | damage_from_parent;

        // ** Note we can get styles like this
        if let Some(ref style) = element_data.styles.primary {
            if style.get_box().display == Display::None {
                element_data.damage = element_damage;
                return element_damage;
            }
        }
    }

    // If we are reconstructing this node, then all of the children should be reconstructed as well.
    // Otherwise, do not propagate down its box damage.
    let mut damage_for_children = element_damage;
    if !element_damage.contains(LayoutDamage::rebuild_box_tree()) {
        damage_for_children.truncate();
    }

    let mut damage_from_children = RestyleDamage::empty();
    for child in node.children() {
        if child.is_element() {
            damage_from_children |=
                compute_damage_and_repair_style_inner(context, child, damage_for_children);
        }
    }

    // If one of our children needed to be reconstructed, we need to recollect children
    // during box tree construction.
    if damage_from_children.contains(LayoutDamage::recollect_box_tree_children()) {
        element_damage.insert(LayoutDamage::recollect_box_tree_children());
    }

    // If this node's box will not be preserved, we need to relayout its box tree.
    let element_layout_damage = LayoutDamage::from(element_damage);
    if element_layout_damage.has_box_damage() {
        element_damage.insert(RestyleDamage::RELAYOUT);
    }

    // Only propagate up layout phases from children, as other types of damage are
    // incorporated into `element_damage` above.
    let mut damage_for_parent = element_damage | (damage_from_children & RestyleDamage::RELAYOUT);

    // If we are going to potentially reuse this box tree node, then clear any cached
    // fragment layout.
    //
    // TODO: If this node has `recollect_box_tree_children` damage, this is unnecessary
    // unless it's entirely above the dirty root.
    if element_damage != RestyleDamage::reconstruct() &&
        damage_for_parent.contains(RestyleDamage::RELAYOUT)
    {
        let outer_inline_content_sizes_depend_on_content = Cell::new(false);
        node.with_layout_box_base_including_pseudos(|base| {
            base.clear_fragments();
            if original_element_damage.contains(RestyleDamage::RELAYOUT) {
                // If the node itself has damage, we must clear both the cached layout results
                // and also the cached intrinsic inline sizes.
                *base.cached_layout_result.borrow_mut() = None;
                *base.cached_inline_content_size.borrow_mut() = None;
            } else if damage_from_children.contains(RestyleDamage::RELAYOUT) {
                // If the damage is propagated from children, then we still need to clear the cached
                // layout results, but sometimes we can keep the cached intrinsic inline sizes.
                *base.cached_layout_result.borrow_mut() = None;
                if !damage_from_children.contains(LayoutDamage::recompute_inline_content_sizes()) {
                    // This happens when there is a node which is a descendant of the current one and
                    // an ancestor of the damaged one, whose inline size doesn't depend on its contents.
                    return;
                }
                *base.cached_inline_content_size.borrow_mut() = None;
            }

            // When a block container has a mix of inline-level and block-level contents,
            // the inline-level ones are wrapped inside an anonymous block associated with
            // the block container. The anonymous block has an `auto` size, so its intrinsic
            // contribution depends on content, but it can't affect the intrinsic size of
            // ancestors if the block container is sized extrinsically.
            if !base.base_fragment_info.is_anonymous() {
                // TODO: Use `Cell::update()` once it becomes stable.
                outer_inline_content_sizes_depend_on_content.set(
                    outer_inline_content_sizes_depend_on_content.get() ||
                        base.outer_inline_content_sizes_depend_on_content
                            .load(Ordering::Relaxed),
                );
            }
        });

        // If the intrinsic contributions of this node depend on content, we will need to clear
        // the cached intrinsic sizes of the parent. But if the contributions are purely extrinsic,
        // then the intrinsic sizes of the ancestors won't be affected, and we can keep the cache.
        if outer_inline_content_sizes_depend_on_content.get() {
            damage_for_parent.insert(LayoutDamage::recompute_inline_content_sizes())
        }
    }

    // If the box will be preserved, update the box's style and also in any fragments
    // that haven't been cleared. Meanwhile, clear the damage to avoid affecting the
    // next reflow.
    if !element_layout_damage.has_box_damage() {
        if !original_element_damage.is_empty() {
            node.repair_style(context);
        }

        element_damage = RestyleDamage::empty();
    }

    if element_damage != original_element_damage {
        element_data.borrow_mut().damage = element_damage;
    }

    damage_for_parent
}
