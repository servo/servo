/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLCollectionBinding::HTMLCollectionMethods;
use dom::bindings::codegen::Bindings::HTMLFormControlsCollectionBinding;
use dom::bindings::codegen::Bindings::HTMLFormControlsCollectionBinding::HTMLFormControlsCollectionMethods;
use dom::bindings::codegen::UnionTypes::RadioNodeListOrElement;
use dom::bindings::reflector::{DomObject, reflect_dom_object};
use dom::bindings::root::DomRoot;
use dom::bindings::str::DOMString;
use dom::element::Element;
use dom::htmlcollection::{CollectionFilter, HTMLCollection};
use dom::node::Node;
use dom::radionodelist::RadioNodeList;
use dom::window::Window;
use dom_struct::dom_struct;
use std::iter;

#[dom_struct]
pub struct HTMLFormControlsCollection {
    collection: HTMLCollection,
}

impl HTMLFormControlsCollection {
    fn new_inherited(root: &Node, filter: Box<CollectionFilter + 'static>) -> HTMLFormControlsCollection {
        HTMLFormControlsCollection {
            collection: HTMLCollection::new_inherited(root, filter)
        }
    }

    pub fn new(window: &Window, root: &Node, filter: Box<CollectionFilter + 'static>)
        -> DomRoot<HTMLFormControlsCollection>
    {
        reflect_dom_object(Box::new(HTMLFormControlsCollection::new_inherited(root, filter)),
                           window,
                           HTMLFormControlsCollectionBinding::Wrap)
    }

    // FIXME: This shouldn't need to be implemented here since HTMLCollection (the parent of
    // HTMLFormControlsCollection) implements Length
    pub fn Length(&self) -> u32 {
        self.collection.Length()
    }
}

impl HTMLFormControlsCollectionMethods for HTMLFormControlsCollection {
    // https://html.spec.whatwg.org/multipage/#dom-htmlformcontrolscollection-nameditem
    fn NamedItem(&self, name: DOMString) -> Option<RadioNodeListOrElement> {
        // Step 1
        if name.is_empty() { return None; }

        let mut filter_map = self.collection.elements_iter().filter_map(|elem| {
            if elem.get_string_attribute(&local_name!("name")) == name
               || elem.get_string_attribute(&local_name!("id")) == name {
                Some(elem)
            } else { None }
        });

        if let Some(elem) = filter_map.next() {
            let mut peekable = filter_map.peekable();
            // Step 2
            if peekable.peek().is_none() {
                Some(RadioNodeListOrElement::Element(elem))
            } else {
                // Step 4-5
                let once = iter::once(DomRoot::upcast::<Node>(elem));
                let list = once.chain(peekable.map(DomRoot::upcast));
                let global = self.global();
                let window = global.as_window();
                Some(RadioNodeListOrElement::RadioNodeList(RadioNodeList::new_simple_list(window, list)))
            }
        // Step 3
        } else { None }

    }

    // https://html.spec.whatwg.org/multipage/#dom-htmlformcontrolscollection-nameditem
    fn NamedGetter(&self, name: DOMString) -> Option<RadioNodeListOrElement> {
        self.NamedItem(name)
    }

    // https://html.spec.whatwg.org/multipage/#the-htmlformcontrolscollection-interface:supported-property-names
    fn SupportedPropertyNames(&self) -> Vec<DOMString> {
        self.collection.SupportedPropertyNames()
    }

    // FIXME: This shouldn't need to be implemented here since HTMLCollection (the parent of
    // HTMLFormControlsCollection) implements IndexedGetter.
    // https://github.com/servo/servo/issues/5875
    //
    // https://dom.spec.whatwg.org/#dom-htmlcollection-item
    fn IndexedGetter(&self, index: u32) -> Option<DomRoot<Element>> {
        self.collection.IndexedGetter(index)
    }
}
