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
use crate::dom::htmlelement::HTMLElement;
use crate::dom::node::{Node, NodeTraits};
use crate::script_runtime::CanGc;

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

    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    pub(crate) fn new(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
        proto: Option<HandleObject>,
        can_gc: CanGc,
    ) -> DomRoot<HTMLFrameSetElement> {
        let n = Node::reflect_node_with_proto(
            Box::new(HTMLFrameSetElement::new_inherited(
                local_name, prefix, document,
            )),
            document,
            proto,
            can_gc,
        );
        n.upcast::<Node>().set_weird_parser_insertion_mode();
        n
    }
}

impl HTMLFrameSetElementMethods<crate::DomTypeHolder> for HTMLFrameSetElement {
    // https://html.spec.whatwg.org/multipage/#windoweventhandlers
    window_event_handlers!(ForwardToWindow);
}
