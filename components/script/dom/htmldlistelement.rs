/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLDListElementBinding;
use dom::bindings::root::DomRoot;
use dom::document::Document;
use dom::htmlelement::HTMLElement;
use dom::node::Node;
use dom_struct::dom_struct;
use html5ever::{LocalName, Prefix};
use typeholder::TypeHolderTrait;

#[dom_struct]
pub struct HTMLDListElement<TH: TypeHolderTrait> {
    htmlelement: HTMLElement<TH>
}

impl<TH: TypeHolderTrait> HTMLDListElement<TH> {
    fn new_inherited(local_name: LocalName, prefix: Option<Prefix>, document: &Document<TH>) -> HTMLDListElement<TH> {
        HTMLDListElement {
            htmlelement:
                HTMLElement::new_inherited(local_name, prefix, document)
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(local_name: LocalName,
               prefix: Option<Prefix>,
               document: &Document<TH>) -> DomRoot<HTMLDListElement<TH>> {
        Node::<TH>::reflect_node(Box::new(HTMLDListElement::new_inherited(local_name, prefix, document)),
                           document,
                           HTMLDListElementBinding::Wrap)
    }
}
