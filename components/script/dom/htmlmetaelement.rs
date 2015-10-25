/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLMetaElementBinding;
use dom::bindings::codegen::Bindings::HTMLMetaElementBinding::HTMLMetaElementMethods;
use dom::bindings::conversions::Castable;
use dom::bindings::js::{Root, RootedReference};
use dom::document::Document;
use dom::element::Element;
use dom::htmlelement::HTMLElement;
use dom::node::{Node, window_from_node};
use dom::virtualmethods::VirtualMethods;
use layout_interface::{LayoutChan, Msg};
use std::ascii::AsciiExt;
use style::viewport::ViewportRule;
use util::str::{DOMString, HTML_SPACE_CHARACTERS};

#[dom_struct]
pub struct HTMLMetaElement {
    htmlelement: HTMLElement,
}

impl HTMLMetaElement {
    fn new_inherited(localName: DOMString,
                     prefix: Option<DOMString>,
                     document: &Document) -> HTMLMetaElement {
        HTMLMetaElement {
            htmlelement: HTMLElement::new_inherited(localName, prefix, document)
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: DOMString,
               prefix: Option<DOMString>,
               document: &Document) -> Root<HTMLMetaElement> {
        let element = HTMLMetaElement::new_inherited(localName, prefix, document);
        Node::reflect_node(box element, document, HTMLMetaElementBinding::Wrap)
    }

    fn process_attributes(&self) {
        let element = self.upcast::<Element>();
        if let Some(name) = element.get_attribute(&ns!(""), &atom!("name")).r() {
            let name = name.value().to_ascii_lowercase();
            let name = name.trim_matches(HTML_SPACE_CHARACTERS);

            match name {
                "viewport" => self.translate_viewport(),
                _ => {}
            }
        }
    }

    fn translate_viewport(&self) {
        let element = self.upcast::<Element>();
        if let Some(content) = element.get_attribute(&ns!(""), &atom!("content")).r() {
            let content = content.value();
            if !content.is_empty() {
                if let Some(translated_rule) = ViewportRule::from_meta(&**content) {
                    let win = window_from_node(self);
                    let LayoutChan(ref layout_chan) = win.r().layout_chan();

                    layout_chan.send(Msg::AddMetaViewport(translated_rule)).unwrap();
                }
            }
        }
    }
}

impl HTMLMetaElementMethods for HTMLMetaElement {
    // https://html.spec.whatwg.org/multipage/#dom-meta-name
    make_getter!(Name, "name");

    // https://html.spec.whatwg.org/multipage/#dom-meta-name
    make_setter!(SetName, "name");

    // https://html.spec.whatwg.org/multipage/#dom-meta-content
    make_getter!(Content, "content");

    // https://html.spec.whatwg.org/multipage/#dom-meta-content
    make_setter!(SetContent, "content");
}

impl VirtualMethods for HTMLMetaElement {
    fn super_type(&self) -> Option<&VirtualMethods> {
        Some(self.upcast::<HTMLElement>() as &VirtualMethods)
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
