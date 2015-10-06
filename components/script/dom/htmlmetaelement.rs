/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::HTMLMetaElementBinding;
use dom::bindings::codegen::Bindings::HTMLMetaElementBinding::HTMLMetaElementMethods;
use dom::bindings::codegen::InheritTypes::HTMLMetaElementDerived;
use dom::bindings::codegen::InheritTypes::{ElementCast, HTMLElementCast};
use dom::bindings::js::{Root, RootedReference};
use dom::document::Document;
use dom::element::ElementTypeId;
use dom::eventtarget::{EventTarget, EventTargetTypeId};
use dom::htmlelement::{HTMLElement, HTMLElementTypeId};
use dom::node::{Node, NodeTypeId, document_from_node};
use dom::virtualmethods::VirtualMethods;
use std::ascii::AsciiExt;
use std::sync::Arc;
use style::stylesheets::{CSSRule, Origin, Stylesheet};
use style::viewport::ViewportRule;
use util::str::{DOMString, HTML_SPACE_CHARACTERS};

#[dom_struct]
pub struct HTMLMetaElement {
    htmlelement: HTMLElement,
    stylesheet: DOMRefCell<Option<Arc<Stylesheet>>>,
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
                     document: &Document) -> HTMLMetaElement {
        HTMLMetaElement {
            htmlelement: HTMLElement::new_inherited(HTMLElementTypeId::HTMLMetaElement, localName, prefix, document),
            stylesheet: DOMRefCell::new(None),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: DOMString,
               prefix: Option<DOMString>,
               document: &Document) -> Root<HTMLMetaElement> {
        let element = HTMLMetaElement::new_inherited(localName, prefix, document);
        Node::reflect_node(box element, document, HTMLMetaElementBinding::Wrap)
    }

    pub fn get_stylesheet(&self) -> Option<Arc<Stylesheet>> {
        self.stylesheet.borrow().clone()
    }

    fn process_attributes(&self) {
        let element = ElementCast::from_ref(self);
        if let Some(name) = element.get_attribute(&ns!(""), &atom!("name")).r() {
            let name = name.value().to_ascii_lowercase();
            let name = name.trim_matches(HTML_SPACE_CHARACTERS);

            match name {
                "viewport" => self.apply_viewport(),
                _ => {}
            }
        }
    }

    fn apply_viewport(&self) {
        let element = ElementCast::from_ref(self);
        if let Some(content) = element.get_attribute(&ns!(""), &atom!("content")).r() {
            let content = content.value();
            if !content.is_empty() {
                if let Some(translated_rule) = ViewportRule::from_meta(&**content) {
                    *self.stylesheet.borrow_mut() = Some(Arc::new(Stylesheet {
                        rules: vec![CSSRule::Viewport(translated_rule)],
                        origin: Origin::Author,
                        media: None,
                    }));
                    let doc = document_from_node(self);
                    doc.r().invalidate_stylesheets();
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
    fn super_type<'b>(&'b self) -> Option<&'b VirtualMethods> {
        let htmlelement: &HTMLElement = HTMLElementCast::from_ref(self);
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
