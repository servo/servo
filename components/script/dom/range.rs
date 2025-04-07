/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::UnsafeCell;
use std::cmp::{Ordering, PartialOrd};
use std::iter;

use dom_struct::dom_struct;
use js::jsapi::JSTracer;
use js::rust::HandleObject;
use malloc_size_of::{MallocSizeOf, MallocSizeOfOps};

use crate::dom::abstractrange::{AbstractRange, BoundaryPoint, bp_position};
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::AbstractRangeBinding::AbstractRangeMethods;
use crate::dom::bindings::codegen::Bindings::CharacterDataBinding::CharacterDataMethods;
use crate::dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use crate::dom::bindings::codegen::Bindings::NodeListBinding::NodeListMethods;
use crate::dom::bindings::codegen::Bindings::RangeBinding::{RangeConstants, RangeMethods};
use crate::dom::bindings::codegen::Bindings::TextBinding::TextMethods;
use crate::dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use crate::dom::bindings::error::{Error, ErrorResult, Fallible};
use crate::dom::bindings::inheritance::{Castable, CharacterDataTypeId, NodeTypeId};
use crate::dom::bindings::reflector::reflect_dom_object_with_proto;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::bindings::trace::JSTraceable;
use crate::dom::bindings::weakref::{WeakRef, WeakRefVec};
use crate::dom::characterdata::CharacterData;
use crate::dom::document::Document;
use crate::dom::documentfragment::DocumentFragment;
use crate::dom::domrect::DOMRect;
use crate::dom::domrectlist::DOMRectList;
use crate::dom::element::Element;
use crate::dom::htmlscriptelement::HTMLScriptElement;
use crate::dom::node::{Node, NodeTraits, ShadowIncluding, UnbindContext};
use crate::dom::selection::Selection;
use crate::dom::text::Text;
use crate::dom::window::Window;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct Range {
    abstract_range: AbstractRange,
    // A range that belongs to a Selection needs to know about it
    // so selectionchange can fire when the range changes.
    // A range shouldn't belong to more than one Selection at a time,
    // but from the spec as of Feb 1 2020 I can't rule out a corner case like:
    // * Select a range R in document A, from node X to Y
    // * Insert everything from X to Y into document B
    // * Set B's selection's range to R
    // which leaves R technically, and observably, associated with A even though
    // it will fail the same-root-node check on many of A's selection's methods.
    associated_selections: DomRefCell<Vec<Dom<Selection>>>,
}

struct ContainedChildren {
    first_partially_contained_child: Option<DomRoot<Node>>,
    last_partially_contained_child: Option<DomRoot<Node>>,
    contained_children: Vec<DomRoot<Node>>,
}

impl Range {
    fn new_inherited(
        start_container: &Node,
        start_offset: u32,
        end_container: &Node,
        end_offset: u32,
    ) -> Range {
        debug_assert!(start_offset <= start_container.len());
        debug_assert!(end_offset <= end_container.len());
        Range {
            abstract_range: AbstractRange::new_inherited(
                start_container,
                start_offset,
                end_container,
                end_offset,
            ),
            associated_selections: DomRefCell::new(vec![]),
        }
    }

    pub(crate) fn new_with_doc(
        document: &Document,
        proto: Option<HandleObject>,
        can_gc: CanGc,
    ) -> DomRoot<Range> {
        let root = document.upcast();
        Range::new_with_proto(document, proto, root, 0, root, 0, can_gc)
    }

    pub(crate) fn new(
        document: &Document,
        start_container: &Node,
        start_offset: u32,
        end_container: &Node,
        end_offset: u32,
        can_gc: CanGc,
    ) -> DomRoot<Range> {
        Self::new_with_proto(
            document,
            None,
            start_container,
            start_offset,
            end_container,
            end_offset,
            can_gc,
        )
    }

    fn new_with_proto(
        document: &Document,
        proto: Option<HandleObject>,
        start_container: &Node,
        start_offset: u32,
        end_container: &Node,
        end_offset: u32,
        can_gc: CanGc,
    ) -> DomRoot<Range> {
        let range = reflect_dom_object_with_proto(
            Box::new(Range::new_inherited(
                start_container,
                start_offset,
                end_container,
                end_offset,
            )),
            document.window(),
            proto,
            can_gc,
        );
        start_container.ranges().push(WeakRef::new(&range));
        if start_container != end_container {
            end_container.ranges().push(WeakRef::new(&range));
        }
        range
    }

    /// <https://dom.spec.whatwg.org/#contained>
    fn contains(&self, node: &Node) -> bool {
        matches!(
            (
                bp_position(node, 0, &self.start_container(), self.start_offset()),
                bp_position(node, node.len(), &self.end_container(), self.end_offset()),
            ),
            (Some(Ordering::Greater), Some(Ordering::Less))
        )
    }

    /// <https://dom.spec.whatwg.org/#partially-contained>
    fn partially_contains(&self, node: &Node) -> bool {
        self.start_container()
            .inclusive_ancestors(ShadowIncluding::No)
            .any(|n| &*n == node) !=
            self.end_container()
                .inclusive_ancestors(ShadowIncluding::No)
                .any(|n| &*n == node)
    }

    /// <https://dom.spec.whatwg.org/#concept-range-clone>
    fn contained_children(&self) -> Fallible<ContainedChildren> {
        let start_node = self.start_container();
        let end_node = self.end_container();
        // Steps 5-6.
        let common_ancestor = self.CommonAncestorContainer();

        let first_partially_contained_child = if start_node.is_inclusive_ancestor_of(&end_node) {
            // Step 7.
            None
        } else {
            // Step 8.
            common_ancestor
                .children()
                .find(|node| Range::partially_contains(self, node))
        };

        let last_partially_contained_child = if end_node.is_inclusive_ancestor_of(&start_node) {
            // Step 9.
            None
        } else {
            // Step 10.
            common_ancestor
                .rev_children()
                .find(|node| Range::partially_contains(self, node))
        };

        // Step 11.
        let contained_children: Vec<DomRoot<Node>> = common_ancestor
            .children()
            .filter(|n| self.contains(n))
            .collect();

        // Step 12.
        if contained_children.iter().any(|n| n.is_doctype()) {
            return Err(Error::HierarchyRequest);
        }

        Ok(ContainedChildren {
            first_partially_contained_child,
            last_partially_contained_child,
            contained_children,
        })
    }

    /// <https://dom.spec.whatwg.org/#concept-range-bp-set>
    fn set_start(&self, node: &Node, offset: u32) {
        if self.start().node() != node || self.start_offset() != offset {
            self.report_change();
        }
        if self.start().node() != node {
            if self.start().node() == self.end().node() {
                node.ranges().push(WeakRef::new(self));
            } else if self.end().node() == node {
                self.start_container().ranges().remove(self);
            } else {
                node.ranges()
                    .push(self.start_container().ranges().remove(self));
            }
        }
        self.start().set(node, offset);
    }

    /// <https://dom.spec.whatwg.org/#concept-range-bp-set>
    fn set_end(&self, node: &Node, offset: u32) {
        if self.end().node() != node || self.end_offset() != offset {
            self.report_change();
        }
        if self.end().node() != node {
            if self.end().node() == self.start().node() {
                node.ranges().push(WeakRef::new(self));
            } else if self.start().node() == node {
                self.end_container().ranges().remove(self);
            } else {
                node.ranges()
                    .push(self.end_container().ranges().remove(self));
            }
        }
        self.end().set(node, offset);
    }

    /// <https://dom.spec.whatwg.org/#dom-range-comparepointnode-offset>
    fn compare_point(&self, node: &Node, offset: u32) -> Fallible<Ordering> {
        let start_node = self.start_container();
        let start_node_root = start_node
            .inclusive_ancestors(ShadowIncluding::No)
            .last()
            .unwrap();
        let node_root = node
            .inclusive_ancestors(ShadowIncluding::No)
            .last()
            .unwrap();
        if start_node_root != node_root {
            // Step 1.
            return Err(Error::WrongDocument);
        }
        if node.is_doctype() {
            // Step 2.
            return Err(Error::InvalidNodeType);
        }
        if offset > node.len() {
            // Step 3.
            return Err(Error::IndexSize);
        }
        if let Ordering::Less = bp_position(node, offset, &start_node, self.start_offset()).unwrap()
        {
            // Step 4.
            return Ok(Ordering::Less);
        }
        if let Ordering::Greater =
            bp_position(node, offset, &self.end_container(), self.end_offset()).unwrap()
        {
            // Step 5.
            return Ok(Ordering::Greater);
        }
        // Step 6.
        Ok(Ordering::Equal)
    }

    pub(crate) fn associate_selection(&self, selection: &Selection) {
        let mut selections = self.associated_selections.borrow_mut();
        if !selections.iter().any(|s| &**s == selection) {
            selections.push(Dom::from_ref(selection));
        }
    }

    pub(crate) fn disassociate_selection(&self, selection: &Selection) {
        self.associated_selections
            .borrow_mut()
            .retain(|s| &**s != selection);
    }

    fn report_change(&self) {
        self.associated_selections
            .borrow()
            .iter()
            .for_each(|s| s.queue_selectionchange_task());
    }

    fn abstract_range(&self) -> &AbstractRange {
        &self.abstract_range
    }

    fn start(&self) -> &BoundaryPoint {
        self.abstract_range().start()
    }

    fn end(&self) -> &BoundaryPoint {
        self.abstract_range().end()
    }

    pub(crate) fn start_container(&self) -> DomRoot<Node> {
        self.abstract_range().StartContainer()
    }

    pub(crate) fn start_offset(&self) -> u32 {
        self.abstract_range().StartOffset()
    }

    pub(crate) fn end_container(&self) -> DomRoot<Node> {
        self.abstract_range().EndContainer()
    }

    pub(crate) fn end_offset(&self) -> u32 {
        self.abstract_range().EndOffset()
    }

    pub(crate) fn collapsed(&self) -> bool {
        self.abstract_range().Collapsed()
    }

    fn client_rects(
        &self,
        can_gc: CanGc,
    ) -> impl Iterator<Item = euclid::Rect<app_units::Au, euclid::UnknownUnit>> {
        // FIXME: For text nodes that are only partially selected, this should return the client
        // rect of the selected part, not the whole text node.
        let start = self.start_container();
        let end = self.end_container();
        let document = start.owner_doc();
        let end_clone = end.clone();
        start
            .following_nodes(document.upcast::<Node>())
            .take_while(move |node| node != &end)
            .chain(iter::once(end_clone))
            .flat_map(move |node| node.content_boxes(can_gc))
    }
}

impl RangeMethods<crate::DomTypeHolder> for Range {
    /// <https://dom.spec.whatwg.org/#dom-range>
    fn Constructor(
        window: &Window,
        proto: Option<HandleObject>,
        can_gc: CanGc,
    ) -> Fallible<DomRoot<Range>> {
        let document = window.Document();
        Ok(Range::new_with_doc(&document, proto, can_gc))
    }

    /// <https://dom.spec.whatwg.org/#dom-range-commonancestorcontainer>
    fn CommonAncestorContainer(&self) -> DomRoot<Node> {
        self.end_container()
            .common_ancestor(&self.start_container(), ShadowIncluding::No)
            .expect("Couldn't find common ancestor container")
    }

    /// <https://dom.spec.whatwg.org/#dom-range-setstart>
    #[allow(clippy::neg_cmp_op_on_partial_ord)]
    fn SetStart(&self, node: &Node, offset: u32) -> ErrorResult {
        if node.is_doctype() {
            // Step 1.
            Err(Error::InvalidNodeType)
        } else if offset > node.len() {
            // Step 2.
            Err(Error::IndexSize)
        } else {
            // Step 3.
            self.set_start(node, offset);
            if !(self.start() <= self.end()) {
                // Step 4.
                self.set_end(node, offset);
            }
            Ok(())
        }
    }

    /// <https://dom.spec.whatwg.org/#dom-range-setend>
    #[allow(clippy::neg_cmp_op_on_partial_ord)]
    fn SetEnd(&self, node: &Node, offset: u32) -> ErrorResult {
        if node.is_doctype() {
            // Step 1.
            Err(Error::InvalidNodeType)
        } else if offset > node.len() {
            // Step 2.
            Err(Error::IndexSize)
        } else {
            // Step 3.
            self.set_end(node, offset);
            if !(self.end() >= self.start()) {
                // Step 4.
                self.set_start(node, offset);
            }
            Ok(())
        }
    }

    /// <https://dom.spec.whatwg.org/#dom-range-setstartbefore>
    fn SetStartBefore(&self, node: &Node) -> ErrorResult {
        let parent = node.GetParentNode().ok_or(Error::InvalidNodeType)?;
        self.SetStart(&parent, node.index())
    }

    /// <https://dom.spec.whatwg.org/#dom-range-setstartafter>
    fn SetStartAfter(&self, node: &Node) -> ErrorResult {
        let parent = node.GetParentNode().ok_or(Error::InvalidNodeType)?;
        self.SetStart(&parent, node.index() + 1)
    }

    /// <https://dom.spec.whatwg.org/#dom-range-setendbefore>
    fn SetEndBefore(&self, node: &Node) -> ErrorResult {
        let parent = node.GetParentNode().ok_or(Error::InvalidNodeType)?;
        self.SetEnd(&parent, node.index())
    }

    /// <https://dom.spec.whatwg.org/#dom-range-setendafter>
    fn SetEndAfter(&self, node: &Node) -> ErrorResult {
        let parent = node.GetParentNode().ok_or(Error::InvalidNodeType)?;
        self.SetEnd(&parent, node.index() + 1)
    }

    /// <https://dom.spec.whatwg.org/#dom-range-collapse>
    fn Collapse(&self, to_start: bool) {
        if to_start {
            self.set_end(&self.start_container(), self.start_offset());
        } else {
            self.set_start(&self.end_container(), self.end_offset());
        }
    }

    /// <https://dom.spec.whatwg.org/#dom-range-selectnode>
    fn SelectNode(&self, node: &Node) -> ErrorResult {
        // Steps 1, 2.
        let parent = node.GetParentNode().ok_or(Error::InvalidNodeType)?;
        // Step 3.
        let index = node.index();
        // Step 4.
        self.set_start(&parent, index);
        // Step 5.
        self.set_end(&parent, index + 1);
        Ok(())
    }

    /// <https://dom.spec.whatwg.org/#dom-range-selectnodecontents>
    fn SelectNodeContents(&self, node: &Node) -> ErrorResult {
        if node.is_doctype() {
            // Step 1.
            return Err(Error::InvalidNodeType);
        }
        // Step 2.
        let length = node.len();
        // Step 3.
        self.set_start(node, 0);
        // Step 4.
        self.set_end(node, length);
        Ok(())
    }

    /// <https://dom.spec.whatwg.org/#dom-range-compareboundarypoints>
    fn CompareBoundaryPoints(&self, how: u16, other: &Range) -> Fallible<i16> {
        if how > RangeConstants::END_TO_START {
            // Step 1.
            return Err(Error::NotSupported);
        }
        let this_root = self
            .start_container()
            .inclusive_ancestors(ShadowIncluding::No)
            .last()
            .unwrap();
        let other_root = other
            .start_container()
            .inclusive_ancestors(ShadowIncluding::No)
            .last()
            .unwrap();
        if this_root != other_root {
            // Step 2.
            return Err(Error::WrongDocument);
        }
        // Step 3.
        let (this_point, other_point) = match how {
            RangeConstants::START_TO_START => (self.start(), other.start()),
            RangeConstants::START_TO_END => (self.end(), other.start()),
            RangeConstants::END_TO_END => (self.end(), other.end()),
            RangeConstants::END_TO_START => (self.start(), other.end()),
            _ => unreachable!(),
        };
        // step 4.
        match this_point.partial_cmp(other_point).unwrap() {
            Ordering::Less => Ok(-1),
            Ordering::Equal => Ok(0),
            Ordering::Greater => Ok(1),
        }
    }

    /// <https://dom.spec.whatwg.org/#dom-range-clonerange>
    fn CloneRange(&self, can_gc: CanGc) -> DomRoot<Range> {
        let start_node = self.start_container();
        let owner_doc = start_node.owner_doc();
        Range::new(
            &owner_doc,
            &start_node,
            self.start_offset(),
            &self.end_container(),
            self.end_offset(),
            can_gc,
        )
    }

    /// <https://dom.spec.whatwg.org/#dom-range-ispointinrange>
    fn IsPointInRange(&self, node: &Node, offset: u32) -> Fallible<bool> {
        match self.compare_point(node, offset) {
            Ok(Ordering::Less) => Ok(false),
            Ok(Ordering::Equal) => Ok(true),
            Ok(Ordering::Greater) => Ok(false),
            Err(Error::WrongDocument) => {
                // Step 2.
                Ok(false)
            },
            Err(error) => Err(error),
        }
    }

    /// <https://dom.spec.whatwg.org/#dom-range-comparepoint>
    fn ComparePoint(&self, node: &Node, offset: u32) -> Fallible<i16> {
        self.compare_point(node, offset).map(|order| match order {
            Ordering::Less => -1,
            Ordering::Equal => 0,
            Ordering::Greater => 1,
        })
    }

    /// <https://dom.spec.whatwg.org/#dom-range-intersectsnode>
    fn IntersectsNode(&self, node: &Node) -> bool {
        let start_node = self.start_container();
        let start_node_root = self
            .start_container()
            .inclusive_ancestors(ShadowIncluding::No)
            .last()
            .unwrap();
        let node_root = node
            .inclusive_ancestors(ShadowIncluding::No)
            .last()
            .unwrap();
        if start_node_root != node_root {
            // Step 1.
            return false;
        }
        let parent = match node.GetParentNode() {
            Some(parent) => parent,
            None => {
                // Step 3.
                return true;
            },
        };
        // Step 4.
        let offset = node.index();
        // Step 5.
        Ordering::Greater ==
            bp_position(&parent, offset + 1, &start_node, self.start_offset()).unwrap() &&
            Ordering::Less ==
                bp_position(&parent, offset, &self.end_container(), self.end_offset())
                    .unwrap()
    }

    /// <https://dom.spec.whatwg.org/#dom-range-clonecontents>
    /// <https://dom.spec.whatwg.org/#concept-range-clone>
    fn CloneContents(&self, can_gc: CanGc) -> Fallible<DomRoot<DocumentFragment>> {
        // Step 3.
        let start_node = self.start_container();
        let start_offset = self.start_offset();
        let end_node = self.end_container();
        let end_offset = self.end_offset();

        // Step 1.
        let fragment = DocumentFragment::new(&start_node.owner_doc(), can_gc);

        // Step 2.
        if self.start() == self.end() {
            return Ok(fragment);
        }

        if end_node == start_node {
            if let Some(cdata) = start_node.downcast::<CharacterData>() {
                // Steps 4.1-2.
                let data = cdata
                    .SubstringData(start_offset, end_offset - start_offset)
                    .unwrap();
                let clone = cdata.clone_with_data(data, &start_node.owner_doc(), can_gc);
                // Step 4.3.
                fragment.upcast::<Node>().AppendChild(&clone, can_gc)?;
                // Step 4.4
                return Ok(fragment);
            }
        }

        // Steps 5-12.
        let ContainedChildren {
            first_partially_contained_child,
            last_partially_contained_child,
            contained_children,
        } = self.contained_children()?;

        if let Some(child) = first_partially_contained_child {
            // Step 13.
            if let Some(cdata) = child.downcast::<CharacterData>() {
                assert!(child == start_node);
                // Steps 13.1-2.
                let data = cdata
                    .SubstringData(start_offset, start_node.len() - start_offset)
                    .unwrap();
                let clone = cdata.clone_with_data(data, &start_node.owner_doc(), can_gc);
                // Step 13.3.
                fragment.upcast::<Node>().AppendChild(&clone, can_gc)?;
            } else {
                // Step 14.1.
                let clone = child.CloneNode(/* deep */ false, can_gc)?;
                // Step 14.2.
                fragment.upcast::<Node>().AppendChild(&clone, can_gc)?;
                // Step 14.3.
                let subrange = Range::new(
                    &clone.owner_doc(),
                    &start_node,
                    start_offset,
                    &child,
                    child.len(),
                    can_gc,
                );
                // Step 14.4.
                let subfragment = subrange.CloneContents(can_gc)?;
                // Step 14.5.
                clone.AppendChild(subfragment.upcast(), can_gc)?;
            }
        }

        // Step 15.
        for child in contained_children {
            // Step 15.1.
            let clone = child.CloneNode(/* deep */ true, can_gc)?;
            // Step 15.2.
            fragment.upcast::<Node>().AppendChild(&clone, can_gc)?;
        }

        if let Some(child) = last_partially_contained_child {
            // Step 16.
            if let Some(cdata) = child.downcast::<CharacterData>() {
                assert!(child == end_node);
                // Steps 16.1-2.
                let data = cdata.SubstringData(0, end_offset).unwrap();
                let clone = cdata.clone_with_data(data, &start_node.owner_doc(), can_gc);
                // Step 16.3.
                fragment.upcast::<Node>().AppendChild(&clone, can_gc)?;
            } else {
                // Step 17.1.
                let clone = child.CloneNode(/* deep */ false, can_gc)?;
                // Step 17.2.
                fragment.upcast::<Node>().AppendChild(&clone, can_gc)?;
                // Step 17.3.
                let subrange =
                    Range::new(&clone.owner_doc(), &child, 0, &end_node, end_offset, can_gc);
                // Step 17.4.
                let subfragment = subrange.CloneContents(can_gc)?;
                // Step 17.5.
                clone.AppendChild(subfragment.upcast(), can_gc)?;
            }
        }

        // Step 18.
        Ok(fragment)
    }

    /// <https://dom.spec.whatwg.org/#dom-range-extractcontents>
    /// <https://dom.spec.whatwg.org/#concept-range-extract>
    fn ExtractContents(&self, can_gc: CanGc) -> Fallible<DomRoot<DocumentFragment>> {
        // Step 3.
        let start_node = self.start_container();
        let start_offset = self.start_offset();
        let end_node = self.end_container();
        let end_offset = self.end_offset();

        // Step 1.
        let fragment = DocumentFragment::new(&start_node.owner_doc(), can_gc);

        // Step 2.
        if self.collapsed() {
            return Ok(fragment);
        }

        if end_node == start_node {
            if let Some(end_data) = end_node.downcast::<CharacterData>() {
                // Step 4.1.
                let clone = end_node.CloneNode(/* deep */ true, can_gc)?;
                // Step 4.2.
                let text = end_data.SubstringData(start_offset, end_offset - start_offset);
                clone
                    .downcast::<CharacterData>()
                    .unwrap()
                    .SetData(text.unwrap());
                // Step 4.3.
                fragment.upcast::<Node>().AppendChild(&clone, can_gc)?;
                // Step 4.4.
                end_data.ReplaceData(start_offset, end_offset - start_offset, DOMString::new())?;
                // Step 4.5.
                return Ok(fragment);
            }
        }

        // Steps 5-12.
        let ContainedChildren {
            first_partially_contained_child,
            last_partially_contained_child,
            contained_children,
        } = self.contained_children()?;

        let (new_node, new_offset) = if start_node.is_inclusive_ancestor_of(&end_node) {
            // Step 13.
            (DomRoot::from_ref(&*start_node), start_offset)
        } else {
            // Step 14.1-2.
            let reference_node = start_node
                .ancestors()
                .take_while(|n| !n.is_inclusive_ancestor_of(&end_node))
                .last()
                .unwrap_or(DomRoot::from_ref(&start_node));
            // Step 14.3.
            (
                reference_node.GetParentNode().unwrap(),
                reference_node.index() + 1,
            )
        };

        if let Some(child) = first_partially_contained_child {
            if let Some(start_data) = child.downcast::<CharacterData>() {
                assert!(child == start_node);
                // Step 15.1.
                let clone = start_node.CloneNode(/* deep */ true, can_gc)?;
                // Step 15.2.
                let text = start_data.SubstringData(start_offset, start_node.len() - start_offset);
                clone
                    .downcast::<CharacterData>()
                    .unwrap()
                    .SetData(text.unwrap());
                // Step 15.3.
                fragment.upcast::<Node>().AppendChild(&clone, can_gc)?;
                // Step 15.4.
                start_data.ReplaceData(
                    start_offset,
                    start_node.len() - start_offset,
                    DOMString::new(),
                )?;
            } else {
                // Step 16.1.
                let clone = child.CloneNode(/* deep */ false, can_gc)?;
                // Step 16.2.
                fragment.upcast::<Node>().AppendChild(&clone, can_gc)?;
                // Step 16.3.
                let subrange = Range::new(
                    &clone.owner_doc(),
                    &start_node,
                    start_offset,
                    &child,
                    child.len(),
                    can_gc,
                );
                // Step 16.4.
                let subfragment = subrange.ExtractContents(can_gc)?;
                // Step 16.5.
                clone.AppendChild(subfragment.upcast(), can_gc)?;
            }
        }

        // Step 17.
        for child in contained_children {
            fragment.upcast::<Node>().AppendChild(&child, can_gc)?;
        }

        if let Some(child) = last_partially_contained_child {
            if let Some(end_data) = child.downcast::<CharacterData>() {
                assert!(child == end_node);
                // Step 18.1.
                let clone = end_node.CloneNode(/* deep */ true, can_gc)?;
                // Step 18.2.
                let text = end_data.SubstringData(0, end_offset);
                clone
                    .downcast::<CharacterData>()
                    .unwrap()
                    .SetData(text.unwrap());
                // Step 18.3.
                fragment.upcast::<Node>().AppendChild(&clone, can_gc)?;
                // Step 18.4.
                end_data.ReplaceData(0, end_offset, DOMString::new())?;
            } else {
                // Step 19.1.
                let clone = child.CloneNode(/* deep */ false, can_gc)?;
                // Step 19.2.
                fragment.upcast::<Node>().AppendChild(&clone, can_gc)?;
                // Step 19.3.
                let subrange =
                    Range::new(&clone.owner_doc(), &child, 0, &end_node, end_offset, can_gc);
                // Step 19.4.
                let subfragment = subrange.ExtractContents(can_gc)?;
                // Step 19.5.
                clone.AppendChild(subfragment.upcast(), can_gc)?;
            }
        }

        // Step 20.
        self.SetStart(&new_node, new_offset)?;
        self.SetEnd(&new_node, new_offset)?;

        // Step 21.
        Ok(fragment)
    }

    /// <https://dom.spec.whatwg.org/#dom-range-detach>
    fn Detach(&self) {
        // This method intentionally left blank.
    }

    /// <https://dom.spec.whatwg.org/#dom-range-insertnode>
    /// <https://dom.spec.whatwg.org/#concept-range-insert>
    fn InsertNode(&self, node: &Node, can_gc: CanGc) -> ErrorResult {
        let start_node = self.start_container();
        let start_offset = self.start_offset();

        // Step 1.
        if &*start_node == node {
            return Err(Error::HierarchyRequest);
        }
        match start_node.type_id() {
            // Handled under step 2.
            NodeTypeId::CharacterData(CharacterDataTypeId::Text(_)) => (),
            NodeTypeId::CharacterData(_) => return Err(Error::HierarchyRequest),
            _ => (),
        }

        // Step 2.
        let (reference_node, parent) = match start_node.type_id() {
            NodeTypeId::CharacterData(CharacterDataTypeId::Text(_)) => {
                // Step 3.
                let parent = match start_node.GetParentNode() {
                    Some(parent) => parent,
                    // Step 1.
                    None => return Err(Error::HierarchyRequest),
                };
                // Step 5.
                (Some(DomRoot::from_ref(&*start_node)), parent)
            },
            _ => {
                // Steps 4-5.
                let child = start_node.ChildNodes(can_gc).Item(start_offset);
                (child, DomRoot::from_ref(&*start_node))
            },
        };

        // Step 6.
        Node::ensure_pre_insertion_validity(node, &parent, reference_node.as_deref())?;

        // Step 7.
        let split_text;
        let reference_node = match start_node.downcast::<Text>() {
            Some(text) => {
                split_text = text.SplitText(start_offset, can_gc)?;
                let new_reference = DomRoot::upcast::<Node>(split_text);
                assert!(new_reference.GetParentNode().as_deref() == Some(&parent));
                Some(new_reference)
            },
            _ => reference_node,
        };

        // Step 8.
        let reference_node = if Some(node) == reference_node.as_deref() {
            node.GetNextSibling()
        } else {
            reference_node
        };

        // Step 9.
        node.remove_self(can_gc);

        // Step 10.
        let new_offset = reference_node
            .as_ref()
            .map_or(parent.len(), |node| node.index());

        // Step 11
        let new_offset = new_offset +
            if let NodeTypeId::DocumentFragment(_) = node.type_id() {
                node.len()
            } else {
                1
            };

        // Step 12.
        Node::pre_insert(node, &parent, reference_node.as_deref(), can_gc)?;

        // Step 13.
        if self.collapsed() {
            self.set_end(&parent, new_offset);
        }

        Ok(())
    }

    /// <https://dom.spec.whatwg.org/#dom-range-deletecontents>
    fn DeleteContents(&self) -> ErrorResult {
        // Step 1.
        if self.collapsed() {
            return Ok(());
        }

        // Step 2.
        let start_node = self.start_container();
        let end_node = self.end_container();
        let start_offset = self.start_offset();
        let end_offset = self.end_offset();

        // Step 3.
        if start_node == end_node {
            if let Some(text) = start_node.downcast::<CharacterData>() {
                if end_offset > start_offset {
                    self.report_change();
                }
                return text.ReplaceData(start_offset, end_offset - start_offset, DOMString::new());
            }
        }

        // Step 4.
        rooted_vec!(let mut contained_children);
        let ancestor = self.CommonAncestorContainer();

        let mut iter = start_node.following_nodes(&ancestor);

        let mut next = iter.next();
        while let Some(child) = next {
            if self.contains(&child) {
                contained_children.push(Dom::from_ref(&*child));
                next = iter.next_skipping_children();
            } else {
                next = iter.next();
            }
        }

        let (new_node, new_offset) = if start_node.is_inclusive_ancestor_of(&end_node) {
            // Step 5.
            (DomRoot::from_ref(&*start_node), start_offset)
        } else {
            // Step 6.
            fn compute_reference(start_node: &Node, end_node: &Node) -> (DomRoot<Node>, u32) {
                let mut reference_node = DomRoot::from_ref(start_node);
                while let Some(parent) = reference_node.GetParentNode() {
                    if parent.is_inclusive_ancestor_of(end_node) {
                        return (parent, reference_node.index() + 1);
                    }
                    reference_node = parent;
                }
                unreachable!()
            }

            compute_reference(&start_node, &end_node)
        };

        // Step 7.
        if let Some(text) = start_node.downcast::<CharacterData>() {
            text.ReplaceData(
                start_offset,
                start_node.len() - start_offset,
                DOMString::new(),
            )
            .unwrap();
        }

        // Step 8.
        for child in &*contained_children {
            child.remove_self(CanGc::note());
        }

        // Step 9.
        if let Some(text) = end_node.downcast::<CharacterData>() {
            text.ReplaceData(0, end_offset, DOMString::new()).unwrap();
        }

        // Step 10.
        self.SetStart(&new_node, new_offset).unwrap();
        self.SetEnd(&new_node, new_offset).unwrap();
        Ok(())
    }

    /// <https://dom.spec.whatwg.org/#dom-range-surroundcontents>
    fn SurroundContents(&self, new_parent: &Node, can_gc: CanGc) -> ErrorResult {
        // Step 1.
        let start = self.start_container();
        let end = self.end_container();

        if start
            .inclusive_ancestors(ShadowIncluding::No)
            .any(|n| !n.is_inclusive_ancestor_of(&end) && !n.is::<Text>()) ||
            end.inclusive_ancestors(ShadowIncluding::No)
                .any(|n| !n.is_inclusive_ancestor_of(&start) && !n.is::<Text>())
        {
            return Err(Error::InvalidState);
        }

        // Step 2.
        match new_parent.type_id() {
            NodeTypeId::Document(_) |
            NodeTypeId::DocumentType |
            NodeTypeId::DocumentFragment(_) => {
                return Err(Error::InvalidNodeType);
            },
            _ => (),
        }

        // Step 3.
        let fragment = self.ExtractContents(can_gc)?;

        // Step 4.
        Node::replace_all(None, new_parent, can_gc);

        // Step 5.
        self.InsertNode(new_parent, can_gc)?;

        // Step 6.
        new_parent.AppendChild(fragment.upcast(), can_gc)?;

        // Step 7.
        self.SelectNode(new_parent)
    }

    /// <https://dom.spec.whatwg.org/#dom-range-stringifier>
    fn Stringifier(&self) -> DOMString {
        let start_node = self.start_container();
        let end_node = self.end_container();

        // Step 1.
        let mut s = DOMString::new();

        if let Some(text_node) = start_node.downcast::<Text>() {
            let char_data = text_node.upcast::<CharacterData>();

            // Step 2.
            if start_node == end_node {
                return char_data
                    .SubstringData(self.start_offset(), self.end_offset() - self.start_offset())
                    .unwrap();
            }

            // Step 3.
            s.push_str(
                &char_data
                    .SubstringData(
                        self.start_offset(),
                        char_data.Length() - self.start_offset(),
                    )
                    .unwrap(),
            );
        }

        // Step 4.
        let ancestor = self.CommonAncestorContainer();
        let iter = start_node
            .following_nodes(&ancestor)
            .filter_map(DomRoot::downcast::<Text>);

        for child in iter {
            if self.contains(child.upcast()) {
                s.push_str(&child.upcast::<CharacterData>().Data());
            }
        }

        // Step 5.
        if let Some(text_node) = end_node.downcast::<Text>() {
            let char_data = text_node.upcast::<CharacterData>();
            s.push_str(&char_data.SubstringData(0, self.end_offset()).unwrap());
        }

        // Step 6.
        s
    }

    /// <https://dvcs.w3.org/hg/innerhtml/raw-file/tip/index.html#extensions-to-the-range-interface>
    fn CreateContextualFragment(
        &self,
        fragment: DOMString,
        can_gc: CanGc,
    ) -> Fallible<DomRoot<DocumentFragment>> {
        // Step 1.
        let node = self.start_container();
        let owner_doc = node.owner_doc();
        let element = match node.type_id() {
            NodeTypeId::Document(_) | NodeTypeId::DocumentFragment(_) => None,
            NodeTypeId::Element(_) => Some(DomRoot::downcast::<Element>(node).unwrap()),
            NodeTypeId::CharacterData(CharacterDataTypeId::Comment) |
            NodeTypeId::CharacterData(CharacterDataTypeId::Text(_)) => node.GetParentElement(),
            NodeTypeId::CharacterData(CharacterDataTypeId::ProcessingInstruction) |
            NodeTypeId::DocumentType => unreachable!(),
            NodeTypeId::Attr => unreachable!(),
        };

        // Step 2.
        let element = Element::fragment_parsing_context(&owner_doc, element.as_deref(), can_gc);

        // Step 3.
        let fragment_node = element.parse_fragment(fragment, can_gc)?;

        // Step 4.
        for node in fragment_node
            .upcast::<Node>()
            .traverse_preorder(ShadowIncluding::No)
        {
            if let Some(script) = node.downcast::<HTMLScriptElement>() {
                script.set_already_started(false);
                script.set_parser_inserted(false);
            }
        }

        // Step 5.
        Ok(fragment_node)
    }

    /// <https://drafts.csswg.org/cssom-view/#dom-range-getclientrects>
    fn GetClientRects(&self, can_gc: CanGc) -> DomRoot<DOMRectList> {
        let start = self.start_container();
        let window = start.owner_window();

        let client_rects = self
            .client_rects(can_gc)
            .map(|rect| {
                DOMRect::new(
                    window.upcast(),
                    rect.origin.x.to_f64_px(),
                    rect.origin.y.to_f64_px(),
                    rect.size.width.to_f64_px(),
                    rect.size.height.to_f64_px(),
                    can_gc,
                )
            })
            .collect();

        DOMRectList::new(&window, client_rects, can_gc)
    }

    /// <https://drafts.csswg.org/cssom-view/#dom-range-getboundingclientrect>
    fn GetBoundingClientRect(&self, can_gc: CanGc) -> DomRoot<DOMRect> {
        let window = self.start_container().owner_window();

        // Step 1. Let list be the result of invoking getClientRects() on the same range this method was invoked on.
        let list = self.client_rects(can_gc);

        // Step 2. If list is empty return a DOMRect object whose x, y, width and height members are zero.
        // Step 3. If all rectangles in list have zero width or height, return the first rectangle in list.
        // Step 4. Otherwise, return a DOMRect object describing the smallest rectangle that includes all
        // of the rectangles in list of which the height or width is not zero.
        let bounding_rect = list.fold(euclid::Rect::zero(), |acc, rect| acc.union(&rect));

        DOMRect::new(
            window.upcast(),
            bounding_rect.origin.x.to_f64_px(),
            bounding_rect.origin.y.to_f64_px(),
            bounding_rect.size.width.to_f64_px(),
            bounding_rect.size.height.to_f64_px(),
            can_gc,
        )
    }
}

pub(crate) struct WeakRangeVec {
    cell: UnsafeCell<WeakRefVec<Range>>,
}

impl Default for WeakRangeVec {
    fn default() -> Self {
        WeakRangeVec {
            cell: UnsafeCell::new(WeakRefVec::new()),
        }
    }
}

#[allow(unsafe_code)]
impl WeakRangeVec {
    /// Whether that vector of ranges is empty.
    pub(crate) fn is_empty(&self) -> bool {
        unsafe { (*self.cell.get()).is_empty() }
    }

    /// Used for steps 2.1-2. when inserting a node.
    /// <https://dom.spec.whatwg.org/#concept-node-insert>
    pub(crate) fn increase_above(&self, node: &Node, offset: u32, delta: u32) {
        self.map_offset_above(node, offset, |offset| offset + delta);
    }

    /// Used for steps 4-5. when removing a node.
    /// <https://dom.spec.whatwg.org/#concept-node-remove>
    pub(crate) fn decrease_above(&self, node: &Node, offset: u32, delta: u32) {
        self.map_offset_above(node, offset, |offset| offset - delta);
    }

    /// Used for steps 2-3. when removing a node.
    /// <https://dom.spec.whatwg.org/#concept-node-remove>
    pub(crate) fn drain_to_parent(&self, context: &UnbindContext, child: &Node) {
        if self.is_empty() {
            return;
        }

        let offset = context.index();
        let parent = context.parent;
        unsafe {
            let ranges = &mut *self.cell.get();

            ranges.update(|entry| {
                let range = entry.root().unwrap();
                if range.start().node() == parent || range.end().node() == parent {
                    entry.remove();
                }
                if range.start().node() == child {
                    range.report_change();
                    range.start().set(context.parent, offset);
                }
                if range.end().node() == child {
                    range.report_change();
                    range.end().set(context.parent, offset);
                }
            });

            (*context.parent.ranges().cell.get()).extend(ranges.drain(..));
        }
    }

    /// Used for steps 7.1-2. when normalizing a node.
    /// <https://dom.spec.whatwg.org/#dom-node-normalize>
    pub(crate) fn drain_to_preceding_text_sibling(&self, node: &Node, sibling: &Node, length: u32) {
        if self.is_empty() {
            return;
        }

        unsafe {
            let ranges = &mut *self.cell.get();

            ranges.update(|entry| {
                let range = entry.root().unwrap();
                if range.start().node() == sibling || range.end().node() == sibling {
                    entry.remove();
                }
                if range.start().node() == node {
                    range.report_change();
                    range.start().set(sibling, range.start_offset() + length);
                }
                if range.end().node() == node {
                    range.report_change();
                    range.end().set(sibling, range.end_offset() + length);
                }
            });

            (*sibling.ranges().cell.get()).extend(ranges.drain(..));
        }
    }

    /// Used for steps 7.3-4. when normalizing a node.
    /// <https://dom.spec.whatwg.org/#dom-node-normalize>
    pub(crate) fn move_to_text_child_at(
        &self,
        node: &Node,
        offset: u32,
        child: &Node,
        new_offset: u32,
    ) {
        unsafe {
            let child_ranges = &mut *child.ranges().cell.get();

            (*self.cell.get()).update(|entry| {
                let range = entry.root().unwrap();

                let node_is_start = range.start().node() == node;
                let node_is_end = range.end().node() == node;

                let move_start = node_is_start && range.start_offset() == offset;
                let move_end = node_is_end && range.end_offset() == offset;

                let remove_from_node =
                    move_start && (move_end || !node_is_end) || move_end && !node_is_start;

                let already_in_child = range.start().node() == child || range.end().node() == child;
                let push_to_child = !already_in_child && (move_start || move_end);

                if remove_from_node {
                    let ref_ = entry.remove();
                    if push_to_child {
                        child_ranges.push(ref_);
                    }
                } else if push_to_child {
                    child_ranges.push(WeakRef::new(&range));
                }

                if move_start {
                    range.report_change();
                    range.start().set(child, new_offset);
                }
                if move_end {
                    range.report_change();
                    range.end().set(child, new_offset);
                }
            });
        }
    }

    /// Used for steps 8-11. when replacing character data.
    /// <https://dom.spec.whatwg.org/#concept-cd-replace>
    pub(crate) fn replace_code_units(
        &self,
        node: &Node,
        offset: u32,
        removed_code_units: u32,
        added_code_units: u32,
    ) {
        self.map_offset_above(node, offset, |range_offset| {
            if range_offset <= offset + removed_code_units {
                offset
            } else {
                range_offset + added_code_units - removed_code_units
            }
        });
    }

    /// Used for steps 7.2-3. when splitting a text node.
    /// <https://dom.spec.whatwg.org/#concept-text-split>
    pub(crate) fn move_to_following_text_sibling_above(
        &self,
        node: &Node,
        offset: u32,
        sibling: &Node,
    ) {
        unsafe {
            let sibling_ranges = &mut *sibling.ranges().cell.get();

            (*self.cell.get()).update(|entry| {
                let range = entry.root().unwrap();
                let start_offset = range.start_offset();
                let end_offset = range.end_offset();

                let node_is_start = range.start().node() == node;
                let node_is_end = range.end().node() == node;

                let move_start = node_is_start && start_offset > offset;
                let move_end = node_is_end && end_offset > offset;

                let remove_from_node =
                    move_start && (move_end || !node_is_end) || move_end && !node_is_start;

                let already_in_sibling =
                    range.start().node() == sibling || range.end().node() == sibling;
                let push_to_sibling = !already_in_sibling && (move_start || move_end);

                if remove_from_node {
                    let ref_ = entry.remove();
                    if push_to_sibling {
                        sibling_ranges.push(ref_);
                    }
                } else if push_to_sibling {
                    sibling_ranges.push(WeakRef::new(&range));
                }

                if move_start {
                    range.report_change();
                    range.start().set(sibling, start_offset - offset);
                }
                if move_end {
                    range.report_change();
                    range.end().set(sibling, end_offset - offset);
                }
            });
        }
    }

    /// Used for steps 7.4-5. when splitting a text node.
    /// <https://dom.spec.whatwg.org/#concept-text-split>
    pub(crate) fn increment_at(&self, node: &Node, offset: u32) {
        unsafe {
            (*self.cell.get()).update(|entry| {
                let range = entry.root().unwrap();
                if range.start().node() == node && offset == range.start_offset() {
                    range.report_change();
                    range.start().set_offset(offset + 1);
                }
                if range.end().node() == node && offset == range.end_offset() {
                    range.report_change();
                    range.end().set_offset(offset + 1);
                }
            });
        }
    }

    fn map_offset_above<F: FnMut(u32) -> u32>(&self, node: &Node, offset: u32, mut f: F) {
        unsafe {
            (*self.cell.get()).update(|entry| {
                let range = entry.root().unwrap();
                let start_offset = range.start_offset();
                if range.start().node() == node && start_offset > offset {
                    range.report_change();
                    range.start().set_offset(f(start_offset));
                }
                let end_offset = range.end_offset();
                if range.end().node() == node && end_offset > offset {
                    range.report_change();
                    range.end().set_offset(f(end_offset));
                }
            });
        }
    }

    pub(crate) fn push(&self, ref_: WeakRef<Range>) {
        unsafe {
            (*self.cell.get()).push(ref_);
        }
    }

    fn remove(&self, range: &Range) -> WeakRef<Range> {
        unsafe {
            let ranges = &mut *self.cell.get();
            let position = ranges.iter().position(|ref_| ref_ == range).unwrap();
            ranges.swap_remove(position)
        }
    }
}

#[allow(unsafe_code)]
impl MallocSizeOf for WeakRangeVec {
    fn size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        unsafe { (*self.cell.get()).size_of(ops) }
    }
}

#[allow(unsafe_code)]
unsafe impl JSTraceable for WeakRangeVec {
    unsafe fn trace(&self, _: *mut JSTracer) {
        (*self.cell.get()).retain_alive()
    }
}
