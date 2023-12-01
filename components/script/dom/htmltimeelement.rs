/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use html5ever::{LocalName, Prefix};
use js::rust::HandleObject;

use crate::dom::bindings::codegen::Bindings::HTMLTimeElementBinding::HTMLTimeElementMethods;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::document::Document;
use crate::dom::htmlelement::HTMLElement;
use crate::dom::node::Node;

#[dom_struct]
pub struct HTMLTimeElement {
    htmlelement: HTMLElement,
}

impl HTMLTimeElement {
    fn new_inherited(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
    ) -> HTMLTimeElement {
        HTMLTimeElement {
            htmlelement: HTMLElement::new_inherited(local_name, prefix, document),
        }
    }

    #[allow(crown::unrooted_must_root)]
    pub fn new(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
        proto: Option<HandleObject>,
    ) -> DomRoot<HTMLTimeElement> {
        Node::reflect_node_with_proto(
            Box::new(HTMLTimeElement::new_inherited(local_name, prefix, document)),
            document,
            proto,
        )
    }
}

impl HTMLTimeElementMethods for HTMLTimeElement {
    // https://html.spec.whatwg.org/multipage/#dom-time-datetime
    make_getter!(DateTime, "datetime");

    // https://html.spec.whatwg.org/multipage/#dom-time-datetime
    make_setter!(SetDateTime, "datetime");
}
