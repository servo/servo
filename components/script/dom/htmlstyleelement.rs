/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLStyleElementBinding;
use dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use dom::bindings::codegen::InheritTypes::{ElementCast, HTMLElementCast, HTMLStyleElementDerived, NodeCast};
use dom::bindings::js::{JSRef, Temporary, OptionalRootable};
use dom::document::Document;
use dom::eventtarget::{EventTarget, EventTargetTypeId};
use dom::element::{Element, ElementTypeId};
use dom::htmlelement::{HTMLElement, HTMLElementTypeId};
use dom::node::{Node, NodeHelpers, NodeTypeId, window_from_node};
use dom::virtualmethods::VirtualMethods;
use dom::window::WindowHelpers;
use layout_interface::{LayoutChan, Msg};
use util::str::DOMString;
use style::stylesheets::{Origin, Stylesheet};
use style::media_queries::parse_media_query_list;
use style::node::TElement;
use cssparser::Parser as CssParser;

#[dom_struct]
pub struct HTMLStyleElement {
    htmlelement: HTMLElement,
}

impl HTMLStyleElementDerived for EventTarget {
    fn is_htmlstyleelement(&self) -> bool {
        *self.type_id() == EventTargetTypeId::Node(NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLStyleElement)))
    }
}

impl HTMLStyleElement {
    fn new_inherited(localName: DOMString, prefix: Option<DOMString>, document: JSRef<Document>) -> HTMLStyleElement {
        HTMLStyleElement {
            htmlelement: HTMLElement::new_inherited(HTMLElementTypeId::HTMLStyleElement, localName, prefix, document)
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
        let element: JSRef<Element> = ElementCast::from_ref(self);
        assert!(node.is_in_doc());

        let win = window_from_node(node).root();
        let win = win.r();
        let url = win.get_url();

        let mq_str = element.get_attr(&ns!(""), &atom!("media")).unwrap_or("");
        let mut css_parser = CssParser::new(&mq_str);
        let media = parse_media_query_list(&mut css_parser);

        let data = node.GetTextContent().expect("Element.textContent must be a string");
        let sheet = Stylesheet::from_str(&data, url, Origin::Author);
        let LayoutChan(ref layout_chan) = win.layout_chan();
        layout_chan.send(Msg::AddStylesheet(sheet, media)).unwrap();
    }
}

impl<'a> VirtualMethods for JSRef<'a, HTMLStyleElement> {
    fn super_type<'b>(&'b self) -> Option<&'b VirtualMethods> {
        let htmlelement: &JSRef<HTMLElement> = HTMLElementCast::from_borrowed_ref(self);
        Some(htmlelement as &VirtualMethods)
    }

    fn child_inserted(&self, child: JSRef<Node>) {
        if let Some(ref s) = self.super_type() {
            s.child_inserted(child);
        }

        let node: JSRef<Node> = NodeCast::from_ref(*self);
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

