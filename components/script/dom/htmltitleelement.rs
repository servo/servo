/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLTitleElementBinding;
use dom::bindings::codegen::Bindings::HTMLTitleElementBinding::HTMLTitleElementMethods;
use dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use dom::bindings::conversions::Castable;
use dom::bindings::js::Root;
use dom::characterdata::CharacterData;
use dom::document::Document;
use dom::htmlelement::HTMLElement;
use dom::node::{ChildrenMutation, Node};
use dom::text::Text;
use dom::virtualmethods::VirtualMethods;
use util::str::DOMString;

#[dom_struct]
pub struct HTMLTitleElement {
    htmlelement: HTMLElement,
}

impl HTMLTitleElement {
    fn new_inherited(localName: DOMString, prefix: Option<DOMString>, document: &Document) -> HTMLTitleElement {
        HTMLTitleElement {
            htmlelement: HTMLElement::new_inherited(localName, prefix, document)
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: DOMString,
               prefix: Option<DOMString>,
               document: &Document) -> Root<HTMLTitleElement> {
        let element = HTMLTitleElement::new_inherited(localName, prefix, document);
        Node::reflect_node(box element, document, HTMLTitleElementBinding::Wrap)
    }
}

impl HTMLTitleElementMethods for HTMLTitleElement {
    // https://html.spec.whatwg.org/multipage/#dom-title-text
    fn Text(&self) -> DOMString {
        let node = self.upcast::<Node>();
        let mut content = String::new();
        for child in node.children() {
            let text: Option<&Text> = child.downcast::<Text>();
            match text {
                Some(text) => content.push_str(&text.upcast::<CharacterData>().data()),
                None => (),
            }
        }
        content
    }

    // https://html.spec.whatwg.org/multipage/#dom-title-text
    fn SetText(&self, value: DOMString) {
        let node = self.upcast::<Node>();
        node.SetTextContent(Some(value))
    }
}

impl VirtualMethods for HTMLTitleElement {
    fn super_type<'b>(&'b self) -> Option<&'b VirtualMethods> {
        let htmlelement: &HTMLElement = self.upcast::<HTMLElement>();
        Some(htmlelement as &VirtualMethods)
    }

    fn children_changed(&self, mutation: &ChildrenMutation) {
        if let Some(ref s) = self.super_type() {
            s.children_changed(mutation);
        }
        let node = self.upcast::<Node>();
        if node.is_in_doc() {
            node.owner_doc().title_changed();
        }
    }

    fn bind_to_tree(&self, is_in_doc: bool) {
        let node = self.upcast::<Node>();
        if is_in_doc {
            let document = node.owner_doc();
            document.r().title_changed();
        }
    }
}
