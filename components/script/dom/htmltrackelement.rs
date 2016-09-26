/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLTrackElementBinding;
use dom::bindings::js::Root;
use dom::bindings::str::DOMString;
use dom::document::Document;
use dom::htmlelement::HTMLElement;
use dom::node::Node;
use string_cache::Atom;

#[dom_struct]
pub struct HTMLTrackElement {
    htmlelement: HTMLElement,
}

impl HTMLTrackElement {
    fn new_inherited(local_name: Atom, prefix: Option<DOMString>, document: &Document) -> HTMLTrackElement {
        HTMLTrackElement {
            htmlelement: HTMLElement::new_inherited(local_name, prefix, document)
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(local_name: Atom,
               prefix: Option<DOMString>,
               document: &Document) -> Root<HTMLTrackElement> {
        Node::reflect_node(box HTMLTrackElement::new_inherited(local_name, prefix, document),
                           document,
                           HTMLTrackElementBinding::Wrap)
    }
}
