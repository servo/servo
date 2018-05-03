/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLFrameSetElementBinding;
use dom::bindings::codegen::Bindings::HTMLFrameSetElementBinding::HTMLFrameSetElementMethods;
use dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use dom::bindings::root::DomRoot;
use dom::document::Document;
use dom::htmlelement::HTMLElement;
use dom::node::{Node, document_from_node};
use dom_struct::dom_struct;
use html5ever::{LocalName, Prefix};
use typeholder::TypeHolderTrait;

#[dom_struct]
pub struct HTMLFrameSetElement<TH: TypeHolderTrait> {
    htmlelement: HTMLElement<TH>
}

impl<TH: TypeHolderTrait> HTMLFrameSetElement<TH> {
    fn new_inherited(local_name: LocalName,
                     prefix: Option<Prefix>,
                     document: &Document<TH>) -> HTMLFrameSetElement<TH> {
        HTMLFrameSetElement {
            htmlelement:
                HTMLElement::new_inherited(local_name, prefix, document)
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(local_name: LocalName,
               prefix: Option<Prefix>,
               document: &Document<TH>) -> DomRoot<HTMLFrameSetElement<TH>> {
        Node::<TH>::reflect_node(Box::new(HTMLFrameSetElement::new_inherited(local_name, prefix, document)),
                           document,
                           HTMLFrameSetElementBinding::Wrap)
    }
}

impl<TH: TypeHolderTrait> HTMLFrameSetElementMethods<TH> for HTMLFrameSetElement<TH> {
    // https://html.spec.whatwg.org/multipage/#windoweventhandlers
    window_event_handlers!(ForwardToWindow);
}
