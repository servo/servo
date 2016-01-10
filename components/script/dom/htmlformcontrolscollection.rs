/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLCollectionBinding::HTMLCollectionMethods;
use dom::bindings::codegen::Bindings::HTMLFormControlsCollectionBinding;
use dom::bindings::codegen::Bindings::HTMLFormControlsCollectionBinding::HTMLFormControlsCollectionMethods;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::Root;
use dom::bindings::reflector::reflect_dom_object;
use dom::element::Element;
use dom::htmlcollection::{CollectionFilter, HTMLCollection};
use dom::node::Node;
use dom::window::Window;
use util::str::DOMString;

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
    fn NamedGetter(&self, name: DOMString, found: &mut bool) -> Option<Root<Element>> {
        self.collection.NamedGetter(name, found)
    }

    // https://html.spec.whatwg.org/multipage/#the-htmlformcontrolscollection-interface:supported-property-names
    fn SupportedPropertyNames(&self) -> Vec<DOMString> {
        self.collection.SupportedPropertyNames()
    }

    // FIXME: This shouldn't need to be implemented here since HTMLCollection (the parent of
    // HTMLFormControlsCollection) implements IndexedGetter
    //
    // https://dom.spec.whatwg.org/#dom-htmlcollection-item
    fn IndexedGetter(&self, index: u32, found: &mut bool) -> Option<Root<Element>> {
        self.collection.IndexedGetter(index, found)
    }
}
