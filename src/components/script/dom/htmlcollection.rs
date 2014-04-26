/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::InheritTypes::{ElementCast, NodeCast};
use dom::bindings::codegen::HTMLCollectionBinding;
use dom::bindings::js::JS;
use dom::bindings::utils::{Reflectable, Reflector, reflect_dom_object};
use dom::element::{Element, AttributeHandlers};
use dom::node::{Node, NodeHelpers};
use dom::window::Window;
use servo_util::namespace::Namespace;
use servo_util::str::{DOMString, split_html_space_chars};

use serialize::{Encoder, Encodable};

pub trait CollectionFilter {
    fn filter(&self, elem: &JS<Element>, root: &JS<Node>) -> bool;
}

impl<S: Encoder<E>, E> Encodable<S, E> for ~CollectionFilter {
    fn encode(&self, _s: &mut S) -> Result<(), E> {
        Ok(())
    }
}

#[deriving(Encodable)]
pub enum CollectionTypeId {
    Static(Vec<JS<Element>>),
    Live(JS<Node>, ~CollectionFilter)
}

#[deriving(Encodable)]
pub struct HTMLCollection {
    pub collection: CollectionTypeId,
    pub reflector_: Reflector,
    pub window: JS<Window>,
}

impl HTMLCollection {
    pub fn new_inherited(window: JS<Window>, collection: CollectionTypeId) -> HTMLCollection {
        HTMLCollection {
            collection: collection,
            reflector_: Reflector::new(),
            window: window,
        }
    }

    pub fn new(window: &JS<Window>, collection: CollectionTypeId) -> JS<HTMLCollection> {
        reflect_dom_object(~HTMLCollection::new_inherited(window.clone(), collection),
                           window, HTMLCollectionBinding::Wrap)
    }
}

impl HTMLCollection {
    pub fn create(window: &JS<Window>, root: &JS<Node>, filter: ~CollectionFilter) -> JS<HTMLCollection> {
        HTMLCollection::new(window, Live(root.clone(), filter))
    }

    pub fn by_tag_name(window: &JS<Window>, root: &JS<Node>, tag: DOMString)
                       -> JS<HTMLCollection> {
        struct TagNameFilter {
            tag: DOMString
        }
        impl CollectionFilter for TagNameFilter {
            fn filter(&self, elem: &JS<Element>, _root: &JS<Node>) -> bool {
                elem.get().local_name == self.tag
            }
        }
        let filter = TagNameFilter {
            tag: tag
        };
        HTMLCollection::create(window, root, ~filter)
    }

    pub fn by_tag_name_ns(window: &JS<Window>, root: &JS<Node>, tag: DOMString,
                          namespace: Namespace) -> JS<HTMLCollection> {
        struct TagNameNSFilter {
            tag: DOMString,
            namespace: Namespace
        }
        impl CollectionFilter for TagNameNSFilter {
            fn filter(&self, elem: &JS<Element>, _root: &JS<Node>) -> bool {
                elem.get().namespace == self.namespace && elem.get().local_name == self.tag
            }
        }
        let filter = TagNameNSFilter {
            tag: tag,
            namespace: namespace
        };
        HTMLCollection::create(window, root, ~filter)
    }

    pub fn by_class_name(window: &JS<Window>, root: &JS<Node>, classes: DOMString)
                         -> JS<HTMLCollection> {
        struct ClassNameFilter {
            classes: Vec<DOMString>
        }
        impl CollectionFilter for ClassNameFilter {
            fn filter(&self, elem: &JS<Element>, _root: &JS<Node>) -> bool {
                self.classes.iter().all(|class| elem.has_class(*class))
            }
        }
        let filter = ClassNameFilter {
            classes: split_html_space_chars(classes).map(|class| class.into_owned()).collect()
        };
        HTMLCollection::create(window, root, ~filter)
    }

    pub fn children(window: &JS<Window>, root: &JS<Node>) -> JS<HTMLCollection> {
        struct ElementChildFilter;
        impl CollectionFilter for ElementChildFilter {
            fn filter(&self, elem: &JS<Element>, root: &JS<Node>) -> bool {
                root.is_parent_of(&NodeCast::from(elem))
            }
        }
        HTMLCollection::create(window, root, ~ElementChildFilter)
    }
}

impl HTMLCollection {
    // http://dom.spec.whatwg.org/#dom-htmlcollection-length
    pub fn Length(&self) -> u32 {
        match self.collection {
            Static(ref elems) => elems.len() as u32,
            Live(ref root, ref filter) => root.traverse_preorder()
                .count(|child| {
                    let elem: Option<JS<Element>> = ElementCast::to(&child);
                    elem.map_or(false, |elem| filter.filter(&elem, root))
                }) as u32
        }
    }

    // http://dom.spec.whatwg.org/#dom-htmlcollection-item
    pub fn Item(&self, index: u32) -> Option<JS<Element>> {
        match self.collection {
            Static(ref elems) => elems
                .as_slice()
                .get(index as uint)
                .map(|elem| elem.clone()),
            Live(ref root, ref filter) => root.traverse_preorder()
                .filter_map(|node| ElementCast::to(&node))
                .filter(|elem| filter.filter(elem, root))
                .nth(index as uint).clone()
        }
    }

    // http://dom.spec.whatwg.org/#dom-htmlcollection-nameditem
    pub fn NamedItem(&self, key: DOMString) -> Option<JS<Element>> {
        // Step 1.
        if key.is_empty() {
            return None;
        }

        // Step 2.
        match self.collection {
            Static(ref elems) => elems.iter()
                .find(|elem| {
                    elem.get_string_attribute("name") == key ||
                    elem.get_string_attribute("id") == key })
                .map(|maybe_elem| maybe_elem.clone()),
            Live(ref root, ref filter) => root.traverse_preorder()
                .filter_map(|node| ElementCast::to(&node))
                .filter(|elem| filter.filter(elem, root))
                .find(|elem| {
                    elem.get_string_attribute("name") == key ||
                    elem.get_string_attribute("id") == key })
                .map(|maybe_elem| maybe_elem.clone())
        }
    }
}

impl HTMLCollection {
    pub fn IndexedGetter(&self, index: u32, found: &mut bool) -> Option<JS<Element>> {
        let maybe_elem = self.Item(index);
        *found = maybe_elem.is_some();
        maybe_elem
    }

    pub fn NamedGetter(&self, maybe_name: Option<DOMString>, found: &mut bool) -> Option<JS<Element>> {
        match maybe_name {
            Some(name) => {
                let maybe_elem = self.NamedItem(name);
                *found = maybe_elem.is_some();
                maybe_elem
            },
            None => {
                *found = false;
                None
            }
        }
    }
}

impl Reflectable for HTMLCollection {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        &self.reflector_
    }

    fn mut_reflector<'a>(&'a mut self) -> &'a mut Reflector {
        &mut self.reflector_
    }
}
