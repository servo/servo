/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLLabelElementBinding;
use dom::bindings::codegen::Bindings::HTMLLabelElementBinding::HTMLLabelElementMethods;
use dom::bindings::js::Root;
use dom::document::Document;
use dom::htmlelement::HTMLElement;
use dom::htmlformelement::{FormControl, HTMLFormElement};
use dom::node::Node;
use util::str::DOMString;

#[dom_struct]
pub struct HTMLLabelElement {
    htmlelement: HTMLElement,
}

impl HTMLLabelElement {
    fn new_inherited(localName: DOMString,
                     prefix: Option<DOMString>,
                     document: &Document) -> HTMLLabelElement {
        HTMLLabelElement {
            htmlelement:
                HTMLElement::new_inherited(localName, prefix, document)
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: DOMString,
               prefix: Option<DOMString>,
               document: &Document) -> Root<HTMLLabelElement> {
        let element = HTMLLabelElement::new_inherited(localName, prefix, document);
        Node::reflect_node(box element, document, HTMLLabelElementBinding::Wrap)
    }
}

impl HTMLLabelElementMethods for HTMLLabelElement {
    // https://html.spec.whatwg.org/multipage/#dom-fae-form
    fn GetForm(&self) -> Option<Root<HTMLFormElement>> {
        self.form_owner()
    }
}

impl FormControl for HTMLLabelElement {}
