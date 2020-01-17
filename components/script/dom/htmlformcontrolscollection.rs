/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::HTMLCollectionBinding::HTMLCollectionMethods;
use crate::dom::bindings::codegen::Bindings::HTMLFormControlsCollectionBinding;
use crate::dom::bindings::codegen::Bindings::HTMLFormControlsCollectionBinding::HTMLFormControlsCollectionMethods;
use crate::dom::bindings::codegen::UnionTypes::RadioNodeListOrElement;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::element::Element;
use crate::dom::htmlcollection::{CollectionFilter, HTMLCollection};
use crate::dom::htmlformelement::HTMLFormElement;
use crate::dom::node::Node;
use crate::dom::radionodelist::RadioNodeList;
use crate::dom::window::Window;
use dom_struct::dom_struct;

#[dom_struct]
pub struct HTMLFormControlsCollection {
    collection: HTMLCollection,
}

impl HTMLFormControlsCollection {
    fn new_inherited(
        root: &HTMLFormElement,
        filter: Box<dyn CollectionFilter + 'static>,
    ) -> HTMLFormControlsCollection {
        HTMLFormControlsCollection {
            collection: HTMLCollection::new_inherited(root.upcast::<Node>(), filter),
        }
    }

    pub fn new(
        window: &Window,
        root: &HTMLFormElement,
        filter: Box<dyn CollectionFilter + 'static>,
    ) -> DomRoot<HTMLFormControlsCollection> {
        reflect_dom_object(
            Box::new(HTMLFormControlsCollection::new_inherited(root, filter)),
            window,
            HTMLFormControlsCollectionBinding::Wrap,
        )
    }

    // FIXME: This shouldn't need to be implemented here since HTMLCollection (the parent of
    // HTMLFormControlsCollection) implements Length
    #[allow(non_snake_case)]
    pub fn Length(&self) -> u32 {
        self.collection.Length()
    }
}

impl HTMLFormControlsCollectionMethods for HTMLFormControlsCollection {
    // https://html.spec.whatwg.org/multipage/#dom-htmlformcontrolscollection-nameditem
    fn NamedItem(&self, name: DOMString) -> Option<RadioNodeListOrElement> {
        // Step 1
        if name.is_empty() {
            return None;
        }

        let mut filter_map = self.collection.elements_iter().filter_map(|elem| {
            if elem.get_string_attribute(&local_name!("name")) == name ||
                elem.get_string_attribute(&local_name!("id")) == name
            {
                Some(elem)
            } else {
                None
            }
        });

        if let Some(elem) = filter_map.next() {
            let mut peekable = filter_map.peekable();
            // Step 2
            if peekable.peek().is_none() {
                Some(RadioNodeListOrElement::Element(elem))
            } else {
                // Step 4-5
                let global = self.global();
                let window = global.as_window();
                // okay to unwrap: root's type was checked in the constructor
                let collection_root = self.collection.root_node();
                let form = collection_root.downcast::<HTMLFormElement>().unwrap();
                // There is only one way to get an HTMLCollection,
                // specifically HTMLFormElement::Elements(),
                // and the collection filter excludes image inputs.
                Some(RadioNodeListOrElement::RadioNodeList(
                    RadioNodeList::new_controls_except_image_inputs(window, form, name),
                ))
            }
        // Step 3
        } else {
            None
        }
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
