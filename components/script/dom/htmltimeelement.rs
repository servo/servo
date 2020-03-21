/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::HTMLTimeElementBinding::HTMLTimeElementMethods;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::document::Document;
use crate::dom::htmlelement::HTMLElement;
use crate::dom::node::Node;
use dom_struct::dom_struct;
use html5ever::{LocalName, Prefix};

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

    #[allow(unrooted_must_root)]
    pub fn new(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
    ) -> DomRoot<HTMLTimeElement> {
        Node::reflect_node(
            Box::new(HTMLTimeElement::new_inherited(local_name, prefix, document)),
            document,
        )
    }
}

impl HTMLTimeElementMethods for HTMLTimeElement {
    // https://html.spec.whatwg.org/multipage/#dom-time-datetime
    make_getter!(DateTime, "datetime");

    // https://html.spec.whatwg.org/multipage/#dom-time-datetime
    make_setter!(SetDateTime, "datetime");
}
