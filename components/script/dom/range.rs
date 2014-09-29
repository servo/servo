/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::RangeBinding;
use dom::bindings::codegen::Bindings::RangeBinding::RangeMethods;
use dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use dom::bindings::codegen::InheritTypes::NodeCast;
use dom::bindings::error::Fallible;
use dom::bindings::global::{GlobalRef, Window};
use dom::bindings::js::{JS, JSRef, Temporary};
use dom::bindings::utils::{Reflectable, Reflector, reflect_dom_object};
use dom::document::Document;
use dom::node::Node;

#[jstraceable]
#[must_root]
pub struct Range {
    reflector_: Reflector,
    start_node: JS<Node>,
    start_point: u32,
    end_node: JS<Node>,
    end_point: u32,
}

impl Range {
    fn new_inherited(document: JSRef<Document>) -> Range {
        let node: JSRef<Node> = NodeCast::from_ref(document);
        Range {
            reflector_: Reflector::new(),
            start_node: node.unrooted(),
            start_point: 0,
            end_node: node.unrooted(),
            end_point: 0,
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
        Temporary::new(self.start_node)
    }

    /// http://dom.spec.whatwg.org/#dom-range-startoffset
    fn StartOffset(self) -> u32 {
        self.start_point
    }

    /// http://dom.spec.whatwg.org/#dom-range-endcontainer
    fn EndContainer(self) -> Temporary<Node> {
        Temporary::new(self.end_node)
    }

    /// http://dom.spec.whatwg.org/#dom-range-endoffset
    fn EndOffset(self) -> u32 {
        self.end_point
    }

    /// http://dom.spec.whatwg.org/#dom-range-collapsed
    fn Collapsed(self) -> bool {
        self.start_node == self.end_node
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
