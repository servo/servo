/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::{Cell, LazyCell};
use std::cmp::Ordering;

use bitflags::bitflags;
use dom_struct::dom_struct;
use js::context::{JSContext, NoGC};
use rustc_hash::FxHashSet;
use script_bindings::cell::DomRefCell;
use script_bindings::codegen::GenericBindings::ShadowRootBinding::ShadowRootMethods;
use script_bindings::dom::UnrootedDom;
use script_bindings::reflector::{Reflector, reflect_dom_object};
use servo_base::text::{RangeAny, Utf16CodeUnits, Utf32CodeUnits, Utf32CodeUnitsOrNodeOffset};

use crate::dom::abstractrange::bp_position;
use crate::dom::bindings::codegen::Bindings::NodeBinding::{GetRootNodeOptions, NodeMethods};
use crate::dom::bindings::codegen::Bindings::RangeBinding::RangeMethods;
use crate::dom::bindings::codegen::Bindings::SelectionBinding::{
    GetComposedRangesOptions, SelectionMethods,
};
use crate::dom::bindings::error::{Error, ErrorResult, Fallible};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::DomGlobal;
use crate::dom::bindings::root::{Dom, DomRoot, LayoutDom, MutNullableDom};
use crate::dom::bindings::str::DOMString;
use crate::dom::document::Document;
use crate::dom::eventtarget::EventTarget;
use crate::dom::iterators::PrePostIteration;
use crate::dom::node::{Node, NodeTraits};
use crate::dom::range::Range;
use crate::dom::selection_range::{SelectionBoundary, SelectionRange};
use crate::dom::staticrange::StaticRange;
use crate::dom::types::ShadowRoot;
use crate::dom::{CharacterData, FlatTreeParent, NodeDamage, NodeFlags};

#[derive(Clone, Copy, JSTraceable, MallocSizeOf)]
enum Direction {
    Forwards,
    Backwards,
    Directionless,
}

/// <https://w3c.github.io/selection-api/#dfn-selection>
#[dom_struct]
pub(crate) struct Selection {
    reflector_: Reflector,
    document: Dom<Document>,
    /// A range that holds the start and end of this selection, which may potentially
    /// cross shadow roots.
    range: DomRefCell<Option<SelectionRange>>,
    /// The live range version of this selection, which will never cross shadow roots.
    live_range: MutNullableDom<Range>,
    /// The [`Direction`] of this [`Selection`] which determines which endpoint of
    /// [`Self::range`] is the anchor and which is the focus.
    direction: Cell<Direction>,
    /// <https://w3c.github.io/selection-api/#dfn-has-scheduled-selectionchange-event>
    has_scheduled_selectionchange_event: Cell<bool>,
    /// Whether or not this [`Selection`] needs to remark DOM nodes with selection flags
    /// after a change to its underlying [`Range`].
    visible_selection_dirty: Cell<bool>,
}

impl Selection {
    fn new_inherited(document: &Document) -> Selection {
        Selection {
            reflector_: Reflector::new(),
            document: Dom::from_ref(document),
            range: Default::default(),
            live_range: MutNullableDom::new(None),
            direction: Cell::new(Direction::Directionless),
            has_scheduled_selectionchange_event: Cell::new(false),
            visible_selection_dirty: Cell::new(false),
        }
    }

    pub(crate) fn new(cx: &mut JSContext, document: &Document) -> DomRoot<Selection> {
        reflect_dom_object(
            cx,
            Box::new(Selection::new_inherited(document)),
            &*document.global(),
        )
    }

    pub(crate) fn visible_selection_dirty(&self) -> bool {
        self.visible_selection_dirty.get()
    }

    fn clear_cached_live_range(&self) {
        if let Some(old_range) = self.live_range.take() {
            old_range.disassociate_selection(self);
        }
    }

    pub(crate) fn update_from_live_range(
        &self,
        live_range: &Range,
        notification: SelectionLiveRangeNotification,
    ) {
        let start_changed;
        let end_changed;
        {
            let mut range = self.range.borrow_mut();
            let range = range
                .as_mut()
                .expect("A live range implies a selection range");

            start_changed = notification.contains(SelectionLiveRangeNotification::Start) &&
                range.start != *live_range.start();
            if start_changed {
                range.start =
                    SelectionBoundary::new(&live_range.start_container(), live_range.start_offset())
            }

            end_changed = notification.contains(SelectionLiveRangeNotification::End) &&
                range.end != *live_range.end();
            if end_changed {
                range.end =
                    SelectionBoundary::new(&live_range.end_container(), live_range.end_offset())
            }
        }

        if start_changed || end_changed {
            self.selection_boundaries_changed();
        }
    }

    #[cfg_attr(crown, expect(crown::unrooted_must_root))]
    fn set_range(&self, _no_gc: &NoGC, new_range: Option<SelectionRange>) -> bool {
        let changed;
        {
            let mut range = self.range.borrow_mut();
            changed = *range != new_range;
            *range = new_range;

            if range.is_none() {
                self.direction.set(Direction::Directionless);
            }
        }

        // Any changes must unconditionally install a new live range.
        self.clear_cached_live_range();

        if changed {
            self.selection_boundaries_changed();
            #[cfg(debug_assertions)]
            self.assert_valid_selection(_no_gc);
        }

        changed
    }

    pub(crate) fn set_live_range(&self, no_gc: &NoGC, new_range: Option<&Range>) {
        if new_range == self.live_range.get().as_deref() {
            return;
        }

        let boundaries_changed = self.set_range(no_gc, new_range.map(|new_range| new_range.into()));

        // It's possible that `set_range` was a no-op, but still in that case we need to
        // replace the live range per-specification.
        if let Some(old_range) = self.live_range.take() {
            old_range.disassociate_selection(self);
        }
        if let Some(new_range) = new_range {
            self.live_range.set(Some(new_range));
            new_range.associate_selection(self);
        }

        // From <https://w3c.github.io/selection-api/#selectionchange-event>:
        // > When the selection is dissociated with its range, associated with a new
        // > range, or the associated range's boundary point is mutated either by the user
        // > or the content script, the user agent must schedule a selectionchange event on
        // > document.
        //
        // This means we should fire the event even if the boundaries themselves did not change. A
        // change to the range object is enough. Normally, this happens in `set_range`, but
        // only when the boundaries changed. In this case the call to `set_range` above did
        // not queue the task.
        if !boundaries_changed {
            self.queue_selectionchange_task();
        }
    }

    fn selection_boundaries_changed(&self) {
        self.set_visible_selection_dirty();
        self.queue_selectionchange_task();

        // See:
        //  - <https://w3c.github.io/editing/docs/execCommand/#state-override> and
        //  - <https://w3c.github.io/editing/docs/execCommand/#value-override>
        //
        // > Whenever the number of ranges in the selection changes to something
        // > different, and whenever a boundary point of the range at a given index in the
        // > selection changes to something different, the state override and value
        // > override must be unset for every command.
        self.document.clear_command_overrides();
    }

    fn iter_nodes_with_overlaps_document_selection_flag<'no_gc>(
        &self,
        no_gc: &'no_gc NoGC,
    ) -> impl Iterator<Item = UnrootedDom<'no_gc, Node>> {
        let mut traversal = self
            .document
            .upcast::<Node>()
            .following_flat_tree_nodes_unrooted(no_gc);
        let mut next = traversal.next();
        std::iter::from_fn(move || {
            while let Some(node) = next.take() {
                match node {
                    PrePostIteration::Enter(node) => {
                        if node.get_flag(NodeFlags::OVERLAPS_DOCUMENT_SELECTION) {
                            next = traversal.next();
                            return Some(node);
                        } else {
                            // This relies on flags being set consistently: this node
                            // with the flag unset claims that no part of it overlaps selection,
                            // which implies that none of its descendant either have any part
                            // of them overlapping selection, meaning none of them have the flag
                            next = traversal.next_skipping_subtree();
                        }
                    },
                    PrePostIteration::Leave(_) => next = traversal.next(),
                }
            }
            None
        })
    }

    pub(crate) fn update_overlaps_document_selection_flags<'no_gc>(&self, no_gc: &'no_gc NoGC) {
        if !self.visible_selection_dirty.take() {
            return;
        }

        let previously_flagged_nodes = self.iter_nodes_with_overlaps_document_selection_flag(no_gc);

        let needs_new_display_list = Cell::new(false);
        let set_text_run_selection =
            |character_data: &CharacterData, range: Option<RangeAny<Utf32CodeUnits>>| {
                if character_data.set_text_run_selection(range) {
                    needs_new_display_list.set(true)
                } else {
                    character_data
                        .upcast::<Node>()
                        .dirty(no_gc, NodeDamage::ContentOrHeritage);
                }
            };
        let remove_selection = |node: &Node| {
            node.set_flag(NodeFlags::OVERLAPS_DOCUMENT_SELECTION, false);
            // Currently only `CharacterData` nodes show visible selection.
            if let Some(character_data) = node.downcast::<CharacterData>() {
                set_text_run_selection(character_data, None)
            }
        };

        let range = self.range.borrow();
        let Some(range) = range.as_ref() else {
            for node in previously_flagged_nodes {
                remove_selection(&node)
            }
            if needs_new_display_list.get() {
                self.document.window().layout().set_needs_new_display_list();
            }
            return;
        };

        // Hash keys are pointer addresses which are not directly controlled by web content
        // so we don’t need HashDoS resistance and can use a faster hasher than `std`’s default
        let mut previously_flagged_nodes: FxHashSet<_> = previously_flagged_nodes.collect();

        let start_offset = range.start.offset as usize;
        let end_offset = range.end.offset as usize;
        let start_container = range.start.container.as_rooted();
        let end_container = range.end.container.as_rooted();
        let start_position =
            position_in_flat_tree_for_selection(no_gc, start_container.clone(), start_offset);
        let end_position =
            position_in_flat_tree_for_selection(no_gc, end_container.clone(), end_offset);

        let start_node = start_position.node();
        let end_node = end_position.node();

        // In case the range hasn't changed, but the offsets within the start/end end node have
        // changed, always update the selection on the start and end nodes, if they paint selection.

        // TODO(mrobinson): We should handle changes only to the offsets within a single
        // boundary node explicitly and not traversing the whole range.
        // But that requires keeping track of the previous range, to compare.
        if let Some(character_data) = start_container.downcast::<CharacterData>() {
            let text = character_data.data();
            let range = RangeAny {
                start: Some(Utf16CodeUnits(start_offset).to_utf32_code_units_in(&text)),
                end: (start_node == end_node)
                    .then_some(Utf16CodeUnits(end_offset).to_utf32_code_units_in(&text)),
            };
            set_text_run_selection(character_data, Some(range))
        }
        if end_container != start_container &&
            let Some(character_data) = end_container.downcast::<CharacterData>()
        {
            let text = character_data.data();
            let range = RangeAny {
                start: None,
                end: Some(Utf16CodeUnits(end_offset).to_utf32_code_units_in(&text)),
            };
            set_text_run_selection(character_data, Some(range))
        }

        let mut set_selection_flag = |node: &UnrootedDom<'no_gc, Node>| {
            if !node.get_flag(NodeFlags::OVERLAPS_DOCUMENT_SELECTION) {
                node.set_flag(NodeFlags::OVERLAPS_DOCUMENT_SELECTION, true);
                debug_assert!(!previously_flagged_nodes.contains(node));
            } else {
                previously_flagged_nodes.remove(node);
            }
        };

        // We mark the ancestors of the start node as containing a selection. Two notes:
        // - The traversal itself will take care of marking ancestors of all other nodes,
        //   as the in-order tree walk will be guaranteed to walk them.
        // - We do not need to mark these nodes as dirty as they are guaranteed to not be
        //   leaves (the only nodes that show visible selection).
        let mut maybe_parent = start_node.parent_in_flat_tree(no_gc);
        while let FlatTreeParent::Parent(parent) = maybe_parent {
            set_selection_flag(&parent);
            maybe_parent = parent.parent_in_flat_tree(no_gc);
        }

        let mut traversal = start_node.following_flat_tree_nodes_unrooted(no_gc);

        // If the selection starts after the first node, skip that node and all descendants
        // before setting flags in the selection range.
        if matches!(start_position, FlatTreeNodePosition::After(_)) {
            let leaving_start = traversal.next_skipping_subtree();
            debug_assert!(
                matches!(leaving_start, Some(PrePostIteration::Leave(node)) if node == *start_node)
            );
        }

        for iteration in traversal {
            match &iteration {
                PrePostIteration::Enter(node) => {
                    if node == end_node && matches!(end_position, FlatTreeNodePosition::Before(_)) {
                        break;
                    }
                    if node == start_node {
                        continue;
                    }
                    set_selection_flag(node);
                },
                PrePostIteration::Leave(node) => {
                    set_selection_flag(node);
                    if node == end_node {
                        break;
                    }
                    if let Some(character_data) = node.downcast::<CharacterData>() {
                        set_text_run_selection(character_data, Some(RangeAny::full()))
                    }
                },
            }
        }

        // Nodes that haven’t been removed from the `HashSet` by `add_selection_flag`
        // should no longer have the flag:
        for node in &previously_flagged_nodes {
            remove_selection(node)
        }
        if needs_new_display_list.get() {
            self.document.window().layout().set_needs_new_display_list();
        }
    }

    /// <https://w3c.github.io/selection-api/#dfn-schedule-a-selectionchange-event>
    pub(crate) fn queue_selectionchange_task(&self) {
        // Step 1. If target's has scheduled selectionchange event is true, abort these steps.
        if self.has_scheduled_selectionchange_event.get() {
            return;
        }
        // Step 2. Set target's has scheduled selectionchange event to true.
        self.has_scheduled_selectionchange_event.set(true);
        // Step 3. Queue a task on the user interaction task source to fire a
        // selectionchange event on target.
        let this = Trusted::new(self);
        self.document
            .owner_global()
            .task_manager()
            .user_interaction_task_source() // w3c/selection-api#117
            .queue(
                // https://w3c.github.io/selection-api/#firing-selectionchange-event
                task!(selectionchange_task_steps: move |cx| {
                    let this = this.root();
                    // Step 1. Set target's has scheduled selectionchange event to false.
                    this.has_scheduled_selectionchange_event.set(false);
                    // Step 2. If target is an element, fire an event named
                    // selectionchange, which bubbles and not cancelable, at target.
                    //
                    // n/a

                    // Step 3. Otherwise, if target is a document, fire an event named
                    // selectionchange, which does not bubble and not cancelable, at
                    // target.
                    this.document.upcast::<EventTarget>().fire_event(cx, atom!("selectionchange"));
                }),
            );
    }

    fn is_in_document_of_range(&self, node: &Node) -> bool {
        // TODO(mrobinson): This should eventually allow nodes in the same composed tree (and
        // not just the same tree), but this requires more work to allow `Selection` to cross
        // shadow tree boundaries.
        &*node.GetRootNode(&GetRootNodeOptions { composed: false }) ==
            self.document.upcast::<Node>()
    }

    pub(crate) fn start_boundary(&self, cx: &mut JSContext) -> (DomRoot<Node>, u32) {
        let range = self.expect_active_range(cx);
        (range.start_container(), range.start_offset())
    }

    pub(crate) fn end_boundary(&self, cx: &mut JSContext) -> (DomRoot<Node>, u32) {
        let range = self.expect_active_range(cx);
        (range.end_container(), range.end_offset())
    }

    #[cfg(debug_assertions)]
    fn assert_valid_selection(&self, no_gc: &NoGC) {
        let range_borrow = self.range.borrow();
        let Some(range) = range_borrow.as_ref() else {
            return;
        };
        debug_assert_eq!(
            range.start.container.GetRootNode(&Default::default()),
            range.end.container.GetRootNode(&Default::default())
        );
        debug_assert!(
            bp_position(
                no_gc,
                &range.start.container,
                range.start.offset,
                &range.end.container,
                range.end.offset
            ) != Ordering::Greater
        );
    }

    #[cfg(debug_assertions)]
    fn assert_valid_selection_and_live_range(&self, no_gc: &NoGC) {
        self.assert_valid_selection(no_gc);

        let Some(active_range) = self.live_range.get() else {
            return;
        };

        // TODO: For now the live range is equal to the selection range, but one selections
        // can span shadow root boundaries they will be different.
        let range = self.range.borrow();
        let range = range
            .as_ref()
            .expect("Should always have a range if we have an live range");
        debug_assert!(*range.start.container == *active_range.start_container());
        debug_assert_eq!(range.start.offset, active_range.start_offset());
        debug_assert!(*range.end.container == *active_range.end_container());
        debug_assert_eq!(range.end.offset, active_range.end_offset());
        debug_assert!(
            bp_position(
                no_gc,
                &active_range.start_container(),
                active_range.start_offset(),
                &active_range.end_container(),
                active_range.end_offset()
            ) != Ordering::Greater
        );
    }

    /// <https://w3c.github.io/editing/docs/execCommand/#active-range>
    ///
    /// > The active range is the range of the selection given by calling
    /// > getSelection() on the context object. (Thus the active range may be null.)
    pub(crate) fn active_range(&self, cx: &mut JSContext) -> Option<DomRoot<Range>> {
        #[cfg(debug_assertions)]
        self.assert_valid_selection_and_live_range(cx.no_gc());

        if let Some(active_range) = self.live_range.get() {
            return Some(active_range);
        }

        // TODO: This should eventually be the projection of the composed range stored in
        // `self.range` into the boundaries of a single DOM tree.
        let active_range = {
            let range = self.range.borrow();
            let range = range.as_ref()?;
            Range::new(
                cx,
                &self.document,
                &range.start.container,
                range.start.offset,
                &range.end.container,
                range.end.offset,
            )
        };

        self.live_range.set(Some(&active_range));
        active_range.associate_selection(self);
        Some(active_range)
    }

    pub(crate) fn expect_active_range(&self, cx: &mut JSContext) -> DomRoot<Range> {
        self.active_range(cx)
            .expect("Should always have an active range")
    }

    pub(crate) fn set_visible_selection_dirty(&self) {
        self.visible_selection_dirty.set(true);
    }

    fn composed_anchor_position(&self) -> Option<(DomRoot<Node>, u32)> {
        let range = self.range.borrow();
        let range = range.as_ref()?;
        Some(match self.direction.get() {
            Direction::Forwards => (range.start.container.as_rooted(), range.start.offset),
            _ => (range.end.container.as_rooted(), range.end.offset),
        })
    }

    /// <https://w3c.github.io/selection-api/#dfn-anchor>
    fn live_anchor_node(&self, cx: &mut JSContext) -> Option<DomRoot<Node>> {
        self.active_range(cx)
            .map(|range| match self.direction.get() {
                Direction::Forwards => range.start_container(),
                _ => range.end_container(),
            })
    }

    /// <https://w3c.github.io/selection-api/#dfn-anchor>
    fn live_anchor_offset(&self, cx: &mut JSContext) -> u32 {
        self.active_range(cx)
            .map_or(0, |range| match self.direction.get() {
                Direction::Forwards => range.start_offset(),
                _ => range.end_offset(),
            })
    }

    /// <https://w3c.github.io/selection-api/#dfn-focus>
    fn live_focus_node(&self, cx: &mut JSContext) -> Option<DomRoot<Node>> {
        self.active_range(cx)
            .map(|range| match self.direction.get() {
                Direction::Forwards => range.end_container(),
                _ => range.start_container(),
            })
    }

    /// <https://w3c.github.io/selection-api/#dfn-focus>
    fn live_focus_offset(&self, cx: &mut JSContext) -> u32 {
        self.active_range(cx)
            .map_or(0, |range| match self.direction.get() {
                Direction::Forwards => range.end_offset(),
                _ => range.start_offset(),
            })
    }

    /// <https://dom.spec.whatwg.org/#concept-node-insert> steps 5.1-5.2
    /// and
    /// <https://dom.spec.whatwg.org/#move> steps 17.1-17.2
    /// adapted for selections.
    pub(crate) fn insert_steps(&self, parent: &Node, child: &Node, count: u32) {
        let mut range_borrow = self.range.borrow_mut();
        let Some(range) = &mut *range_borrow else {
            return;
        };
        let child_index = LazyCell::new(|| child.index());
        // Step 5.1: For each live range whose start node is parent and start offset is
        // greater than child’s index: increase its start offset by count.
        if range.start.container == parent && range.start.offset > *child_index {
            range.start.offset += count;
            self.selection_boundaries_changed();
        }
        // Step 5.2: For each live range whose end node is parent and end offset is
        // greater than child’s index: increase its end offset by count.
        if range.end.container == parent && range.end.offset > *child_index {
            range.end.offset += count;
            self.selection_boundaries_changed();
        }
    }

    /// <https://dom.spec.whatwg.org/#live-range-pre-remove-steps> steps 4 and 5
    /// adapted for selections.
    ///
    /// These steps are run on the inclusive descendants of a removed node, but to avoid
    /// having to iterate through those nodes twice, they are run when the inclusive
    /// descendants themselves are unbound from the tree.
    pub(crate) fn remove_steps_for_removed_subtree(
        &self,
        inclusive_descendant_of_removed_node: &Node, // "node" in the specification
        parent_of_removed_node: &Node,               // "parent" in the specification
        index_of_removed_node: &mut dyn FnMut() -> u32, // "index" in the specification
    ) {
        // The steps are only supposed to run on DOM tree inclusive descendants of the removal
        // root and elements in shadow trees are not, so they shouldn't run for them.
        //
        // TODO: This won't be true once selections can span shadow tree roots.
        if inclusive_descendant_of_removed_node.is_in_a_shadow_tree() {
            return;
        }

        let mut range_borrow = self.range.borrow_mut();
        let Some(range) = &mut *range_borrow else {
            return;
        };
        // Step 4: For each live range whose start node is an inclusive descendant of
        // node, set its start to (parent, index).
        if range.start.container == inclusive_descendant_of_removed_node {
            range.start = SelectionBoundary::new(parent_of_removed_node, index_of_removed_node());
            self.selection_boundaries_changed();
        }
        // Step 5: For each live range whose end node is an inclusive descendant of node,
        // set its end to (parent, index).
        if range.end.container == inclusive_descendant_of_removed_node {
            range.end = SelectionBoundary::new(parent_of_removed_node, index_of_removed_node());
            self.selection_boundaries_changed();
        }
    }

    /// <https://dom.spec.whatwg.org/#live-range-pre-remove-steps> steps 6 and 7
    /// adapted for selections.
    pub(crate) fn remove_steps_for_parent(
        &self,
        parent: &Node,
        node_index: &mut dyn FnMut() -> u32,
    ) {
        let mut range_borrow = self.range.borrow_mut();
        let Some(range) = &mut *range_borrow else {
            return;
        };

        // Step 6: For each live range whose start node is parent and start offset is
        // greater than index, decrease its start offset by 1.
        if range.start.container == parent && range.start.offset > node_index() {
            range.start.offset -= 1;
            self.selection_boundaries_changed();
        }
        // Step 7: For each live range whose end node is parent and end offset is greater than
        // index, decrease its end offset by 1.
        if range.end.container == parent && range.end.offset > node_index() {
            range.end.offset -= 1;
            self.selection_boundaries_changed();
        }
    }

    /// <https://dom.spec.whatwg.org/#dom-node-normalize> Steps 6.1-6.4 adapted for selections.
    ///
    /// - `parent`: The parent of both other node arguments.
    /// - `node`: The node that text is being merged into.
    /// - `current_node`: The node which has text being merged into `node` and will be
    ///   removed from the DOM.
    /// - `length`: The length of the text content that was merged into `node` from
    ///   siblings before `current_node`.
    pub(crate) fn normalization_steps(
        &self,
        parent: &Node,
        node: &Node,
        current_node: &Node,
        current_node_index: &dyn Fn() -> u32,
        length: u32,
    ) {
        let mut range_borrow = self.range.borrow_mut();
        let Some(range) = &mut *range_borrow else {
            return;
        };
        // Step 6.1: For each live range whose start node is currentNode: add length to its start
        // offset and set its start node to node.
        if range.start.container == current_node {
            range.start.offset += length;
            range.start.container = Dom::from_ref(node);
            self.selection_boundaries_changed();
        }
        // Step 6.2 For each live range whose end node is currentNode: add length to its end offset
        // and set its end node to node.
        if range.end.container == current_node {
            range.end.offset += length;
            range.end.container = Dom::from_ref(node);
            self.selection_boundaries_changed();
        }
        // Step 6.3: For each live range whose start node is currentNode’s parent and start
        // offset is currentNode’s index: set its start node to node and its start offset
        // to length.
        if range.start.container == parent && range.start.offset == current_node_index() {
            range.start.container = Dom::from_ref(node);
            range.start.offset = length;
            self.selection_boundaries_changed();
        }
        // Step 6.4: For each live range whose end node is currentNode’s parent and end offset is
        // currentNode’s index: set its end node to node and its end offset to length.
        if range.end.container == parent && range.end.offset == current_node_index() {
            range.end.container = Dom::from_ref(node);
            range.end.offset = length;
            self.selection_boundaries_changed();
        }
    }

    /// <https://dom.spec.whatwg.org/#concept-cd-replace> steps 8-11
    /// adapted for selections.
    pub(crate) fn replace_data_steps(
        &self,
        node: &Node,
        offset: u32,
        removed_code_units: u32,
        added_code_units: &mut dyn FnMut() -> u32,
    ) {
        let mut range_borrow = self.range.borrow_mut();
        let Some(range) = &mut *range_borrow else {
            return;
        };
        // Step 8: For each live range whose start node is node and start offset is
        // greater than offset but less than or equal to offset + count: set its start
        // offset to offset.
        let start_container = &range.start.container;
        let start_offset = range.start.offset;
        if &**start_container == node &&
            start_offset > offset &&
            start_offset <= offset + removed_code_units
        {
            range.start.offset = offset;
            self.selection_boundaries_changed();
        }
        // Step 9: For each live range whose end node is node and end offset is
        // greater than offset but less than or equal to offset + count: set its end
        // offset to offset.
        let end_container = &range.end.container;
        let end_offset = range.end.offset;
        if &**end_container == node &&
            end_offset > offset &&
            end_offset <= offset + removed_code_units
        {
            range.end.offset = offset;
            self.selection_boundaries_changed();
        }
        // Step 10: For each live range whose start node is node and start offset is
        // greater than offset + count: increase its start offset by data’s length and
        // decrease it by count.
        if &**start_container == node && start_offset > offset + removed_code_units {
            range.start.offset = start_offset + added_code_units() - removed_code_units;
            self.selection_boundaries_changed();
        }
        // Step 11: For each live range whose end node is node and end offset is
        // greater than offset + count: increase its end offset by data’s length and
        // decrease it by count.
        if &**end_container == node && end_offset > offset + removed_code_units {
            range.end.offset = end_offset + added_code_units() - removed_code_units;
            self.selection_boundaries_changed();
        }
    }

    /// <https://dom.spec.whatwg.org/#concept-text-split> steps 7.2-7.3
    /// adapted for selections.
    pub(crate) fn text_split_steps(
        &self,
        node: &Node,
        offset: u32,
        parent_node: &Node,
        new_node: &Node,
    ) {
        let mut range_borrow = self.range.borrow_mut();
        let Some(range) = &mut *range_borrow else {
            return;
        };
        // Step 7.2: For each live range whose start node is node and start offset is
        // greater than offset, set its start node to newNode and decrease its start
        // offset by offset.
        if range.start.container == node && range.start.offset > offset {
            range.start.container = Dom::from_ref(new_node);
            range.start.offset -= offset;
            self.selection_boundaries_changed();
        }
        // Step 7.3: For each live range whose end node is node and end offset is greater
        // than offset, set its end node to newNode and decrease its end offset by offset.
        if range.end.container == node && range.end.offset > offset {
            range.end.container = Dom::from_ref(new_node);
            range.end.offset -= offset;
            self.selection_boundaries_changed();
        }
        // Step 7.4: For each live range whose start node is parent and start offset is
        // equal to the index of node plus 1, increase its start offset by 1.
        let node_index = LazyCell::new(|| node.index());
        if range.start.container == parent_node && range.start.offset == *node_index + 1 {
            range.start.offset += 1;
            self.selection_boundaries_changed();
        }
        // Step 7.5: For each live range whose end node is parent and end offset is equal
        // to the index of node plus 1, increase its end offset by 1.
        if range.end.container == parent_node && range.end.offset == *node_index + 1 {
            range.end.offset += 1;
            self.selection_boundaries_changed();
        }
    }

    pub(crate) fn collapse_to_dom_position(
        &self,
        cx: &mut JSContext,
        container: &Node,
        offset: Utf32CodeUnitsOrNodeOffset,
    ) {
        let _ = self.Collapse(
            cx,
            Some(container),
            container.to_sibling_or_utf16_offset(offset),
        );
    }

    pub(crate) fn collapse_or_extend_to_dom_position(
        &self,
        cx: &mut JSContext,
        container: &Node,
        offset: Utf32CodeUnitsOrNodeOffset,
    ) {
        let offset = container.to_sibling_or_utf16_offset(offset);
        let is_anchor =
            self.composed_anchor_position()
                .is_some_and(|(anchor_node, anchor_offset)| {
                    &*anchor_node == container && anchor_offset == offset
                });

        if self.range.borrow().is_none() || is_anchor {
            let _ = self.Collapse(cx, Some(container), offset);
        } else {
            let _ = self.Extend(cx, container, offset);
        }
    }
}

impl SelectionMethods<crate::DomTypeHolder> for Selection {
    /// <https://w3c.github.io/selection-api/#dom-selection-anchornode>
    fn GetAnchorNode(&self, cx: &mut JSContext) -> Option<DomRoot<Node>> {
        // > The attribute must return the anchor node of this, or null if the anchor is
        // > null or anchor is not in the document tree.
        let anchor_node = self.live_anchor_node(cx)?;
        if !anchor_node.is_in_a_document_tree() {
            return None;
        }
        Some(anchor_node)
    }

    /// <https://w3c.github.io/selection-api/#dom-selection-anchoroffset>
    fn AnchorOffset(&self, cx: &mut JSContext) -> u32 {
        // > The attribute must return the anchor offset of this, or 0 if the anchor is null
        // > or anchor is not in the document tree.
        if self
            .live_anchor_node(cx)
            .is_none_or(|anchor_node| !anchor_node.is_in_a_document_tree())
        {
            return 0;
        }
        self.live_anchor_offset(cx)
    }

    /// <https://w3c.github.io/selection-api/#dom-selection-focusnode>
    fn GetFocusNode(&self, cx: &mut JSContext) -> Option<DomRoot<Node>> {
        // > The attribute must return the focus node of this, or null if the focus is
        // > null or focus is not in the document tree.
        let focus_node = self.live_focus_node(cx)?;
        if !focus_node.is_in_a_document_tree() {
            return None;
        }
        Some(focus_node)
    }

    /// <https://w3c.github.io/selection-api/#dom-selection-focusoffset>
    fn FocusOffset(&self, cx: &mut JSContext) -> u32 {
        // > The attribute must return the focus offset of this, or 0 if the focus is null
        // > or focus is not in the document tree.
        if self
            .live_focus_node(cx)
            .is_none_or(|focus_node| !focus_node.is_in_a_document_tree())
        {
            return 0;
        }
        self.live_focus_offset(cx)
    }

    /// <https://w3c.github.io/selection-api/#dom-selection-iscollapsed>
    fn IsCollapsed(&self, cx: &mut JSContext) -> bool {
        // > The attribute must return true if and only if the anchor and focus are the
        // > same (including if both are null). Otherwise it must return false.
        self.active_range(cx).is_none_or(|range| range.collapsed())
    }

    /// <https://w3c.github.io/selection-api/#dom-selection-rangecount>
    fn RangeCount(&self) -> u32 {
        // > The attribute must return 0 if this is empty or either focus or anchor is not
        // > in the document tree, and must return 1 otherwise.
        let range = self.range.borrow();
        let Some(range) = range.as_ref() else {
            return 0;
        };
        if !range.start_and_end_are_in_document_tree() {
            return 0;
        }
        1
    }

    /// <https://w3c.github.io/selection-api/#dom-selection-type>
    fn Type(&self) -> DOMString {
        // > The attribute must return "None" if this is empty or either focus or anchor
        // > is not in the document tree, "Caret" if this's range is collapsed, and "Range"
        // > otherwise.
        let range = self.range.borrow();
        let Some(range) = range.as_ref() else {
            return DOMString::from_static("None");
        };
        if !range.start_and_end_are_in_document_tree() {
            return DOMString::from_static("None");
        }

        if range.collapsed() {
            DOMString::from_static("Caret")
        } else {
            DOMString::from_static("Range")
        }
    }

    /// <https://w3c.github.io/selection-api/#dom-selection-direction>
    fn Direction(&self) -> DOMString {
        // > The attribute must return "none" if this is empty or this selection is
        // > directionless. "forward" if this selection's direction is forwards and
        // > "backward" if this selection's direction is backwards.
        match self.direction.get() {
            Direction::Directionless => DOMString::from_static("none"),
            Direction::Forwards => DOMString::from_static("forward"),
            Direction::Backwards => DOMString::from_static("backward"),
        }
    }

    /// <https://w3c.github.io/selection-api/#dom-selection-getrangeat>
    fn GetRangeAt(&self, cx: &mut JSContext, index: u32) -> Fallible<DomRoot<Range>> {
        // > The method must throw an IndexSizeError exception if index is not 0, or if this
        // > is empty or either focus or anchor is not in the document tree. Otherwise, it
        // > must return a reference to (not a copy of) this's range.
        if index != 0 {
            return Err(Error::IndexSize(Some("Index must be zero".into())));
        }

        let range = self.range.borrow();
        let Some(range) = range.as_ref() else {
            return Err(Error::IndexSize(Some("Selection is empty".into())));
        };
        if !range.start_and_end_are_in_document_tree() {
            return Err(Error::IndexSize(Some(
                "Start and end are not in document tree".into(),
            )));
        }
        self.active_range(cx).ok_or(Error::IndexSize(Some(
            "Could not create live range for selection".into(),
        )))
    }

    /// <https://w3c.github.io/selection-api/#dom-selection-addrange>
    fn AddRange(&self, no_gc: &NoGC, range: &Range) {
        // Step 1. If the root of the range's boundary points are not the document
        // associated with this, abort these steps.
        if !self.is_in_document_of_range(&range.start_container()) {
            return;
        }

        // Step 2. If rangeCount is not 0, abort these steps.
        if self.RangeCount() != 0 {
            return;
        }

        // Step 3. Set this's range to range by a strong reference (not by making a copy).
        self.set_live_range(no_gc, Some(range));

        // Are we supposed to set Direction here? w3c/selection-api#116
        self.direction.set(Direction::Forwards);
    }

    /// <https://w3c.github.io/selection-api/#dom-selection-removerange>
    fn RemoveRange(&self, no_gc: &NoGC, range: &Range) -> ErrorResult {
        // > The method must make this empty by disassociating its range if this's range
        // > is range. Otherwise, it must throw a NotFoundError.
        if let Some(own_range) = self.live_range.get() &&
            &*own_range == range
        {
            self.set_range(no_gc, None);
            return Ok(());
        }
        Err(Error::NotFound(None))
    }

    /// <https://w3c.github.io/selection-api/#dom-selection-removeallranges>
    fn RemoveAllRanges(&self, no_gc: &NoGC) {
        // > The method must make this empty by disassociating its range if this has an
        // > associated range.
        self.set_range(no_gc, None);
    }

    /// <https://w3c.github.io/selection-api/#dom-selection-empty>
    fn Empty(&self, no_gc: &NoGC) {
        // > The method must be an alias, and behave identically, to removeAllRanges().
        self.RemoveAllRanges(no_gc);
    }

    /// <https://w3c.github.io/selection-api/#dom-selection-getcomposedranges>
    fn GetComposedRanges(
        &self,
        cx: &mut JSContext,
        options: &GetComposedRangesOptions,
    ) -> Vec<DomRoot<StaticRange>> {
        // Step 1. If this is empty, return an empty array.
        let borrowed_range = self.range.borrow();
        let Some(range) = borrowed_range.as_ref() else {
            return Vec::new();
        };

        // Step 2. Otherwise, let startNode be start node of the range associated with
        // this, and let startOffset be start offset of the range.
        let mut start_node = range.start.container.as_rooted();
        let mut start_offset = range.start.offset;

        let is_ancestor_of_provided_shadow_roots = |shadow_root: &ShadowRoot| {
            let shadow_root_node = shadow_root.upcast::<Node>();
            options.shadowRoots.iter().any(|option_shadow_root| {
                shadow_root_node
                    .is_shadow_including_inclusive_ancestor_of(option_shadow_root.upcast())
            })
        };

        // Step 3. While startNode is a node, startNode's root is a shadow root, and
        // startNode's root is not a shadow-including inclusive ancestor of any of
        // options["shadowRoots"], repeat these steps:
        while let Some(containing_shadow_root) = start_node.containing_shadow_root() &&
            !is_ancestor_of_provided_shadow_roots(&containing_shadow_root)
        {
            // Step 3.1. Set startOffset to index of startNode's root's host.
            let host = DomRoot::upcast::<Node>(containing_shadow_root.Host());
            start_offset = host.index();

            // Step 3.2. Set startNode to startNode's root's host's parent.
            // See <https://github.com/w3c/selection-api/issues/161> for why
            // we always know that the start_node is a node.
            start_node = host
                .GetParentNode()
                .expect("The host should always have a parent node");
        }

        // Step 4. Let endNode be end node of the range associated with this, and let
        // endOffset be end offset of the range.
        let mut end_node = range.end.container.as_rooted();
        let mut end_offset = range.end.offset;

        // Step 5. While endNode is a node, endNode's root is a shadow root, and endNode's
        // root is not a shadow-including inclusive ancestor of any of
        // options["shadowRoots"], repeat these steps:
        while let Some(containing_shadow_root) = end_node.containing_shadow_root() &&
            !is_ancestor_of_provided_shadow_roots(&containing_shadow_root)
        {
            // Step 5.1. Set endOffset to index of endNode's root's host plus 1.
            let host = DomRoot::upcast::<Node>(containing_shadow_root.Host());
            end_offset = host.index() + 1;

            // Step 5.2. Set endNode to endNode's root's host's parent.
            // See <https://github.com/w3c/selection-api/issues/161> for why
            // we always know that the end_node is a node.
            end_node = host
                .GetParentNode()
                .expect("The host should always have a parent node.");
        }

        drop(borrowed_range);

        // Step 6. Return an array consisting of new StaticRange whose start node is
        // startNode, start offset is startOffset, end node is endNode, and end offset is
        // endOffset.
        vec![DomRoot::from_ref(&StaticRange::new(
            cx,
            &self.document,
            &start_node,
            start_offset,
            &end_node,
            end_offset,
        ))]
    }

    /// <https://w3c.github.io/selection-api/#dom-selection-collapse>
    fn Collapse(&self, cx: &mut JSContext, node: Option<&Node>, offset: u32) -> ErrorResult {
        // Step 1. If node is null, this method must behave identically as
        // removeAllRanges() and abort these steps.
        let Some(node) = node else {
            self.set_range(cx.no_gc(), None);
            return Ok(());
        };

        // Step 2. If node is a DocumentType, throw an InvalidNodeTypeError exception and
        // abort these steps.
        if node.is_doctype() {
            return Err(Error::InvalidNodeType(None));
        }

        // Step 3. The method must throw an IndexSizeError exception if offset is longer
        // than node's length and abort these steps.
        if offset > node.len() {
            return Err(Error::IndexSize(None));
        }

        // Step 4. If document associated with this is not a shadow-including inclusive
        // ancestor of node, abort these steps.
        //
        // TODO(mrobinson): This should eventually allow nodes in the same composed tree (and
        // not just the same tree), but this requires more work to allow `Selection` to cross
        // shadow tree boundaries.
        if &*node.GetRootNode(&GetRootNodeOptions { composed: false }) !=
            self.document.upcast::<Node>()
        {
            return Ok(());
        }

        // Step 5. Otherwise, let newRange be a new range.
        // Step 6. Set the start the start and the end of newRange to (node, offset).
        // Step 7. Set this's range to newRange.
        self.set_range(
            cx.no_gc(),
            Some(SelectionRange::collapsed_at(SelectionBoundary::new(
                node, offset,
            ))),
        );

        // Are we supposed to set Direction here? w3c/selection-api#116
        self.direction.set(Direction::Forwards);

        Ok(())
    }

    /// <https://w3c.github.io/selection-api/#dom-selection-setposition>
    fn SetPosition(&self, cx: &mut JSContext, node: Option<&Node>, offset: u32) -> ErrorResult {
        // > The method must be an alias, and behave identically, to collapse().
        self.Collapse(cx, node, offset)
    }

    /// <https://w3c.github.io/selection-api/#dom-selection-collapsetostart>
    fn CollapseToStart(&self, cx: &mut JSContext) -> ErrorResult {
        // > The method must throw InvalidStateError exception if the this is empty.
        // > Otherwise, it must create a new range, set the start both its start and end to
        // > the start of this's range, and then set this's range to the newly-created
        // > range.
        let Some((start_container, start_offset)) = self
            .range
            .borrow()
            .as_ref()
            .map(|range| (range.start.container.as_rooted(), range.start.offset))
        else {
            return Err(Error::InvalidState(None));
        };
        self.Collapse(cx, Some(&*start_container), start_offset)
    }

    /// <https://w3c.github.io/selection-api/#dom-selection-collapsetoend>
    fn CollapseToEnd(&self, cx: &mut JSContext) -> ErrorResult {
        // > The method must throw InvalidStateError exception if the this is empty.
        // > Otherwise, it must create a new range, set the start both its start and end to
        // > the end of this's range, and then set this's range to the newly-created range.
        let Some((end_container, end_offset)) = self
            .range
            .borrow()
            .as_ref()
            .map(|range| (range.end.container.as_rooted(), range.end.offset))
        else {
            return Err(Error::InvalidState(None));
        };
        self.Collapse(cx, Some(&*end_container), end_offset)
    }

    /// <https://w3c.github.io/selection-api/#dom-selection-extend>
    fn Extend(&self, cx: &mut JSContext, node: &Node, offset: u32) -> ErrorResult {
        // Step 1. If the document associated with this is not a shadow-including
        // inclusive ancestor of node, abort these steps.
        //
        // TODO(mrobinson): This should eventually allow nodes in the same composed tree (and
        // not just the same tree), but this requires more work to allow `Selection` to cross
        // shadow tree boundaries.
        if &*node.GetRootNode(&GetRootNodeOptions { composed: false }) !=
            self.document.upcast::<Node>()
        {
            return Ok(());
        }

        // Step 2. If this is empty, throw an InvalidStateError exception and abort these steps.
        let range_borrow = self.range.borrow();
        let Some(range) = range_borrow.as_ref() else {
            return Err(Error::InvalidState(None));
        };

        // This isn't specified, but it appears to be implementation behavior of other
        // browsers. See w3c/selection-api#118.
        if node.is_doctype() {
            return Err(Error::InvalidNodeType(None));
        }

        // As with is_doctype, this is not explicit in the selection specification steps
        // here but implied by which exceptions are thrown in WPT tests.
        if offset > node.len() {
            return Err(Error::IndexSize(None));
        }

        // Step 3. Let oldAnchor and oldFocus be the this's anchor and focus, and let
        // newFocus be the boundary point (node, offset).
        //
        // Note: oldFocus is unused, so we do not set it here.
        let (old_anchor_node, old_anchor_offset) = self
            .composed_anchor_position()
            .expect("has range, therefore has anchor node");

        // Step 4. Let newRange be a new range.
        // Note: Set directly to satisfy crown.
        let direction;

        // Step 5. If node's root is not the same as the this's range's root, set the
        // start newRange's start and end to newFocus.
        let is_in_document_of_range = self.is_in_document_of_range(&range.start.container);
        drop(range_borrow);

        if !is_in_document_of_range {
            self.set_range(
                cx.no_gc(),
                Some(SelectionRange::collapsed_at(SelectionBoundary::new(
                    node, offset,
                ))),
            );
            direction = Direction::Forwards;
        } else {
            let is_old_anchor_before_or_equal = matches!(
                bp_position(
                    cx.no_gc(),
                    &old_anchor_node,
                    old_anchor_offset,
                    node,
                    offset
                ),
                Ordering::Less | Ordering::Equal
            );
            if is_old_anchor_before_or_equal {
                // Step 6. Otherwise, if oldAnchor is before or equal to newFocus, set the start
                // newRange's start to oldAnchor, then set its end to newFocus.
                self.set_range(
                    cx.no_gc(),
                    Some(SelectionRange::new(
                        SelectionBoundary::new(&old_anchor_node, old_anchor_offset),
                        SelectionBoundary::new(node, offset),
                    )),
                );
                direction = Direction::Forwards;
            } else {
                // Step 7. Otherwise, set the start newRange's start to newFocus, then set
                // its end to oldAnchor.
                self.set_range(
                    cx.no_gc(),
                    Some(SelectionRange::new(
                        SelectionBoundary::new(node, offset),
                        SelectionBoundary::new(&old_anchor_node, old_anchor_offset),
                    )),
                );
                direction = Direction::Backwards;
            }
        }

        // Step 8. Set this's range to newRange.
        // Note: Done above to satisfy crown.

        // Step 9. If newFocus is before oldAnchor, set this's direction to backwards.
        // Otherwise, set it to forwards.
        self.direction.set(direction);

        Ok(())
    }

    /// <https://w3c.github.io/selection-api/#dom-selection-setbaseandextent>
    fn SetBaseAndExtent(
        &self,
        cx: &mut JSContext,
        anchor_node: &Node,
        anchor_offset: u32,
        focus_node: &Node,
        focus_offset: u32,
    ) -> ErrorResult {
        // This isn't specified, but it appears to be implementation behavior of other
        // browsers. See w3c/selection-api#118.
        if anchor_node.is_doctype() || focus_node.is_doctype() {
            return Err(Error::InvalidNodeType(None));
        }

        // Step 1. If anchorOffset is longer than anchorNode's length or if focusOffset is
        // longer than focusNode's length, throw an IndexSizeError exception and abort
        // these steps.
        if anchor_offset > anchor_node.len() || focus_offset > focus_node.len() {
            return Err(Error::IndexSize(None));
        }

        // Step 2. If document associated with this is not a shadow-including inclusive
        // ancestor of anchorNode or focusNode, abort these steps.
        //
        // TODO(mrobinson): This should eventually allow nodes in the same composed tree (and
        // not just the same tree), but this requires more work to allow `Selection` to cross
        // shadow tree boundaries.
        if &*anchor_node.GetRootNode(&GetRootNodeOptions { composed: false }) !=
            self.document.upcast::<Node>()
        {
            return Ok(());
        }
        if &*focus_node.GetRootNode(&GetRootNodeOptions { composed: false }) !=
            self.document.upcast::<Node>()
        {
            return Ok(());
        }

        // Step 3. Let anchor be the boundary point (anchorNode, anchorOffset) and let
        // focus be the boundary point (focusNode, focusOffset).
        //
        // Note: We do not model the boundary point in this way.

        // Step 4. Let newRange be a new range.
        // Note: We set the range directly to satisfy crown.

        // Step 5. If anchor is before focus, set the start the newRange's start to anchor
        // and its end to focus. Otherwise, set the start them to focus and anchor
        // respectively.
        let is_anchor_before_focus = bp_position(
            cx.no_gc(),
            anchor_node,
            anchor_offset,
            focus_node,
            focus_offset,
        ) == Ordering::Less;
        let direction = if is_anchor_before_focus {
            self.set_range(
                cx.no_gc(),
                Some(SelectionRange::new(
                    SelectionBoundary::new(anchor_node, anchor_offset),
                    SelectionBoundary::new(focus_node, focus_offset),
                )),
            );
            Direction::Forwards
        } else {
            self.set_range(
                cx.no_gc(),
                Some(SelectionRange::new(
                    SelectionBoundary::new(focus_node, focus_offset),
                    SelectionBoundary::new(anchor_node, anchor_offset),
                )),
            );
            Direction::Backwards
        };

        // Step 6. Set this's range to newRange.
        // Note: Done above to satisfy crown.

        // Step 7. If focus is before anchor, set this's direction to backwards.
        // Otherwise, set it to forwards
        self.direction.set(direction);

        Ok(())
    }

    /// <https://w3c.github.io/selection-api/#dom-selection-selectallchildren>
    fn SelectAllChildren(&self, cx: &mut JSContext, node: &Node) -> ErrorResult {
        // Step 1. If node is a DocumentType, throw an InvalidNodeTypeError exception and
        // abort these steps.
        if node.is_doctype() {
            return Err(Error::InvalidNodeType(None));
        }

        // Step 2. If node's root is not the document associated with this, abort these
        // steps.
        if !self.is_in_document_of_range(node) {
            return Ok(());
        }

        // Let newRange be a new range and childCount be the number of children of node.
        let child_count = node.children_count();

        // Step 4. Set newRange's start to (node, 0).
        // Step 5. Set newRange's end to (node, childCount).
        // Step 6. Set this's range to newRange.
        self.set_range(
            cx.no_gc(),
            Some(SelectionRange::new(
                SelectionBoundary::new(node, 0),
                SelectionBoundary::new(node, child_count),
            )),
        );

        // Step 7. Set this's direction to forwards.
        self.direction.set(Direction::Forwards);

        Ok(())
    }

    /// <https://w3c.github.io/selection-api/#dom-selection-deletecontents>
    fn DeleteFromDocument(&self, cx: &mut JSContext) -> ErrorResult {
        // > The method must invoke deleteContents() on this's range if this is not empty
        // > and both focus and anchor are in the document tree. Otherwise the method must
        // > do nothing.
        if self
            .range
            .borrow()
            .as_ref()
            .is_none_or(|range| !range.start_and_end_are_in_document_tree())
        {
            return Ok(());
        }

        self.active_range(cx)
            .map_or(Ok(()), |active_range| active_range.DeleteContents(cx))
    }

    /// <https://w3c.github.io/selection-api/#dom-selection-containsnode>
    fn ContainsNode(&self, no_gc: &NoGC, node: &Node, allow_partial_containment: bool) -> bool {
        // > The method must return false if this is empty or if node's root is not the document
        // > associated with this.
        // >
        // > Otherwise, if allowPartialContainment is false, the method must return true if and only
        // > if start of its range is before or visually equivalent to the first boundary point in
        // > the node *and* end of its range is after or visually equivalent to the last boundary
        // > point in the node.
        // >
        // > If allowPartialContainment is true, the method must return true if and only if start of
        // > its range is before or visually equivalent to the last boundary point in the node *and*
        // > end of its range is after or visually equivalent to the first boundary point in the
        // > node.
        if !self.is_in_document_of_range(node) {
            return false;
        }
        let range = self.range.borrow();
        let Some(range) = range.as_ref() else {
            return false;
        };
        let start_node = &*range.start.container;
        if !self.is_in_document_of_range(start_node) {
            return false;
        }
        let end_node = &*range.end.container;

        let first_offset = 0;
        let last_offset = node.len();
        let (compare_start_to, compare_end_to) = if allow_partial_containment {
            (last_offset, first_offset)
        } else {
            (first_offset, last_offset)
        };

        // TODO: find out what "visually equivalent" means for boundary points and implement it.
        // https://github.com/w3c/selection-api/issues/6
        // For now it is simplified to "position is equal".
        matches!(
            bp_position(
                no_gc,
                start_node,
                range.start.offset,
                node,
                compare_start_to
            ),
            Ordering::Less | Ordering::Equal
        ) && matches!(
            bp_position(no_gc, end_node, range.end.offset, node, compare_end_to),
            Ordering::Greater | Ordering::Equal
        )
    }

    /// <https://w3c.github.io/selection-api/#dom-selection-stringifier>
    fn Stringifier(&self, cx: &mut JSContext) -> DOMString {
        // > The stringification must return the string, which is the concatenation of the
        // > rendered text if there is a range associated with this.
        // >
        // > If the selection is within a textarea or input element, it must return the
        // > selected substring in its value.
        //
        // TODO: This implementation should be examined in depth. Does rendered text take
        // into account `display: none`. The case for textarea and input elements is
        // completely unhandled here.
        self.GetRangeAt(cx, 0)
            .map(|range| range.Stringifier(cx.no_gc()))
            .unwrap_or_default()
    }
}

impl<'dom> LayoutDom<'dom, Selection> {
    #[expect(unsafe_code)]
    pub(crate) fn range_for_layout(&self) -> &Option<SelectionRange> {
        unsafe { self.unsafe_get().range.borrow_for_layout() }
    }
}

enum FlatTreeNodePosition {
    Before(DomRoot<Node>),
    Inside(DomRoot<Node>),
    After(DomRoot<Node>),
}

impl FlatTreeNodePosition {
    fn node(&self) -> &Node {
        match self {
            FlatTreeNodePosition::Before(node) => node,
            FlatTreeNodePosition::Inside(node) => node,
            FlatTreeNodePosition::After(node) => node,
        }
    }
}

/// Find the position of a node and offset in the flat tree for the purposes of selection
/// boundaries. This projects the given position onto the flat tree, accounting for origin
/// nodes that may not actually be in the flat tree at all.
fn position_in_flat_tree_for_selection(
    no_gc: &NoGC,
    container: DomRoot<Node>,
    offset: usize,
) -> FlatTreeNodePosition {
    if container.is::<CharacterData>() {
        return FlatTreeNodePosition::Inside(container);
    }

    let shadow_host_or_node = |node: &Node| {
        container
            .downcast::<ShadowRoot>()
            .map(|shadow_root| DomRoot::upcast(shadow_root.Host()))
            .unwrap_or(DomRoot::from_ref(node))
    };

    if let Some(child) = container.children().nth(offset) {
        if let FlatTreeParent::Parent(_) = child.parent_in_flat_tree(no_gc) {
            return FlatTreeNodePosition::Before(child);
        }
    } else if let Some(last_child) = container.GetLastChild() &&
        let FlatTreeParent::Parent(_) = last_child.parent_in_flat_tree(no_gc)
    {
        return FlatTreeNodePosition::After(shadow_host_or_node(&container));
    }

    // The container has no child in the flat tree or the child indicated by the index
    // isn't in the flat tree, so just return a position inside that container.
    FlatTreeNodePosition::Inside(shadow_host_or_node(&container))
}

impl Node {
    /// Get the `Utf16CodeUnits` offset for the given offset if `self` is a
    /// `CharacterData` or else return the offset in the child list.
    fn to_sibling_or_utf16_offset(&self, offset: Utf32CodeUnitsOrNodeOffset) -> u32 {
        if let Some(character_data) = self.downcast::<CharacterData>() {
            offset.to_utf16_code_units_in(&character_data.data()).0 as u32
        } else {
            offset.0 as u32
        }
    }
}

bitflags! {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub(crate) struct SelectionLiveRangeNotification: u8 {
        const Start = 1 << 0;
        const End = 1 << 1;
    }
}
