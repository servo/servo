/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use html5ever::{LocalName, Prefix};
use js::rust::HandleObject;
use script_bindings::codegen::GenericBindings::HTMLFrameElementBinding::HTMLFrameElement_Binding::HTMLFrameElementMethods;

use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::USVString;
use crate::dom::document::Document;
use crate::dom::html::htmlelement::HTMLElement;
use crate::dom::node::Node;

#[dom_struct]
pub(crate) struct HTMLFrameElement {
    htmlelement: HTMLElement,
}

impl HTMLFrameElement {
    fn new_inherited(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
    ) -> HTMLFrameElement {
        HTMLFrameElement {
            htmlelement: HTMLElement::new_inherited(local_name, prefix, document),
        }
    }

    pub(crate) fn new(
        cx: &mut js::context::JSContext,
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
        proto: Option<HandleObject>,
    ) -> DomRoot<HTMLFrameElement> {
        Node::reflect_node_with_proto(
            cx,
            Box::new(HTMLFrameElement::new_inherited(
                local_name, prefix, document,
            )),
            document,
            proto,
        )
    }
}

impl HTMLFrameElementMethods<crate::DomTypeHolder> for HTMLFrameElement {
    // https://html.spec.whatwg.org/multipage/#dom-frame-longdesc
    make_url_getter!(LongDesc, "longdesc");

    // https://html.spec.whatwg.org/multipage/#dom-frame-longdesc
    make_url_setter!(SetLongDesc, "longdesc");
}
