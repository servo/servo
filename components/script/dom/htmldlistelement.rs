/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLDListElementBinding;
use dom::bindings::js::Root;
use dom::document::Document;
use dom::htmlelement::HTMLElement;
use dom::node::Node;
use dom_struct::dom_struct;
use html5ever::{LocalName, Prefix};

#[dom_struct]
pub struct HTMLDListElement {
    htmlelement: HTMLElement
}

impl HTMLDListElement {
    fn new_inherited(local_name: LocalName, prefix: Option<Prefix>, document: &Document) -> HTMLDListElement {
        HTMLDListElement {
            htmlelement:
                HTMLElement::new_inherited(local_name, prefix, document)
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(local_name: LocalName,
               prefix: Option<Prefix>,
               document: &Document) -> Root<HTMLDListElement> {
        Node::reflect_node(box HTMLDListElement::new_inherited(local_name, prefix, document),
                           document,
                           HTMLDListElementBinding::Wrap)
    }
}
