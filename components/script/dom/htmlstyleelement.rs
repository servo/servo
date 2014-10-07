/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLStyleElementBinding;
use dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use dom::bindings::codegen::InheritTypes::{HTMLElementCast, HTMLStyleElementDerived, NodeCast};
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::utils::{Reflectable, Reflector};
use dom::document::Document;
use dom::element::HTMLStyleElementTypeId;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, NodeHelpers, ElementNodeTypeId, window_from_node};
use dom::virtualmethods::VirtualMethods;
use layout_interface::{AddStylesheetMsg, LayoutChan};
use servo_util::str::DOMString;
use style::Stylesheet;

#[jstraceable]
#[must_root]
pub struct HTMLStyleElement {
    pub htmlelement: HTMLElement,
}

impl HTMLStyleElementDerived for EventTarget {
    fn is_htmlstyleelement(&self) -> bool {
        self.type_id == NodeTargetTypeId(ElementNodeTypeId(HTMLStyleElementTypeId))
    }
}

impl HTMLStyleElement {
    fn new_inherited(localName: DOMString, prefix: Option<DOMString>, document: JSRef<Document>) -> HTMLStyleElement {
        HTMLStyleElement {
            htmlelement: HTMLElement::new_inherited(HTMLStyleElementTypeId, localName, prefix, document)
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: DOMString, prefix: Option<DOMString>, document: JSRef<Document>) -> Temporary<HTMLStyleElement> {
        let element = HTMLStyleElement::new_inherited(localName, prefix, document);
        Node::reflect_node(box element, document, HTMLStyleElementBinding::Wrap)
    }
}

pub trait StyleElementHelpers {
    fn parse_own_css(self);
}

impl<'a> StyleElementHelpers for JSRef<'a, HTMLStyleElement> {
    fn parse_own_css(self) {
        let node: JSRef<Node> = NodeCast::from_ref(self);
        assert!(node.is_in_doc());

        let win = window_from_node(node).root();
        let url = win.deref().page().get_url();

        let data = node.GetTextContent().expect("Element.textContent must be a string");
        let sheet = Stylesheet::from_str(data.as_slice(), url);
        let LayoutChan(ref layout_chan) = win.deref().page().layout_chan;
        layout_chan.send(AddStylesheetMsg(sheet));
    }
}

impl<'a> VirtualMethods for JSRef<'a, HTMLStyleElement> {
    fn super_type<'a>(&'a self) -> Option<&'a VirtualMethods> {
        let htmlelement: &JSRef<HTMLElement> = HTMLElementCast::from_borrowed_ref(self);
        Some(htmlelement as &VirtualMethods)
    }

    fn child_inserted(&self, child: JSRef<Node>) {
        match self.super_type() {
            Some(ref s) => s.child_inserted(child),
            _ => (),
        }

        let node: JSRef<Node> = NodeCast::from_ref(*self);
        if node.is_in_doc() {
            self.parse_own_css();
        }
    }

    fn bind_to_tree(&self, tree_in_doc: bool) {
        match self.super_type() {
            Some(ref s) => s.bind_to_tree(tree_in_doc),
            _ => ()
        }

        if tree_in_doc {
            self.parse_own_css();
        }
    }
}

impl Reflectable for HTMLStyleElement {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.htmlelement.reflector()
    }
}
