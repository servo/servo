/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::sync::Arc;

use bitflags::Flags;
use layout_api::LayoutDamage;
use layout_api::wrapper_traits::{LayoutNode, ThreadSafeLayoutNode};
use script::layout_dom::{ServoLayoutNode, ServoThreadSafeLayoutNode};
use style::context::{SharedStyleContext, StyleContext};
use style::data::ElementData;
use style::dom::{NodeInfo, TElement, TNode};
use style::selector_parser::RestyleDamage;
use style::traversal::{DomTraversal, PerLevelTraversalData, recalc_style_at};

use crate::BoxTree;
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
pub(crate) fn compute_damage_and_rebuild_box_tree(
    box_tree: &mut Option<Arc<BoxTree>>,
    layout_context: &LayoutContext,
    dirty_root: ServoLayoutNode<'_>,
    root_node: ServoLayoutNode<'_>,
    damage_from_environment: RestyleDamage,
) -> RestyleDamage {
    let restyle_damage = compute_damage_and_rebuild_box_tree_inner(
        layout_context,
        dirty_root.to_threadsafe(),
        damage_from_environment,
    );

    let layout_damage: LayoutDamage = restyle_damage.into();
    if box_tree.is_none() {
        *box_tree = Some(Arc::new(BoxTree::construct(layout_context, root_node)));
        return restyle_damage;
    }

    // There are two cases where we need to do more work:
    //
    // 1. Fragment tree layout needs to run again, in which case we should invalidate all
    //    fragments to the root of the DOM.
    // 2. Box tree reconstruction needs to run at the dirty root, in which case we need to
    //    find an appropriate place to run box tree reconstruction and *also* invalidate all
    //    fragments to the root of the DOM.
    if !restyle_damage.contains(RestyleDamage::RELAYOUT) {
        return restyle_damage;
    }

    // If the damage traversal indicated that the dirty root needs a new box, walk up the
    // tree to find an appropriate place to run box tree reconstruction.
    let mut needs_box_tree_rebuild = layout_damage.needs_new_box();

    let mut damage_for_ancestors = LayoutDamage::RECOMPUTE_INLINE_CONTENT_SIZES;
    let mut maybe_parent_node = dirty_root.traversal_parent();
    while let Some(parent_node) = maybe_parent_node {
        let threadsafe_parent_node = parent_node.as_node().to_threadsafe();

        // If we need box tree reconstruction, try it here.
        if needs_box_tree_rebuild &&
            threadsafe_parent_node
                .rebuild_box_tree_from_independent_formatting_context(layout_context)
        {
            needs_box_tree_rebuild = false;
        }

        if needs_box_tree_rebuild {
            // We have not yet found a place to run box tree reconstruction, so clear this
            // node's boxes to ensure that they are invalidated for the reconstruction we
            // will run later.
            threadsafe_parent_node.unset_all_boxes();
        } else {
            // Reconstruction has already run or was not necessary, so we just need to
            // ensure that fragment tree layout does not reuse any cached fragments.
            let new_damage_for_ancestors = Cell::new(LayoutDamage::empty());
            threadsafe_parent_node.with_layout_box_base_including_pseudos(|base| {
                new_damage_for_ancestors.set(
                    new_damage_for_ancestors.get() |
                        base.add_damage(Default::default(), damage_for_ancestors),
                );
            });
            damage_for_ancestors = new_damage_for_ancestors.get();
        }

        maybe_parent_node = parent_node.traversal_parent();
    }

    // We could not find a place in the middle of the tree to run box tree reconstruction,
    // so just rebuild the whole tree.
    if needs_box_tree_rebuild {
        *box_tree = Some(Arc::new(BoxTree::construct(layout_context, root_node)));
    }

    restyle_damage
}

pub(crate) fn compute_damage_and_rebuild_box_tree_inner(
    layout_context: &LayoutContext,
    node: ServoThreadSafeLayoutNode<'_>,
    damage_from_parent: RestyleDamage,
) -> RestyleDamage {
    let element_data = &node
        .style_data()
        .expect("Should not run `compute_damage` before styling.")
        .element_data;
    let (element_damage, is_display_none) = {
        let mut element_data = element_data.borrow_mut();
        (
            std::mem::take(&mut element_data.damage),
            element_data.styles.is_display_none(),
        )
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
    let rebuild_children = element_damage.contains(LayoutDamage::box_damage()) ||
        (damage_from_parent.contains(LayoutDamage::box_damage()) &&
            !node.isolates_damage_for_damage_propagation());
    if rebuild_children {
        damage_for_children.insert(LayoutDamage::box_damage());
    } else if element_and_parent_damage.contains(RestyleDamage::RELAYOUT) &&
        !element_damage.contains(RestyleDamage::RELAYOUT) &&
        node.isolates_damage_for_damage_propagation()
    {
        // If not rebuilding the boxes for this node, but fragments need to be rebuilt
        // only because of an ancestor, fragment layout caches should still be valid when
        // crossing down into new independent formatting contexts.
        damage_for_children.remove(RestyleDamage::RELAYOUT);
        element_and_parent_damage.remove(RestyleDamage::RELAYOUT);
    }

    let mut damage_from_children = RestyleDamage::empty();
    for child in node.children() {
        if child.is_element() {
            damage_from_children |= compute_damage_and_rebuild_box_tree_inner(
                layout_context,
                child,
                damage_for_children,
            );
        }
    }

    // Only propagate up layout phases from children. Other types of damage can be
    // propagated from children but via the `LayoutBoxBase::add_damage` return value.
    let mut layout_damage_for_parent =
        element_and_parent_damage | (damage_from_children & RestyleDamage::RELAYOUT);

    let element_or_ancestors_need_rebuild =
        element_and_parent_damage.contains(LayoutDamage::descendant_has_box_damage());
    let descendant_needs_rebuild =
        damage_from_children.contains(LayoutDamage::descendant_has_box_damage());
    if element_or_ancestors_need_rebuild || descendant_needs_rebuild {
        if damage_from_parent.contains(LayoutDamage::descendant_has_box_damage()) ||
            !node.rebuild_box_tree_from_independent_formatting_context(layout_context)
        {
            // In this case:
            //  - an ancestor needs to be completely rebuilt, or
            //  - a descendant needs to be rebuilt, but we are still propagating the rebuild
            //    damage to an independent formatting context with a compatible box level.
            //
            // This means that this box is no longer valid and also needs to be rebuilt
            // (perhaps some of its descendants do not though). In this case, unset all existing
            // boxes for the node and ensure that the appropriate rebuild-type damage
            // propagates up the tree.
            node.unset_all_boxes();
            layout_damage_for_parent
                .insert(LayoutDamage::descendant_has_box_damage() | RestyleDamage::RELAYOUT);
        } else {
            // In this case, we have rebuilt the box tree from this point and we do not
            // have to propagate rebuild box tree damage up the tree any further.
            layout_damage_for_parent.remove(LayoutDamage::box_damage());
            layout_damage_for_parent
                .insert(RestyleDamage::RELAYOUT | LayoutDamage::recompute_inline_content_sizes());
        }
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
            node.repair_style(&layout_context.style_context);
        }
    }

    layout_damage_for_parent
}
