/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use html5ever::{LocalName, Prefix};
use js::rust::HandleObject;

use crate::dom::bindings::codegen::Bindings::HTMLDivElementBinding::HTMLDivElementMethods;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::document::Document;
use crate::dom::html::htmlelement::HTMLElement;
use crate::dom::node::Node;

#[dom_struct]
pub(crate) struct HTMLDivElement {
    htmlelement: HTMLElement,
}

impl HTMLDivElement {
    fn new_inherited(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
    ) -> HTMLDivElement {
        HTMLDivElement {
            htmlelement: HTMLElement::new_inherited(local_name, prefix, document),
        }
    }

    pub(crate) fn new(
        cx: &mut js::context::JSContext,
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
        proto: Option<HandleObject>,
    ) -> DomRoot<HTMLDivElement> {
        Node::reflect_node_with_proto(
            cx,
            Box::new(HTMLDivElement::new_inherited(local_name, prefix, document)),
            document,
            proto,
        )
    }
}

impl HTMLDivElementMethods<crate::DomTypeHolder> for HTMLDivElement {
    // https://html.spec.whatwg.org/multipage/#dom-div-align
    make_getter!(Align, "align");

    // https://html.spec.whatwg.org/multipage/#dom-div-align
    make_setter!(SetAlign, "align");
}
