/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLParamElementBinding;
use dom::bindings::js::Root;
use dom::document::Document;
use dom::htmlelement::HTMLElement;
use dom::node::Node;
use string_cache::Atom;
use util::str::DOMString;

#[dom_struct]
pub struct HTMLParamElement {
    htmlelement: HTMLElement
}

impl HTMLParamElement {
    fn new_inherited(localName: Atom,
                     prefix: Option<DOMString>,
                     document: &Document) -> HTMLParamElement {
        HTMLParamElement {
            htmlelement:
                HTMLElement::new_inherited(localName, prefix, document)
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: Atom,
               prefix: Option<DOMString>,
               document: &Document) -> Root<HTMLParamElement> {
        let element = HTMLParamElement::new_inherited(localName, prefix, document);
        Node::reflect_node(box element, document, HTMLParamElementBinding::Wrap)
    }
}
