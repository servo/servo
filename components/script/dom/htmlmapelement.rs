/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLMapElementBinding;
use dom::bindings::inheritance::Castable;
use dom::bindings::root::DomRoot;
use dom::document::Document;
use dom::htmlareaelement::HTMLAreaElement;
use dom::htmlelement::HTMLElement;
use dom::node::Node;
use dom_struct::dom_struct;
use html5ever::{LocalName, Prefix};
use typeholder::TypeHolderTrait;

#[dom_struct]
pub struct HTMLMapElement<TH: TypeHolderTrait> {
    htmlelement: HTMLElement<TH>
}

impl<TH: TypeHolderTrait> HTMLMapElement<TH> {
    fn new_inherited(local_name: LocalName,
                     prefix: Option<Prefix>,
                     document: &Document<TH>) -> HTMLMapElement<TH> {
        HTMLMapElement {
            htmlelement: HTMLElement::new_inherited(local_name, prefix, document)
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(local_name: LocalName,
               prefix: Option<Prefix>,
               document: &Document<TH>) -> DomRoot<HTMLMapElement<TH>> {
        Node::<TH>::reflect_node(Box::new(HTMLMapElement::new_inherited(local_name, prefix, document)),
                           document,
                           HTMLMapElementBinding::Wrap)
    }

    pub fn get_area_elements(&self) -> Vec<DomRoot<HTMLAreaElement<TH>>> {
        self.upcast::<Node<TH>>()
            .traverse_preorder()
            .filter_map(DomRoot::downcast::<HTMLAreaElement<TH>>).collect()
    }
}
