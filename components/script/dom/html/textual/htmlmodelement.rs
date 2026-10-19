/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use html5ever::{LocalName, Prefix};
use js::rust::HandleObject;

use crate::dom::bindings::codegen::Bindings::HTMLModElementBinding::HTMLModElementMethods;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::{DOMString, USVString};
use crate::dom::document::Document;
use crate::dom::html::htmlelement::HTMLElement;
use crate::dom::node::Node;

#[dom_struct]
pub(crate) struct HTMLModElement {
    htmlelement: HTMLElement,
}

impl HTMLModElement {
    fn new_inherited(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
    ) -> HTMLModElement {
        HTMLModElement {
            htmlelement: HTMLElement::new_inherited(local_name, prefix, document),
        }
    }

    pub(crate) fn new(
        cx: &mut js::context::JSContext,
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
        proto: Option<HandleObject>,
    ) -> DomRoot<HTMLModElement> {
        Node::reflect_node_with_proto(
            cx,
            Box::new(HTMLModElement::new_inherited(local_name, prefix, document)),
            document,
            proto,
        )
    }
}

impl HTMLModElementMethods<crate::DomTypeHolder> for HTMLModElement {
    // https://html.spec.whatwg.org/multipage/#dom-mod-cite
    make_url_getter!(Cite, "cite");

    // https://html.spec.whatwg.org/multipage/#dom-mod-cite
    make_url_setter!(SetCite, "cite");

    // https://html.spec.whatwg.org/multipage/#dom-mod-datetime
    make_getter!(DateTime, "datetime");

    // https://html.spec.whatwg.org/multipage/#dom-mod-datetime
    make_setter!(SetDateTime, "datetime");
}
