/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use script::layout_dom::ServoLayoutNode;
use script_layout_interface::wrapper_traits::LayoutNode;
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
    pub fn new(context: &'a LayoutContext<'a>) -> Self {
        RecalcStyle { context }
    }

    pub fn context(&self) -> &LayoutContext<'a> {
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

pub(crate) fn compute_damage_and_repair_style(
    context: &SharedStyleContext,
    node: ServoLayoutNode<'_>,
) -> RestyleDamage {
    compute_damage_and_repair_style_inner(context, node, RestyleDamage::empty())
}

fn need_repair_style(damage: RestyleDamage) -> bool {
    // Repair the style at the node's layout objects only when itself has
    // restyle damage but it's boxes will be kept unchanged.
    if damage.is_empty() || damage.will_change_box_subtree() {
        return false;
    }

    return true;
}

pub(crate) fn compute_damage_and_repair_style_inner(
    context: &SharedStyleContext,
    node: ServoLayoutNode<'_>,
    parent_restyle_damage: RestyleDamage,
) -> RestyleDamage {
    let original_damage;
    let damage;
    {
        let mut element_data = node
            .style_data()
            .expect("Should not run `compute_damage` before styling.")
            .element_data
            .borrow_mut();

        original_damage = element_data.damage;
        // TODO: The `REPAINT` only is used to ensure whether the reflow is repaint-only
        // incremental reflow, and the `REFLOW` is inactive currently. Thus, both of them
        // are unused and cleaned to avoid to trigger the next reflow. The other kind of
        // damage will be cleaned after incremeental box tree update.
        element_data
            .damage
            .remove(RestyleDamage::REPAINT | RestyleDamage::REFLOW);

        damage = original_damage | parent_restyle_damage;

        if let Some(ref style) = element_data.styles.primary {
            if style.get_box().display == Display::None {
                return damage.propagate_up_damage();
            }
        }
    };

    let mut propagated_damage = damage.propagate_up_damage();
    let propagated_down_damage = damage.propagate_down_damage();

    for child in iter_child_nodes(node) {
        if child.is_element() {
            propagated_damage |=
                compute_damage_and_repair_style_inner(context, child, propagated_down_damage);
        }
    }

    if need_repair_style(propagated_damage) && need_repair_style(original_damage) {
        node.repair_style(context);
    }

    if propagated_damage.contains(RestyleDamage::REPAIR) {
        let mut element_data = node.style_data().unwrap().element_data.borrow_mut();
        element_data.damage |= RestyleDamage::REPAIR;
    }

    propagated_damage
}
