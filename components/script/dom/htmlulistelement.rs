/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use html5ever::{LocalName, Prefix};
use js::rust::HandleObject;

use crate::dom::bindings::codegen::Bindings::HTMLUListElementBinding::HTMLUListElementMethods;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::document::Document;
use crate::dom::htmlelement::HTMLElement;
use crate::dom::node::Node;

#[dom_struct]
pub struct HTMLUListElement {
    htmlelement: HTMLElement,
}

impl HTMLUListElement {
    fn new_inherited(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
    ) -> HTMLUListElement {
        HTMLUListElement {
            htmlelement: HTMLElement::new_inherited(local_name, prefix, document),
        }
    }

    #[allow(crown::unrooted_must_root)]
    pub fn new(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
        proto: Option<HandleObject>,
    ) -> DomRoot<HTMLUListElement> {
        Node::reflect_node_with_proto(
            Box::new(HTMLUListElement::new_inherited(
                local_name, prefix, document,
            )),
            document,
            proto,
        )
    }
}

impl HTMLUListElementMethods for HTMLUListElement {
    // https://html.spec.whatwg.org/multipage/#dom-ul-compact
    make_bool_getter!(Compact, "compact");

    // https://html.spec.whatwg.org/multipage/#dom-ul-compact
    make_bool_setter!(SetCompact, "compact");

    // https://html.spec.whatwg.org/multipage/#dom-ul-type
    make_getter!(Type, "type");

    // https://html.spec.whatwg.org/multipage/#dom-ul-type
    make_setter!(SetType, "type");
}
