/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLCollectionBinding::HTMLCollectionMethods;
use dom::bindings::codegen::Bindings::HTMLFormControlsCollectionBinding;
use dom::bindings::codegen::Bindings::HTMLFormControlsCollectionBinding::HTMLFormControlsCollectionMethods;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::JS;
use dom::bindings::js::Root;
use dom::bindings::utils::reflect_dom_object;
use dom::element::Element;
use dom::htmlcollection::{Collection, CollectionFilter, HTMLCollection};
use dom::node::Node;
use dom::window::Window;
use util::str::{DOMString, StaticStringVec};

// https://html.spec.whatwg.org/multipage/#category-listed
static LISTED_ELEMENTS: StaticStringVec = &[
    "button", "fieldset", "input", "keygen", "object", "output", "select", "textarea"];

#[derive(JSTraceable, HeapSizeOf)]
struct ElementsFilter;
impl CollectionFilter for ElementsFilter {
    fn filter<'a>(&self, elem: &'a Element, _root: &'a Node) -> bool {
        LISTED_ELEMENTS.iter().any(|&tag_name| tag_name == &**elem.local_name())
    }
}


#[dom_struct]
pub struct HTMLFormControlsCollection {
    collection: HTMLCollection,
}

impl HTMLFormControlsCollection {
    fn new_inherited(root: &Node) -> HTMLFormControlsCollection {
        HTMLFormControlsCollection {
            collection: HTMLCollection::new_inherited(
                Collection(JS::from_ref(root),
                box ElementsFilter))
        }
    }

    pub fn new(window: &Window, root: &Node) -> Root<HTMLFormControlsCollection> {
        reflect_dom_object(box HTMLFormControlsCollection::new_inherited(root),
                           GlobalRef::Window(window),
                           HTMLFormControlsCollectionBinding::Wrap)
    }

    pub fn Length(&self) -> u32 {
        self.collection.Length()
    }
}

impl HTMLFormControlsCollectionMethods for HTMLFormControlsCollection {
    // check-tidy: no specs after this line
    fn NamedGetter(&self, name: DOMString, found: &mut bool) -> Option<Root<Element>> {
        self.collection.NamedGetter(name, found)
    }

    fn SupportedPropertyNames(&self) -> Vec<DOMString> {
        self.collection.SupportedPropertyNames()
    }

    fn IndexedGetter(&self, index: u32, found: &mut bool) -> Option<Root<Element>> {
        self.collection.IndexedGetter(index, found)
    }
}
