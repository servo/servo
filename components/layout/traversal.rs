/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;

use bitflags::Flags;
use layout_api::LayoutDamage;
use layout_api::wrapper_traits::{LayoutNode, ThreadSafeLayoutNode};
use script::layout_dom::ServoThreadSafeLayoutNode;
use style::context::{SharedStyleContext, StyleContext};
use style::data::ElementData;
use style::dom::{NodeInfo, TElement, TNode};
use style::selector_parser::RestyleDamage;
use style::traversal::{DomTraversal, PerLevelTraversalData, recalc_style_at};

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
    let element_data = &node
        .style_data()
        .expect("Should not run `compute_damage` before styling.")
        .element_data;
    let (element_damage, is_display_none) = {
        let element_data = element_data.borrow();
        (element_data.damage, element_data.styles.is_display_none())
    };

    let mut element_and_parent_damage = element_damage | damage_from_parent;
    if is_display_none {
        node.unset_all_boxes();
        return element_and_parent_damage;
    }

    // Children only receive layout mode damage from their parents, except when an ancestor
    // needs to be completely rebuilt. In that case, descendants are rebuilt down to the
    // first independent formatting context, which should isolate that tree from further
    // box damage.
    let mut damage_for_children = element_and_parent_damage;
    damage_for_children.truncate();
    let rebuild_children = element_damage.contains(LayoutDamage::rebuild_box_tree()) ||
        (damage_from_parent.contains(LayoutDamage::rebuild_box_tree()) &&
            !node.isolates_box_tree_rebuild_damage());
    if rebuild_children {
        damage_for_children.insert(LayoutDamage::rebuild_box_tree());
    }

    let mut damage_from_children = RestyleDamage::empty();
    for child in node.children() {
        if child.is_element() {
            damage_from_children |=
                compute_damage_and_repair_style_inner(context, child, damage_for_children);
        }
    }

    // Only propagate up layout phases from children. Other types of damage can be
    // propagated from children but via the `LayoutBoxBase::add_damage` return value.
    let mut layout_damage_for_parent =
        element_and_parent_damage | (damage_from_children & RestyleDamage::RELAYOUT);

    if damage_from_children.contains(LayoutDamage::recollect_box_tree_children()) ||
        element_and_parent_damage.contains(LayoutDamage::recollect_box_tree_children())
    {
        // In this case, this node, an ancestor, or a descendant needs to be completely
        // rebuilt. That means that this box is no longer valid and also needs to be rebuilt
        // (perhaps some of its children do not though). In this case, unset all existing
        // boxes for the node and ensure that the appropriate rebuild-type damage propagates
        // up the tree.
        node.unset_all_boxes();
        layout_damage_for_parent
            .insert(LayoutDamage::recollect_box_tree_children() | RestyleDamage::RELAYOUT);
    } else {
        // In this case, this node's boxes are preserved! It's possible that we still need
        // to run fragment tree layout in this subtree due to an ancestor, this node, or a
        // descendant changing style. In that case, we ask the `LayoutBoxBase` to clear
        // any cached information that cannot be used.
        if (element_and_parent_damage | damage_from_children).contains(RestyleDamage::RELAYOUT) {
            let extra_layout_damage_for_parent = Cell::new(LayoutDamage::empty());
            node.with_layout_box_base_including_pseudos(|base| {
                extra_layout_damage_for_parent.set(
                    extra_layout_damage_for_parent.get() |
                        base.add_damage(element_damage.into(), damage_from_children.into()),
                );
            });
            layout_damage_for_parent.insert(extra_layout_damage_for_parent.get().into());
        }

        // The box is preserved. Whether or not we run fragment tree layout, we need to
        // update any preserved layout data structures' style references, if *this*
        // element's style has changed.
        if !element_damage.is_empty() {
            node.repair_style(context);
        }

        // Since damage is cleared during box tree construction, which will not run for
        // this node, clear the damage now to avoid it affecting the next reflow.
        //
        // TODO: It would be cleaner to clear all damage during the damage traversal.
        element_and_parent_damage = RestyleDamage::empty();
    }

    if element_and_parent_damage != element_damage {
        element_data.borrow_mut().damage = element_and_parent_damage;
    }

    layout_damage_for_parent
}
