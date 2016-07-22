/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLProgressElementBinding::{self, HTMLProgressElementMethods};
use dom::bindings::inheritance::Castable;
use dom::bindings::js::Root;
use dom::bindings::str::DOMString;
use dom::document::Document;
use dom::htmlelement::HTMLElement;
use dom::node::Node;
use dom::nodelist::NodeList;
use string_cache::Atom;

#[dom_struct]
pub struct HTMLProgressElement {
    htmlelement: HTMLElement,
}

impl HTMLProgressElement {
    fn new_inherited(localName: Atom,
                     prefix: Option<DOMString>,
                     document: &Document) -> HTMLProgressElement {
        HTMLProgressElement {
            htmlelement:
                HTMLElement::new_inherited(localName, prefix, document)
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: Atom,
               prefix: Option<DOMString>,
               document: &Document) -> Root<HTMLProgressElement> {
        Node::reflect_node(box HTMLProgressElement::new_inherited(localName, prefix, document),
                           document,
                           HTMLProgressElementBinding::Wrap)
    }
}

impl HTMLProgressElementMethods for HTMLProgressElement {
    // https://html.spec.whatwg.org/multipage/#dom-lfe-labels
    fn Labels(&self) -> Root<NodeList> {
        self.upcast::<HTMLElement>().labels()
    }
}
