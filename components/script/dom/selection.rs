/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;

use dom_struct::dom_struct;

use crate::dom::bindings::codegen::Bindings::NodeBinding::{GetRootNodeOptions, NodeMethods};
use crate::dom::bindings::codegen::Bindings::RangeBinding::RangeMethods;
use crate::dom::bindings::codegen::Bindings::SelectionBinding::SelectionMethods;
use crate::dom::bindings::error::{Error, ErrorResult, Fallible};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::{reflect_dom_object, DomGlobal, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot, MutNullableDom};
use crate::dom::bindings::str::DOMString;
use crate::dom::document::Document;
use crate::dom::eventtarget::EventTarget;
use crate::dom::node::{Node, NodeTraits};
use crate::dom::range::Range;
use crate::script_runtime::CanGc;

#[derive(Clone, Copy, JSTraceable, MallocSizeOf)]
enum Direction {
    Forwards,
    Backwards,
    Directionless,
}

#[dom_struct]
pub(crate) struct Selection {
    reflector_: Reflector,
    document: Dom<Document>,
    range: MutNullableDom<Range>,
    direction: Cell<Direction>,
    task_queued: Cell<bool>,
}

impl Selection {
    fn new_inherited(document: &Document) -> Selection {
        Selection {
            reflector_: Reflector::new(),
            document: Dom::from_ref(document),
            range: MutNullableDom::new(None),
            direction: Cell::new(Direction::Directionless),
            task_queued: Cell::new(false),
        }
    }

    pub(crate) fn new(document: &Document, can_gc: CanGc) -> DomRoot<Selection> {
        reflect_dom_object(
            Box::new(Selection::new_inherited(document)),
            &*document.global(),
            can_gc,
        )
    }

    fn set_range(&self, range: &Range) {
        // If we are setting to literally the same Range object
        // (not just the same positions), then there's nothing changing
        // and no task to queue.
        if let Some(existing) = self.range.get() {
            if &*existing == range {
                return;
            }
        }
        self.range.set(Some(range));
        range.associate_selection(self);
        self.queue_selectionchange_task();
    }

    fn clear_range(&self) {
        // If we already don't have a a Range object, then there's
        // nothing changing and no task to queue.
        if let Some(range) = self.range.get() {
            range.disassociate_selection(self);
            self.range.set(None);
            self.queue_selectionchange_task();
        }
    }

    pub(crate) fn queue_selectionchange_task(&self) {
        if self.task_queued.get() {
            // Spec doesn't specify not to queue multiple tasks,
            // but it's much easier to code range operations if
            // change notifications within a method are idempotent.
            return;
        }
        let this = Trusted::new(self);
        self.document
            .owner_global()
            .task_manager()
            .user_interaction_task_source() // w3c/selection-api#117
            .queue(
                task!(selectionchange_task_steps: move || {
                    let this = this.root();
                    this.task_queued.set(false);
                    this.document.upcast::<EventTarget>().fire_event(atom!("selectionchange"), CanGc::note());
                })
            );
        self.task_queued.set(true);
    }

    fn is_same_root(&self, node: &Node) -> bool {
        &*node.GetRootNode(&GetRootNodeOptions::empty()) == self.document.upcast::<Node>()
    }
}

impl SelectionMethods<crate::DomTypeHolder> for Selection {
    // https://w3c.github.io/selection-api/#dom-selection-anchornode
    fn GetAnchorNode(&self) -> Option<DomRoot<Node>> {
        if let Some(range) = self.range.get() {
            match self.direction.get() {
                Direction::Forwards => Some(range.start_container()),
                _ => Some(range.end_container()),
            }
        } else {
            None
        }
    }

    // https://w3c.github.io/selection-api/#dom-selection-anchoroffset
    fn AnchorOffset(&self) -> u32 {
        if let Some(range) = self.range.get() {
            match self.direction.get() {
                Direction::Forwards => range.start_offset(),
                _ => range.end_offset(),
            }
        } else {
            0
        }
    }

    // https://w3c.github.io/selection-api/#dom-selection-focusnode
    fn GetFocusNode(&self) -> Option<DomRoot<Node>> {
        if let Some(range) = self.range.get() {
            match self.direction.get() {
                Direction::Forwards => Some(range.end_container()),
                _ => Some(range.start_container()),
            }
        } else {
            None
        }
    }

    // https://w3c.github.io/selection-api/#dom-selection-focusoffset
    fn FocusOffset(&self) -> u32 {
        if let Some(range) = self.range.get() {
            match self.direction.get() {
                Direction::Forwards => range.end_offset(),
                _ => range.start_offset(),
            }
        } else {
            0
        }
    }

    // https://w3c.github.io/selection-api/#dom-selection-iscollapsed
    fn IsCollapsed(&self) -> bool {
        if let Some(range) = self.range.get() {
            range.collapsed()
        } else {
            true
        }
    }

    // https://w3c.github.io/selection-api/#dom-selection-rangecount
    fn RangeCount(&self) -> u32 {
        if self.range.get().is_some() {
            1
        } else {
            0
        }
    }

    // https://w3c.github.io/selection-api/#dom-selection-type
    fn Type(&self) -> DOMString {
        if let Some(range) = self.range.get() {
            if range.collapsed() {
                DOMString::from("Caret")
            } else {
                DOMString::from("Range")
            }
        } else {
            DOMString::from("None")
        }
    }

    // https://w3c.github.io/selection-api/#dom-selection-getrangeat
    fn GetRangeAt(&self, index: u32) -> Fallible<DomRoot<Range>> {
        if index != 0 {
            Err(Error::IndexSize)
        } else if let Some(range) = self.range.get() {
            Ok(DomRoot::from_ref(&range))
        } else {
            Err(Error::IndexSize)
        }
    }

    // https://w3c.github.io/selection-api/#dom-selection-addrange
    fn AddRange(&self, range: &Range) {
        // Step 1
        if !self.is_same_root(&range.start_container()) {
            return;
        }

        // Step 2
        if self.RangeCount() != 0 {
            return;
        }

        // Step 3
        self.set_range(range);
        // Are we supposed to set Direction here? w3c/selection-api#116
        self.direction.set(Direction::Forwards);
    }

    // https://w3c.github.io/selection-api/#dom-selection-removerange
    fn RemoveRange(&self, range: &Range) -> ErrorResult {
        if let Some(own_range) = self.range.get() {
            if &*own_range == range {
                self.clear_range();
                return Ok(());
            }
        }
        Err(Error::NotFound)
    }

    // https://w3c.github.io/selection-api/#dom-selection-removeallranges
    fn RemoveAllRanges(&self) {
        self.clear_range();
    }

    // https://w3c.github.io/selection-api/#dom-selection-empty
    // TODO: When implementing actual selection UI, this may be the correct
    // method to call as the abandon-selection action
    fn Empty(&self) {
        self.clear_range();
    }

    // https://w3c.github.io/selection-api/#dom-selection-collapse
    fn Collapse(&self, node: Option<&Node>, offset: u32, can_gc: CanGc) -> ErrorResult {
        if let Some(node) = node {
            if node.is_doctype() {
                // w3c/selection-api#118
                return Err(Error::InvalidNodeType);
            }
            if offset > node.len() {
                // Step 2
                return Err(Error::IndexSize);
            }

            if !self.is_same_root(node) {
                // Step 3
                return Ok(());
            }

            // Steps 4-5
            let range = Range::new(&self.document, node, offset, node, offset, can_gc);

            // Step 6
            self.set_range(&range);
            // Are we supposed to set Direction here? w3c/selection-api#116
            //
            self.direction.set(Direction::Forwards);
        } else {
            // Step 1
            self.clear_range();
        }
        Ok(())
    }

    // https://w3c.github.io/selection-api/#dom-selection-setposition
    // TODO: When implementing actual selection UI, this may be the correct
    // method to call as the start-of-selection action, after a
    // selectstart event has fired and not been cancelled.
    fn SetPosition(&self, node: Option<&Node>, offset: u32, can_gc: CanGc) -> ErrorResult {
        self.Collapse(node, offset, can_gc)
    }

    // https://w3c.github.io/selection-api/#dom-selection-collapsetostart
    fn CollapseToStart(&self, can_gc: CanGc) -> ErrorResult {
        if let Some(range) = self.range.get() {
            self.Collapse(
                Some(&*range.start_container()),
                range.start_offset(),
                can_gc,
            )
        } else {
            Err(Error::InvalidState)
        }
    }

    // https://w3c.github.io/selection-api/#dom-selection-collapsetoend
    fn CollapseToEnd(&self, can_gc: CanGc) -> ErrorResult {
        if let Some(range) = self.range.get() {
            self.Collapse(Some(&*range.end_container()), range.end_offset(), can_gc)
        } else {
            Err(Error::InvalidState)
        }
    }

    // https://w3c.github.io/selection-api/#dom-selection-extend
    // TODO: When implementing actual selection UI, this may be the correct
    // method to call as the continue-selection action
    fn Extend(&self, node: &Node, offset: u32, can_gc: CanGc) -> ErrorResult {
        if !self.is_same_root(node) {
            // Step 1
            return Ok(());
        }
        if let Some(range) = self.range.get() {
            if node.is_doctype() {
                // w3c/selection-api#118
                return Err(Error::InvalidNodeType);
            }

            if offset > node.len() {
                // As with is_doctype, not explicit in selection spec steps here
                // but implied by which exceptions are thrown in WPT tests
                return Err(Error::IndexSize);
            }

            // Step 4
            if !self.is_same_root(&range.start_container()) {
                // Step 5, and its following 8 and 9
                self.set_range(&Range::new(
                    &self.document,
                    node,
                    offset,
                    node,
                    offset,
                    can_gc,
                ));
                self.direction.set(Direction::Forwards);
            } else {
                let old_anchor_node = &*self.GetAnchorNode().unwrap(); // has range, therefore has anchor node
                let old_anchor_offset = self.AnchorOffset();
                let is_old_anchor_before_or_equal = {
                    if old_anchor_node == node {
                        old_anchor_offset <= offset
                    } else {
                        old_anchor_node.is_before(node)
                    }
                };
                if is_old_anchor_before_or_equal {
                    // Step 6, and its following 8 and 9
                    self.set_range(&Range::new(
                        &self.document,
                        old_anchor_node,
                        old_anchor_offset,
                        node,
                        offset,
                        can_gc,
                    ));
                    self.direction.set(Direction::Forwards);
                } else {
                    // Step 7, and its following 8 and 9
                    self.set_range(&Range::new(
                        &self.document,
                        node,
                        offset,
                        old_anchor_node,
                        old_anchor_offset,
                        can_gc,
                    ));
                    self.direction.set(Direction::Backwards);
                }
            };
        } else {
            // Step 2
            return Err(Error::InvalidState);
        }
        Ok(())
    }

    // https://w3c.github.io/selection-api/#dom-selection-setbaseandextent
    fn SetBaseAndExtent(
        &self,
        anchor_node: &Node,
        anchor_offset: u32,
        focus_node: &Node,
        focus_offset: u32,
        can_gc: CanGc,
    ) -> ErrorResult {
        // Step 1
        if anchor_node.is_doctype() || focus_node.is_doctype() {
            // w3c/selection-api#118
            return Err(Error::InvalidNodeType);
        }

        if anchor_offset > anchor_node.len() || focus_offset > focus_node.len() {
            return Err(Error::IndexSize);
        }

        // Step 2
        if !self.is_same_root(anchor_node) || !self.is_same_root(focus_node) {
            return Ok(());
        }

        // Steps 5-7
        let is_focus_before_anchor = {
            if anchor_node == focus_node {
                focus_offset < anchor_offset
            } else {
                focus_node.is_before(anchor_node)
            }
        };
        if is_focus_before_anchor {
            self.set_range(&Range::new(
                &self.document,
                focus_node,
                focus_offset,
                anchor_node,
                anchor_offset,
                can_gc,
            ));
            self.direction.set(Direction::Backwards);
        } else {
            self.set_range(&Range::new(
                &self.document,
                anchor_node,
                anchor_offset,
                focus_node,
                focus_offset,
                can_gc,
            ));
            self.direction.set(Direction::Forwards);
        }
        Ok(())
    }

    // https://w3c.github.io/selection-api/#dom-selection-selectallchildren
    fn SelectAllChildren(&self, node: &Node, can_gc: CanGc) -> ErrorResult {
        if node.is_doctype() {
            // w3c/selection-api#118
            return Err(Error::InvalidNodeType);
        }
        if !self.is_same_root(node) {
            return Ok(());
        }

        // Spec wording just says node length here, but WPT specifically
        // wants number of children (the main difference is that it's 0
        // for cdata).
        self.set_range(&Range::new(
            &self.document,
            node,
            0,
            node,
            node.children_count(),
            can_gc,
        ));

        self.direction.set(Direction::Forwards);
        Ok(())
    }

    // https://w3c.github.io/selection-api/#dom-selection-deletecontents
    fn DeleteFromDocument(&self) -> ErrorResult {
        if let Some(range) = self.range.get() {
            // Since the range is changing, it should trigger a
            // selectionchange event as it would if if mutated any other way
            return range.DeleteContents();
        }
        Ok(())
    }

    // https://w3c.github.io/selection-api/#dom-selection-containsnode
    fn ContainsNode(&self, node: &Node, allow_partial_containment: bool) -> bool {
        // TODO: Spec requires a "visually equivalent to" check, which is
        // probably up to a layout query. This is therefore not a full implementation.
        if !self.is_same_root(node) {
            return false;
        }
        if let Some(range) = self.range.get() {
            let start_node = &*range.start_container();
            if !self.is_same_root(start_node) {
                // node can't be contained in a range with a different root
                return false;
            }
            if allow_partial_containment {
                // Spec seems to be incorrect here, w3c/selection-api#116
                if node.is_before(start_node) {
                    return false;
                }
                let end_node = &*range.end_container();
                if end_node.is_before(node) {
                    return false;
                }
                if node == start_node {
                    return range.start_offset() < node.len();
                }
                if node == end_node {
                    return range.end_offset() > 0;
                }
                true
            } else {
                if node.is_before(start_node) {
                    return false;
                }
                let end_node = &*range.end_container();
                if end_node.is_before(node) {
                    return false;
                }
                if node == start_node {
                    return range.start_offset() == 0;
                }
                if node == end_node {
                    return range.end_offset() == node.len();
                }
                true
            }
        } else {
            // No range
            false
        }
    }

    // https://w3c.github.io/selection-api/#dom-selection-stringifier
    fn Stringifier(&self) -> DOMString {
        // The spec as of Jan 31 2020 just says
        // "See W3C bug 10583." for this method.
        // Stringifying the range seems at least approximately right
        // and passes the non-style-dependent case in the WPT tests.
        if let Some(range) = self.range.get() {
            range.Stringifier()
        } else {
            DOMString::from("")
        }
    }
}
