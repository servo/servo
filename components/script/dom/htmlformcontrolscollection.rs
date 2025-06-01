/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use stylo_atoms::Atom;

use crate::dom::bindings::codegen::Bindings::HTMLCollectionBinding::HTMLCollectionMethods;
use crate::dom::bindings::codegen::Bindings::HTMLFormControlsCollectionBinding::HTMLFormControlsCollectionMethods;
use crate::dom::bindings::codegen::Bindings::NodeBinding::{GetRootNodeOptions, NodeMethods};
use crate::dom::bindings::codegen::UnionTypes::RadioNodeListOrElement;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::{DomGlobal, reflect_dom_object};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::element::Element;
use crate::dom::htmlcollection::{CollectionFilter, HTMLCollection};
use crate::dom::htmlformelement::HTMLFormElement;
use crate::dom::node::Node;
use crate::dom::radionodelist::RadioNodeList;
use crate::dom::window::Window;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct HTMLFormControlsCollection {
    collection: HTMLCollection,
    form: Dom<HTMLFormElement>,
}

impl HTMLFormControlsCollection {
    fn new_inherited(
        form: &HTMLFormElement,
        filter: Box<dyn CollectionFilter + 'static>,
    ) -> HTMLFormControlsCollection {
        let root_of_form = form
            .upcast::<Node>()
            .GetRootNode(&GetRootNodeOptions::empty());
        HTMLFormControlsCollection {
            collection: HTMLCollection::new_inherited(&root_of_form, filter),
            form: Dom::from_ref(form),
        }
    }

    pub(crate) fn new(
        window: &Window,
        form: &HTMLFormElement,
        filter: Box<dyn CollectionFilter + 'static>,
        can_gc: CanGc,
    ) -> DomRoot<HTMLFormControlsCollection> {
        reflect_dom_object(
            Box::new(HTMLFormControlsCollection::new_inherited(form, filter)),
            window,
            can_gc,
        )
    }
}

impl HTMLFormControlsCollectionMethods<crate::DomTypeHolder> for HTMLFormControlsCollection {
    // FIXME: This shouldn't need to be implemented here since HTMLCollection (the parent of
    // HTMLFormControlsCollection) implements Length
    // https://dom.spec.whatwg.org/#dom-htmlcollection-length
    fn Length(&self) -> u32 {
        self.collection.Length()
    }

    // https://html.spec.whatwg.org/multipage/#dom-htmlformcontrolscollection-nameditem
    fn NamedItem(&self, name: DOMString, can_gc: CanGc) -> Option<RadioNodeListOrElement> {
        // Step 1
        if name.is_empty() {
            return None;
        }

        let name = Atom::from(name);

        let mut filter_map = self.collection.elements_iter().filter_map(|elem| {
            if elem.get_name().is_some_and(|n| n == name) ||
                elem.get_id().is_some_and(|i| i == name)
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
                // There is only one way to get an HTMLCollection,
                // specifically HTMLFormElement::Elements(),
                // and the collection filter excludes image inputs.
                Some(RadioNodeListOrElement::RadioNodeList(
                    RadioNodeList::new_controls_except_image_inputs(
                        window, &self.form, &name, can_gc,
                    ),
                ))
            }
        // Step 3
        } else {
            None
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-htmlformcontrolscollection-nameditem
    fn NamedGetter(&self, name: DOMString, can_gc: CanGc) -> Option<RadioNodeListOrElement> {
        self.NamedItem(name, can_gc)
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
