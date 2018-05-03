/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLDataListElementBinding;
use dom::bindings::codegen::Bindings::HTMLDataListElementBinding::HTMLDataListElementMethods;
use dom::bindings::inheritance::Castable;
use dom::bindings::root::DomRoot;
use dom::document::Document;
use dom::element::Element;
use dom::htmlcollection::{CollectionFilter, HTMLCollection};
use dom::htmlelement::HTMLElement;
use dom::htmloptionelement::HTMLOptionElement;
use dom::node::{Node, window_from_node};
use dom_struct::dom_struct;
use html5ever::{LocalName, Prefix};
use std::marker::PhantomData;
use typeholder::TypeHolderTrait;

#[dom_struct]
pub struct HTMLDataListElement<TH: TypeHolderTrait> {
    htmlelement: HTMLElement<TH>
}

impl<TH: TypeHolderTrait> HTMLDataListElement<TH> {
    fn new_inherited(local_name: LocalName,
                     prefix: Option<Prefix>,
                     document: &Document<TH>) -> HTMLDataListElement<TH> {
        HTMLDataListElement {
            htmlelement:
                HTMLElement::new_inherited(local_name, prefix, document)
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(local_name: LocalName,
               prefix: Option<Prefix>,
               document: &Document<TH>) -> DomRoot<HTMLDataListElement<TH>> {
        Node::<TH>::reflect_node(Box::new(HTMLDataListElement::new_inherited(local_name, prefix, document)),
                           document,
                           HTMLDataListElementBinding::Wrap)
    }
}

impl<TH: TypeHolderTrait> HTMLDataListElementMethods<TH> for HTMLDataListElement<TH> {
    // https://html.spec.whatwg.org/multipage/#dom-datalist-options
    fn Options(&self) -> DomRoot<HTMLCollection<TH>> {
        #[derive(JSTraceable, MallocSizeOf)]
        struct HTMLDataListOptionsFilter<THH: TypeHolderTrait + 'static>(PhantomData<THH>);
        impl<THH: TypeHolderTrait> CollectionFilter<THH> for HTMLDataListOptionsFilter<THH> {
            fn filter(&self, elem: &Element<THH>, _root: &Node<THH>) -> bool {
                elem.is::<HTMLOptionElement<THH>>()
            }
        }
        let filter = Box::new(HTMLDataListOptionsFilter(Default::default()));
        let window = window_from_node(self);
        HTMLCollection::create(&window, self.upcast(), filter)
    }
}
