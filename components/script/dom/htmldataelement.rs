/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use html5ever::{LocalName, Prefix};
use js::rust::HandleObject;

use crate::dom::bindings::codegen::Bindings::HTMLDataElementBinding::HTMLDataElementMethods;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::document::Document;
use crate::dom::htmlelement::HTMLElement;
use crate::dom::node::Node;

#[dom_struct]
pub struct HTMLDataElement {
    htmlelement: HTMLElement,
}

impl HTMLDataElement {
    fn new_inherited(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
    ) -> HTMLDataElement {
        HTMLDataElement {
            htmlelement: HTMLElement::new_inherited(local_name, prefix, document),
        }
    }

    #[allow(crown::unrooted_must_root)]
    pub fn new(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
        proto: Option<HandleObject>,
    ) -> DomRoot<HTMLDataElement> {
        Node::reflect_node_with_proto(
            Box::new(HTMLDataElement::new_inherited(local_name, prefix, document)),
            document,
            proto,
        )
    }
}

impl HTMLDataElementMethods for HTMLDataElement {
    // https://html.spec.whatwg.org/multipage/#dom-data-value
    make_getter!(Value, "value");

    // https://html.spec.whatwg.org/multipage/#dom-data-value
    make_setter!(SetValue, "value");
}
