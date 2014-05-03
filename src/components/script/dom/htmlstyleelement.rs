/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::BindingDeclarations::HTMLStyleElementBinding;
use dom::bindings::codegen::InheritTypes::{HTMLElementCast, HTMLStyleElementDerived, NodeCast};
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::error::ErrorResult;
use dom::document::Document;
use dom::element::HTMLStyleElementTypeId;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, NodeMethods, ElementNodeTypeId, window_from_node};
use dom::virtualmethods::VirtualMethods;
use html::cssparse::parse_inline_css;
use layout_interface::{AddStylesheetMsg, LayoutChan};
use servo_util::str::DOMString;

#[deriving(Encodable)]
pub struct HTMLStyleElement {
    pub htmlelement: HTMLElement,
}

impl HTMLStyleElementDerived for EventTarget {
    fn is_htmlstyleelement(&self) -> bool {
        match self.type_id {
            NodeTargetTypeId(ElementNodeTypeId(HTMLStyleElementTypeId)) => true,
            _ => false
        }
    }
}

impl HTMLStyleElement {
    pub fn new_inherited(localName: DOMString, document: &JSRef<Document>) -> HTMLStyleElement {
        HTMLStyleElement {
            htmlelement: HTMLElement::new_inherited(HTMLStyleElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: &JSRef<Document>) -> Temporary<HTMLStyleElement> {
        let element = HTMLStyleElement::new_inherited(localName, document);
        Node::reflect_node(~element, document, HTMLStyleElementBinding::Wrap)
    }
}

pub trait HTMLStyleElementMethods {
    fn Disabled(&self) -> bool;
    fn SetDisabled(&self, _disabled: bool);
    fn Media(&self) -> DOMString;
    fn SetMedia(&mut self, _media: DOMString) -> ErrorResult;
    fn Type(&self) -> DOMString;
    fn SetType(&mut self, _type: DOMString) -> ErrorResult;
    fn Scoped(&self) -> bool;
    fn SetScoped(&self, _scoped: bool) -> ErrorResult;
}

impl<'a> HTMLStyleElementMethods for JSRef<'a, HTMLStyleElement> {
    fn Disabled(&self) -> bool {
        false
    }

    fn SetDisabled(&self, _disabled: bool) {
    }

    fn Media(&self) -> DOMString {
        "".to_owned()
    }

    fn SetMedia(&mut self, _media: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Type(&self) -> DOMString {
        "".to_owned()
    }

    fn SetType(&mut self, _type: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Scoped(&self) -> bool {
        false
    }

    fn SetScoped(&self, _scoped: bool) -> ErrorResult {
        Ok(())
    }
}

pub trait StyleElementHelpers {
    fn parse_own_css(&self);
}

impl<'a> StyleElementHelpers for JSRef<'a, HTMLStyleElement> {
    fn parse_own_css(&self) {
        let node: &JSRef<Node> = NodeCast::from_ref(self);
        let win = window_from_node(node).root();
        let url = win.deref().page().get_url();

        let data = node.GetTextContent().expect("Element.textContent must be a string");
        let sheet = parse_inline_css(url, data);
        let LayoutChan(ref layout_chan) = *win.deref().page().layout_chan;
        layout_chan.send(AddStylesheetMsg(sheet));
    }
}

impl<'a> VirtualMethods for JSRef<'a, HTMLStyleElement> {
    fn super_type<'a>(&'a mut self) -> Option<&'a mut VirtualMethods:> {
        let htmlelement: &mut JSRef<HTMLElement> = HTMLElementCast::from_mut_ref(self);
        Some(htmlelement as &mut VirtualMethods:)
    }

    fn child_inserted(&mut self, child: &JSRef<Node>) {
        match self.super_type() {
            Some(ref mut s) => s.child_inserted(child),
            _ => (),
        }
        self.parse_own_css();
    }
}
