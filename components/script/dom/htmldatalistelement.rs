/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use html5ever::{LocalName, Prefix};
use js::rust::HandleObject;

use crate::dom::bindings::codegen::Bindings::HTMLDataListElementBinding::HTMLDataListElementMethods;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::DomRoot;
use crate::dom::document::Document;
use crate::dom::htmlcollection::HTMLCollection;
use crate::dom::htmlelement::HTMLElement;
use crate::dom::htmloptionelement::HTMLOptionElement;
use crate::dom::node::{Node, NodeTraits};
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct HTMLDataListElement {
    htmlelement: HTMLElement,
}

impl HTMLDataListElement {
    fn new_inherited(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
    ) -> HTMLDataListElement {
        HTMLDataListElement {
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
    ) -> DomRoot<HTMLDataListElement> {
        Node::reflect_node_with_proto(
            Box::new(HTMLDataListElement::new_inherited(
                local_name, prefix, document,
            )),
            document,
            proto,
            can_gc,
        )
    }
}

impl HTMLDataListElementMethods<crate::DomTypeHolder> for HTMLDataListElement {
    // https://html.spec.whatwg.org/multipage/#dom-datalist-options
    fn Options(&self, can_gc: CanGc) -> DomRoot<HTMLCollection> {
        HTMLCollection::new_with_filter_fn(
            &self.owner_window(),
            self.upcast(),
            |element, _| element.is::<HTMLOptionElement>(),
            can_gc,
        )
    }
}
