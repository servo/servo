/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLUnknownElementBinding;
use dom::bindings::js::Root;
use dom::bindings::str::DOMString;
use dom::document::Document;
use dom::htmlelement::HTMLElement;
use dom::node::Node;
use dom_struct::dom_struct;
use html5ever::LocalName;

#[dom_struct]
pub struct HTMLUnknownElement {
    htmlelement: HTMLElement
}

impl HTMLUnknownElement {
    fn new_inherited(local_name: LocalName,
                     prefix: Option<DOMString>,
                     document: &Document) -> HTMLUnknownElement {
        HTMLUnknownElement {
            htmlelement:
                HTMLElement::new_inherited(local_name, prefix, document)
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(local_name: LocalName,
               prefix: Option<DOMString>,
               document: &Document) -> Root<HTMLUnknownElement> {
        Node::reflect_node(box HTMLUnknownElement::new_inherited(local_name, prefix, document),
                           document,
                           HTMLUnknownElementBinding::Wrap)
    }
}
