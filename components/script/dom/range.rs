/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::CharacterDataBinding::CharacterDataMethods;
use dom::bindings::codegen::Bindings::NodeBinding::NodeConstants;
use dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use dom::bindings::codegen::Bindings::NodeListBinding::NodeListMethods;
use dom::bindings::codegen::Bindings::RangeBinding::{self, RangeConstants};
use dom::bindings::codegen::Bindings::RangeBinding::RangeMethods;
use dom::bindings::codegen::Bindings::TextBinding::TextMethods;
use dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use dom::bindings::codegen::InheritTypes::{CharacterDataCast, NodeCast, TextCast};
use dom::bindings::error::{Error, ErrorResult, Fallible};
use dom::bindings::error::Error::HierarchyRequest;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JS, Root, RootedReference};
use dom::bindings::utils::{Reflector, reflect_dom_object};
use dom::characterdata::CharacterDataTypeId;
use dom::document::{Document, DocumentHelpers};
use dom::documentfragment::DocumentFragment;
use dom::node::{Node, NodeHelpers, NodeTypeId};

use std::cell::RefCell;
use std::cmp::{Ord, Ordering, PartialEq, PartialOrd};
use std::rc::Rc;

#[dom_struct]
#[derive(HeapSizeOf)]
pub struct Range {
    reflector_: Reflector,
    inner: Rc<RefCell<RangeInner>>,
}

impl Range {
    fn new_inherited(start_container: &Node, start_offset: u32,
                     end_container: &Node, end_offset: u32) -> Range {
        Range {
            reflector_: Reflector::new(),
            inner: Rc::new(RefCell::new(RangeInner::new(
                BoundaryPoint::new(start_container, start_offset),
                BoundaryPoint::new(end_container, end_offset)))),
        }
    }

    pub fn new_with_doc(document: &Document) -> Root<Range> {
        let root = NodeCast::from_ref(document);
        Range::new(document, root, 0, root, 0)
    }

    pub fn new(document: &Document,
               start_container: &Node, start_offset: u32,
               end_container: &Node, end_offset: u32)
               -> Root<Range> {
        let window = document.window();
        reflect_dom_object(box Range::new_inherited(start_container, start_offset,
                                                    end_container, end_offset),
                           GlobalRef::Window(window.r()),
                           RangeBinding::Wrap)
    }

    // https://dom.spec.whatwg.org/#dom-range
    pub fn Constructor(global: GlobalRef) -> Fallible<Root<Range>> {
        let document = global.as_window().Document();
        Ok(Range::new_with_doc(document.r()))
    }

    // https://dom.spec.whatwg.org/#contained
    fn contains(&self, node: &Node) -> bool {
        let inner = self.inner.borrow();
        let start = &inner.start;
        let end = &inner.end;
        match (bp_position(node, 0, start.node().r(), start.offset()),
               bp_position(node, node.len(), end.node().r(), end.offset())) {
            (Some(Ordering::Greater), Some(Ordering::Less)) => true,
            _ => false
        }
    }

    // https://dom.spec.whatwg.org/#partially-contained
    fn partially_contains(&self, node: &Node) -> bool {
        let inner = self.inner.borrow();
        inner.start.node().inclusive_ancestors().any(|n| n.r() == node) !=
            inner.end.node().inclusive_ancestors().any(|n| n.r() == node)
    }

    // https://dom.spec.whatwg.org/#concept-range-clone
    fn contained_children(&self) -> Fallible<(Option<Root<Node>>,
                                              Option<Root<Node>>,
                                              Vec<Root<Node>>)> {
        let start_node = self.StartContainer();
        let end_node = self.EndContainer();
        // Steps 5-6.
        let common_ancestor = self.CommonAncestorContainer();

        let first_contained_child =
            if start_node.is_inclusive_ancestor_of(end_node.r()) {
                // Step 7.
                None
            } else {
                // Step 8.
                common_ancestor.children()
                               .find(|node| Range::partially_contains(self, node))
            };

        let last_contained_child =
            if end_node.is_inclusive_ancestor_of(start_node.r()) {
                // Step 9.
                None
            } else {
                // Step 10.
                common_ancestor.rev_children()
                               .find(|node| Range::partially_contains(self, node))
            };

        // Step 11.
        let contained_children : Vec<Root<Node>> =
            common_ancestor.children().filter(|n| Range::contains(self, n)).collect();

        // Step 12.
        if contained_children.iter().any(|n| n.is_doctype()) {
            return Err(HierarchyRequest);
        }

        return Ok((first_contained_child, last_contained_child, contained_children));
    }
}

pub trait RangeHelpers<'a> {
    fn inner(self) -> &'a Rc<RefCell<RangeInner>>;
}

impl<'a> RangeHelpers<'a> for &'a Range {
    fn inner(self) -> &'a Rc<RefCell<RangeInner>> {
        &self.inner
    }
}

impl<'a> RangeMethods for &'a Range {
    // https://dom.spec.whatwg.org/#dom-range-startcontainer
    fn StartContainer(self) -> Root<Node> {
        self.inner().borrow().start.node()
    }

    // https://dom.spec.whatwg.org/#dom-range-startoffset
    fn StartOffset(self) -> u32 {
        self.inner().borrow().start.offset
    }

    // https://dom.spec.whatwg.org/#dom-range-endcontainer
    fn EndContainer(self) -> Root<Node> {
        self.inner().borrow().end.node()
    }

    // https://dom.spec.whatwg.org/#dom-range-endoffset
    fn EndOffset(self) -> u32 {
        self.inner().borrow().end.offset
    }

    // https://dom.spec.whatwg.org/#dom-range-collapsed
    fn Collapsed(self) -> bool {
        let inner = self.inner().borrow();
        inner.start == inner.end
    }

    // https://dom.spec.whatwg.org/#dom-range-commonancestorcontainer
    fn CommonAncestorContainer(self) -> Root<Node> {
        self.inner().borrow().common_ancestor_container()
    }

    // https://dom.spec.whatwg.org/#dom-range-setstartnode-offset
    fn SetStart(self, node: &Node, offset: u32) -> ErrorResult {
        if node.is_doctype() {
            // Step 1.
            Err(Error::InvalidNodeType)
        } else if offset > node.len() {
            // Step 2.
            Err(Error::IndexSize)
        } else {
            // Step 3-4.
            self.inner().borrow_mut().set_start(node, offset);
            Ok(())
        }
    }

    // https://dom.spec.whatwg.org/#dom-range-setendnode-offset
    fn SetEnd(self, node: &Node, offset: u32) -> ErrorResult {
        if node.is_doctype() {
            // Step 1.
            Err(Error::InvalidNodeType)
        } else if offset > node.len() {
            // Step 2.
            Err(Error::IndexSize)
        } else {
            // Step 3-4.
            self.inner().borrow_mut().set_end(node, offset);
            Ok(())
        }
    }

    // https://dom.spec.whatwg.org/#dom-range-setstartbeforenode
    fn SetStartBefore(self, node: &Node) -> ErrorResult {
        let parent = try!(node.GetParentNode().ok_or(Error::InvalidNodeType));
        self.SetStart(parent.r(), node.index())
    }

    // https://dom.spec.whatwg.org/#dom-range-setstartafternode
    fn SetStartAfter(self, node: &Node) -> ErrorResult {
        let parent = try!(node.GetParentNode().ok_or(Error::InvalidNodeType));
        self.SetStart(parent.r(), node.index() + 1)
    }

    // https://dom.spec.whatwg.org/#dom-range-setendbeforenode
    fn SetEndBefore(self, node: &Node) -> ErrorResult {
        let parent = try!(node.GetParentNode().ok_or(Error::InvalidNodeType));
        self.SetEnd(parent.r(), node.index())
    }

    // https://dom.spec.whatwg.org/#dom-range-setendafternode
    fn SetEndAfter(self, node: &Node) -> ErrorResult {
        let parent = try!(node.GetParentNode().ok_or(Error::InvalidNodeType));
        self.SetEnd(parent.r(), node.index() + 1)
    }

    // https://dom.spec.whatwg.org/#dom-range-collapsetostart
    fn Collapse(self, to_start: bool) {
        self.inner().borrow_mut().collapse(to_start);
    }

    // https://dom.spec.whatwg.org/#dom-range-selectnodenode
    fn SelectNode(self, node: &Node) -> ErrorResult {
        self.inner().borrow_mut().select_node(node)
    }

    // https://dom.spec.whatwg.org/#dom-range-selectnodecontentsnode
    fn SelectNodeContents(self, node: &Node) -> ErrorResult {
        self.inner().borrow_mut().select_node_contents(node)
    }

    // https://dom.spec.whatwg.org/#dom-range-compareboundarypointshow-sourcerange
    fn CompareBoundaryPoints(self, how: u16, source_range: &Range)
                             -> Fallible<i16> {
        if how > RangeConstants::END_TO_START {
            // Step 1.
            return Err(Error::NotSupported);
        }
        let this_inner = self.inner().borrow();
        let other_inner = source_range.inner().borrow();
        let this_start_node = this_inner.start.node();
        let other_start_node = other_inner.start.node();
        let this_root = this_start_node.r().inclusive_ancestors().last().unwrap();
        let other_root = other_start_node.r().inclusive_ancestors().last().unwrap();
        if this_root != other_root {
            // Step 2.
            return Err(Error::WrongDocument);
        }
        // Step 3.
        let (this_point, other_point) = match how {
            RangeConstants::START_TO_START => {
                (&this_inner.start, &other_inner.start)
            },
            RangeConstants::START_TO_END => {
                (&this_inner.end, &other_inner.start)
            },
            RangeConstants::END_TO_END => {
                (&this_inner.end, &other_inner.end)
            },
            RangeConstants::END_TO_START => {
                (&this_inner.start, &other_inner.end)
            },
            _ => unreachable!(),
        };
        // step 4.
        match this_point.partial_cmp(other_point).unwrap() {
            Ordering::Less => Ok(-1),
            Ordering::Equal => Ok(0),
            Ordering::Greater => Ok(1),
        }
    }

    // https://dom.spec.whatwg.org/#dom-range-clonerange
    fn CloneRange(self) -> Root<Range> {
        let inner = self.inner().borrow();
        let start = &inner.start;
        let end = &inner.end;
        let start_node = start.node();
        let owner_doc = NodeCast::from_ref(start_node.r()).owner_doc();
        Range::new(owner_doc.r(), start_node.r(), start.offset,
                   end.node().r(), end.offset)
    }

    // https://dom.spec.whatwg.org/#dom-range-ispointinrangenode-offset
    fn IsPointInRange(self, node: &Node, offset: u32) -> Fallible<bool> {
        match self.inner().borrow().compare_point(node, offset) {
            Ok(Ordering::Less) => Ok(false),
            Ok(Ordering::Equal) => Ok(true),
            Ok(Ordering::Greater) => Ok(false),
            Err(Error::WrongDocument) => {
                // Step 2.
                Ok(false)
            }
            Err(error) => Err(error),
        }
    }

    // https://dom.spec.whatwg.org/#dom-range-comparepointnode-offset
    fn ComparePoint(self, node: &Node, offset: u32) -> Fallible<i16> {
        self.inner().borrow().compare_point(node, offset).map(|order| {
            match order {
                Ordering::Less => -1,
                Ordering::Equal => 0,
                Ordering::Greater => 1,
            }
        })
    }

    // https://dom.spec.whatwg.org/#dom-range-intersectsnode
    fn IntersectsNode(self, node: &Node) -> bool {
        let inner = self.inner().borrow();
        let start = &inner.start;
        let start_node = start.node();
        let start_offset = start.offset;
        let start_node_root = start_node.r().inclusive_ancestors().last().unwrap();
        let node_root = node.inclusive_ancestors().last().unwrap();
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
        let end = &inner.end;
        let end_node = end.node();
        let end_offset = end.offset;
        match (bp_position(parent.r(), offset + 1, start_node.r(), start_offset).unwrap(),
               bp_position(parent.r(), offset, end_node.r(), end_offset).unwrap()) {
            (Ordering::Greater, Ordering::Less) => {
                // Step 5.
                true
            },
            _ => {
                // Step 6.
                false
            }
        }
    }

    // https://dom.spec.whatwg.org/#dom-range-clonecontents
    // https://dom.spec.whatwg.org/#concept-range-clone
    fn CloneContents(self) -> Fallible<Root<DocumentFragment>> {
        let inner = self.inner.borrow();
        let start = &inner.start;
        let end = &inner.end;

        // Step 3.
        let start_node = start.node();
        let start_offset = start.offset();
        let end_node = end.node();
        let end_offset = end.offset();

        // Step 1.
        let fragment = DocumentFragment::new(start_node.owner_doc().r());

        // Step 2.
        if start == end {
            return Ok(fragment);
        }

        if end_node == start_node {
            if let Some(text) = CharacterDataCast::to_ref(start_node.r()) {
                // Step 4.1.
                let clone = start_node.CloneNode(true);
                // Step 4.2.
                let text = text.SubstringData(start_offset, end_offset - start_offset);
                CharacterDataCast::to_ref(clone.r()).unwrap().SetData(text.unwrap());
                // Step 4.3.
                try!(NodeCast::from_ref(fragment.r()).AppendChild(clone.r()));
                // Step 4.4
                return Ok(fragment);
            }
        }

        // Steps 5-12.
        let (first_contained_child, last_contained_child, contained_children) =
            try!(self.contained_children());

        if let Some(child) = first_contained_child {
            // Step 13.
            if let Some(text) = CharacterDataCast::to_ref(child.r()) {
                assert!(child == start_node);
                // Step 13.1.
                let clone = start_node.CloneNode(true); // CharacterData has no children.
                // Step 13.2
                let text = text.SubstringData(start_offset, start_node.len() - start_offset);
                CharacterDataCast::to_ref(clone.r()).unwrap().SetData(text.unwrap());
                // Step 13.3.
                try!(NodeCast::from_ref(fragment.r()).AppendChild(clone.r()));
            } else {
                // Step 14.1.
                let clone = child.CloneNode(false);
                // Step 14.2.
                try!(NodeCast::from_ref(fragment.r()).AppendChild(clone.r()));
                // Step 14.3.
                let subrange = Range::new(clone.owner_doc().r(),
                                          start_node.r(),
                                          start_offset,
                                          child.r(),
                                          child.len());
                // Step 14.4.
                let subfragment = try!(subrange.CloneContents());
                // Step 14.5.
                try!(clone.AppendChild(NodeCast::from_ref(subfragment.r())));
            }
        }

        // Step 15.
        for child in contained_children {
            // Step 15.1.
            let clone = child.CloneNode(true);
            // Step 15.2.
            try!(NodeCast::from_ref(fragment.r()).AppendChild(clone.r()));
        }

        if let Some(child) = last_contained_child {
            // Step 16.
            if let Some(text) = CharacterDataCast::to_ref(child.r()) {
                assert!(child == end_node);
                // Step 16.1.
                let clone = end_node.CloneNode(true); // CharacterData has no children.
                // Step 16.2.
                let text = text.SubstringData(0, end_offset);
                CharacterDataCast::to_ref(clone.r()).unwrap().SetData(text.unwrap());
                // Step 16.3.
                try!(NodeCast::from_ref(fragment.r()).AppendChild(clone.r()));
            } else {
                // Step 17.1.
                let clone = child.CloneNode(false);
                // Step 17.2.
                try!(NodeCast::from_ref(fragment.r()).AppendChild(clone.r()));
                // Step 17.3.
                let subrange = Range::new(clone.owner_doc().r(),
                                          child.r(),
                                          0,
                                          end_node.r(),
                                          end_offset);
                // Step 17.4.
                let subfragment = try!(subrange.CloneContents());
                // Step 17.5.
                try!(clone.AppendChild(NodeCast::from_ref(subfragment.r())));
            }
        }

        // Step 18.
        Ok(fragment)
    }

    // https://dom.spec.whatwg.org/#dom-range-extractcontents
    // https://dom.spec.whatwg.org/#concept-range-extract
    fn ExtractContents(self) -> Fallible<Root<DocumentFragment>> {

        // Step 3.
        let (start_node, start_offset, end_node, end_offset) = {
            let inner = self.inner.borrow();
            let start = &inner.start;
            let end = &inner.end;
            (start.node(), start.offset(), end.node(), end.offset())
        };

        // Step 1.
        let fragment = DocumentFragment::new(start_node.owner_doc().r());

        // Step 2.
        if self.Collapsed() {
            return Ok(fragment);
        }

        if end_node == start_node {
            if let Some(end_data) = CharacterDataCast::to_ref(end_node.r()) {
                // Step 4.1.
                let clone = end_node.CloneNode(true);
                // Step 4.2.
                let text = end_data.SubstringData(start_offset, end_offset - start_offset);
                CharacterDataCast::to_ref(clone.r()).unwrap().SetData(text.unwrap());
                // Step 4.3.
                try!(NodeCast::from_ref(fragment.r()).AppendChild(clone.r()));
                // Step 4.4.
                try!(end_data.ReplaceData(start_offset,
                                          end_offset - start_offset,
                                          "".to_owned()));
                // Step 4.5.
                return Ok(fragment);
            }
        }

        // Steps 5-12.
        let (first_contained_child, last_contained_child, contained_children) =
            try!(self.contained_children());

        let (new_node, new_offset) = if start_node.is_inclusive_ancestor_of(end_node.r()) {
            // Step 13.
            (Root::from_ref(start_node.r()), start_offset)
        } else {
            // Step 14.1-2.
            let reference_node = start_node.ancestors()
                                           .find(|n| n.is_inclusive_ancestor_of(end_node.r()))
                                           .unwrap();
            // Step 14.3.
            (reference_node.GetParentNode().unwrap(), reference_node.index() + 1)
        };

        if let Some(child) = first_contained_child {
            if let Some(start_data) = CharacterDataCast::to_ref(child.r()) {
                assert!(child == start_node);
                // Step 15.1.
                let clone = start_node.CloneNode(true);
                // Step 15.2.
                let text = start_data.SubstringData(start_offset,
                                                    start_node.len() - start_offset);
                CharacterDataCast::to_ref(clone.r()).unwrap().SetData(text.unwrap());
                // Step 15.3.
                try!(NodeCast::from_ref(fragment.r()).AppendChild(clone.r()));
                // Step 15.4.
                try!(start_data.ReplaceData(start_offset,
                                            start_node.len() - start_offset,
                                            "".to_owned()));
            } else {
                // Step 16.1.
                let clone = child.CloneNode(false);
                // Step 16.2.
                try!(NodeCast::from_ref(fragment.r()).AppendChild(clone.r()));
                // Step 16.3.
                let subrange = Range::new(clone.owner_doc().r(),
                                          start_node.r(),
                                          start_offset,
                                          child.r(),
                                          child.len());
                // Step 16.4.
                let subfragment = try!(subrange.ExtractContents());
                // Step 16.5.
                try!(clone.AppendChild(NodeCast::from_ref(subfragment.r())));
            }
        }

        // Step 17.
        for child in contained_children {
            try!(NodeCast::from_ref(fragment.r()).AppendChild(child.r()));
        }

        if let Some(child) = last_contained_child {
            if let Some(end_data) = CharacterDataCast::to_ref(child.r()) {
                assert!(child == end_node);
                // Step 18.1.
                let clone = end_node.CloneNode(true);
                // Step 18.2.
                let text = end_data.SubstringData(0, end_offset);
                CharacterDataCast::to_ref(clone.r()).unwrap().SetData(text.unwrap());
                // Step 18.3.
                try!(NodeCast::from_ref(fragment.r()).AppendChild(clone.r()));
                // Step 18.4.
                try!(end_data.ReplaceData(0, end_offset, "".to_owned()));
            } else {
                // Step 19.1.
                let clone = child.CloneNode(false);
                // Step 19.2.
                try!(NodeCast::from_ref(fragment.r()).AppendChild(clone.r()));
                // Step 19.3.
                let subrange = Range::new(clone.owner_doc().r(),
                                          child.r(),
                                          0,
                                          end_node.r(),
                                          end_offset);
                // Step 19.4.
                let subfragment = try!(subrange.ExtractContents());
                // Step 19.5.
                try!(clone.AppendChild(NodeCast::from_ref(subfragment.r())));
            }
        }

        // Step 20.
        try!(self.SetStart(new_node.r(), new_offset));
        try!(self.SetEnd(new_node.r(), new_offset));

        // Step 21.
        Ok(fragment)
    }

    // https://dom.spec.whatwg.org/#dom-range-detach
    fn Detach(self) {
        // This method intentionally left blank.
    }

    // https://dom.spec.whatwg.org/#dom-range-insertnode
    // https://dom.spec.whatwg.org/#concept-range-insert
    fn InsertNode(self, node: &Node) -> ErrorResult {
        let (start_node, start_offset) = {
            let inner = self.inner().borrow();
            let start = &inner.start;
            (start.node(), start.offset())
        };

        // Step 1.
        match start_node.type_id() {
            // Handled under step 2.
            NodeTypeId::CharacterData(CharacterDataTypeId::Text) => (),
            NodeTypeId::CharacterData(_) => return Err(HierarchyRequest),
            _ => ()
        }

        // Step 2.
        let (reference_node, parent) =
            if start_node.type_id() == NodeTypeId::CharacterData(CharacterDataTypeId::Text) {
                // Step 3.
                let parent = match start_node.GetParentNode() {
                    Some(parent) => parent,
                    // Step 1.
                    None => return Err(HierarchyRequest)
                };
                // Step 5.
                (Some(Root::from_ref(start_node.r())), parent)
            } else {
                // Steps 4-5.
                let child = start_node.ChildNodes().Item(start_offset);
                (child, Root::from_ref(start_node.r()))
            };

        // Step 6.
        try!(Node::ensure_pre_insertion_validity(node,
                                                 parent.r(),
                                                 reference_node.r()));

        // Step 7.
        let split_text;
        let reference_node =
            match TextCast::to_ref(start_node.r()) {
                Some(text) => {
                    split_text = try!(text.SplitText(start_offset));
                    let new_reference = NodeCast::from_root(split_text);
                    assert!(new_reference.GetParentNode().r() == Some(parent.r()));
                    Some(new_reference)
                },
                _ => reference_node
            };

        // Step 8.
        let reference_node = if Some(node) == reference_node.r() {
            node.GetNextSibling()
        } else {
            reference_node
        };

        // Step 9.
        node.remove_self();

        // Step 10.
        let new_offset =
            reference_node.r().map_or(parent.len(), |node| node.index());

        // Step 11
        let new_offset = new_offset + if node.type_id() == NodeTypeId::DocumentFragment {
            node.len()
        } else {
            1
        };

        // Step 12.
        try!(Node::pre_insert(node, parent.r(), reference_node.r()));

        // Step 13.
        if self.Collapsed() {
            self.inner().borrow_mut().set_end(parent.r(), new_offset);
        }

        Ok(())
    }
}

#[derive(JSTraceable)]
#[must_root]
#[privatize]
#[derive(HeapSizeOf)]
pub struct RangeInner {
    start: BoundaryPoint,
    end: BoundaryPoint,
}

impl RangeInner {
    fn new(start: BoundaryPoint, end: BoundaryPoint) -> RangeInner {
        RangeInner { start: start, end: end }
    }

    // https://dom.spec.whatwg.org/#dom-range-commonancestorcontainer
    fn common_ancestor_container(&self) -> Root<Node> {
        let start_container = self.start.node();
        let end_container = self.end.node();
        // Step 1.
        for container in start_container.r().inclusive_ancestors() {
            // Step 2.
            if container.r().is_inclusive_ancestor_of(end_container.r()) {
                // Step 3.
                return container;
            }
        }
        unreachable!();
    }

    // https://dom.spec.whatwg.org/#concept-range-bp-set
    pub fn set_start(&mut self, bp_node: &Node, bp_offset: u32) {
        // Steps 1-3 handled in Range caller.
        let end_node = self.end.node();
        let end_offset = self.end.offset;
        match bp_position(bp_node, bp_offset, end_node.r(), end_offset) {
            None | Some(Ordering::Greater) => {
                // Step 4-1.
                self.end.set(bp_node, bp_offset);
            },
            _ => {},
        };
        // Step 4-2.
        self.start.set(bp_node, bp_offset);
    }

    // https://dom.spec.whatwg.org/#concept-range-bp-set
    pub fn set_end(&mut self, bp_node: &Node, bp_offset: u32) {
        // Steps 1-3 handled in Range caller.
        let start_node = self.start.node();
        let start_offset = self.start.offset;
        match bp_position(bp_node, bp_offset, start_node.r(), start_offset) {
            None | Some(Ordering::Less) => {
                // Step 4-1.
                self.start.set(bp_node, bp_offset);
            },
            _ => {},
        };
        // Step 4-2.
        self.end.set(bp_node, bp_offset);
    }

    // https://dom.spec.whatwg.org/#dom-range-collapsetostart
    fn collapse(&mut self, to_start: bool) {
        if to_start {
            let start_node = self.start.node();
            self.end.set(start_node.r(), self.start.offset);
        } else {
            let end_node = self.end.node();
            self.start.set(end_node.r(), self.end.offset);
        }
    }

    // https://dom.spec.whatwg.org/#dom-range-selectnodenode
    fn select_node(&mut self, node: &Node) -> ErrorResult {
        // Steps 1, 2.
        let parent = try!(node.GetParentNode().ok_or(Error::InvalidNodeType));
        // Step 3.
        let index = node.index();
        // Step 4.
        self.start.set(parent.r(), index);
        // Step 5.
        self.end.set(parent.r(), index + 1);
        Ok(())
    }

    // https://dom.spec.whatwg.org/#dom-range-selectnodecontentsnode
    fn select_node_contents(&mut self, node: &Node) -> ErrorResult {
        if node.is_doctype() {
            // Step 1.
            return Err(Error::InvalidNodeType);
        }
        // Step 2.
        let length = node.len();
        // Step 3.
        self.start.set(node, 0);
        // Step 4.
        self.end.set(node, length);
        Ok(())
    }

    // https://dom.spec.whatwg.org/#dom-range-comparepointnode-offset
    fn compare_point(&self, node: &Node, offset: u32) -> Fallible<Ordering> {
        let start = &self.start;
        let start_node = start.node();
        let start_offset = start.offset;
        let start_node_root = start_node.r().inclusive_ancestors().last().unwrap();
        let node_root = node.inclusive_ancestors().last().unwrap();
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
        if let Ordering::Less = bp_position(node, offset, start_node.r(), start_offset).unwrap() {
            // Step 4.
            return Ok(Ordering::Less);
        }
        let end = &self.end;
        let end_node = end.node();
        let end_offset = end.offset;
        if let Ordering::Greater = bp_position(node, offset, end_node.r(), end_offset).unwrap() {
            // Step 5.
            return Ok(Ordering::Greater);
        }
        // Step 6.
        Ok(Ordering::Equal)
    }
}

#[derive(JSTraceable)]
#[must_root]
#[privatize]
#[derive(HeapSizeOf)]
pub struct BoundaryPoint {
    node: JS<Node>,
    offset: u32,
}

impl BoundaryPoint {
    fn new(node: &Node, offset: u32) -> BoundaryPoint {
        debug_assert!(!node.is_doctype());
        debug_assert!(offset <= node.len());
        BoundaryPoint {
            node: JS::from_ref(node),
            offset: offset,
        }
    }

    pub fn node(&self) -> Root<Node> {
        self.node.root()
    }

    pub fn offset(&self) -> u32 {
        self.offset
    }

    fn set(&mut self, node: &Node, offset: u32) {
        debug_assert!(!node.is_doctype());
        debug_assert!(offset <= node.len());
        self.node = JS::from_ref(node);
        self.offset = offset;
    }
}

#[allow(unrooted_must_root)]
impl PartialOrd for BoundaryPoint {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        bp_position(self.node().r(), self.offset,
                    other.node().r(), other.offset)
    }
}

#[allow(unrooted_must_root)]
impl PartialEq for BoundaryPoint {
    fn eq(&self, other: &Self) -> bool {
        self.node() == other.node() &&
        self.offset == other.offset
    }
}

// https://dom.spec.whatwg.org/#concept-range-bp-position
fn bp_position(a_node: &Node, a_offset: u32,
               b_node: &Node, b_offset: u32)
               -> Option<Ordering> {
    if a_node as *const Node == b_node as *const Node {
        // Step 1.
        return Some(a_offset.cmp(&b_offset));
    }
    let position = b_node.CompareDocumentPosition(a_node);
    if position & NodeConstants::DOCUMENT_POSITION_DISCONNECTED != 0 {
        // No order is defined for nodes not in the same tree.
        None
    } else if position & NodeConstants::DOCUMENT_POSITION_FOLLOWING != 0 {
        // Step 2.
        match bp_position(b_node, b_offset, a_node, a_offset).unwrap() {
            Ordering::Less => Some(Ordering::Greater),
            Ordering::Greater => Some(Ordering::Less),
            Ordering::Equal => unreachable!(),
        }
    } else if position & NodeConstants::DOCUMENT_POSITION_CONTAINS != 0 {
        // Step 3-1, 3-2.
        let mut b_ancestors = b_node.inclusive_ancestors();
        let ref child = b_ancestors.find(|child| {
            child.r().GetParentNode().unwrap().r() == a_node
        }).unwrap();
        // Step 3-3.
        if child.r().index() < a_offset {
            Some(Ordering::Greater)
        } else {
            // Step 4.
            Some(Ordering::Less)
        }
    } else {
        // Step 4.
        Some(Ordering::Less)
    }
}
