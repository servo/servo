/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::activation::Activatable;
use dom::attr::AttrValue;
use dom::bindings::codegen::Bindings::AttrBinding::AttrMethods;
use dom::bindings::codegen::Bindings::HTMLAnchorElementBinding;
use dom::bindings::codegen::Bindings::HTMLAnchorElementBinding::HTMLAnchorElementMethods;
use dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use dom::bindings::codegen::InheritTypes::HTMLAnchorElementDerived;
use dom::bindings::codegen::InheritTypes::{ElementCast, HTMLElementCast, NodeCast};
use dom::bindings::js::{MutNullableJS, JSRef, Temporary, OptionalRootable};
use dom::document::{Document, DocumentHelpers};
use dom::domtokenlist::DOMTokenList;
use dom::element::{Element, AttributeHandlers, ElementTypeId};
use dom::event::Event;
use dom::eventtarget::{EventTarget, EventTargetTypeId};
use dom::htmlelement::{HTMLElement, HTMLElementTypeId};
use dom::node::{Node, NodeHelpers, NodeTypeId, document_from_node};
use dom::virtualmethods::VirtualMethods;

use std::default::Default;
use string_cache::Atom;
use util::str::DOMString;

#[dom_struct]
pub struct HTMLAnchorElement {
    htmlelement: HTMLElement,
    rel_list: MutNullableJS<DOMTokenList>,
}

impl HTMLAnchorElementDerived for EventTarget {
    fn is_htmlanchorelement(&self) -> bool {
        *self.type_id() == EventTargetTypeId::Node(NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLAnchorElement)))
    }
}

impl HTMLAnchorElement {
    fn new_inherited(localName: DOMString, prefix: Option<DOMString>, document: JSRef<Document>) -> HTMLAnchorElement {
        HTMLAnchorElement {
            htmlelement: HTMLElement::new_inherited(HTMLElementTypeId::HTMLAnchorElement, localName, prefix, document),
            rel_list: Default::default(),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: DOMString, prefix: Option<DOMString>, document: JSRef<Document>) -> Temporary<HTMLAnchorElement> {
        let element = HTMLAnchorElement::new_inherited(localName, prefix, document);
        Node::reflect_node(box element, document, HTMLAnchorElementBinding::Wrap)
    }
}

impl<'a> VirtualMethods for JSRef<'a, HTMLAnchorElement> {
    fn super_type<'b>(&'b self) -> Option<&'b VirtualMethods> {
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
    }

    fn parse_plain_attribute(&self, name: &Atom, value: DOMString) -> AttrValue {
        match name {
            &atom!("rel") => AttrValue::from_serialized_tokenlist(value),
            _ => self.super_type().unwrap().parse_plain_attribute(name, value),
        }
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

    fn RelList(self) -> Temporary<DOMTokenList> {
        self.rel_list.or_init(|| {
            DOMTokenList::new(ElementCast::from_ref(self), &atom!("rel"))
        })
    }
}

impl<'a> Activatable for JSRef<'a, HTMLAnchorElement> {
    fn as_element(&self) -> Temporary<Element> {
        Temporary::from_rooted(ElementCast::from_ref(*self))
    }

    fn is_instance_activatable(&self) -> bool {
        true
    }


    //TODO:https://html.spec.whatwg.org/multipage/semantics.html#the-a-element
    fn pre_click_activation(&self) {
    }

    //TODO:https://html.spec.whatwg.org/multipage/semantics.html#the-a-element
    // https://html.spec.whatwg.org/multipage/interaction.html#run-canceled-activation-steps
    fn canceled_activation(&self) {
    }

    //https://html.spec.whatwg.org/multipage/semantics.html#the-a-element:activation-behaviour
    fn activation_behavior(&self) {
        //Step 1. If the node document is not fully active, abort.
        let doc = document_from_node(*self).root();
        if !doc.r().is_fully_active() {
            return;
        }
        //TODO: Step 2. Check if browsing context is specified and act accordingly.
        //TODO: Step 3. Handle <img ismap/>.
        //TODO: Step 4. Download the link is `download` attribute is set.
        let element: JSRef<Element> = ElementCast::from_ref(*self);
        let attr = element.get_attribute(ns!(""), &atom!("href")).root();
        match attr {
            Some(ref href) => {
                let value = href.r().Value();
                debug!("clicked on link to {}", value);
                doc.r().load_anchor_href(value);
            }
            None => ()
        }
    }

    //TODO:https://html.spec.whatwg.org/multipage/semantics.html#the-a-element
    fn implicit_submission(&self, _ctrlKey: bool, _shiftKey: bool, _altKey: bool, _metaKey: bool) {
    }
}
