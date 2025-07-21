/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use bitflags::Flags;
use layout_api::LayoutDamage;
use layout_api::wrapper_traits::LayoutNode;
use script::layout_dom::ServoLayoutNode;
use style::context::{SharedStyleContext, StyleContext};
use style::data::ElementData;
use style::dom::{NodeInfo, TElement, TNode};
use style::selector_parser::RestyleDamage;
use style::traversal::{DomTraversal, PerLevelTraversalData, recalc_style_at};
use style::values::computed::Display;

use crate::context::LayoutContext;
use crate::dom::{DOMLayoutData, NodeExt};
use crate::dom_traversal::iter_child_nodes;

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

#[allow(unsafe_code)]
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

    fn shared_context(&self) -> &SharedStyleContext {
        &self.context.style_context
    }
}

#[servo_tracing::instrument(skip_all)]
pub(crate) fn compute_damage_and_repair_style(
    context: &SharedStyleContext,
    node: ServoLayoutNode<'_>,
    damage_from_environment: RestyleDamage,
) -> RestyleDamage {
    compute_damage_and_repair_style_inner(context, node, damage_from_environment)
}

pub(crate) fn compute_damage_and_repair_style_inner(
    context: &SharedStyleContext,
    node: ServoLayoutNode<'_>,
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
    for child in iter_child_nodes(node) {
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
    let damage_for_parent = element_damage | (damage_from_children & RestyleDamage::RELAYOUT);

    // If we are going to potentially reuse this box tree node, then clear any cached
    // fragment layout.
    //
    // TODO: If this node has `recollect_box_tree_children` damage, this is unecessary
    // unless it's entirely above the dirty root.
    if element_damage != RestyleDamage::reconstruct() &&
        damage_for_parent.contains(RestyleDamage::RELAYOUT)
    {
        node.clear_fragment_layout_cache();
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
