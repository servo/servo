/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLMapElementBinding;
use dom::bindings::js::Root;
use dom::bindings::str::DOMString;
use dom::document::Document;
use dom::htmlelement::HTMLElement;
use dom::node::Node;
use string_cache::Atom;

#[dom_struct]
pub struct HTMLMapElement {
    htmlelement: HTMLElement
}

impl HTMLMapElement {
    fn new_inherited(localName: Atom,
                     prefix: Option<DOMString>,
                     document: &Document) -> HTMLMapElement {
        HTMLMapElement {
            htmlelement: HTMLElement::new_inherited(localName, prefix, document)
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: Atom,
               prefix: Option<DOMString>,
               document: &Document) -> Root<HTMLMapElement> {
        Node::reflect_node(box HTMLMapElement::new_inherited(localName, prefix, document),
                           document,
                           HTMLMapElementBinding::Wrap)
    }
}
