/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::sync::Arc;

use layout_api::{
    DangerousStyleElement, DangerousStyleNode, LayoutDamage, LayoutElement, LayoutNode,
};
use script::layout_dom::ServoLayoutNode;
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

impl<'dom, E> DomTraversal<E> for RecalcStyle<'_>
where
    E: DangerousStyleElement<'dom> + TElement,
    E::ConcreteNode: 'dom + DangerousStyleNode<'dom>,
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
        let Some(dangerous_style_element) = node.as_element() else {
            return;
        };

        let layout_element = dangerous_style_element.layout_element();
        let had_style_data = layout_element.style_data().is_some();
        layout_element.initialize_style_and_layout_data::<DOMLayoutData>();

        let mut element_data = dangerous_style_element.mutate_data().unwrap();
        if !had_style_data {
            element_data.damage = RestyleDamage::reconstruct();
        }

        recalc_style_at(
            self,
            traversal_data,
            context,
            dangerous_style_element,
            &mut element_data,
            note_child,
        );
    }

    #[inline]
    fn needs_postorder_traversal() -> bool {
        false
    }

    fn process_postorder(&self, _style_context: &mut StyleContext<E>, _node: E::ConcreteNode) {
        panic!("this should never be called")
    }

    fn text_node_needs_traversal(node: E::ConcreteNode, parent_data: &ElementData) -> bool {
        node.layout_node().layout_data().is_none() || !parent_data.damage.is_empty()
    }

    fn shared_context(&self) -> &SharedStyleContext<'_> {
        &self.context.style_context
    }
}

#[expect(unsafe_code)]
#[servo_tracing::instrument(skip_all)]
pub(crate) fn compute_damage_and_rebuild_box_tree(
    box_tree: &mut Option<Arc<BoxTree>>,
    layout_context: &LayoutContext,
    dirty_root: ServoLayoutNode<'_>,
    root_node: ServoLayoutNode<'_>,
    damage_from_environment: LayoutDamage,
) -> LayoutDamage {
    let layout_damage = compute_damage_and_rebuild_box_tree_inner(
        layout_context,
        dirty_root,
        damage_from_environment,
    );

    if box_tree.is_none() {
        *box_tree = Some(Arc::new(BoxTree::construct(layout_context, root_node)));
        return layout_damage;
    }

    // There are two cases where we need to do more work:
    //
    // 1. Fragment tree layout needs to run again, in which case we should invalidate all
    //    fragments to the root of the DOM.
    // 2. Box tree reconstruction needs to run at the dirty root, in which case we need to
    //    find an appropriate place to run box tree reconstruction and *also* invalidate all
    //    fragments to the root of the DOM.
    let needs_fragment_tree_rebuild = layout_damage.contains(LayoutDamage::Relayout);
    let needs_overflow_recalculation = layout_damage.contains(LayoutDamage::RecalculateOverflow);
    if !needs_fragment_tree_rebuild && !needs_overflow_recalculation {
        return layout_damage;
    }

    // If the damage traversal indicated that the dirty root needs a new box, walk up the
    // tree to find an appropriate place to run box tree reconstruction.
    let mut needs_box_tree_rebuild = layout_damage.contains(LayoutDamage::DescendantHasBoxDamage);

    let mut damage_for_ancestors = LayoutDamage::RecomputeInlineContentSizes;
    if layout_damage.contains(LayoutDamage::LayoutAffectedByInflowDescendant) {
        damage_for_ancestors.insert(LayoutDamage::LayoutAffectedByInflowDescendant);
    }

    let mut maybe_parent_node = unsafe { dirty_root.dangerous_flat_tree_parent() };
    while let Some(parent_node) = maybe_parent_node {
        // If we need box tree reconstruction, try it here.
        if needs_box_tree_rebuild &&
            parent_node.rebuild_box_tree_from_independent_formatting_context(layout_context)
        {
            needs_box_tree_rebuild = false;
        }

        if needs_box_tree_rebuild {
            // We have not yet found a place to run box tree reconstruction, so clear this
            // node's boxes to ensure that they are invalidated for the reconstruction we
            // will run later.
            parent_node.unset_all_boxes();
        } else if needs_fragment_tree_rebuild {
            // Reconstruction has already run or was not necessary, so we just need to
            // ensure that fragment tree layout does not reuse any cached fragments.
            let mut new_damage_for_ancestors = LayoutDamage::empty();
            parent_node.with_layout_box_base_including_pseudos(|base| {
                new_damage_for_ancestors |= base.add_damage(&ElementDamageSet::from_children(
                    parent_node,
                    damage_for_ancestors,
                ));
            });
            damage_for_ancestors = new_damage_for_ancestors;

            // If doing a fragment tree layout, we also need to apply the LAYOUT_AFFECTED_BY_INFLOW_DESCENDANT
            // damage flag, unless this node is out of flow. In that case our ancestors are rebuilt, but
            // their resulting fragments should be equivalent to the previous ones.
            if damage_for_ancestors.contains(LayoutDamage::LayoutAffectedByInflowDescendant) &&
                parent_node.is_absolutely_positioned()
            {
                damage_for_ancestors.remove(LayoutDamage::LayoutAffectedByInflowDescendant);
            }
        } else {
            // No fragment layout is necessary, but a descendant had scrollable overflow
            // damage. In this case, clear any preexisting scrollable overflow so that it
            // gets recalculated the next time it is queried.
            assert!(needs_overflow_recalculation);
            parent_node.with_layout_box_base_including_pseudos(|base| {
                base.clear_scrollable_overflow_all_on_fragments();
            });
        }

        maybe_parent_node = unsafe { parent_node.dangerous_flat_tree_parent() };
    }

    // We could not find a place in the middle of the tree to run box tree reconstruction,
    // so just rebuild the whole tree.
    if needs_box_tree_rebuild {
        *box_tree = Some(Arc::new(BoxTree::construct(layout_context, root_node)));
    }

    layout_damage
}

pub(crate) fn compute_damage_and_rebuild_box_tree_inner(
    layout_context: &LayoutContext,
    node: ServoLayoutNode<'_>,
    damage_from_parent: LayoutDamage,
) -> LayoutDamage {
    // Don't do any kind of damage propagation or box tree construction for non-Element
    // nodes, such as text and comments.
    let Some(element) = node.as_element() else {
        return damage_from_parent;
    };

    let (element_damage, is_display_none) = {
        let mut element_data = element.element_data_mut();
        (
            LayoutDamage::from(std::mem::take(&mut element_data.damage)),
            element_data.styles.is_display_none(),
        )
    };

    let has_dirty_descendants;
    #[expect(unsafe_code)]
    unsafe {
        let dangerous_style_element = element.dangerous_style_element();
        has_dirty_descendants = dangerous_style_element.has_dirty_descendants();
        dangerous_style_element.unset_dirty_descendants();
    };

    if is_display_none {
        node.unset_all_boxes();
        return element_damage | damage_from_parent;
    }

    let mut damage_set = ElementDamageSet {
        node,
        from_parent: damage_from_parent,
        on_element: element_damage,
        from_children: LayoutDamage::empty(),
    };

    // Depending on the incoming damage, it can be isolated, meaning that some damage
    // doesn't get passed down to children.
    let damage_for_children = damage_set.isolate_incoming_damage();

    // Propagate damage to children and gather the resulting damage into `from_children`.
    damage_set.propagate_damage_to_children(
        layout_context,
        has_dirty_descendants,
        damage_for_children,
    );

    // Apply the calculated damage to this element (perhaps triggering box tree layout),
    // and propagate resulting damage to ancestors.
    damage_set.apply_damage(layout_context)
}

enum BoxDamageAction {
    RebuildAncestor,
    TryRebuild,
    InvalidateFragmentCache,
    InvalidateScrollableOverflow,
    None,
}

impl BoxDamageAction {
    fn preserves_boxes(&self) -> bool {
        matches!(
            self,
            Self::InvalidateFragmentCache | Self::InvalidateScrollableOverflow | Self::None
        )
    }
}

pub(crate) struct ElementDamageSet<'a> {
    node: ServoLayoutNode<'a>,
    pub from_parent: LayoutDamage,
    pub on_element: LayoutDamage,
    pub from_children: LayoutDamage,
}

impl<'a> ElementDamageSet<'a> {
    fn from_children(node: ServoLayoutNode<'a>, from_children: LayoutDamage) -> Self {
        Self {
            node,
            from_parent: LayoutDamage::empty(),
            on_element: LayoutDamage::empty(),
            from_children,
        }
    }

    /// Given the damage on the element and damage from parents, determine which damage
    /// should be passed to children, returning that value.
    fn isolate_incoming_damage(&mut self) -> LayoutDamage {
        // Children only receive layout mode damage from their parents, except when an ancestor
        // needs to be completely rebuilt. In that case, descendants are rebuilt down to the
        // first independent formatting context, which should isolate that tree from further
        // box damage.
        let mut damage_for_children = (self.on_element | self.from_parent).only_layout_modes();
        let rebuild_children = self.on_element.contains(LayoutDamage::BoxDamage) ||
            (self.from_parent.contains(LayoutDamage::BoxDamage) &&
                !self.node.isolates_damage_for_damage_propagation());

        if rebuild_children {
            damage_for_children.insert(LayoutDamage::BoxDamage);
        } else if self.from_parent.contains(LayoutDamage::Relayout) &&
            !self.on_element.contains(LayoutDamage::Relayout) &&
            self.node.isolates_damage_for_damage_propagation()
        {
            // If not rebuilding the boxes for this node, but fragments need to be laid out
            // only because of an ancestor, fragment layout caches should still be valid when
            // crossing down into new independent formatting contexts.
            damage_for_children.remove(LayoutDamage::Relayout);
            self.from_parent.remove(LayoutDamage::Relayout);
        }

        damage_for_children
    }

    /// Given the damage the damage to children and whether or not this element had any
    /// dirty descendants, conditionally propagated damage to children and set the resulting
    /// damage from children on this [`ElementDamageSet`].
    fn propagate_damage_to_children(
        &mut self,
        layout_context: &LayoutContext<'_>,
        has_dirty_descendants: bool,
        damage_for_children: LayoutDamage,
    ) {
        // Propagate damage into children, but only if:
        //  1. There is a descendant that was dirty / possibly restyled.
        //  2. We detected that we need to rebuild child boxes.
        //  3. An ancestor will be laid out and children need to have their fragment caches cleared.
        //
        // In other situations, such as when layout will not run at all or when we are
        // guaranteed that children are undamaged, we can skip traversing children entirely.
        if has_dirty_descendants ||
            damage_for_children.intersects(LayoutDamage::BoxDamage | LayoutDamage::Relayout)
        {
            for child in self.node.flat_tree_children() {
                if child.is_element() {
                    self.from_children |= compute_damage_and_rebuild_box_tree_inner(
                        layout_context,
                        child,
                        damage_for_children,
                    );
                }
            }
        }
    }

    /// Given the damage from this element, the parent, and children, determine what action to
    /// take for this element's boxes and return the damage that should be propagated to parents.
    fn apply_damage(&self, layout_context: &LayoutContext<'_>) -> LayoutDamage {
        // Only propagate up layout phases and whether layout affects inflow descendants from
        // children. Other types of damage can be propagated from children but via the
        // `LayoutBoxBase::add_damage` return value.
        let forwarded_damage = (self.from_parent | self.on_element | self.from_children)
            .only_layout_modes() |
            (self.from_children | LayoutDamage::LayoutAffectedByInflowDescendant);

        let invalidate_for_rebuild = || {
            self.node.unset_all_boxes();
            LayoutDamage::DescendantHasBoxDamage | LayoutDamage::Relayout
        };

        let action = self.box_damage_action();
        let mut damage_for_parent = match action {
            BoxDamageAction::TryRebuild => {
                if self
                    .node
                    .rebuild_box_tree_from_independent_formatting_context(layout_context)
                {
                    // In this case, we have rebuilt the box tree from this point and we do not
                    // have to propagate rebuild box tree damage up the tree any further.
                    LayoutDamage::Relayout | LayoutDamage::RecomputeInlineContentSizes
                } else {
                    // A descendant needs to be rebuilt, but couldn't be rebuilt here,
                    // because this node was an not a rebuild-compatible independent
                    // formatting context. In this case do the same thing as if we needed
                    // to rebuild an ancestor.
                    invalidate_for_rebuild()
                }
            },
            BoxDamageAction::RebuildAncestor => {
                // In this case an ancestor needs to be completely rebuilt.
                //
                // This means that this box is no longer valid and also needs to be rebuilt
                // (perhaps some of its descendants do not though). In this case, unset all existing
                // boxes for the node and ensure that the appropriate rebuild-type damage
                // propagates up the tree.
                invalidate_for_rebuild()
            },
            BoxDamageAction::InvalidateFragmentCache => {
                // In this case, this node's boxes are preserved! It's possible that we still need
                // to run fragment tree layout in this subtree due to an ancestor, this node, or a
                // descendant changing style. In that case, we ask the `LayoutBoxBase` to clear
                // any cached information that cannot be used.
                let mut damage_for_parent = forwarded_damage;
                self.node.with_layout_box_base_including_pseudos(|base| {
                    damage_for_parent |= base.add_damage(self);
                });
                damage_for_parent
            },
            BoxDamageAction::InvalidateScrollableOverflow => {
                // In this case the node's fragments are preserved, but it or one of its descendants
                // had scrollable overflow damage, which means that scrollable overflow should be
                // cleared. This causes it to be recalculated the next time it's queried.
                self.node.with_layout_box_base_including_pseudos(|base| {
                    base.clear_scrollable_overflow_all_on_fragments();
                });
                forwarded_damage
            },
            BoxDamageAction::None => forwarded_damage,
        };

        // The box is preserved. Whether or not we run fragment tree layout, we need to
        // update any preserved layout data structures' style references, if *this*
        // element's style has changed.
        if !self.on_element.is_empty() && action.preserves_boxes() {
            self.node.repair_style(&layout_context.style_context);
        }

        // If doing a fragment tree layout, we also need to apply the LAYOUT_AFFECTED_BY_INFLOW_DESCENDANT
        // damage flag, unless this node is out of flow. In that case our ancestors are rebuilt, but
        // their resulting fragments should be equivalent to the previous ones.
        if !damage_for_parent.contains(LayoutDamage::DescendantHasBoxDamage) {
            if self.on_element.contains(LayoutDamage::Relayout) {
                damage_for_parent.insert(LayoutDamage::LayoutAffectedByInflowDescendant);
            }

            if damage_for_parent.contains(LayoutDamage::LayoutAffectedByInflowDescendant) &&
                self.node.is_absolutely_positioned()
            {
                damage_for_parent.remove(LayoutDamage::LayoutAffectedByInflowDescendant);
            }
        }

        damage_for_parent
    }

    fn box_damage_action(&self) -> BoxDamageAction {
        // When a parent box is going to be reconstructed, that overrides everything else.
        if self
            .from_parent
            .contains(LayoutDamage::DescendantHasBoxDamage)
        {
            return BoxDamageAction::RebuildAncestor;
        }

        // When this element or one of its descendants needs to be reconstructed, try to
        // rebuild it here. If that fails, an ancestor box will be reconstructed instead.
        let element_and_children_damage = self.on_element | self.from_children;
        if element_and_children_damage.contains(LayoutDamage::DescendantHasBoxDamage) {
            return BoxDamageAction::TryRebuild;
        }

        // If this element needs a new fragment layout, then invalidate fragment caches.
        if (self.from_parent | element_and_children_damage).contains(LayoutDamage::Relayout) {
            return BoxDamageAction::InvalidateFragmentCache;
        }

        // If the scrollable overflow of this element has changed, invalidate the
        // scrollable overflow.
        if element_and_children_damage.contains(LayoutDamage::RecalculateOverflow) {
            return BoxDamageAction::InvalidateScrollableOverflow;
        }

        BoxDamageAction::None
    }
}
