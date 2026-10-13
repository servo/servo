/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use html5ever::{LocalName, Prefix};
use js::rust::HandleObject;

use crate::dom::bindings::codegen::Bindings::HTMLFrameSetElementBinding::HTMLFrameSetElementMethods;
use crate::dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::DomRoot;
use crate::dom::document::Document;
use crate::dom::html::htmlelement::HTMLElement;
use crate::dom::node::{Node, NodeTraits};

#[dom_struct]
pub(crate) struct HTMLFrameSetElement {
    htmlelement: HTMLElement,
}

impl HTMLFrameSetElement {
    fn new_inherited(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
    ) -> HTMLFrameSetElement {
        HTMLFrameSetElement {
            htmlelement: HTMLElement::new_inherited(local_name, prefix, document),
        }
    }

    pub(crate) fn new(
        cx: &mut js::context::JSContext,
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
        proto: Option<HandleObject>,
    ) -> DomRoot<HTMLFrameSetElement> {
        let n = Node::reflect_node_with_proto(
            cx,
            Box::new(HTMLFrameSetElement::new_inherited(
                local_name, prefix, document,
            )),
            document,
            proto,
        );
        n.upcast::<Node>().set_weird_parser_insertion_mode();
        n
    }
}

impl HTMLFrameSetElementMethods<crate::DomTypeHolder> for HTMLFrameSetElement {
    // https://html.spec.whatwg.org/multipage/#windoweventhandlers
    window_event_handlers!(ForwardToWindow);
}
