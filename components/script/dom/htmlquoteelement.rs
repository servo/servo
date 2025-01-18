/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use html5ever::{LocalName, Prefix};
use js::rust::HandleObject;

use crate::dom::bindings::codegen::Bindings::HTMLQuoteElementBinding::HTMLQuoteElementMethods;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::USVString;
use crate::dom::document::Document;
use crate::dom::htmlelement::HTMLElement;
use crate::dom::node::Node;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct HTMLQuoteElement {
    htmlelement: HTMLElement,
}

impl HTMLQuoteElement {
    fn new_inherited(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
    ) -> HTMLQuoteElement {
        HTMLQuoteElement {
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
    ) -> DomRoot<HTMLQuoteElement> {
        Node::reflect_node_with_proto(
            Box::new(HTMLQuoteElement::new_inherited(
                local_name, prefix, document,
            )),
            document,
            proto,
            can_gc,
        )
    }
}

impl HTMLQuoteElementMethods<crate::DomTypeHolder> for HTMLQuoteElement {
    // https://html.spec.whatwg.org/multipage/#dom-quote-cite
    make_url_getter!(Cite, "cite");

    // https://html.spec.whatwg.org/multipage/#dom-quote-cite
    make_url_setter!(SetCite, "cite");
}
