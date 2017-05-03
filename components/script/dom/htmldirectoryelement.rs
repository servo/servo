/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLDirectoryElementBinding;
use dom::bindings::js::Root;
use dom::bindings::str::DOMString;
use dom::document::Document;
use dom::htmlelement::HTMLElement;
use dom::node::Node;
use dom_struct::dom_struct;
use html5ever::LocalName;

#[dom_struct]
pub struct HTMLDirectoryElement {
    htmlelement: HTMLElement
}

impl HTMLDirectoryElement {
    fn new_inherited(local_name: LocalName,
                     prefix: Option<DOMString>,
                     document: &Document) -> HTMLDirectoryElement {
        HTMLDirectoryElement {
            htmlelement:
                HTMLElement::new_inherited(local_name, prefix, document)
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(local_name: LocalName,
               prefix: Option<DOMString>,
               document: &Document) -> Root<HTMLDirectoryElement> {
        Node::reflect_node(box HTMLDirectoryElement::new_inherited(local_name, prefix, document),
                           document,
                           HTMLDirectoryElementBinding::Wrap)
    }
}
