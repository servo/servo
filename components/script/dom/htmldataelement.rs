/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLDataElementBinding;
use dom::bindings::codegen::Bindings::HTMLDataElementBinding::HTMLDataElementMethods;
use dom::bindings::js::Root;
use dom::bindings::str::DOMString;
use dom::document::Document;
use dom::htmlelement::HTMLElement;
use dom::node::Node;
use dom_struct::dom_struct;
use html5ever_atoms::LocalName;

#[dom_struct]
pub struct HTMLDataElement {
    htmlelement: HTMLElement
}

impl HTMLDataElement {
    fn new_inherited(local_name: LocalName,
                     prefix: Option<DOMString>,
                     document: &Document) -> HTMLDataElement {
        HTMLDataElement {
            htmlelement: HTMLElement::new_inherited(local_name, prefix, document)
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(local_name: LocalName,
               prefix: Option<DOMString>,
               document: &Document) -> Root<HTMLDataElement> {
        Node::reflect_node(box HTMLDataElement::new_inherited(local_name, prefix, document),
                           document,
                           HTMLDataElementBinding::Wrap)
    }
}

impl HTMLDataElementMethods for HTMLDataElement {
    // https://html.spec.whatwg.org/multipage/#dom-data-value
    make_getter!(Value, "value");

    // https://html.spec.whatwg.org/multipage/#dom-data-value
    make_setter!(SetValue, "value");
}
