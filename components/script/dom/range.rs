/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::NodeBinding::NodeConstants;
use dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use dom::bindings::codegen::Bindings::RangeBinding::{self, RangeConstants};
use dom::bindings::codegen::Bindings::RangeBinding::RangeMethods;
use dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use dom::bindings::codegen::InheritTypes::NodeCast;
use dom::bindings::error::{Error, ErrorResult, Fallible};
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JS, Root};
use dom::bindings::utils::{Reflector, reflect_dom_object};
use dom::document::{Document, DocumentHelpers};
use dom::node::{Node, NodeHelpers};

use std::cell::RefCell;
use std::cmp::{Ord, Ordering, PartialEq, PartialOrd};
use std::rc::Rc;

#[dom_struct]
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
    // http://dom.spec.whatwg.org/#dom-range-startcontainer
    fn StartContainer(self) -> Root<Node> {
        self.inner().borrow().start.node()
    }

    /// http://dom.spec.whatwg.org/#dom-range-startoffset
    fn StartOffset(self) -> u32 {
        self.inner().borrow().start.offset
    }

    /// http://dom.spec.whatwg.org/#dom-range-endcontainer
    fn EndContainer(self) -> Root<Node> {
        self.inner().borrow().end.node()
    }

    /// http://dom.spec.whatwg.org/#dom-range-endoffset
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

    // https://dom.spec.whatwg.org/#dom-range-intersectsnodenode
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

    // http://dom.spec.whatwg.org/#dom-range-detach
    fn Detach(self) {
        // This method intentionally left blank.
    }
}

#[derive(JSTraceable)]
#[must_root]
#[privatize]
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
