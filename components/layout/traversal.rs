/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

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
) -> RestyleDamage {
    compute_damage_and_repair_style_inner(context, node, RestyleDamage::empty())
}

pub(crate) fn compute_damage_and_repair_style_inner(
    context: &SharedStyleContext,
    node: ServoLayoutNode<'_>,
    damage_from_parent: RestyleDamage,
) -> RestyleDamage {
    let mut element_damage;
    let element_data = &node
        .style_data()
        .expect("Should not run `compute_damage` before styling.")
        .element_data;

    {
        let mut element_data = element_data.borrow_mut();
        element_data.damage.insert(damage_from_parent);
        element_damage = element_data.damage;

        if let Some(ref style) = element_data.styles.primary {
            if style.get_box().display == Display::None {
                return element_damage;
            }
        }
    }

    // If we are reconstructing this node, then all of the children should be reconstructed as well.
    let damage_for_children = element_damage | damage_from_parent;
    let mut damage_from_children = RestyleDamage::empty();
    for child in iter_child_nodes(node) {
        if child.is_element() {
            damage_from_children |=
                compute_damage_and_repair_style_inner(context, child, damage_for_children);
        }
    }

    if element_damage != RestyleDamage::reconstruct() && !element_damage.is_empty() {
        node.repair_style(context);
    }

    // If one of our children needed to be reconstructed, we need to recollect children
    // during box tree construction.
    if damage_from_children.contains(LayoutDamage::recollect_box_tree_children()) {
        element_damage.insert(LayoutDamage::recollect_box_tree_children());
        element_data.borrow_mut().damage.insert(element_damage);
    }

    // Only propagate up layout phases from children, as other types of damage are
    // incorporated into `element_damage` above.
    element_damage | (damage_from_children & RestyleDamage::RELAYOUT)
}
