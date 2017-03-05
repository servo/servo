/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLDataListElementBinding;
use dom::bindings::codegen::Bindings::HTMLDataListElementBinding::HTMLDataListElementMethods;
use dom::bindings::inheritance::Castable;
use dom::bindings::js::Root;
use dom::bindings::str::DOMString;
use dom::document::Document;
use dom::element::Element;
use dom::htmlcollection::{CollectionFilter, HTMLCollection};
use dom::htmlelement::HTMLElement;
use dom::htmloptionelement::HTMLOptionElement;
use dom::node::{Node, window_from_node};
use dom_struct::dom_struct;
use html5ever_atoms::LocalName;

#[dom_struct]
pub struct HTMLDataListElement {
    htmlelement: HTMLElement
}

impl HTMLDataListElement {
    fn new_inherited(local_name: LocalName,
                     prefix: Option<DOMString>,
                     document: &Document) -> HTMLDataListElement {
        HTMLDataListElement {
            htmlelement:
                HTMLElement::new_inherited(local_name, prefix, document)
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(local_name: LocalName,
               prefix: Option<DOMString>,
               document: &Document) -> Root<HTMLDataListElement> {
        Node::reflect_node(box HTMLDataListElement::new_inherited(local_name, prefix, document),
                           document,
                           HTMLDataListElementBinding::Wrap)
    }
}

impl HTMLDataListElementMethods for HTMLDataListElement {
    // https://html.spec.whatwg.org/multipage/#dom-datalist-options
    fn Options(&self) -> Root<HTMLCollection> {
        #[derive(JSTraceable, HeapSizeOf)]
        struct HTMLDataListOptionsFilter;
        impl CollectionFilter for HTMLDataListOptionsFilter {
            fn filter(&self, elem: &Element, _root: &Node) -> bool {
                elem.is::<HTMLOptionElement>()
            }
        }
        let filter = box HTMLDataListOptionsFilter;
        let window = window_from_node(self);
        HTMLCollection::create(&window, self.upcast(), filter)
    }
}
