/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::{LazyCell, RefCell};
use std::cmp::{Ordering, PartialOrd};
use std::iter;
use std::rc::Rc;

use app_units::Au;
use dom_struct::dom_struct;
use euclid::Rect;
use js::context::{JSContext, NoGC};
use js::jsapi::JSTracer;
use js::rust::HandleObject;
use script_bindings::cell::DomRefCell;
use script_bindings::dom::UnrootedDom;
use script_bindings::reflector::reflect_weak_referenceable_dom_object_with_proto;
use smallvec::SmallVec;
use style_traits::CSSPixel;

use crate::dom::abstractrange::{AbstractRange, BoundaryPoint, bp_position};
use crate::dom::bindings::codegen::Bindings::AbstractRangeBinding::AbstractRangeMethods;
use crate::dom::bindings::codegen::Bindings::CharacterDataBinding::CharacterDataMethods;
use crate::dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use crate::dom::bindings::codegen::Bindings::NodeListBinding::NodeListMethods;
use crate::dom::bindings::codegen::Bindings::RangeBinding::{RangeConstants, RangeMethods};
use crate::dom::bindings::codegen::Bindings::TextBinding::TextMethods;
use crate::dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use crate::dom::bindings::codegen::UnionTypes::TrustedHTMLOrString;
use crate::dom::bindings::error::{Error, ErrorResult, Fallible};
use crate::dom::bindings::inheritance::{Castable, CharacterDataTypeId, NodeTypeId};
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
use crate::dom::html::htmlscriptelement::HTMLScriptElement;
use crate::dom::iterators::ShadowIncluding;
use crate::dom::node::{Node, NodeTraits};
use crate::dom::selection::Selection;
use crate::dom::text::Text;
use crate::dom::trustedtypes::trustedhtml::TrustedHTML;
use crate::dom::window::Window;

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

pub(crate) struct ContainedChildren {
    pub(crate) first_partially_contained_child: Option<DomRoot<Node>>,
    pub(crate) last_partially_contained_child: Option<DomRoot<Node>>,
    pub(crate) contained_children: Vec<DomRoot<Node>>,
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
        cx: &mut JSContext,
        document: &Document,
        proto: Option<HandleObject>,
    ) -> DomRoot<Range> {
        let root = document.upcast();
        Range::new_with_proto(cx, document, proto, root, 0, root, 0)
    }

    pub(crate) fn new(
        cx: &mut JSContext,
        document: &Document,
        start_container: &Node,
        start_offset: u32,
        end_container: &Node,
        end_offset: u32,
    ) -> DomRoot<Range> {
        Self::new_with_proto(
            cx,
            document,
            None,
            start_container,
            start_offset,
            end_container,
            end_offset,
        )
    }

    fn new_with_proto(
        cx: &mut JSContext,
        document: &Document,
        proto: Option<HandleObject>,
        start_container: &Node,
        start_offset: u32,
        end_container: &Node,
        end_offset: u32,
    ) -> DomRoot<Range> {
        let range = reflect_weak_referenceable_dom_object_with_proto(
            cx,
            Rc::new(Range::new_inherited(
                start_container,
                start_offset,
                end_container,
                end_offset,
            )),
            document.window(),
            proto,
        );
        start_container
            .ensure_weak_ranges()
            .push(WeakRef::new(&range));
        if start_container != end_container {
            end_container
                .ensure_weak_ranges()
                .push(WeakRef::new(&range));
        }
        range
    }

    /// <https://dom.spec.whatwg.org/#concept-range-root>
    ///
    /// > The root of a live range is the root of its start node.
    pub(crate) fn root(&self) -> DomRoot<Node> {
        self.start_container().GetRootNode(&Default::default())
    }

    /// <https://dom.spec.whatwg.org/#contained>
    pub(crate) fn contains(&self, node: &Node) -> bool {
        // > A node node is contained in a live range range if node’s root is range’s root,
        // > and (node, 0) is after range’s start, and (node, node’s length) is before range’s end.
        node.GetRootNode(&Default::default()) == self.root() &&
            matches!(
                (
                    bp_position(node, 0, &self.start_container(), self.start_offset()),
                    bp_position(node, node.len(), &self.end_container(), self.end_offset()),
                ),
                (Ordering::Greater, Ordering::Less)
            )
    }

    /// <https://dom.spec.whatwg.org/#partially-contained>
    fn partially_contains(&self, node: &Node) -> bool {
        // > A node is partially contained in a live range if it’s an inclusive ancestor
        // > of the live range’s start node but not its end node, or vice versa.
        self.start_container()
            .inclusive_ancestors(ShadowIncluding::No)
            .any(|n| &*n == node) !=
            self.end_container()
                .inclusive_ancestors(ShadowIncluding::No)
                .any(|n| &*n == node)
    }

    /// <https://dom.spec.whatwg.org/#concept-range-clone>
    pub(crate) fn contained_children(&self) -> Fallible<ContainedChildren> {
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
            return Err(Error::HierarchyRequest(None));
        }

        Ok(ContainedChildren {
            first_partially_contained_child,
            last_partially_contained_child,
            contained_children,
        })
    }

    /// <https://dom.spec.whatwg.org/#concept-range-bp-set>
    pub(crate) fn set_start(&self, node: &Node, offset: u32) {
        if self.set_start_without_reporting(node, offset) {
            self.report_change();
        }
    }

    fn set_start_without_reporting(&self, node: &Node, offset: u32) -> bool {
        if self.start().node() == node && self.start_offset() == offset {
            return false;
        }

        if self.start().node() != node {
            if self.start().node() == self.end().node() {
                node.ensure_weak_ranges().push(WeakRef::new(self));
            } else if self.end().node() == node {
                self.start_container().ensure_weak_ranges().remove(self);
            } else {
                node.ensure_weak_ranges()
                    .push(self.start_container().ensure_weak_ranges().remove(self));
            }
        }

        self.start().set(node, offset);
        true
    }

    /// <https://dom.spec.whatwg.org/#concept-range-bp-set>
    pub(crate) fn set_end(&self, node: &Node, offset: u32) {
        if self.set_end_without_reporting(node, offset) {
            self.report_change();
        }
    }

    fn set_end_without_reporting(&self, node: &Node, offset: u32) -> bool {
        if self.end().node() == node && self.end_offset() == offset {
            return false;
        }

        if self.end().node() != node {
            if self.end().node() == self.start().node() {
                node.ensure_weak_ranges().push(WeakRef::new(self));
            } else if self.start().node() == node {
                self.end_container().ensure_weak_ranges().remove(self);
            } else {
                node.ensure_weak_ranges()
                    .push(self.end_container().ensure_weak_ranges().remove(self));
            }
        }

        self.end().set(node, offset);
        true
    }

    /// <https://dom.spec.whatwg.org/#dom-range-comparepointnode-offset>
    fn compare_point(&self, node: &Node, offset: u32) -> Fallible<Ordering> {
        // Step 1. If node’s root is not this’s root, then throw a "WrongDocumentError"
        // DOMException.
        if node.GetRootNode(&Default::default()) != self.root() {
            return Err(Error::WrongDocument(None));
        }
        // Step 2. If node is a doctype, then throw an "InvalidNodeTypeError"
        // DOMException.
        if node.is_doctype() {
            return Err(Error::InvalidNodeType(None));
        }
        // Step 3. If offset is greater than node’s length, then throw an "IndexSizeError"
        // DOMException.
        if offset > node.len() {
            return Err(Error::IndexSize(None));
        }
        // Step 4. If (node, offset) is before start, then return −1.
        let start_node = self.start_container();
        if let Ordering::Less = bp_position(node, offset, &start_node, self.start_offset()) {
            return Ok(Ordering::Less);
        }
        // Step 5. If (node, offset) is after end, then return 1.
        if let Ordering::Greater =
            bp_position(node, offset, &self.end_container(), self.end_offset())
        {
            return Ok(Ordering::Greater);
        }
        // Step 6. Return 0.
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
            .for_each(|selection| {
                selection.queue_selectionchange_task();
                selection.set_visible_selection_dirty();
                selection.clear_command_overrides();
            });
    }

    fn abstract_range(&self) -> &AbstractRange {
        &self.abstract_range
    }

    pub(crate) fn start(&self) -> &BoundaryPoint {
        self.abstract_range().start()
    }

    pub(crate) fn end(&self) -> &BoundaryPoint {
        self.abstract_range().end()
    }

    pub(crate) fn start_and_end_are_in_document_tree(&self) -> bool {
        self.start_container().is_in_a_document_tree() &&
            self.end_container().is_in_a_document_tree()
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

    /// <https://drafts.csswg.org/cssom-view/#dom-range-getclientrects>
    fn client_rects(&self, no_gc: &NoGC) -> Vec<Rect<Au, CSSPixel>> {
        // FIXME: For text nodes that are only partially selected, this should return the client
        // rect of the selected part, not the whole text node.
        let start = self.start_container();
        let end = self.end_container();
        // > The getClientRects() method, when invoked, must return an empty DOMRectList
        // > object if the range is not in the document.
        if !start.is_connected() || !end.is_connected() {
            return vec![];
        }

        // Per the spec, only Text nodes contribute rects when the range is collapsed
        // (including when the boundary points are identical).
        if self.collapsed() {
            if start.is::<CharacterData>() {
                return start.border_boxes();
            } else {
                return vec![];
            }
        }

        let document = start.owner_doc();
        let end_clone = UnrootedDom::from_dom(Dom::from_ref(&*end), no_gc);
        start
            .following_nodes_unrooted(no_gc, document.upcast::<Node>(), ShadowIncluding::No)
            .take_while(move |node| *node != *end)
            .chain(iter::once(end_clone))
            .flat_map(move |node| node.border_boxes())
            .collect()
    }

    /// <https://dom.spec.whatwg.org/#concept-range-bp-set>
    fn set_the_start_or_end(
        &self,
        node: &Node,
        offset: u32,
        start_or_end: StartOrEnd,
    ) -> ErrorResult {
        // Step 1. If node is a doctype, then throw an "InvalidNodeTypeError"
        // DOMException.
        if node.is_doctype() {
            return Err(Error::InvalidNodeType(None));
        }

        // Step 2. If offset is greater than node’s length, then throw an "IndexSizeError"
        // DOMException.
        if offset > node.len() {
            return Err(Error::IndexSize(None));
        }

        // Step 3. Let bp be the boundary point (node, offset).
        // NOTE: We don't need this part.
        let mut set_start = false;
        let mut set_end = false;
        match start_or_end {
            // If these steps were invoked as "set the start"
            StartOrEnd::Start => {
                // Step 4.1. If range’s root is not equal to node’s root, or if bp is after
                // the range’s end, set range’s end to bp.
                if self.root() != node.GetRootNode(&Default::default()) ||
                    bp_position(node, offset, &self.end_container(), self.end_offset()) ==
                        Ordering::Greater
                {
                    set_end = self.set_end_without_reporting(node, offset);
                }

                // Step 4.2. Set range’s start to bp.
                set_start = self.set_start_without_reporting(node, offset);
            },
            // If these steps were invoked as "set the end"
            StartOrEnd::End => {
                // Step 4.1. If range’s root is not equal to node’s root, or if bp is
                // before the range’s start, set range’s start to bp.
                if self.root() != node.GetRootNode(&Default::default()) ||
                    bp_position(node, offset, &self.start_container(), self.start_offset()) ==
                        Ordering::Less
                {
                    set_start = self.set_start_without_reporting(node, offset);
                }

                // Step 4.2. Set range’s end to bp.
                set_end = self.set_end_without_reporting(node, offset);
            },
        }

        if set_start || set_end {
            self.report_change();
        }

        Ok(())
    }
}

impl std::fmt::Debug for Range {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "[({:?}, {}) -> ({:?}, {})]",
            self.start_container(),
            self.start_offset(),
            self.end_container(),
            self.end_offset()
        )
    }
}

enum StartOrEnd {
    Start,
    End,
}

impl RangeMethods<crate::DomTypeHolder> for Range {
    /// <https://dom.spec.whatwg.org/#dom-range>
    fn Constructor(
        cx: &mut JSContext,
        window: &Window,
        proto: Option<HandleObject>,
    ) -> Fallible<DomRoot<Range>> {
        let document = window.Document();
        Ok(Range::new_with_doc(cx, &document, proto))
    }

    /// <https://dom.spec.whatwg.org/#dom-range-commonancestorcontainer>
    fn CommonAncestorContainer(&self) -> DomRoot<Node> {
        self.end_container()
            .common_ancestor(&self.start_container(), ShadowIncluding::No)
            .expect("Couldn't find common ancestor container")
    }

    /// <https://dom.spec.whatwg.org/#dom-range-setstart>
    fn SetStart(&self, node: &Node, offset: u32) -> ErrorResult {
        self.set_the_start_or_end(node, offset, StartOrEnd::Start)
    }

    /// <https://dom.spec.whatwg.org/#dom-range-setend>
    fn SetEnd(&self, node: &Node, offset: u32) -> ErrorResult {
        self.set_the_start_or_end(node, offset, StartOrEnd::End)
    }

    /// <https://dom.spec.whatwg.org/#dom-range-setstartbefore>
    fn SetStartBefore(&self, node: &Node) -> ErrorResult {
        let parent = node.GetParentNode().ok_or(Error::InvalidNodeType(None))?;
        self.SetStart(&parent, node.index())
    }

    /// <https://dom.spec.whatwg.org/#dom-range-setstartafter>
    fn SetStartAfter(&self, node: &Node) -> ErrorResult {
        let parent = node.GetParentNode().ok_or(Error::InvalidNodeType(None))?;
        self.SetStart(&parent, node.index() + 1)
    }

    /// <https://dom.spec.whatwg.org/#dom-range-setendbefore>
    fn SetEndBefore(&self, node: &Node) -> ErrorResult {
        let parent = node.GetParentNode().ok_or(Error::InvalidNodeType(None))?;
        self.SetEnd(&parent, node.index())
    }

    /// <https://dom.spec.whatwg.org/#dom-range-setendafter>
    fn SetEndAfter(&self, node: &Node) -> ErrorResult {
        let parent = node.GetParentNode().ok_or(Error::InvalidNodeType(None))?;
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
        let parent = node.GetParentNode().ok_or(Error::InvalidNodeType(None))?;
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
            return Err(Error::InvalidNodeType(None));
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
    fn CompareBoundaryPoints(&self, how: u16, source_range: &Range) -> Fallible<i16> {
        // Step 1. If how is not one of
        //    * START_TO_START,
        //    * START_TO_END,
        //    * END_TO_END, and
        //    * END_TO_START,
        // then throw a "NotSupportedError" DOMException.
        if how > RangeConstants::END_TO_START {
            return Err(Error::NotSupported(None));
        }
        // Step 2. If this’s root is not sourceRange’s root, then throw a
        // "WrongDocumentError" DOMException.
        if self.root() != source_range.root() {
            return Err(Error::WrongDocument(None));
        }
        // Step 3. Let thisPoint and sourcePoint be null.
        // Step 4.  Switch on how:
        //  ↪ START_TO_START:
        //     Set thisPoint to this’s start and sourcePoint to sourceRange’s start.
        //  ↪ START_TO_END:
        //     Set thisPoint to this’s end and sourcePoint to sourceRange’s start.
        //  ↪ END_TO_END:
        //     Set thisPoint to this’s end and sourcePoint to sourceRange’s end.
        //  ↪ END_TO_START:
        //     Set thisPoint to this’s start and sourcePoint to sourceRange’s end.
        let (this_point, source_point) = match how {
            RangeConstants::START_TO_START => (self.start(), source_range.start()),
            RangeConstants::START_TO_END => (self.end(), source_range.start()),
            RangeConstants::END_TO_END => (self.end(), source_range.end()),
            RangeConstants::END_TO_START => (self.start(), source_range.end()),
            _ => unreachable!(),
        };
        // Step 5. Switch on the position of thisPoint relative to sourcePoint:
        //  ↪ before
        //      Return −1.
        //  ↪ equal
        //      Return 0.
        //  ↪ after
        //      Return 1.
        match this_point.partial_cmp(source_point).unwrap() {
            Ordering::Less => Ok(-1),
            Ordering::Equal => Ok(0),
            Ordering::Greater => Ok(1),
        }
    }

    /// <https://dom.spec.whatwg.org/#dom-range-clonerange>
    fn CloneRange(&self, cx: &mut JSContext) -> DomRoot<Range> {
        let start_node = self.start_container();
        let owner_doc = start_node.owner_doc();
        Range::new(
            cx,
            &owner_doc,
            &start_node,
            self.start_offset(),
            &self.end_container(),
            self.end_offset(),
        )
    }

    /// <https://dom.spec.whatwg.org/#dom-range-ispointinrange>
    fn IsPointInRange(&self, node: &Node, offset: u32) -> Fallible<bool> {
        match self.compare_point(node, offset) {
            Ok(Ordering::Less) => Ok(false),
            Ok(Ordering::Equal) => Ok(true),
            Ok(Ordering::Greater) => Ok(false),
            Err(Error::WrongDocument(None)) => {
                // Step 2.  If node’s root is not this’s root, then return false.
                // Note: This is the only step that differs from `Self::compare_point`.
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
        // Step 1. If node’s root is not this’s root, then return false.
        if self.root() != node.GetRootNode(&Default::default()) {
            return false;
        }
        // Step 2. Let parent be node’s parent.
        let Some(parent) = node.GetParentNode() else {
            // Step 3. If parent is null, then return true.
            return true;
        };
        // Step 4. Let offset be node’s index.
        let offset = node.index();
        // Step 5. If (parent, offset) is before end and (parent, offset + 1) is after
        // start, then return true.
        // Step 6. Return false.
        let start_node = self.start_container();
        Ordering::Greater == bp_position(&parent, offset + 1, &start_node, self.start_offset()) &&
            Ordering::Less ==
                bp_position(&parent, offset, &self.end_container(), self.end_offset())
    }

    /// <https://dom.spec.whatwg.org/#dom-range-clonecontents>
    /// <https://dom.spec.whatwg.org/#concept-range-clone>
    fn CloneContents(&self, cx: &mut JSContext) -> Fallible<DomRoot<DocumentFragment>> {
        // Step 3.
        let start_node = self.start_container();
        let start_offset = self.start_offset();
        let end_node = self.end_container();
        let end_offset = self.end_offset();

        // Step 1.
        let fragment = DocumentFragment::new(cx, &start_node.owner_doc());

        // Step 2.
        if self.start() == self.end() {
            return Ok(fragment);
        }

        if end_node == start_node &&
            let Some(cdata) = start_node.downcast::<CharacterData>()
        {
            // Steps 4.1-2.
            let data = cdata
                .SubstringData(start_offset, end_offset - start_offset)
                .unwrap();
            let clone = cdata.clone_with_data(cx, data, &start_node.owner_doc());
            // Step 4.3.
            fragment.upcast::<Node>().AppendChild(cx, &clone)?;
            // Step 4.4
            return Ok(fragment);
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
                let clone = cdata.clone_with_data(cx, data, &start_node.owner_doc());
                // Step 13.3.
                fragment.upcast::<Node>().AppendChild(cx, &clone)?;
            } else {
                // Step 14.1.
                let clone = child.CloneNode(cx, /* deep */ false)?;
                // Step 14.2.
                fragment.upcast::<Node>().AppendChild(cx, &clone)?;
                // Step 14.3.
                let subrange = Range::new(
                    cx,
                    &clone.owner_doc(),
                    &start_node,
                    start_offset,
                    &child,
                    child.len(),
                );
                // Step 14.4.
                let subfragment = subrange.CloneContents(cx)?;
                // Step 14.5.
                clone.AppendChild(cx, subfragment.upcast())?;
            }
        }

        // Step 15.
        for child in contained_children {
            // Step 15.1.
            let clone = child.CloneNode(cx, /* deep */ true)?;
            // Step 15.2.
            fragment.upcast::<Node>().AppendChild(cx, &clone)?;
        }

        if let Some(child) = last_partially_contained_child {
            // Step 16.
            if let Some(cdata) = child.downcast::<CharacterData>() {
                assert!(child == end_node);
                // Steps 16.1-2.
                let data = cdata.SubstringData(0, end_offset).unwrap();
                let clone = cdata.clone_with_data(cx, data, &start_node.owner_doc());
                // Step 16.3.
                fragment.upcast::<Node>().AppendChild(cx, &clone)?;
            } else {
                // Step 17.1.
                let clone = child.CloneNode(cx, /* deep */ false)?;
                // Step 17.2.
                fragment.upcast::<Node>().AppendChild(cx, &clone)?;
                // Step 17.3.
                let subrange = Range::new(cx, &clone.owner_doc(), &child, 0, &end_node, end_offset);
                // Step 17.4.
                let subfragment = subrange.CloneContents(cx)?;
                // Step 17.5.
                clone.AppendChild(cx, subfragment.upcast())?;
            }
        }

        // Step 18.
        Ok(fragment)
    }

    /// <https://dom.spec.whatwg.org/#dom-range-extractcontents>
    /// <https://dom.spec.whatwg.org/#concept-range-extract>
    fn ExtractContents(&self, cx: &mut JSContext) -> Fallible<DomRoot<DocumentFragment>> {
        // Step 3.
        let start_node = self.start_container();
        let start_offset = self.start_offset();
        let end_node = self.end_container();
        let end_offset = self.end_offset();

        // Step 1.
        let fragment = DocumentFragment::new(cx, &start_node.owner_doc());

        // Step 2.
        if self.collapsed() {
            return Ok(fragment);
        }

        if end_node == start_node &&
            let Some(end_data) = end_node.downcast::<CharacterData>()
        {
            // Step 4.1.
            let clone = end_node.CloneNode(cx, /* deep */ true)?;
            // Step 4.2.
            let text = end_data.SubstringData(start_offset, end_offset - start_offset);
            clone
                .downcast::<CharacterData>()
                .unwrap()
                .SetData(cx, text.unwrap());
            // Step 4.3.
            fragment.upcast::<Node>().AppendChild(cx, &clone)?;
            // Step 4.4.
            end_data.ReplaceData(
                cx,
                start_offset,
                end_offset - start_offset,
                DOMString::new(),
            )?;
            // Step 4.5.
            return Ok(fragment);
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
                let clone = start_node.CloneNode(cx, /* deep */ true)?;
                // Step 15.2.
                let text = start_data.SubstringData(start_offset, start_node.len() - start_offset);
                clone
                    .downcast::<CharacterData>()
                    .unwrap()
                    .SetData(cx, text.unwrap());
                // Step 15.3.
                fragment.upcast::<Node>().AppendChild(cx, &clone)?;
                // Step 15.4.
                start_data.ReplaceData(
                    cx,
                    start_offset,
                    start_node.len() - start_offset,
                    DOMString::new(),
                )?;
            } else {
                // Step 16.1.
                let clone = child.CloneNode(cx, /* deep */ false)?;
                // Step 16.2.
                fragment.upcast::<Node>().AppendChild(cx, &clone)?;
                // Step 16.3.
                let subrange = Range::new(
                    cx,
                    &clone.owner_doc(),
                    &start_node,
                    start_offset,
                    &child,
                    child.len(),
                );
                // Step 16.4.
                let subfragment = subrange.ExtractContents(cx)?;
                // Step 16.5.
                clone.AppendChild(cx, subfragment.upcast())?;
            }
        }

        // Step 17.
        for child in contained_children {
            fragment.upcast::<Node>().AppendChild(cx, &child)?;
        }

        if let Some(child) = last_partially_contained_child {
            if let Some(end_data) = child.downcast::<CharacterData>() {
                assert!(child == end_node);
                // Step 18.1.
                let clone = end_node.CloneNode(cx, /* deep */ true)?;
                // Step 18.2.
                let text = end_data.SubstringData(0, end_offset);
                clone
                    .downcast::<CharacterData>()
                    .unwrap()
                    .SetData(cx, text.unwrap());
                // Step 18.3.
                fragment.upcast::<Node>().AppendChild(cx, &clone)?;
                // Step 18.4.
                end_data.ReplaceData(cx, 0, end_offset, DOMString::new())?;
            } else {
                // Step 19.1.
                let clone = child.CloneNode(cx, /* deep */ false)?;
                // Step 19.2.
                fragment.upcast::<Node>().AppendChild(cx, &clone)?;
                // Step 19.3.
                let subrange = Range::new(cx, &clone.owner_doc(), &child, 0, &end_node, end_offset);
                // Step 19.4.
                let subfragment = subrange.ExtractContents(cx)?;
                // Step 19.5.
                clone.AppendChild(cx, subfragment.upcast())?;
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
    fn InsertNode(&self, cx: &mut JSContext, node: &Node) -> ErrorResult {
        let start_node = self.start_container();
        let start_offset = self.start_offset();

        // Step 1.
        if &*start_node == node {
            return Err(Error::HierarchyRequest(None));
        }
        match start_node.type_id() {
            // Handled under step 2.
            NodeTypeId::CharacterData(CharacterDataTypeId::Text(_)) => (),
            NodeTypeId::CharacterData(_) => return Err(Error::HierarchyRequest(None)),
            _ => (),
        }

        // Step 2.
        let (reference_node, parent) = match start_node.type_id() {
            NodeTypeId::CharacterData(CharacterDataTypeId::Text(_)) => {
                // Step 3.
                let parent = match start_node.GetParentNode() {
                    Some(parent) => parent,
                    // Step 1.
                    None => return Err(Error::HierarchyRequest(None)),
                };
                // Step 5.
                (Some(DomRoot::from_ref(&*start_node)), parent)
            },
            _ => {
                // Steps 4-5.
                let child = start_node.ChildNodes(cx).Item(cx, start_offset);
                (child, DomRoot::from_ref(&*start_node))
            },
        };

        // Step 6.
        Node::ensure_pre_insertion_validity(cx.no_gc(), node, &parent, reference_node.as_deref())?;

        // Step 7.
        let split_text;
        let reference_node = match start_node.downcast::<Text>() {
            Some(text) => {
                split_text = text.SplitText(cx, start_offset)?;
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
        node.remove_self(cx);

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
        Node::pre_insert(cx, node, &parent, reference_node.as_deref())?;

        // Step 13.
        if self.collapsed() {
            self.set_end(&parent, new_offset);
        }

        Ok(())
    }

    /// <https://dom.spec.whatwg.org/#dom-range-deletecontents>
    fn DeleteContents(&self, cx: &mut JSContext) -> ErrorResult {
        // Step 1. If this is collapsed, then return.
        if self.collapsed() {
            return Ok(());
        }

        // Step 2. Let originalStartNode, originalStartOffset, originalEndNode,
        // and originalEndOffset be this’s start node, start offset, end node, and end offset, respectively.
        let start_node = self.start_container();
        let end_node = self.end_container();
        let start_offset = self.start_offset();
        let end_offset = self.end_offset();

        // Step 3. If originalStartNode is originalEndNode and it is a CharacterData node:
        if start_node == end_node &&
            let Some(text) = start_node.downcast::<CharacterData>()
        {
            if end_offset > start_offset {
                self.report_change();
            }

            // Step 3.1. Replace data of originalStartNode with originalStartOffset,
            // originalEndOffset − originalStartOffset, and the empty string.
            // Step 3.2. Return.
            return text.ReplaceData(
                cx,
                start_offset,
                end_offset - start_offset,
                DOMString::new(),
            );
        }

        // Step 4. Let nodesToRemove be a list of all the nodes that are contained in this,
        // in tree order, omitting any node whose parent is also contained in this.
        rooted_vec!(let mut contained_children);
        let ancestor = self.CommonAncestorContainer();

        let mut iter = start_node.following_nodes(&ancestor, ShadowIncluding::No);

        let mut next = iter.next();
        while let Some(child) = next {
            if self.contains(&child) {
                contained_children.push(Dom::from_ref(&*child));
                next = iter.next_skipping_children();
            } else {
                next = iter.next();
            }
        }

        // Step 5. Let newNode and newOffset be null.
        // Step 6. If originalStartNode is an inclusive ancestor of originalEndNode,
        // then set newNode to originalStartNode and newOffset to originalStartOffset.
        let (new_node, new_offset) = if start_node.is_inclusive_ancestor_of(&end_node) {
            (DomRoot::from_ref(&*start_node), start_offset)
        } else {
            // Step 7. Otherwise:
            fn compute_reference(start_node: &Node, end_node: &Node) -> (DomRoot<Node>, u32) {
                // Step 7.1. Let referenceNode be originalStartNode.
                let mut reference_node = DomRoot::from_ref(start_node);
                // Step 7.2. While referenceNode’s parent is non-null and
                // is not an inclusive ancestor of originalEndNode: set referenceNode to its parent.
                while let Some(parent) = reference_node.GetParentNode() {
                    if parent.is_inclusive_ancestor_of(end_node) {
                        // Step 7.3. Set newNode to referenceNode’s parent and newOffset to referenceNode’s index + 1.
                        return (parent, reference_node.index() + 1);
                    }
                    reference_node = parent;
                }
                unreachable!()
            }

            compute_reference(&start_node, &end_node)
        };

        // Step 8. Set this’s start and end to (newNode, newOffset).
        self.SetStart(&new_node, new_offset).unwrap();
        self.SetEnd(&new_node, new_offset).unwrap();

        // Step 9. If originalStartNode is a CharacterData node,
        // then replace data of originalStartNode with originalStartOffset,
        // originalStartNode’s length − originalStartOffset, and the empty string.
        if let Some(text) = start_node.downcast::<CharacterData>() {
            text.ReplaceData(
                cx,
                start_offset,
                start_node.len() - start_offset,
                DOMString::new(),
            )
            .unwrap();
        }

        // Step 10. For each node of nodesToRemove, in tree order: remove node.
        for child in &*contained_children {
            child.remove_self(cx);
        }

        // Step 11. If originalEndNode is a CharacterData node,
        // then replace data of originalEndNode with 0, originalEndOffset, and the empty string.
        if let Some(text) = end_node.downcast::<CharacterData>() {
            text.ReplaceData(cx, 0, end_offset, DOMString::new())
                .unwrap();
        }

        Ok(())
    }

    /// <https://dom.spec.whatwg.org/#dom-range-surroundcontents>
    fn SurroundContents(&self, cx: &mut JSContext, new_parent: &Node) -> ErrorResult {
        // Step 1.
        let start = self.start_container();
        let end = self.end_container();

        if start
            .inclusive_ancestors(ShadowIncluding::No)
            .any(|n| !n.is_inclusive_ancestor_of(&end) && !n.is::<Text>()) ||
            end.inclusive_ancestors(ShadowIncluding::No)
                .any(|n| !n.is_inclusive_ancestor_of(&start) && !n.is::<Text>())
        {
            return Err(Error::InvalidState(None));
        }

        // Step 2.
        match new_parent.type_id() {
            NodeTypeId::Document(_) |
            NodeTypeId::DocumentType |
            NodeTypeId::DocumentFragment(_) => {
                return Err(Error::InvalidNodeType(None));
            },
            _ => (),
        }

        // Step 3.
        let fragment = self.ExtractContents(cx)?;

        // Step 4.
        Node::replace_all(cx, None, new_parent);

        // Step 5.
        self.InsertNode(cx, new_parent)?;

        // Step 6.
        new_parent.AppendChild(cx, fragment.upcast())?;

        // Step 7.
        self.SelectNode(new_parent)
    }

    /// <https://dom.spec.whatwg.org/#dom-range-stringifier>
    fn Stringifier(&self, no_gc: &NoGC) -> DOMString {
        let start_node = self.start_container();
        let end_node = self.end_container();

        // Step 1. Let string be the empty string.
        let mut s = DOMString::new();

        if let Some(text_node) = start_node.downcast::<Text>() {
            let char_data = text_node.upcast::<CharacterData>();

            // Step 2. If this’s start node is this’s end node and it is a Text node,
            // then return the substring of that Text node’s data beginning at
            // this’s start offset and ending at this’s end offset.
            if start_node == end_node {
                return char_data
                    .SubstringData(self.start_offset(), self.end_offset() - self.start_offset())
                    .unwrap();
            }

            // Step 3. If this’s start node is a Text node, then append the substring of
            // that node’s data from this’s start offset until the end to string.
            s.push_str(
                &char_data
                    .SubstringData(
                        self.start_offset(),
                        char_data.Length() - self.start_offset(),
                    )
                    .unwrap()
                    .str(),
            );
        }

        // Step 4. Append the concatenation of the data of all Text nodes that are contained in this,
        // in tree order, to string.
        let ancestor = self.CommonAncestorContainer();
        let iter = start_node
            .following_nodes_unrooted(no_gc, &ancestor, ShadowIncluding::No)
            .filter_map(UnrootedDom::downcast::<Text>);

        for child in iter {
            if self.contains(child.upcast()) {
                s.push_str(&child.upcast::<CharacterData>().Data().str());
            }
        }

        // Step 5. If this’s end node is a Text node, then append the substring of
        // that node’s data from its start until this’s end offset to string.
        if let Some(text_node) = end_node.downcast::<Text>() {
            let char_data = text_node.upcast::<CharacterData>();
            s.push_str(&char_data.SubstringData(0, self.end_offset()).unwrap().str());
        }

        // Step 6. Return string.
        s
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-range-createcontextualfragment>
    fn CreateContextualFragment(
        &self,
        cx: &mut JSContext,
        fragment: TrustedHTMLOrString,
    ) -> Fallible<DomRoot<DocumentFragment>> {
        // Step 2. Let node be this's start node.
        //
        // Required to obtain the global, so we do this first. Shouldn't be an
        // observable difference.
        let node = self.start_container();

        // Step 1. Let compliantString be the result of invoking the
        // Get Trusted Type compliant string algorithm with TrustedHTML,
        // this's relevant global object, string, "Range createContextualFragment", and "script".
        let fragment = TrustedHTML::get_trusted_type_compliant_string(
            cx,
            node.owner_window().upcast(),
            fragment,
            "Range createContextualFragment",
        )?;

        let owner_doc = node.owner_doc();

        // Step 3. Let element be null.
        // Step 4. If node implements Element, set element to node.
        // Step 5. Otherwise, if node implements Text or Comment, set element to node's parent element.
        let element = match node.type_id() {
            NodeTypeId::Element(_) => Some(DomRoot::downcast::<Element>(node).unwrap()),
            NodeTypeId::CharacterData(CharacterDataTypeId::Comment) |
            NodeTypeId::CharacterData(CharacterDataTypeId::Text(_)) => node.GetParentElement(),
            _ => None,
        };

        // Step 6. If element is null or all of the following are true:
        let element = Element::fragment_parsing_context(cx, &owner_doc, element.as_deref());

        // Step 7. Let fragment node be the result of invoking the fragment parsing algorithm steps with element and compliantString.
        let fragment_node = element.parse_fragment(fragment, cx)?;

        // Step 8. For each script of fragment node's script element descendants:
        for node in fragment_node
            .upcast::<Node>()
            .traverse_preorder(ShadowIncluding::No)
        {
            if let Some(script) = node.downcast::<HTMLScriptElement>() {
                // Step 8.1. Set script's already started to false.
                script.set_already_started(false);
                // Step 8.2. Set script's parser document to null.
                script.set_parser_inserted(false);
            }
        }

        // Step 9. Return fragment node.
        Ok(fragment_node)
    }

    /// <https://drafts.csswg.org/cssom-view/#dom-range-getclientrects>
    fn GetClientRects(&self, cx: &mut JSContext) -> DomRoot<DOMRectList> {
        let start = self.start_container();
        let window = start.owner_window();

        let client_rects = self.client_rects(cx.no_gc());
        let client_rects = client_rects
            .iter()
            .map(|rect| {
                DOMRect::new(
                    cx,
                    window.upcast(),
                    rect.origin.x.to_f64_px(),
                    rect.origin.y.to_f64_px(),
                    rect.size.width.to_f64_px(),
                    rect.size.height.to_f64_px(),
                )
            })
            .collect();

        DOMRectList::new(cx, &window, client_rects)
    }

    /// <https://drafts.csswg.org/cssom-view/#dom-range-getboundingclientrect>
    fn GetBoundingClientRect(&self, cx: &mut JSContext) -> DomRoot<DOMRect> {
        let window = self.start_container().owner_window();

        // Step 1. Let list be the result of invoking getClientRects() on the same range this method was invoked on.
        let list = self.client_rects(cx.no_gc());

        // Step 2. If list is empty return a DOMRect object whose x, y, width and height members are zero.
        // Step 3. If all rectangles in list have zero width or height, return the first rectangle in list.
        // Step 4. Otherwise, return a DOMRect object describing the smallest rectangle that includes all
        // of the rectangles in list of which the height or width is not zero.
        let bounding_rect = list
            .into_iter()
            .fold(euclid::Rect::zero(), |acc, rect| acc.union(&rect));

        DOMRect::new(
            cx,
            window.upcast(),
            bounding_rect.origin.x.to_f64_px(),
            bounding_rect.origin.y.to_f64_px(),
            bounding_rect.size.width.to_f64_px(),
            bounding_rect.size.height.to_f64_px(),
        )
    }
}

#[derive(MallocSizeOf)]
pub(crate) struct WeakRangeVec {
    cell: RefCell<WeakRefVec<Range>>,
}

impl Default for WeakRangeVec {
    fn default() -> Self {
        WeakRangeVec {
            cell: RefCell::new(WeakRefVec::new()),
        }
    }
}

impl WeakRangeVec {
    /// Whether that vector of ranges is empty.
    pub(crate) fn is_empty(&self) -> bool {
        self.cell.borrow().is_empty()
    }

    /// Get a rooted version of the contents of this [`WeakRangeVec`]
    pub(crate) fn live_ranges(&self) -> SmallVec<[DomRoot<Range>; 4]> {
        let cell = self.cell.borrow();
        if cell.is_empty() {
            return Default::default();
        }
        cell.iter().filter_map(|range| range.root()).collect()
    }

    pub(crate) fn push(&self, ref_: WeakRef<Range>) {
        self.cell.borrow_mut().push(ref_);
    }

    fn remove(&self, range: &Range) -> WeakRef<Range> {
        let mut ranges = self.cell.borrow_mut();
        let position = ranges.iter().position(|ref_| ref_ == range).unwrap();
        ranges.swap_remove(position)
    }
}

#[expect(unsafe_code)]
unsafe impl JSTraceable for WeakRangeVec {
    unsafe fn trace(&self, _: *mut JSTracer) {
        self.cell.borrow_mut().retain_alive()
    }
}

/// <https://dom.spec.whatwg.org/#concept-node-insert> steps 5.1-5.2
/// and
/// <https://dom.spec.whatwg.org/#move> steps 17.1-17.2.
pub(crate) fn live_range_insert_steps(parent: &Node, child: &Node, count: u32) {
    if parent.has_live_ranges() {
        let child_index = LazyCell::new(|| child.index());
        for range in parent.live_ranges() {
            // Step 5.1: For each live range whose start node is parent and start offset is
            // greater than child’s index: increase its start offset by count.
            if &*range.start_container() == parent && range.start_offset() > *child_index {
                range.set_start(parent, range.start_offset() + count);
            }
            // Step 5.2: For each live range whose end node is parent and end offset is
            // greater than child’s index: increase its end offset by count.
            if &*range.end_container() == parent && range.end_offset() > *child_index {
                range.set_end(parent, range.end_offset() + count);
            }
        }
    }
}

/// <https://dom.spec.whatwg.org/#live-range-pre-remove-steps> steps 4 and 5.
///
/// These steps are run on the inclusive descendants of a removed node, but to avoid
/// having to iterate through those nodes twice, they are run when the inclusive
/// descendants themselves are unbound from the tree.
pub(crate) fn live_range_pre_remove_steps_for_removed_subtree(
    inclusive_descendant_of_removed_node: &Node, // "node" in the specification
    parent_of_removed_node: &Node,               // "parent" in the specification
    index_of_removed_node: &dyn Fn() -> u32,     // "index" in the specification
) {
    // The steps are only supposed to run on DOM tree inclusive descendants of the removal
    // root and elements in shadow trees are not, so they shouldn't run for them.
    if inclusive_descendant_of_removed_node.is_in_a_shadow_tree() {
        return;
    }
    for range in inclusive_descendant_of_removed_node.live_ranges() {
        let index = index_of_removed_node();
        // Step 4: For each live range whose start node is an inclusive descendant of
        // node, set its start to (parent, index).
        if &*range.start_container() == inclusive_descendant_of_removed_node {
            range.set_start(parent_of_removed_node, index);
        }
        // Step 5: For each live range whose end node is an inclusive descendant of node,
        // set its end to (parent, index).
        if &*range.end_container() == inclusive_descendant_of_removed_node {
            range.set_end(parent_of_removed_node, index);
        }
    }
}

/// <https://dom.spec.whatwg.org/#live-range-pre-remove-steps> steps 6 and 7.
pub(crate) fn live_range_pre_remove_steps_for_parent(
    node: &Node,
    parent: &Node,
    cached_node_index: &mut Option<u32>,
) {
    for range in parent.live_ranges() {
        let node_index = *cached_node_index.get_or_insert_with(|| node.index());
        // Step 6: For each live range whose start node is parent and start offset is
        // greater than index, decrease its start offset by 1.
        if &*range.start_container() == parent && range.start_offset() > node_index {
            range.set_start(parent, range.start_offset() - 1);
        }
        // Step 7: For each live range whose end node is parent and end offset is greater than
        // index, decrease its end offset by 1.
        if &*range.end_container() == parent && range.end_offset() > node_index {
            range.set_end(parent, range.end_offset() - 1);
        }
    }
}

/// <https://dom.spec.whatwg.org/#dom-node-normalize> Steps 6.1-6.4.
///
/// - `parent`: The parent of both other node arguments.
/// - `node`: The node that text is being merged into.
/// - `current_node`: The node which has text being merged into `node` and will be
///   removed from the DOM.
/// - `current_node_index`: The index of `current_node` in `parent`.
/// - `length`: The length of the text content in `node`, its orginal length plus
///   the length of all content that has already been merged from siblings before
///   `current_node`.
pub(crate) fn live_range_normalization_steps(
    parent: &Node,
    node: &Node,
    current_node: &Node,
    current_node_index: &dyn Fn() -> u32,
    length: u32,
) {
    for range in current_node.live_ranges() {
        // Step 6.1: For each live range whose start node is currentNode: add length to
        // its start offset and set its start node to node.
        if &*range.start_container() == current_node {
            range.set_start(node, range.start_offset() + length);
        }
        // Step 6.2: For each live range whose end node is currentNode: add length to its
        // end offset and set its end node to node.
        if &*range.end_container() == current_node {
            range.set_end(node, range.end_offset() + length);
        }
    }

    for range in parent.live_ranges() {
        // Step 6.3: For each live range whose start node is currentNode’s parent and
        // start offset is currentNode’s index: set its start node to node and its start
        // offset to length.
        if &*range.start_container() == parent && range.start_offset() == current_node_index() {
            range.set_start(node, length);
        }
        // Step 6.4: For each live range whose end node is currentNode’s parent and end
        // offset is currentNode’s index: set its end node to node and its end offset to
        // length.
        if &*range.end_container() == parent && range.end_offset() == current_node_index() {
            range.set_end(node, length);
        }
    }
}

/// <https://dom.spec.whatwg.org/#concept-cd-replace> steps 8-11.
pub(crate) fn live_range_replace_data_steps(
    node: &Node,
    offset: u32,
    removed_code_units: u32,
    added_code_units: u32,
) {
    for range in node.live_ranges() {
        // Step 8: For each live range whose start node is node and start offset is
        // greater than offset but less than or equal to offset + count: set its start
        // offset to offset.
        let start_container = range.start_container();
        let start_offset = range.start_offset();
        if &*start_container == node &&
            start_offset > offset &&
            start_offset <= offset + removed_code_units
        {
            range.set_start(node, offset);
        }
        // Step 9: For each live range whose end node is node and end offset is
        // greater than offset but less than or equal to offset + count: set its end
        // offset to offset.
        let end_container = range.end_container();
        let end_offset = range.end_offset();
        if &*end_container == node &&
            end_offset > offset &&
            end_offset <= offset + removed_code_units
        {
            range.set_end(node, offset);
        }
        // Step 10: For each live range whose start node is node and start offset is
        // greater than offset + count: increase its start offset by data’s length and
        // decrease it by count.
        if &*start_container == node && start_offset > offset + removed_code_units {
            range.set_start(node, start_offset + added_code_units - removed_code_units);
        }
        // Step 11: For each live range whose end node is node and end offset is
        // greater than offset + count: increase its end offset by data’s length and
        // decrease it by count.
        if &*end_container == node && end_offset > offset + removed_code_units {
            range.set_end(node, end_offset + added_code_units - removed_code_units);
        }
    }
}

/// <https://dom.spec.whatwg.org/#concept-text-split> steps 7.2-7.5.
pub(crate) fn live_range_text_split_steps(
    parent: &Node,
    node: &Node,
    offset: u32,
    new_node: &Node,
) {
    for range in node.live_ranges() {
        // Step 7.2. For each live range whose start node is node and start offset is
        // greater than offset, set its start node to newNode and decrease its start
        // offset by offset.
        if &*range.start_container() == node && range.start_offset() > offset {
            range.set_start(new_node, range.start_offset() - offset);
        }
        // Step 7.3. For each live range whose end node is node and end offset is
        // greater than offset, set its end node to newNode and decrease its end
        // offset by offset.
        if &*range.end_container() == node && range.end_offset() > offset {
            range.set_end(new_node, range.end_offset() - offset);
        }
    }

    let node_index = LazyCell::new(|| node.index());
    for range in parent.live_ranges() {
        // Step 7.4. For each live range whose start node is parent and start offset
        // is equal to the index of node plus 1, increase its start offset by 1.
        if &*range.start_container() == parent && range.start_offset() == *node_index + 1 {
            range.set_start(parent, range.start_offset() + 1);
        }

        // Step 7.5. For each live range whose end node is parent and end offset is
        // equal to the index of node plus 1, increase its end offset by 1.
        if &*range.end_container() == parent && range.end_offset() == *node_index + 1 {
            range.set_end(parent, range.end_offset() + 1);
        }
    }
}
