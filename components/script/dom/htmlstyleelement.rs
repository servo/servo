/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cssparser::Parser as CssParser;
use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::HTMLStyleElementBinding;
use dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use dom::bindings::inheritance::Castable;
use dom::bindings::js::Root;
use dom::bindings::str::DOMString;
use dom::document::Document;
use dom::element::Element;
use dom::htmlelement::HTMLElement;
use dom::node::{ChildrenMutation, Node, document_from_node, window_from_node};
use dom::virtualmethods::VirtualMethods;
use script_layout_interface::message::Msg;
use std::sync::Arc;
use string_cache::Atom;
use style::media_queries::parse_media_query_list;
use style::parser::ParserContextExtraData;
use style::stylesheets::{Stylesheet, Origin};

#[dom_struct]
pub struct HTMLStyleElement {
    htmlelement: HTMLElement,
    stylesheet: DOMRefCell<Option<Arc<Stylesheet>>>,
}

impl HTMLStyleElement {
    fn new_inherited(localName: Atom,
                     prefix: Option<DOMString>,
                     document: &Document) -> HTMLStyleElement {
        HTMLStyleElement {
            htmlelement: HTMLElement::new_inherited(localName, prefix, document),
            stylesheet: DOMRefCell::new(None),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: Atom,
               prefix: Option<DOMString>,
               document: &Document) -> Root<HTMLStyleElement> {
        Node::reflect_node(box HTMLStyleElement::new_inherited(localName, prefix, document),
                           document,
                           HTMLStyleElementBinding::Wrap)
    }

    pub fn parse_own_css(&self) {
        let node = self.upcast::<Node>();
        let element = self.upcast::<Element>();
        assert!(node.is_in_doc());

        let win = window_from_node(node);
        let url = win.get_url();

        let mq_attribute = element.get_attribute(&ns!(), &atom!("media"));
        let mq_str = match mq_attribute {
            Some(a) => String::from(&**a.value()),
            None => String::new(),
        };

        let data = node.GetTextContent().expect("Element.textContent must be a string");
        let mut sheet = Stylesheet::from_str(&data, url, Origin::Author, win.css_error_reporter(),
                                             ParserContextExtraData::default());
        let mut css_parser = CssParser::new(&mq_str);
        let media = parse_media_query_list(&mut css_parser);
        sheet.set_media(Some(media));
        let sheet = Arc::new(sheet);

        win.layout_chan().send(Msg::AddStylesheet(sheet.clone())).unwrap();
        *self.stylesheet.borrow_mut() = Some(sheet);
        let doc = document_from_node(self);
        doc.r().invalidate_stylesheets();
    }

    pub fn get_stylesheet(&self) -> Option<Arc<Stylesheet>> {
        self.stylesheet.borrow().clone()
    }
}

impl VirtualMethods for HTMLStyleElement {
    fn super_type(&self) -> Option<&VirtualMethods> {
        Some(self.upcast::<HTMLElement>() as &VirtualMethods)
    }

    fn children_changed(&self, mutation: &ChildrenMutation) {
        if let Some(ref s) = self.super_type() {
            s.children_changed(mutation);
        }
        if self.upcast::<Node>().is_in_doc() {
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
