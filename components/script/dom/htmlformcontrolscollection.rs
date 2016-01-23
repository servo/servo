/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLCollectionBinding::HTMLCollectionMethods;
use dom::bindings::codegen::Bindings::HTMLFormControlsCollectionBinding;
use dom::bindings::codegen::Bindings::HTMLFormControlsCollectionBinding::HTMLFormControlsCollectionMethods;
use dom::bindings::codegen::UnionTypes::RadioNodeListOrElement::{self, eElement, eRadioNodeList};
use dom::bindings::global::GlobalRef;
use dom::bindings::inheritance::Castable;
use dom::bindings::js::Root;
use dom::bindings::reflector::{Reflectable, reflect_dom_object};
use dom::element::Element;
use dom::htmlcollection::{CollectionFilter, HTMLCollection};
use dom::node::Node;
use dom::radionodelist::RadioNodeList;
use dom::window::Window;
use util::str::DOMString;
use std::vec::Vec;

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
        -> Root<HTMLFormControlsCollection>
    {
        reflect_dom_object(box HTMLFormControlsCollection::new_inherited(root, filter),
                           GlobalRef::Window(window),
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

        let count = self.collection.elements_iter().filter_map(|elem| {
            if elem.get_string_attribute(&atom!("name")) == name
               || elem.get_string_attribute(&atom!("id")) == name {
                Some(elem)
            } else { None }
        }).count();
        let mut filter_map = self.collection.elements_iter().filter_map(|elem| {
            if elem.get_string_attribute(&atom!("name")) == name
               || elem.get_string_attribute(&atom!("id")) == name {
                Some(elem)
            } else { None }
        });
        // Step 2
        if count == 1 { return Some(eElement(filter_map.next().unwrap())); }
        // Step 3
        if count == 0 { return None; }

        // Step 4-5
        let list = filter_map.map(Root::upcast);
        let global = self.global();
        let global = global.r();
        let window = global.as_window();
        Some(eRadioNodeList(RadioNodeList::new_simple_list(window, list)))
    }

    // https://html.spec.whatwg.org/multipage/#dom-htmlformcontrolscollection-nameditem
    fn NamedGetter(&self, name: DOMString, found: &mut bool) -> Option<RadioNodeListOrElement> {
        let maybe_elem = self.NamedItem(name);
        *found = maybe_elem.is_some();
        maybe_elem
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
    fn IndexedGetter(&self, index: u32, found: &mut bool) -> Option<Root<Element>> {
        self.collection.IndexedGetter(index, found)
    }
}
