/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::attr::AttrHelpers;
use dom::bindings::codegen::Bindings::HTMLMetaElementBinding;
use dom::bindings::codegen::Bindings::HTMLMetaElementBinding::HTMLMetaElementMethods;
use dom::bindings::codegen::InheritTypes::{ElementCast, HTMLElementCast, NodeCast};
use dom::bindings::codegen::InheritTypes::HTMLMetaElementDerived;
use dom::bindings::js::{JSRef, OptionalRootable, Rootable, RootedReference, Temporary};
use dom::document::Document;
use dom::eventtarget::{EventTarget, EventTargetTypeId};
use dom::element::{AttributeHandlers, Element, ElementTypeId};
use dom::htmlelement::{HTMLElement, HTMLElementTypeId};
use dom::node::{Node, NodeTypeId, window_from_node};
use dom::virtualmethods::VirtualMethods;
use dom::window::WindowHelpers;
use layout_interface::{LayoutChan, Msg};
use style::viewport::ViewportRule;
use util::str::DOMString;

#[dom_struct]
pub struct HTMLMetaElement {
    htmlelement: HTMLElement,
}

impl HTMLMetaElementDerived for EventTarget {
    fn is_htmlmetaelement(&self) -> bool {
        *self.type_id() ==
            EventTargetTypeId::Node(
                NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLMetaElement)))
    }
}

impl HTMLMetaElement {
    fn new_inherited(localName: DOMString,
                     prefix: Option<DOMString>,
                     document: JSRef<Document>) -> HTMLMetaElement {
        HTMLMetaElement {
            htmlelement: HTMLElement::new_inherited(HTMLElementTypeId::HTMLMetaElement, localName, prefix, document)
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: DOMString,
               prefix: Option<DOMString>,
               document: JSRef<Document>) -> Temporary<HTMLMetaElement> {
        let element = HTMLMetaElement::new_inherited(localName, prefix, document);
        Node::reflect_node(box element, document, HTMLMetaElementBinding::Wrap)
    }
}

pub trait MetaElementHelpers {
    fn process_attributes(self);
    fn translate_viewport(self);
}

impl <'a> MetaElementHelpers for JSRef<'a, HTMLMetaElement> {
    fn process_attributes(self) {
        let element: JSRef<Element> = ElementCast::from_ref(self);
        if let Some(ref name) = element.get_attribute(&ns!(""), &atom!("name")).root() {
            let name = name.r().value();
            if !name.is_empty() {
                match &**name {
                    "viewport" => self.translate_viewport(),
                    _ => {}
                }
            }
        }
    }

    fn translate_viewport(self) {
        let element: JSRef<Element> = ElementCast::from_ref(self);
        if let Some(ref content) = element.get_attribute(&ns!(""), &atom!("content")).root() {
            let content = content.r().value();
            if !content.is_empty() {
                if let Some(translated_rule) = ViewportRule::from_meta(&**content) {
                    let node: JSRef<Node> = NodeCast::from_ref(self);
                    let win = window_from_node(node).root();
                    let win = win.r();

                    let LayoutChan(ref layout_chan) = win.layout_chan();
                    layout_chan.send(Msg::AddMetaViewport(translated_rule)).unwrap();
                }
            }
        }
    }
}

impl<'a> VirtualMethods for JSRef<'a, HTMLMetaElement> {
    fn super_type<'b>(&'b self) -> Option<&'b VirtualMethods> {
        let htmlelement: &JSRef<HTMLElement> = HTMLElementCast::from_borrowed_ref(self);
        Some(htmlelement as &VirtualMethods)
    }

    fn bind_to_tree(&self, tree_in_doc: bool) {
        if let Some(ref s) = self.super_type() {
            s.bind_to_tree(tree_in_doc);
        }

        if tree_in_doc {
            self.process_attributes();
        }
    }
}

impl<'a> HTMLMetaElementMethods for JSRef<'a, HTMLMetaElement> {
    // https://html.spec.whatwg.org/multipage/#dom-meta-name
    make_getter!(Name, "name");

    // https://html.spec.whatwg.org/multipage/#dom-meta-name
    make_setter!(SetName, "name");

    // https://html.spec.whatwg.org/multipage/#dom-meta-content
    make_getter!(Content, "content");

    // https://html.spec.whatwg.org/multipage/#dom-meta-content
    make_setter!(SetContent, "content");
}
