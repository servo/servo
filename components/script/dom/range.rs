/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::RangeBinding;
use dom::bindings::codegen::Bindings::NodeBinding::{NodeConstants, NodeMethods};
use dom::bindings::codegen::Bindings::RangeBinding::RangeMethods;
use dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use dom::bindings::codegen::InheritTypes::NodeCast;
use dom::bindings::error::{Fallible, IndexSize, InvalidNodeTypeError};
use dom::bindings::global::{GlobalRef, Window};
use dom::bindings::js::{JS, JSRef, Temporary};
use dom::bindings::utils::{Reflectable, Reflector, reflect_dom_object};
use dom::document::Document;
use dom::node::{DoctypeNodeTypeId, Node, NodeHelpers};

use std::cell::Cell;

#[jstraceable]
#[must_root]
pub struct Range {
    reflector_: Reflector,
    start_node: Cell<JS<Node>>,
    start_point: Cell<u32>,
    end_node: Cell<JS<Node>>,
    end_point: Cell<u32>,
}

impl Range {
    fn new_inherited(document: JSRef<Document>) -> Range {
        let node: JSRef<Node> = NodeCast::from_ref(document);
        Range {
            reflector_: Reflector::new(),
            start_node: Cell::new(node.unrooted()),
            start_point: Cell::new(0),
            end_node: Cell::new(node.unrooted()),
            end_point: Cell::new(0),
        }
    }

    pub fn new(document: JSRef<Document>) -> Temporary<Range> {
        let window = document.window.root();
        reflect_dom_object(box Range::new_inherited(document),
                           &Window(*window),
                           RangeBinding::Wrap)
    }

    pub fn Constructor(global: &GlobalRef) -> Fallible<Temporary<Range>> {
        let document = global.as_window().Document().root();
        Ok(Range::new(*document))
    }
}

impl<'a> RangeMethods for JSRef<'a, Range> {
    /// http://dom.spec.whatwg.org/#dom-range-startcontainer
    fn StartContainer(self) -> Temporary<Node> {
        Temporary::new(self.start_node.get())
    }

    /// http://dom.spec.whatwg.org/#dom-range-startoffset
    fn StartOffset(self) -> u32 {
        self.start_point.get()
    }

    /// http://dom.spec.whatwg.org/#dom-range-endcontainer
    fn EndContainer(self) -> Temporary<Node> {
        Temporary::new(self.end_node.get())
    }

    /// http://dom.spec.whatwg.org/#dom-range-endoffset
    fn EndOffset(self) -> u32 {
        self.end_point.get()
    }

    /// http://dom.spec.whatwg.org/#dom-range-collapsed
    fn Collapsed(self) -> bool {
        self.start_node.get() == self.end_node.get()
    }

    /// http://dom.spec.whatwg.org/#dom-range-setstart
    fn SetStart(self, node: JSRef<Node>, offset: u32) -> Fallible<()> {
        // step 1
        if node.is_doctype() {
            return Err(InvalidNodeTypeError);
        }

        // step 2
        if offset > node.length() {
            return Err(IndexSize);
        }

        // step 3, 4-1
        match compare_bp_position(node, offset, self.end_node.get().root().root_ref(), self.end_point.get()) {
            After | NoSharedRoot => {
                self.end_node.set(node.unrooted());
                self.end_point.set(offset);
            },
            _ => (),
        }

        // step 4-2
        self.start_node.set(node.unrooted());
        self.start_point.set(offset);

        Ok(())
    }

    /// http://dom.spec.whatwg.org/#dom-range-setstart
    fn SetEnd(self, node: JSRef<Node>, offset: u32) -> Fallible<()> {
        // step 1
        if node.is_doctype() {
            return Err(InvalidNodeTypeError);
        }

        // step 2
        if offset > node.length() {
            return Err(IndexSize);
        }

        // step 3, 4-1
        match compare_bp_position(node, offset, self.start_node.get().root().root_ref(), self.start_point.get()) {
            Before | NoSharedRoot => {
                self.start_node.set(node.unrooted());
                self.start_point.set(offset);
            },
            _ => (),
        }

        // step 4-2
        self.end_node.set(node.unrooted());
        self.end_point.set(offset);

        Ok(())
    }

    /// https://dom.spec.whatwg.org/#dom-range-setstartbefore
    fn SetStartBefore(self, node: JSRef<Node>) -> Fallible<()> {
        // step 1
        match node.parent_node() {
            // step 2
            None => {
                Err(InvalidNodeTypeError)
            },
            // step 3
            Some(parent) => {
                let parent = parent.root();
                let index = parent.children().position(|child| -> bool {
                    child == node
                });
                match index {
                    Some(index) => {
                        self.SetStart(parent.root_ref(), index as u32)
                    },
                    None => {
                        unreachable!()
                    }
                }
            },
        }
    }

    /// https://dom.spec.whatwg.org/#dom-range-setstartafter
    fn SetStartAfter(self, node: JSRef<Node>) -> Fallible<()> {
        // step 1
        match node.parent_node() {
            // step 2
            None => {
                Err(InvalidNodeTypeError)
            },
            // step 3
            Some(parent) => {
                let parent = parent.root();
                let index = parent.children().position(|child| -> bool {
                    child == node
                });
                match index {
                    Some(index) => {
                        self.SetStart(parent.root_ref(), (index + 1) as u32)
                    },
                    None => {
                        unreachable!()
                    }
                }
            },
        }
    }

    /// https://dom.spec.whatwg.org/#dom-range-setendbefore
    fn SetEndBefore(self, node: JSRef<Node>) -> Fallible<()> {
        // step 1
        match node.parent_node() {
            // step 2
            None => {
                Err(InvalidNodeTypeError)
            }
            // step 3
            Some(parent) => {
                let parent = parent.root();
                let index = parent.children().position(|child| -> bool {
                    child == node
                });
                match index {
                    Some(index) => {
                        self.SetEnd(parent.root_ref(), index as u32)
                    },
                    None => {
                        unreachable!()
                    }
                }
            },
        }
    }

    /// https://dom.spec.whatwg.org/#dom-range-setendafter
    fn SetEndAfter(self, node: JSRef<Node>) -> Fallible<()> {
        // step 1
        match node.parent_node() {
            // step 2
            None => {
                Err(InvalidNodeTypeError)
            },
            // step 3
            Some(parent) => {
                let parent = parent.root();
                let index = parent.children().position(|child| -> bool {
                    child == node
                });
                match index {
                    Some(index) => {
                        self.SetEnd(parent.root_ref(), (index + 1) as u32)
                    },
                    None => {
                        unreachable!()
                    }
                }
            },
        }
    }

    /// http://dom.spec.whatwg.org/#dom-range-detach
    fn Detach(self) {
        // This method intentionally left blank.
    }
}

impl Reflectable for Range {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        &self.reflector_
    }
}

enum BoundaryPointRelation {
    Before,
    Equal,
    After,
    NoSharedRoot,
}

// https://dom.spec.whatwg.org/#concept-range-bp-after
fn compare_bp_position(a_node: JSRef<Node>, a_offset: u32,
                       b_node: JSRef<Node>, b_offset: u32) -> BoundaryPointRelation {
    let position = b_node.CompareDocumentPosition(a_node);

    // We can compute whether A and B share the same root or not in this point because
    //  * This function is only used in Range.
    //  * This function always takes start/end boundary point.
    //    We can assume that a boundary point set to Range will always share the root.
    //    This is certain by the behavior of setStart/setEnd().
    //    They don't allow to store boundary points which has the difference roots in the same Range instance.
    if (position & NodeConstants::DOCUMENT_POSITION_DISCONNECTED) == NodeConstants::DOCUMENT_POSITION_DISCONNECTED {
        return NoSharedRoot;
    }

    // step 1
    // Node.CompareDocumentPosition returns `0` if a is the same as b.
    if position == 0 {
        if a_offset == b_offset {
            return Equal;
        }
        else if a_offset < b_offset {
            return Before;
        }
        else {
            return After;
        }
    }

    // step 2
    if (position & NodeConstants::DOCUMENT_POSITION_FOLLOWING) == NodeConstants::DOCUMENT_POSITION_FOLLOWING {
        let result = match compare_bp_position(b_node, b_offset, a_node, a_offset) {
            Before => {
                After
            },
            After => {
                Before
            },
            _ => {
                unreachable!()
            }
        };
        return result;
    }

    // step 3
    if (position & NodeConstants::DOCUMENT_POSITION_CONTAINS) == NodeConstants::DOCUMENT_POSITION_CONTAINS {
        // FIXME: more flat
        for child in b_node.inclusive_ancestors() {
            if !a_node.is_parent_of(child) {
                continue;
            }

            let b_position = a_node.children().position(|a_child| -> bool {
                a_child == child
            });
            match b_position {
                Some(idx) => if idx < (a_offset as uint) {
                    return After;
                },
                None => {
                    // In this path, a is the parent of b,
                    // so we have no situation to enter this path.
                    unreachable!()
                }
            }
        }
    }

    // step 4
    Before
}
