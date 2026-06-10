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
use crate::layout_root::LayoutRoot;

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

#[servo_tracing::instrument(skip_all)]
pub(crate) fn compute_damage_and_rebuild_box_tree<'dom>(
    box_tree: &mut Option<Arc<BoxTree>>,
    layout_context: &LayoutContext,
    dirty_root: ServoLayoutNode<'dom>,
    root_node: ServoLayoutNode<'dom>,
    damage_from_environment: LayoutDamage,
    layout_roots: &mut Vec<LayoutRoot<'dom>>,
) -> LayoutDamage {
    // First process damage below the dirty root, returning the damage that
    // should be propagated upward into the clean part of the tree.
    let layout_damage = compute_damage_and_rebuild_box_tree_below_dirty_root(
        layout_context,
        dirty_root,
        damage_from_environment,
        layout_roots,
    );

    // If there was no box tree at all at this point, a full box tree / fragment
    // tree layout is necessary and there is no point processing any other damage.
    if box_tree.is_none() {
        *box_tree = Some(Arc::new(BoxTree::construct(layout_context, root_node)));
        return layout_damage;
    }

    // Propagate the damage from the dirty part of the tree upward. In this part of
    // the traversal no elements can add damage, but they might isolate damage being
    // propagated upward between the dirty root and the root of the DOM.
    let layout_damage = compute_damage_and_rebuild_box_tree_above_dirty_root(
        layout_context,
        dirty_root,
        layout_damage,
        layout_roots,
    );

    // We could not find a place in the middle of the tree to run box tree reconstruction,
    // so just rebuild the whole tree.
    if layout_damage.contains(LayoutDamage::DescendantHasBoxDamage) {
        *box_tree = Some(Arc::new(BoxTree::construct(layout_context, root_node)));
    }

    layout_damage
}

#[expect(unsafe_code)]
#[servo_tracing::instrument(skip_all)]
pub(crate) fn compute_damage_and_rebuild_box_tree_above_dirty_root<'dom>(
    layout_context: &LayoutContext,
    dirty_root: ServoLayoutNode<'dom>,
    layout_damage: LayoutDamage,
    layout_roots: &mut Vec<LayoutRoot<'dom>>,
) -> LayoutDamage {
    // Cases where propagating damage up the tree is necessary:
    //
    // 1. Box tree layout of the dirty root is necessary, in which case we
    //    search for a place to re-run box tree layout and also invalidate
    //    all fragments and fragment caches to the root.
    // 2. Fragment tree layout needs to run again, in which case fragments
    //    and fragment caches need to be invalidated.
    // 3. Overflow is dirty, in which case overflow needs to be cleared.
    //
    // In every other case, just return early.
    let needs_fragment_tree_rebuild = layout_damage.contains(LayoutDamage::Relayout);
    let needs_overflow_recalculation = layout_damage.contains(LayoutDamage::RecalculateOverflow);
    if !needs_fragment_tree_rebuild && !needs_overflow_recalculation {
        assert!(!layout_damage.contains(LayoutDamage::DescendantCollectedAsLayoutRoot));
        return layout_damage;
    }

    let mut damage_for_parent = layout_damage;
    let mut maybe_parent_node = unsafe { dirty_root.dangerous_flat_tree_parent() };
    while let Some(parent_node) = maybe_parent_node {
        let damage_set = ElementDamageSet {
            node: parent_node,
            from_parent: LayoutDamage::empty(),
            on_element: LayoutDamage::empty(),
            from_children: damage_for_parent,
            // Ancestors above the dirty root do not have damage, so will never subsume
            // any existing layout roots, but they may isolate upward flowing fragment
            // tree damage.
            incoming_layout_root_count: layout_roots.len(),
        };

        damage_for_parent = damage_set.apply_damage(layout_context, layout_roots);
        maybe_parent_node = unsafe { parent_node.dangerous_flat_tree_parent() };
    }

    damage_for_parent
}

pub(crate) fn compute_damage_and_rebuild_box_tree_below_dirty_root<'dom>(
    layout_context: &LayoutContext,
    node: ServoLayoutNode<'dom>,
    damage_from_parent: LayoutDamage,
    layout_roots: &mut Vec<LayoutRoot<'dom>>,
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
        incoming_layout_root_count: layout_roots.len(),
    };

    // Depending on the incoming damage, it can be isolated, meaning that some damage
    // doesn't get passed down to children.
    let damage_for_children = damage_set.isolate_incoming_damage();

    // Propagate damage to children and gather the resulting damage into `from_children`.
    damage_set.propagate_damage_to_children(
        layout_context,
        has_dirty_descendants,
        damage_for_children,
        layout_roots,
    );

    // Apply the calculated damage to this element (perhaps triggering box tree layout),
    // and propagate resulting damage to ancestors.
    damage_set.apply_damage(layout_context, layout_roots)
}

enum BoxDamageAction<'a> {
    RebuildAncestor,
    TryRebuild,
    InvalidateFragmentTreeBelowLayoutRoot,
    CollectLayoutRoot(LayoutRoot<'a>),
    InvalidateFragmentTreeAboveLayoutRoot,
    InvalidateScrollableOverflow,
    None,
}

impl BoxDamageAction<'_> {
    fn rebuilds_box(&self) -> bool {
        matches!(self, Self::RebuildAncestor | Self::TryRebuild)
    }
}

pub(crate) struct ElementDamageSet<'a> {
    node: ServoLayoutNode<'a>,
    pub from_parent: LayoutDamage,
    pub on_element: LayoutDamage,
    pub from_children: LayoutDamage,
    pub incoming_layout_root_count: usize,
}

impl<'a> ElementDamageSet<'a> {
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
        layout_roots: &mut Vec<LayoutRoot<'a>>,
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
                    self.from_children |= compute_damage_and_rebuild_box_tree_below_dirty_root(
                        layout_context,
                        child,
                        damage_for_children,
                        layout_roots,
                    );
                }
            }
        }
    }

    /// Given the damage from this element, the parent, and children, determine what action to
    /// take for this element's boxes and return the damage that should be propagated to parents.
    fn apply_damage(
        self,
        layout_context: &LayoutContext<'_>,
        layout_roots: &mut Vec<LayoutRoot<'a>>,
    ) -> LayoutDamage {
        let only_layout_mode_damage =
            (self.from_parent | self.on_element | self.from_children).only_layout_modes();

        let invalidate_for_rebuild = || {
            self.node.unset_all_boxes();
            LayoutDamage::DescendantHasBoxDamage | LayoutDamage::Relayout
        };

        // This removes any dirty layout roots from descendants.
        let discard_any_descendant_layout_roots = |layout_roots: &mut Vec<LayoutRoot>| {
            layout_roots.truncate(self.incoming_layout_root_count);
        };

        let action = self.box_damage_action();
        let will_rebuild_box = action.rebuilds_box();
        let damage_for_parent = match action {
            BoxDamageAction::TryRebuild => {
                discard_any_descendant_layout_roots(layout_roots);

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
                discard_any_descendant_layout_roots(layout_roots);
                invalidate_for_rebuild()
            },
            BoxDamageAction::InvalidateFragmentTreeBelowLayoutRoot => {
                // In this case, this node's boxes are preserved! It's possible that we still need
                // to run fragment tree layout in this subtree due to an ancestor, this node, or a
                // descendant changing style. In that case, we ask the `LayoutBoxBase` to clear
                // any cached information that cannot be used.
                discard_any_descendant_layout_roots(layout_roots);

                let mut damage_for_parent =
                    (self.on_element | self.from_children) | self.from_parent.only_layout_modes();

                // This node also needed new fragment tree layout, so if any descendant
                // was collected as a layout root, it's now discarded. This means we
                // should also clear the damage (though harmless as Relayout takes
                // precedence) indicating that there was a collected layout root.
                damage_for_parent.remove(LayoutDamage::DescendantCollectedAsLayoutRoot);

                let mut inline_size_depends_on_content = false;
                self.node.with_layout_box_base_including_pseudos(|base| {
                    inline_size_depends_on_content |=
                        base.invalidate_caches_for_fragment_tree_layout(&self);
                });

                self.adjust_inline_content_size_damage(
                    &mut damage_for_parent,
                    inline_size_depends_on_content,
                );

                damage_for_parent
            },
            BoxDamageAction::CollectLayoutRoot(layout_root) => {
                // A layout root should only be collected if a parent node does not
                // produce damage requiring a fragment tree layout. This is essential
                // to ensure the invariant that layout roots are only collected when
                // they isolate damage from ancestors. If an ancestor has damage, a
                // layout root's final position depends on that ancestor's layout
                // and should never be a collected layout root.
                debug_assert!(!self.from_parent.contains(LayoutDamage::Relayout));

                // This removes any dirty layout roots from descendants and then adds this
                // node as a dirty layout root. As this node itself as a dirty layout
                // root, it subsumes all dirty descendant layout roots.
                discard_any_descendant_layout_roots(layout_roots);
                layout_roots.push(layout_root);

                self.node.with_layout_box_base_including_pseudos(|base| {
                    base.invalidate_caches(&self);
                });
                LayoutDamage::RecalculateOverflow |
                    LayoutDamage::DescendantCollectedAsLayoutRoot |
                    LayoutDamage::RecomputeInlineContentSizes
            },
            BoxDamageAction::InvalidateFragmentTreeAboveLayoutRoot => {
                // Damage propagation works exactly the same at the point the layout root is collected
                // and above it. Layout caches are invalidated and damage is adjusted, maybe limited
                // inline content size recalculation.
                let mut damage_for_parent = LayoutDamage::RecalculateOverflow |
                    LayoutDamage::DescendantCollectedAsLayoutRoot;

                let mut inline_size_depends_on_content = false;
                self.node.with_layout_box_base_including_pseudos(|base| {
                    inline_size_depends_on_content |= base.invalidate_caches(&self);
                });
                self.adjust_inline_content_size_damage(
                    &mut damage_for_parent,
                    inline_size_depends_on_content,
                );

                damage_for_parent
            },
            BoxDamageAction::InvalidateScrollableOverflow => {
                // In this case the node's fragments are preserved, but it or one of its descendants
                // had scrollable overflow damage, which means that scrollable overflow should be
                // cleared. This causes it to be recalculated the next time it's queried.
                self.node.with_layout_box_base_including_pseudos(|base| {
                    base.clear_scrollable_overflow_all_on_fragments();
                });
                only_layout_mode_damage
            },
            BoxDamageAction::None => only_layout_mode_damage,
        };

        // If this element's boxes are preserved and its style has changed, whether or not
        // we run fragment tree layout, we need to update any preserved layout data
        // structures' style references.
        if !self.on_element.is_empty() && !will_rebuild_box {
            self.node.repair_style(&layout_context.style_context);
        }

        damage_for_parent
    }

    fn box_damage_action(&self) -> BoxDamageAction<'a> {
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

        if element_and_children_damage.contains(LayoutDamage::Relayout) &&
            !self.from_parent.contains(LayoutDamage::Relayout) &&
            let Ok(layout_root) = LayoutRoot::try_from(self.node)
        {
            return BoxDamageAction::CollectLayoutRoot(layout_root);
        }

        // If this element needs a new fragment layout, then invalidate fragment caches
        // clear the resulting fragments, and clear scrollable overflow.
        if (self.from_parent | element_and_children_damage).contains(LayoutDamage::Relayout) {
            return BoxDamageAction::InvalidateFragmentTreeBelowLayoutRoot;
        }

        // If one of this element's descendants was collected as a layout root, then
        // invalidate fragment caches and clear scrollable overflow.
        if self
            .from_children
            .contains(LayoutDamage::DescendantCollectedAsLayoutRoot)
        {
            return BoxDamageAction::InvalidateFragmentTreeAboveLayoutRoot;
        }

        // If the scrollable overflow of this element has changed, invalidate the
        // scrollable overflow.
        if element_and_children_damage.contains(LayoutDamage::RecalculateOverflow) {
            return BoxDamageAction::InvalidateScrollableOverflow;
        }

        BoxDamageAction::None
    }

    fn adjust_inline_content_size_damage(
        &self,
        damage_for_parent: &mut LayoutDamage,
        inline_size_depends_on_content: bool,
    ) {
        let children_need_inline_content_size_recalculation = self
            .from_children
            .contains(LayoutDamage::RecomputeInlineContentSizes) &&
            inline_size_depends_on_content;
        damage_for_parent.set(
            LayoutDamage::RecomputeInlineContentSizes,
            !self.on_element.is_empty() || children_need_inline_content_size_recalculation,
        );
    }
}
