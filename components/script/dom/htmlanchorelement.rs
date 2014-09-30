/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::AttrBinding::AttrMethods;
use dom::bindings::codegen::Bindings::EventBinding::EventMethods;
use dom::bindings::codegen::Bindings::HTMLAnchorElementBinding;
use dom::bindings::codegen::Bindings::HTMLAnchorElementBinding::HTMLAnchorElementMethods;
use dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use dom::bindings::codegen::InheritTypes::HTMLAnchorElementDerived;
use dom::bindings::codegen::InheritTypes::{ElementCast, HTMLElementCast, NodeCast};
use dom::bindings::js::{JSRef, Temporary, OptionalRootable};
use dom::bindings::utils::{Reflectable, Reflector};
use dom::document::{Document, DocumentHelpers};
use dom::element::{Element, AttributeHandlers, HTMLAnchorElementTypeId};
use dom::event::Event;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, NodeHelpers, ElementNodeTypeId};
use dom::virtualmethods::VirtualMethods;

use servo_util::str::DOMString;

#[jstraceable]
#[must_root]
pub struct HTMLAnchorElement {
    pub htmlelement: HTMLElement
}

impl HTMLAnchorElementDerived for EventTarget {
    fn is_htmlanchorelement(&self) -> bool {
        self.type_id == NodeTargetTypeId(ElementNodeTypeId(HTMLAnchorElementTypeId))
    }
}

impl HTMLAnchorElement {
    fn new_inherited(localName: DOMString, document: JSRef<Document>) -> HTMLAnchorElement {
        HTMLAnchorElement {
            htmlelement: HTMLElement::new_inherited(HTMLAnchorElementTypeId, localName, document)
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: DOMString, document: JSRef<Document>) -> Temporary<HTMLAnchorElement> {
        let element = HTMLAnchorElement::new_inherited(localName, document);
        Node::reflect_node(box element, document, HTMLAnchorElementBinding::Wrap)
    }
}

trait PrivateHTMLAnchorElementHelpers {
    fn handle_event_impl(self, event: JSRef<Event>);
}

impl<'a> PrivateHTMLAnchorElementHelpers for JSRef<'a, HTMLAnchorElement> {
    fn handle_event_impl(self, event: JSRef<Event>) {
        if "click" == event.Type().as_slice() && !event.DefaultPrevented() {
            let element: JSRef<Element> = ElementCast::from_ref(self);
            let attr = element.get_attribute(ns!(""), "href").root();
            match attr {
                Some(ref href) => {
                    let value = href.Value();
                    debug!("clicked on link to {:s}", value);
                    let node: JSRef<Node> = NodeCast::from_ref(self);
                    let doc = node.owner_doc().root();
                    doc.load_anchor_href(value);
                }
                None => ()
            }
        }
    }
}

impl<'a> VirtualMethods for JSRef<'a, HTMLAnchorElement> {
    fn super_type<'a>(&'a self) -> Option<&'a VirtualMethods> {
        let htmlelement: &JSRef<HTMLElement> = HTMLElementCast::from_borrowed_ref(self);
        Some(htmlelement as &VirtualMethods)
    }

    fn handle_event(&self, event: JSRef<Event>) {
        match self.super_type() {
            Some(s) => {
                s.handle_event(event);
            }
            None => {}
        }
        self.handle_event_impl(event);
    }
}

impl Reflectable for HTMLAnchorElement {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.htmlelement.reflector()
    }
}

impl<'a> HTMLAnchorElementMethods for JSRef<'a, HTMLAnchorElement> {
    fn Text(self) -> DOMString {
        let node: JSRef<Node> = NodeCast::from_ref(self);
        node.GetTextContent().unwrap()
    }

    fn SetText(self, value: DOMString) {
        let node: JSRef<Node> = NodeCast::from_ref(self);
        node.SetTextContent(Some(value))
    }
}
