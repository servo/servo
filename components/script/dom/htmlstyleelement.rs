/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cssparser::Parser as CssParser;
use dom::bindings::codegen::Bindings::HTMLStyleElementBinding;
use dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use dom::bindings::codegen::InheritTypes::{ElementCast, HTMLElementCast, HTMLStyleElementDerived, NodeCast};
use dom::bindings::js::Root;
use dom::document::Document;
use dom::element::ElementTypeId;
use dom::eventtarget::{EventTarget, EventTargetTypeId};
use dom::htmlelement::{HTMLElement, HTMLElementTypeId};
use dom::node::window_from_node;
use dom::node::{ChildrenMutation, Node, NodeTypeId};
use dom::virtualmethods::VirtualMethods;
use layout_interface::{LayoutChan, Msg};
use style::media_queries::parse_media_query_list;
use style::stylesheets::{Origin, Stylesheet};
use util::str::DOMString;

#[dom_struct]
pub struct HTMLStyleElement {
    htmlelement: HTMLElement,
}

impl HTMLStyleElementDerived for EventTarget {
    fn is_htmlstyleelement(&self) -> bool {
        *self.type_id() ==
            EventTargetTypeId::Node(
                NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLStyleElement)))
    }
}

impl HTMLStyleElement {
    fn new_inherited(localName: DOMString,
                     prefix: Option<DOMString>,
                     document: &Document) -> HTMLStyleElement {
        HTMLStyleElement {
            htmlelement: HTMLElement::new_inherited(HTMLElementTypeId::HTMLStyleElement, localName, prefix, document)
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: DOMString,
               prefix: Option<DOMString>,
               document: &Document) -> Root<HTMLStyleElement> {
        let element = HTMLStyleElement::new_inherited(localName, prefix, document);
        Node::reflect_node(box element, document, HTMLStyleElementBinding::Wrap)
    }
}


impl HTMLStyleElement {
    pub fn parse_own_css(&self) {
        let node = NodeCast::from_ref(self);
        let element = ElementCast::from_ref(self);
        assert!(node.is_in_doc());

        let win = window_from_node(node);
        let win = win.r();
        let url = win.get_url();

        let mq_attribute = element.get_attribute(&ns!(""), &atom!("media"));
        let mq_str = match mq_attribute {
            Some(a) => String::from(&**a.r().value()),
            None => String::new(),
        };
        let mut css_parser = CssParser::new(&mq_str);
        let media = parse_media_query_list(&mut css_parser);

        let data = node.GetTextContent().expect("Element.textContent must be a string");
        let sheet = Stylesheet::from_str(&data, url, Origin::Author);
        let LayoutChan(ref layout_chan) = win.layout_chan();
        layout_chan.send(Msg::AddStylesheet(sheet, media)).unwrap();
    }
}

impl VirtualMethods for HTMLStyleElement {
    fn super_type<'b>(&'b self) -> Option<&'b VirtualMethods> {
        let htmlelement: &HTMLElement = HTMLElementCast::from_ref(self);
        Some(htmlelement as &VirtualMethods)
    }

    fn children_changed(&self, mutation: &ChildrenMutation) {
        if let Some(ref s) = self.super_type() {
            s.children_changed(mutation);
        }
        let node = NodeCast::from_ref(self);
        if node.is_in_doc() {
            self.parse_own_css();
        }
    }

    fn bind_to_tree(&self, tree_in_doc: bool) {
        if let Some(ref s) = self.super_type() {
            s.bind_to_tree(tree_in_doc);
        }

        if tree_in_doc {
            self.parse_own_css();
        }
    }
}
