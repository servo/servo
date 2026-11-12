/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::VecDeque;
use std::fmt::Debug;
use std::sync::LazyLock;

use accesskit::{Affine, NodeId, Role};
use app_units::Au;
use bitflags::bitflags;
use euclid::Rect;
use layout_api::{
    ARIA_ROLE_TO_ACCESSKIT_ROLE, AccessibilityDamage, BoxAreaType, LayoutElement, LayoutNode,
    LayoutNodeType, SUPPORTED_ARIA_ROLES,
};
use log::trace;
use num_traits::ToPrimitive;
use regex::Regex;
use rustc_hash::{FxHashMap, FxHashSet};
use script::layout_dom::{ServoLayoutElement, ServoLayoutNode};
use servo_base::print_tree::PrintTree;
use servo_config::pref;
use style::dom::{NodeInfo, OpaqueNode};
use style_traits::CSSPixel;
use web_atoms::{LocalName, local_name, ns};
use webrender_api::units::LayoutVector2D;

use super::accessibility_tree::{AccessibilityTree, AccessibilityUpdate, TreeChange};
use crate::ArcRefCell;
use crate::accessibility::AccessibilityContext;
use crate::cell::WeakRefCell;
use crate::query::{BoxAreaInclusion, process_box_area_request};

/// Convert a rectangle as layout reports it into the one [`accesskit`] wants.
fn au_rect_to_accesskit_rect(rect: Rect<Au, CSSPixel>) -> accesskit::Rect {
    accesskit::Rect::new(
        rect.min_x().to_f64_px(),
        rect.min_y().to_f64_px(),
        rect.max_x().to_f64_px(),
        rect.max_y().to_f64_px(),
    )
}

fn scroll_offset_to_affine(layout_vector: LayoutVector2D) -> Affine {
    Affine::translate((
        -layout_vector.x.to_f64().unwrap_or(0.),
        -layout_vector.y.to_f64().unwrap_or(0.),
    ))
}

bitflags! {
    /// Flags which may be used in this or future updates to compute serialized properties for a
    /// given AccessibilityNode.
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    struct Flags: u16 {
        /// For a text node, whether we couldn't determine whether it's irrelevant whitespace.
        /// This occurs when a whitespace-only text node is preceded by a text node which ends with
        /// a non-whitespace character, and we haven't yet encountered the following text node to
        /// learn whether it begins with a non-whitespace character, making the whitespace relevant,
        /// or a whitespace character, making the whitespace irrelevant.
        const MayBeIrrelevantWhitespace = 0b0001;
    }
}

bitflags! {
    /// Flags tracking an [`AccessibilityNode`]'s dirty state during an update. All flags which are
    /// set during the update should be unset by the end of the update.
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub(super) struct DirtyState : u16 {
        /// At least one descendant of this node has unresolved damage from the DOM tree.
        const DescendantHasDamage = 0b0001;
        /// This node has unresolved damage from the DOM tree.
        const HasDamage = 0b0010;
        /// This node's data changed, but it hasn't yet been added to the [`AccessibilityUpdate`].
        const Updated = 0b0100;
        /// This node was added in this update, and any associated local damage in its subtree has
        /// yet to be resolved.
        const New = 0b1000;
    }
}

bitflags! {
    /// Damage which was caused by changes to the accessibility tree. These changes can cause other
    /// properties to need to be re-computed based on the updated values, either on the same node or
    /// on other nodes.
    #[derive(Clone, Copy, Default, Debug, Eq, PartialEq)]
    pub(super) struct LocalAccessibilityDamage: u16 {
        /// This node's children changed, and/or any node in its subtree changed.
        const SubtreeChanged = 0b0001;
        /// This node's computed role changed.
        const RoleChanged = 0b0010;
        /// This node's computed label or text value (for a text node) changed.
        const TextChanged = 0b0100;
        /// This node's visibility changed.
        const VisibilityChanged = 0b1000;
    }
}

bitflags! {
    #[derive(Clone, Copy, Default, Debug, Eq, PartialEq)]
    pub(super) struct SubtreeState: u8 {
        const New = 0b0001;
        const Hidden = 0b0010;
    }
}

#[derive(Clone)]
pub(super) struct AccessibilityNode {
    /// The unique ID for the node. This is used both as a key in [`AccessibilityTree`]'s cache of
    /// nodes, and as an identifier in [`accesskit`] datastructures: [`accesskit::Node`]s,
    /// [`accesskit::TreeUpdate`]s and [`accesskit::ActionRequest`]s.
    id: NodeId,
    /// The computed [`accesskit::Node`] data. This will be copied and serialized into a
    /// [`accesskit::TreeUpdate`] whenever it is changed during an update.
    accesskit_node: accesskit::Node,
    /// This node's parent, if any.
    parent_node: Option<WeakRefCell<AccessibilityNode>>,
    /// All this node's children.
    child_nodes: Vec<ArcRefCell<AccessibilityNode>>,
    /// This node's previous sibling, if any.
    previous_sibling: Option<WeakRefCell<AccessibilityNode>>,
    /// This node's next sibling, if any.
    next_sibling: Option<WeakRefCell<AccessibilityNode>>,
    /// The [`OpaqueNode`] for the DOM node which corresponds to this accessibility node, if any.
    /// An accessibility node may not correspond to a DOM node if it corresponds to a
    /// pseudo-element, or in a test.
    opaque_node: Option<OpaqueNode>,
    /// This node's scroll offset, if it is a scroll container which has scrolled. This is used to
    /// translate this node's children.
    scroll_offset: Option<LayoutVector2D>,
    /// Other data which may be used to calculate properties in this or future updates.
    flags: Flags,
    /// Any dirty state for the current update.
    dirty_state: DirtyState,
}

impl AccessibilityNode {
    pub(super) fn new(id: NodeId) -> Self {
        Self::new_with_role(id, Role::Unknown)
    }

    pub(super) fn new_with_role(id: NodeId, role: Role) -> Self {
        Self {
            id,
            accesskit_node: accesskit::Node::new(role),
            parent_node: None,
            child_nodes: vec![],
            previous_sibling: None,
            next_sibling: None,
            opaque_node: None,
            scroll_offset: None,
            flags: Flags::empty(),
            dirty_state: DirtyState::empty(),
        }
    }

    /// Update this node and its subtree based on damage from the DOM.
    ///
    /// - First, if this node has damage from the DOM to be resolved, update the node from the DOM
    ///   tree, recursively populating any new children.
    /// - Next, recursively call this method for any children which are dirty, or have dirty
    ///   descendants.
    /// - Finally, update any properties on this node which are may have changed due to other
    ///   changes in the tree.
    ///
    /// At the end of this method, both `has_dirty_descendants` and `is_dirty` should be false for
    /// this node and all its descendants.
    pub(super) fn update_subtree<'update>(
        &mut self,
        ref_self: ArcRefCell<Self>,
        damage_from_parent: AccessibilityDamage,
        mut subtree_state: SubtreeState,
        context: &AccessibilityContext,
        tree: &mut AccessibilityTree,
        update: &mut AccessibilityUpdate<'update>,
    ) -> LocalAccessibilityDamage {
        let mut local_damage = LocalAccessibilityDamage::empty();

        let dom_node = update.take_dom_node(&self.id);
        let damage = update.take_damage(&self.id) | damage_from_parent;
        let mut children_changed = false;
        let in_new_subtree = if self.dirty_state.is_new() {
            subtree_state |= SubtreeState::New;
            true
        } else {
            subtree_state.intersects(SubtreeState::New)
        };

        if in_new_subtree && CONTAINER_ROLES.contains(&self.role()) {
            update.closest_previous_text = None;
        }

        if let Some(dom_node) = dom_node {
            local_damage |= self.update_properties_and_children_from_dom_node(
                &ref_self,
                &dom_node,
                damage,
                subtree_state,
                tree,
                update,
            );

            let hidden = subtree_state.intersects(SubtreeState::Hidden);
            local_damage |=
                self.update_node_from_layout(&dom_node, damage, hidden, context, update);

            if local_damage.contains(LocalAccessibilityDamage::SubtreeChanged) {
                children_changed = true;
            }

            self.dirty_state -= DirtyState::HasDamage;
            self.dirty_state -= DirtyState::New;
        }

        let layout_damage = damage & AccessibilityDamage::Layout;
        if self.is_hidden() {
            subtree_state.insert(SubtreeState::Hidden);
        }
        if self.dirty_state.descendant_has_damage() || !layout_damage.is_empty() {
            for child_node in self.children() {
                let child_node_ref = child_node.clone();
                let mut child_node = child_node.borrow_mut();
                let child_local_damage = child_node.update_subtree(
                    child_node_ref,
                    layout_damage,
                    subtree_state,
                    context,
                    tree,
                    update,
                );
                if !child_local_damage.is_empty() {
                    local_damage.insert(LocalAccessibilityDamage::SubtreeChanged);
                    if child_local_damage.contains(LocalAccessibilityDamage::VisibilityChanged) {
                        children_changed = true;
                    }
                }
            }
        }
        self.dirty_state -= DirtyState::DescendantHasDamage;

        if children_changed && let Some(scroll_offset) = self.scroll_offset {
            // If this node may have new, or newly-visible, children, update their scroll offsets.
            self.set_scroll_offset(scroll_offset, update);
        }

        local_damage.insert(self.update_node_local(local_damage, update));

        if self.dirty_state.updated() {
            update.add(self);
        }

        if in_new_subtree && CONTAINER_ROLES.contains(&self.role()) {
            update.closest_previous_text = None;
        }

        local_damage
    }

    /// Update each of this node's ancestors based on changes which have already been applied in the
    /// tree.
    pub(super) fn update_ancestors(
        &self,
        local_damage: LocalAccessibilityDamage,
        update: &mut AccessibilityUpdate,
    ) {
        if local_damage.is_empty() {
            return;
        }
        for node in self.ancestors() {
            let mut node = node.borrow_mut();
            node.update_node_local(LocalAccessibilityDamage::SubtreeChanged, update);
            node.dirty_state -= DirtyState::DescendantHasDamage;
            if node.dirty_state.updated() {
                update.add(&mut node);
            }
        }
    }

    pub(super) fn initialize_from_dom_node(&mut self, dom_node: &ServoLayoutNode) {
        self.opaque_node = Some(dom_node.opaque());
        if let Some(dom_element) = dom_node.as_element() {
            let local_name = dom_element.local_name().to_ascii_lowercase();
            self.set_html_tag(&local_name);
        }
    }

    /// Update the given [`AccessibilityNode`] from its corresponding DOM node and
    /// [`AccessibilityDamage`].
    /// If it has new children, those will be created here, but not yet populated.
    // Any changed nodes will be added to the given [`AccessibilityUpdate`].
    fn update_properties_and_children_from_dom_node<'update>(
        &mut self,
        ref_self: &ArcRefCell<Self>,
        dom_node: &ServoLayoutNode<'update>,
        dom_damage: AccessibilityDamage,
        subtree_state: SubtreeState,
        tree: &mut AccessibilityTree,
        update: &mut AccessibilityUpdate<'update>,
    ) -> LocalAccessibilityDamage {
        let mut local_damage = LocalAccessibilityDamage::empty();

        if dom_damage.is_empty() {
            return local_damage;
        }

        // We check for layout damage here because we need to walk the DOM children of nodes with
        // layout damage in order to be able to recompute their bounds. Text nodes have neither
        // bounds nor child nodes, so if the only damage is layout, we can early return here.
        if dom_damage == AccessibilityDamage::Layout && dom_node.is_text_node() {
            return local_damage;
        }

        update.counters.nodes_updated_from_dom += 1;

        if dom_damage.intersects(AccessibilityDamage::Node) {
            local_damage.insert(self.update_properties_from_dom_node(
                ref_self,
                dom_node,
                subtree_state,
                update,
            ));
        }

        if dom_damage.intersects(AccessibilityDamage::Children | AccessibilityDamage::Layout) {
            // If this node has damage from layout, this ensures that all of its children have
            // their corresponding DOM nodes in `update`.
            local_damage
                .insert(self.update_children_from_dom_node(ref_self, dom_node, tree, update));
        }

        local_damage
    }

    /// Update this node's [`Self::children`] from its corresponding DOM node.
    /// If it has new children, those will be created here, but not yet populated.
    fn update_children_from_dom_node<'update>(
        &mut self,
        ref_self: &ArcRefCell<AccessibilityNode>,
        dom_node: &ServoLayoutNode<'update>,
        tree: &mut AccessibilityTree,
        update: &mut AccessibilityUpdate<'update>,
    ) -> LocalAccessibilityDamage {
        let mut remaining_dom_children = dom_node.flat_tree_children().peekable();
        let mut old_child_ids = self.child_ids().iter().peekable();

        // Iterate over existing children and DOM children while they match. No action is necessary
        // for these nodes.
        let mut unchanged_count = 0usize;
        while let Some(&old_id) = old_child_ids.peek() &&
            let Some(dom_child) = remaining_dom_children.peek()
        {
            if tree.existing_id_for_opaque(dom_child.opaque()) == Some(*old_id) {
                update.insert_dom_node(*old_id, *dom_child);
                unchanged_count += 1;
                old_child_ids.next();
                remaining_dom_children.next();
            } else {
                break;
            }
        }

        // If we iterated over all the DOM children without finding any changes, we're done.
        if old_child_ids.peek().is_none() && remaining_dom_children.peek().is_none() {
            return LocalAccessibilityDamage::empty();
        }

        // Remove all child nodes after the first `unchanged_count`.
        self.child_nodes.truncate(unchanged_count);
        let mut new_child_ids = Vec::from(self.child_ids());
        for removed_child_id in new_child_ids.split_off(unchanged_count) {
            update.set_tree_state_change(removed_child_id, TreeChange::Removed);
        }

        // Then, (re-)add all the remaining DOM children. Note that this means that some children
        // may end up being "Moved" even though they haven't changed parents, and may even be in the
        // same position as previously.
        if let Some(last) = self.child_nodes.last() {
            last.borrow_mut().next_sibling = None;
        }
        let weak_self = ref_self.downgrade();
        let mut previous_sibling = self.child_nodes.last().cloned();
        for dom_child in remaining_dom_children {
            let (child_id, child_ref) = tree.get_or_create_node(&dom_child, update);
            // We need to ensure every AccessibilityNode which may be updated has a corresponding
            // DOM node available to be retrieved from the AccessibilityUpdate.
            update.insert_dom_node(child_id, dom_child);

            // Update self.child_nodes in place.
            self.child_nodes.push(child_ref.clone());
            new_child_ids.push(child_id);

            let mut child = child_ref.borrow_mut();
            child.parent_node = Some(weak_self.clone());
            child.previous_sibling = previous_sibling
                .as_ref()
                .map(|node_ref| node_ref.downgrade());
            child.next_sibling = None;
            if let Some(ref previous_sibling) = previous_sibling {
                previous_sibling.borrow_mut().next_sibling = Some(child_ref.downgrade());
            }
            previous_sibling = Some(child_ref.clone());

            if update.is_new(&child_id) {
                self.dirty_state |= DirtyState::DescendantHasDamage;
            } else {
                update.set_tree_state_change(child_id, TreeChange::PendingMove);
            }

            self.dirty_state
                .propagate_descendant_has_damage(child.dirty_state);
        }

        // We can't update the AccessKit node's `children` in place, so we build up the full list
        // and then set it here.
        self.accesskit_node.set_children(new_child_ids);
        self.dirty_state |= DirtyState::Updated;

        LocalAccessibilityDamage::SubtreeChanged
    }

    /// Update this node's properties from its corresponding DOM node.
    fn update_properties_from_dom_node(
        &mut self,
        ref_self: &ArcRefCell<Self>,
        dom_node: &ServoLayoutNode,
        subtree_state: SubtreeState,
        update: &mut AccessibilityUpdate,
    ) -> LocalAccessibilityDamage {
        let mut local_damage = LocalAccessibilityDamage::empty();
        local_damage.insert(self.set_role(role_from_dom_node(dom_node)));
        if dom_node.type_id() == Some(LayoutNodeType::Text) {
            let text_content = dom_node.text_content();
            trace!("node text content = {text_content:?}");
            // FIXME: this should take into account editing selection units (grapheme clusters?)
            local_damage.insert(self.set_value(&text_content));

            if subtree_state.intersects(SubtreeState::New) {
                // While walking a newly added subtree, compute irrelevant whitespace as we go, so
                // we don't need to repeatedly walk across the entire subtree.
                let closest_previous_text = std::mem::take(&mut update.closest_previous_text);
                let is_irrelevant_whitespace =
                    if let Some(closest_previous_text) = closest_previous_text.as_ref() {
                        let closest_previous_text = closest_previous_text.borrow();
                        self.is_irrelevant_whitespace(Some(&closest_previous_text))
                    } else {
                        self.is_irrelevant_whitespace(None)
                    } || self.flags.contains(Flags::MayBeIrrelevantWhitespace);
                local_damage.insert(self.set_is_hidden(is_irrelevant_whitespace));
                if let Some(closest_previous_text) = closest_previous_text &&
                    !is_irrelevant_whitespace
                {
                    // In theory, we should accumulate local damage from this potential change, but
                    // since the entire subtree is new and this change by definition shouldn't
                    // change the computed text of any ancestor nodes (since only _irrelevant_
                    // whitespace is hidden), we can ignore it here.
                    let _ = self
                        .maybe_unhide_previous_whitespace_only_node(closest_previous_text, update);
                }
                update.closest_previous_text = Some(ref_self.clone());
            }
        }

        local_damage
    }

    /// Update this node's bounds from the current layout geometry.
    fn update_node_from_layout(
        &mut self,
        dom_node: &ServoLayoutNode<'_>,
        layout_damage: AccessibilityDamage,
        hidden: bool,
        context: &AccessibilityContext,
        update: &mut AccessibilityUpdate,
    ) -> LocalAccessibilityDamage {
        let mut local_damage = LocalAccessibilityDamage::empty();

        // Don't update bounds on nodes in hidden subtrees.
        if hidden || !layout_damage.intersects(AccessibilityDamage::Layout) {
            return local_damage;
        }

        if let Some(dom_element) = dom_node.as_element() &&
            dom_element.style_data().is_some()
        {
            let data = dom_element.element_data();
            let style = data.styles.primary();
            if style.get_display().is_none() {
                self.clear_bounds();
                local_damage.insert(self.set_hidden());
            } else {
                local_damage.insert(self.clear_hidden());
            }
        }

        if self.is_hidden() {
            return local_damage;
        }

        update.counters.nodes_updated_bounds += 1;

        // Border box without transforms. Bounds are in CSS pixels, relative to the document origin;
        // scroll containers set translations on their child nodes, and the embedder's graft node
        // carries the transform that composes them into AccessKit's coordinate space (see the
        // "Coordinates" section of
        // <https://docs.rs/accesskit/latest/accesskit/struct.Node.html>).
        // TODO(#47166): This doesn't take any CSS transforms into account.
        let bounds = process_box_area_request(
            context.layout_thread,
            context.stacking_context_tree,
            *dom_node,
            BoxAreaType::Border,
            BoxAreaInclusion::Inlines,
        )
        .map(au_rect_to_accesskit_rect);

        // For now only nodes with a box of their own get bounds; anything else, including
        // `display: none` content, gets its bounds cleared. That leaves two kinds of nodes
        // without geometry which assistive technology would like to have some:
        //
        // TODO(#47164): A text node never has bounds of its own: `LayoutBox::Text` has no
        // `LayoutBoxBase`, and `Fragment::Text` has no box area, so the query above always returns
        // `None` for one. Text nodes should get the union of the rectangles of their own
        // `Fragment::Text` fragments, once `cumulative_box_area_rect()` can handle those.
        //
        // TODO(#47163): A `display: contents` element generates no box either. Other
        // engines (Blink, WebKit, Gecko) compute its bounds as the union of the bounding boxes of
        // its rendered descendants.
        match bounds {
            Some(bounds) => self.set_bounds(bounds),
            None => self.clear_bounds(),
        }
        local_damage
    }

    /// Update this node's properties based on changes already made to the accessibility tree.
    /// For example, if there were nodes added or removed in its subtree, its computed text may have
    /// changed, so that will be recomputed here.
    /// If any changes are made, add this node to the given [`AccessibilityUpdate`].
    fn update_node_local(
        &mut self,
        local_damage: LocalAccessibilityDamage,
        update: &mut AccessibilityUpdate,
    ) -> LocalAccessibilityDamage {
        let mut new_damage = LocalAccessibilityDamage::empty();
        if local_damage.is_empty() {
            return new_damage;
        }
        update.counters.nodes_updated_from_tree += 1;

        if local_damage.contains(LocalAccessibilityDamage::SubtreeChanged) ||
            local_damage.contains(LocalAccessibilityDamage::RoleChanged)
        {
            if let Some(text) = self.label_from_descendants() {
                new_damage.insert(self.set_label(text.as_str()));
            } else {
                new_damage.insert(self.clear_label());
            }
        }

        new_damage
    }

    /// If this node is a text node containing only whitespace, we need to hide it unless the
    /// whitespace is significant.
    ///
    /// The whitespace is significant if this node is between two other text nodes within the same
    /// container node, and the two neighbouring text nodes have non-whitespace characters adjacent
    /// to this node.
    ///
    /// For this method, we pass in only the closest previous text node, since we can determine that
    /// during a tree walk. If the previous text node doesn't provide enough information to
    /// determine whether this node is irrelevant whitespace, this node is marked as
    ///  and this method returns `false`.
    ///
    /// [`Self::maybe_unhide_previous_whitespace_only_node()`] will determine whether a node already
    /// marked as [`Flags::MayBeIrrelevantWhitespace`] should be hidden based on its subsequent text
    /// node.
    fn is_irrelevant_whitespace(
        &mut self,
        closest_previous_text: Option<&AccessibilityNode>,
    ) -> bool {
        static ENDS_WITH_WHITESPACE: LazyLock<Regex> =
            LazyLock::new(|| Regex::new(r"\S\s+$").unwrap());

        self.flags -= Flags::MayBeIrrelevantWhitespace;

        let Some(value) = self.value() else {
            return false;
        };

        // If this node contains non-whitespace characters, keep it.
        if !value.trim().is_empty() {
            return false;
        }

        // If this node is not in between two other text nodes, ignore it.
        let Some(closest_previous_text) = closest_previous_text.as_ref() else {
            return true;
        };

        if closest_previous_text.is_hidden() {
            // By the same logic as the previous node.
            return true;
        }
        let previous_value = closest_previous_text
            .value()
            .expect("All text nodes should have values");
        if ENDS_WITH_WHITESPACE.is_match(previous_value) {
            return true;
        }

        // We don't know for sure whether this node is irrelevant whitespace until we visit the
        // subsequent text node or learn that there isn't one in the same container, so mark it as
        // MayBeIrrelevantWhitespace but return false.
        self.flags |= Flags::MayBeIrrelevantWhitespace;
        false
    }

    /// If this node doesn't begin with a whitespace character, unhide any immediately-preceding
    /// whitespace-only node, and return any [`LocalAccessibilityDamage`] resulting from the change.
    fn maybe_unhide_previous_whitespace_only_node(
        &self,
        closest_previous_text: ArcRefCell<AccessibilityNode>,
        update: &mut AccessibilityUpdate,
    ) -> LocalAccessibilityDamage {
        static STARTS_WITH_WHITESPACE: LazyLock<Regex> =
            LazyLock::new(|| Regex::new(r"^\s+\S").unwrap());
        let Some(value) = self.value() else {
            return LocalAccessibilityDamage::empty();
        };

        // If this node's value starts with whitespace, the previous text node can stay hidden.
        if STARTS_WITH_WHITESPACE.is_match(value) {
            return LocalAccessibilityDamage::empty();
        }

        let mut closest_previous_text = closest_previous_text.borrow_mut();
        if !closest_previous_text
            .flags
            .contains(Flags::MayBeIrrelevantWhitespace)
        {
            return LocalAccessibilityDamage::empty();
        }

        let damage = closest_previous_text.clear_hidden();
        update.add(&mut closest_previous_text);
        closest_previous_text.flags -= Flags::MayBeIrrelevantWhitespace;
        damage
    }

    fn label_from_descendants(&self) -> Option<String> {
        if !NAME_FROM_CONTENTS_ROLES.contains(&self.role()) {
            return None;
        }
        let mut children = VecDeque::from_iter(self.children().cloned());
        let mut text = String::new();
        while let Some(child) = children.pop_front() {
            let child = child.borrow();
            if child.is_hidden() {
                continue;
            }
            match child.role() {
                Role::TextRun => {
                    if let Some(child_text) = child.value() {
                        text.push_str(child_text);
                    }
                },
                _ => {
                    for node in child.children().rev() {
                        children.push_front(node.clone());
                    }
                },
            }
        }
        Some(text.trim().to_owned())
    }

    pub(super) fn print(&self, print_tree: &mut PrintTree, tree: &AccessibilityTree) {
        let focused = if tree.focused_node_id() == Some(self.id) {
            "[focused] "
        } else {
            ""
        };
        let node_string = format!("{focused}{self:?}");

        if self.child_nodes.is_empty() {
            print_tree.add_item(node_string);
            return;
        }

        print_tree.new_level(node_string);

        for child in self.children() {
            child.borrow().print(print_tree, tree);
        }
        print_tree.end_level();
    }

    pub(super) fn clone_accesskit_node(&self) -> accesskit::Node {
        self.accesskit_node.clone()
    }

    pub(super) fn id(&self) -> NodeId {
        self.id
    }

    pub(super) fn opaque_node(&self) -> Option<OpaqueNode> {
        self.opaque_node
    }

    pub(super) fn dirty_state(&self) -> DirtyState {
        self.dirty_state
    }

    pub(super) fn add_dirty_state(&mut self, state: DirtyState) {
        self.dirty_state |= state;
    }

    pub(super) fn remove_dirty_state(&mut self, state: DirtyState) {
        self.dirty_state -= state;
    }

    pub(super) fn parent(&self) -> Option<ArcRefCell<AccessibilityNode>> {
        self.parent_node.as_ref().and_then(|weak| weak.upgrade())
    }

    pub(super) fn children(
        &self,
    ) -> impl DoubleEndedIterator<Item = &ArcRefCell<AccessibilityNode>> {
        self.child_nodes.iter()
    }

    #[cfg(test)]
    pub(super) fn previous_sibling(&self) -> Option<ArcRefCell<AccessibilityNode>> {
        self.previous_sibling
            .as_ref()
            .and_then(|weak| weak.upgrade())
    }

    #[cfg(test)]
    pub(super) fn next_sibling(&self) -> Option<ArcRefCell<AccessibilityNode>> {
        self.next_sibling.as_ref().and_then(|weak| weak.upgrade())
    }

    #[cfg(test)]
    pub(super) fn set_parent_for_testing(&mut self, parent: ArcRefCell<AccessibilityNode>) {
        self.parent_node = Some(parent.downgrade());
    }

    #[cfg(test)]
    pub(super) fn set_children_for_testing(
        &mut self,
        child_ids: Vec<NodeId>,
        child_nodes: Vec<ArcRefCell<AccessibilityNode>>,
    ) {
        self.accesskit_node.set_children(child_ids);
        self.child_nodes = child_nodes;
    }

    #[cfg(test)]
    pub(super) fn set_previous_sibling_for_testing(
        &mut self,
        sibling: ArcRefCell<AccessibilityNode>,
    ) {
        self.previous_sibling = Some(sibling.downgrade());
    }

    #[cfg(test)]
    pub(super) fn set_next_sibling_for_testing(&mut self, sibling: ArcRefCell<AccessibilityNode>) {
        self.next_sibling = Some(sibling.downgrade());
    }

    pub(super) fn ancestors(&self) -> impl Iterator<Item = ArcRefCell<AccessibilityNode>> {
        AccessibilityNodeIterator::new(self.parent(), |node| node.parent_node.clone()?.upgrade())
    }

    #[cfg(test)]
    pub(super) fn after(
        &self,
        stay_within: Option<NodeId>,
    ) -> impl Iterator<Item = ArcRefCell<AccessibilityNode>> {
        AccessibilityNodeTreeOrderIterator::new(self, stay_within, TreeOrder::Forwards)
    }

    #[cfg(test)]
    pub(super) fn before(
        &self,
        stay_within: Option<NodeId>,
    ) -> impl Iterator<Item = ArcRefCell<AccessibilityNode>> {
        AccessibilityNodeTreeOrderIterator::new(self, stay_within, TreeOrder::Backwards)
    }

    pub(super) fn child_ids(&self) -> &[NodeId] {
        self.accesskit_node.children()
    }

    pub(super) fn set_scroll_offset(
        &mut self,
        offset: LayoutVector2D,
        update: &mut AccessibilityUpdate,
    ) {
        self.scroll_offset = Some(offset);
        let transform = scroll_offset_to_affine(offset);
        for child in self.children() {
            let mut child = child.borrow_mut();
            if child.is_hidden() {
                continue;
            }
            child.set_transform(transform);
            if child.dirty_state.updated() {
                update.add(&mut child);
            }
        }
    }

    pub(super) fn role(&self) -> Role {
        self.accesskit_node.role()
    }

    pub(super) fn set_role(&mut self, role: Role) -> LocalAccessibilityDamage {
        if role == self.accesskit_node.role() {
            return LocalAccessibilityDamage::empty();
        }
        self.accesskit_node.set_role(role);
        self.dirty_state |= DirtyState::Updated;
        LocalAccessibilityDamage::RoleChanged
    }

    fn label(&self) -> Option<&str> {
        self.accesskit_node.label()
    }

    fn set_label(&mut self, label: &str) -> LocalAccessibilityDamage {
        if Some(label) == self.accesskit_node.label() {
            return LocalAccessibilityDamage::empty();
        }
        self.accesskit_node.set_label(label);
        self.dirty_state |= DirtyState::Updated;
        LocalAccessibilityDamage::TextChanged
    }

    fn clear_label(&mut self) -> LocalAccessibilityDamage {
        if self.accesskit_node.label().is_none() {
            return LocalAccessibilityDamage::empty();
        }
        self.accesskit_node.clear_label();
        self.dirty_state |= DirtyState::Updated;
        LocalAccessibilityDamage::TextChanged
    }

    fn html_tag(&self) -> Option<&str> {
        self.accesskit_node.html_tag()
    }

    fn set_html_tag(&mut self, html_tag: &str) {
        if Some(html_tag) == self.accesskit_node.html_tag() {
            return;
        }
        self.accesskit_node.set_html_tag(html_tag);
        self.dirty_state |= DirtyState::Updated;
    }

    fn value(&self) -> Option<&str> {
        self.accesskit_node.value()
    }

    fn set_value(&mut self, value: &str) -> LocalAccessibilityDamage {
        if Some(value) == self.accesskit_node.value() {
            return LocalAccessibilityDamage::empty();
        }
        self.accesskit_node.set_value(value);
        self.dirty_state |= DirtyState::Updated;
        LocalAccessibilityDamage::TextChanged
    }

    fn is_hidden(&self) -> bool {
        self.accesskit_node.is_hidden()
    }

    fn set_is_hidden(&mut self, is_hidden: bool) -> LocalAccessibilityDamage {
        if is_hidden {
            self.set_hidden()
        } else {
            self.clear_hidden()
        }
    }

    fn set_hidden(&mut self) -> LocalAccessibilityDamage {
        if self.is_hidden() {
            return LocalAccessibilityDamage::empty();
        }
        self.accesskit_node.set_hidden();
        self.dirty_state |= DirtyState::Updated;
        LocalAccessibilityDamage::VisibilityChanged
    }

    fn clear_hidden(&mut self) -> LocalAccessibilityDamage {
        if !self.is_hidden() {
            return LocalAccessibilityDamage::empty();
        }

        self.accesskit_node.clear_hidden();
        self.dirty_state |= DirtyState::Updated;
        LocalAccessibilityDamage::VisibilityChanged
    }

    fn bounds(&self) -> Option<accesskit::Rect> {
        self.accesskit_node.bounds()
    }

    fn set_bounds(&mut self, bounds: accesskit::Rect) {
        if Some(bounds) == self.accesskit_node.bounds() {
            return;
        }
        self.accesskit_node.set_bounds(bounds);
        self.dirty_state |= DirtyState::Updated;
    }

    fn clear_bounds(&mut self) {
        if self.accesskit_node.bounds().is_none() {
            return;
        }
        self.accesskit_node.clear_bounds();
        self.dirty_state |= DirtyState::Updated;
    }

    fn set_transform(&mut self, transform: Affine) {
        // TODO(#47166): Right now a node will only ever have a single transform from a scroll
        // container, if any. Once we correctly support CSS transforms, a node may have multiple
        // transforms, which we'll need to be able to combine.
        if self.accesskit_node.transform() == Some(&transform) {
            return;
        }
        if transform == Affine::IDENTITY {
            self.clear_transform();
            return;
        }
        self.accesskit_node.set_transform(transform);
        self.dirty_state |= DirtyState::Updated;
    }

    fn clear_transform(&mut self) {
        if self.accesskit_node.transform().is_none() {
            return;
        }
        self.accesskit_node.clear_transform();
        self.dirty_state |= DirtyState::Updated;
    }
}

pub(super) struct NodeExpectations {
    pub(super) parent_node: Option<WeakRefCell<AccessibilityNode>>,
    pub(super) previous_sibling: Option<WeakRefCell<AccessibilityNode>>,
    pub(super) next_sibling: Option<WeakRefCell<AccessibilityNode>>,
}

pub(super) trait Integrity {
    fn assert_integrity(&self, expected: NodeExpectations);
}

impl Integrity for AccessibilityNode {
    fn assert_integrity(&self, expected: NodeExpectations) {
        debug_assert!(pref!(expensive_accessibility_test_assertions_enabled));

        assert_relation(self, &self.parent_node, expected.parent_node, "parent");
        assert_relation(
            self,
            &self.previous_sibling,
            expected.previous_sibling,
            "previous sibling",
        );
        assert_relation(
            self,
            &self.next_sibling,
            expected.next_sibling,
            "next sibling",
        );

        assert!(
            self.dirty_state.is_empty(),
            "{self:?} has dirty state {:?}",
            self.dirty_state
        );

        let children_ids: Vec<_> = self.children().map(|child| child.borrow().id).collect();
        assert_eq!(
            children_ids,
            self.child_ids(),
            "children() IDs didn't match child_ids() for {self:?}"
        );
    }
}

fn assert_relation(
    node: &AccessibilityNode,
    actual: &Option<WeakRefCell<AccessibilityNode>>,
    expected: Option<WeakRefCell<AccessibilityNode>>,
    relation: &str,
) {
    let Some(actual) = actual else {
        assert!(
            expected.is_none(),
            "Expected {relation} but no actual {relation} on {node:?}"
        );
        return;
    };
    let expected = expected
        .unwrap_or_else(|| panic!("Actual {relation} but no expected {relation} on {node:?}",));
    let expected = expected
        .upgrade()
        .unwrap_or_else(|| panic!("Expected {relation} on {node:?} was dropped"));
    let actual = actual
        .upgrade()
        .unwrap_or_else(|| panic!("Actual {relation} on {node:?} was dropped"));
    assert!(
        actual.ptr_eq(&expected),
        "actual {:?} did not match expected {:?} for {relation} of {:?}",
        actual,
        expected,
        node
    );
}

impl Debug for AccessibilityNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_hidden() {
            write!(f, "[hidden] ")?;
        }
        write!(f, "{:?}: {:?}", self.id, self.role())?;
        if let Some(html_tag) = self.html_tag() {
            write!(f, " (html_tag: {html_tag:?})")?;
        }
        if let Some(label) = self.label() {
            write!(f, "\nlabel: {label:?}")?;
        }
        if let Some(bounds) = self.bounds() {
            write!(f, "\nbounds: {bounds:?}")?;
        }
        if !self.child_ids().is_empty() {
            write!(f, "\nchildren: {:?}", self.child_ids())?;
        }
        Ok(())
    }
}

/// <https://w3c.github.io/aria/#host_general_role>
fn role_from_role_attribute(dom_element: &ServoLayoutElement<'_>) -> Option<Role> {
    let role_attribute = dom_element.attribute(&ns!(), &local_name!("role"))?;
    role_attribute
        .as_tokens()
        .iter()
        .filter_map(|role_name_in_attribute| SUPPORTED_ARIA_ROLES.get(role_name_in_attribute))
        .next()
        .and_then(|aria_role| ARIA_ROLE_TO_ACCESSKIT_ROLE.get(aria_role))
        .cloned()
}

fn role_from_dom_node(dom_node: &ServoLayoutNode<'_>) -> Role {
    if let Some(dom_element) = dom_node.as_element() {
        role_from_role_attribute(&dom_element).unwrap_or_else(|| {
            let local_name = dom_element.local_name().to_ascii_lowercase();
            *HTML_ELEMENT_ROLE_MAPPINGS
                .get(&local_name)
                .unwrap_or(&Role::GenericContainer)
        })
    } else if dom_node.type_id() == Some(LayoutNodeType::Text) {
        Role::TextRun
    } else {
        Role::GenericContainer
    }
}

struct AccessibilityNodeIterator<I>
where
    I: Fn(&AccessibilityNode) -> Option<ArcRefCell<AccessibilityNode>>,
{
    next_value: Option<ArcRefCell<AccessibilityNode>>,
    next_fn: I,
}

impl<I> AccessibilityNodeIterator<I>
where
    I: Fn(&AccessibilityNode) -> Option<ArcRefCell<AccessibilityNode>>,
{
    fn new(next_value: Option<ArcRefCell<AccessibilityNode>>, next_fn: I) -> Self {
        AccessibilityNodeIterator {
            next_value,
            next_fn,
        }
    }
}

impl<I> Iterator for AccessibilityNodeIterator<I>
where
    I: Fn(&AccessibilityNode) -> Option<ArcRefCell<AccessibilityNode>>,
{
    type Item = ArcRefCell<AccessibilityNode>;

    fn next(&mut self) -> Option<Self::Item> {
        let next_value = self.next_value.take();
        self.next_value = next_value
            .as_ref()
            .and_then(|node| (self.next_fn)(&node.borrow()));
        next_value
    }
}

#[cfg(test)]
#[derive(Copy, Clone)]
enum TreeOrder {
    Forwards,
    Backwards,
}

#[cfg(test)]
struct AccessibilityNodeTreeOrderIterator {
    previous: Option<ArcRefCell<AccessibilityNode>>,
    next_value: Option<ArcRefCell<AccessibilityNode>>,
    stay_within: Option<NodeId>,
    order: TreeOrder,
}

#[cfg(test)]
impl AccessibilityNodeTreeOrderIterator {
    fn new(start_node: &AccessibilityNode, stay_within: Option<NodeId>, order: TreeOrder) -> Self {
        let previous = None;
        let next_value = Self::compute_next_value(start_node, &previous, &stay_within, order);
        AccessibilityNodeTreeOrderIterator {
            previous,
            next_value,
            stay_within,
            order,
        }
    }
}

#[cfg(test)]
impl Iterator for AccessibilityNodeTreeOrderIterator {
    type Item = ArcRefCell<AccessibilityNode>;

    fn next(&mut self) -> Option<Self::Item> {
        let next_value = self.next_value.take();
        self.next_value = next_value.as_ref().and_then(|node| {
            Self::compute_next_value(
                &node.borrow(),
                &self.previous,
                &self.stay_within,
                self.order,
            )
        });
        self.previous = next_value.clone();
        next_value
    }
}

#[cfg(test)]
impl AccessibilityNodeTreeOrderIterator {
    fn compute_next_value(
        node: &AccessibilityNode,
        previous: &Option<ArcRefCell<AccessibilityNode>>,
        stay_within: &Option<NodeId>,
        order: TreeOrder,
    ) -> Option<ArcRefCell<AccessibilityNode>> {
        match order {
            TreeOrder::Forwards => {
                let mut descend = false;
                if let Some(previous) = previous.as_ref() {
                    let previous_id = previous.borrow().id();
                    if let Some(parent) = node.parent() &&
                        parent.borrow().id() == previous_id
                    {
                        descend = true;
                    } else if let Some(sibling) = node.previous_sibling() {
                        let sibling = sibling.borrow();
                        if sibling.id() == previous_id {
                            descend = true;
                        } else {
                            for ancestor in previous.borrow().ancestors() {
                                if sibling.id() == ancestor.borrow().id() {
                                    descend = true;
                                    break;
                                }
                            }
                        }
                    }
                } else {
                    descend = true;
                }

                if descend && let Some(first_child) = node.children().next() {
                    return Some(first_child.clone());
                }

                if let Some(next_sibling) = node.next_sibling() {
                    return Some(next_sibling);
                }

                for ancestor in node.ancestors() {
                    let ancestor = ancestor.borrow();
                    if stay_within == &Some(ancestor.id()) {
                        return None;
                    }
                    if let Some(next_sibling) = ancestor.next_sibling() {
                        return Some(next_sibling);
                    }
                }

                None
            },
            TreeOrder::Backwards => {
                if let Some(prev_sibling) = node.previous_sibling() {
                    let mut child_ref = prev_sibling;
                    while let Some(last_child) = child_ref.clone().borrow().children().last() {
                        child_ref = last_child.clone();
                    }
                    return Some(child_ref);
                }

                let parent = node.parent()?;

                if stay_within == &Some(parent.borrow().id()) {
                    return None;
                }

                Some(parent)
            },
        }
    }
}

impl DirtyState {
    pub(super) fn is_new(&self) -> bool {
        self.contains(DirtyState::New)
    }

    pub(super) fn updated(&self) -> bool {
        self.contains(DirtyState::Updated)
    }

    pub(super) fn descendant_has_damage(&self) -> bool {
        self.contains(DirtyState::DescendantHasDamage)
    }

    pub(super) fn propagate_descendant_has_damage(&mut self, child_dirty_state: DirtyState) {
        if child_dirty_state.self_or_descendant_has_damage() {
            self.insert(DirtyState::DescendantHasDamage)
        }
    }

    pub(super) fn self_or_descendant_has_damage(&self) -> bool {
        self.intersects(DirtyState::HasDamage | DirtyState::DescendantHasDamage)
    }
}

static HTML_ELEMENT_ROLE_MAPPINGS: LazyLock<FxHashMap<LocalName, Role>> = LazyLock::new(|| {
    [
        // FIXME: only a with href!
        (local_name!("a"), Role::Link),
        (local_name!("article"), Role::Article),
        (local_name!("aside"), Role::Complementary),
        (local_name!("body"), Role::RootWebArea),
        (local_name!("footer"), Role::ContentInfo),
        (local_name!("h1"), Role::Heading),
        (local_name!("h2"), Role::Heading),
        (local_name!("h3"), Role::Heading),
        (local_name!("h4"), Role::Heading),
        (local_name!("h5"), Role::Heading),
        (local_name!("h6"), Role::Heading),
        (local_name!("header"), Role::Banner),
        (local_name!("hr"), Role::Splitter),
        (local_name!("main"), Role::Main),
        (local_name!("nav"), Role::Navigation),
        (local_name!("p"), Role::Paragraph),
    ]
    .into_iter()
    .collect()
});

/// <https://w3c.github.io/aria/#namefromcontent>
static NAME_FROM_CONTENTS_ROLES: LazyLock<FxHashSet<Role>> =
    LazyLock::new(|| [(Role::Heading), (Role::Link)].into_iter().collect());

static CONTAINER_ROLES: LazyLock<FxHashSet<Role>> = LazyLock::new(|| {
    [
        (Role::Article),
        (Role::Banner),
        (Role::Complementary),
        (Role::Main),
        (Role::Navigation),
        (Role::Paragraph),
        (Role::RootWebArea),
    ]
    .into_iter()
    .collect()
});
