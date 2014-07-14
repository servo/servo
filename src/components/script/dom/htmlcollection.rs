/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::InheritTypes::{ElementCast, NodeCast};
use dom::bindings::codegen::Bindings::HTMLCollectionBinding;
use dom::bindings::global::Window;
use dom::bindings::js::{JS, JSRef, Temporary};
use dom::bindings::utils::{Reflectable, Reflector, reflect_dom_object};
use dom::element::{Element, AttributeHandlers};
use dom::node::{Node, NodeHelpers};
use dom::window::Window;
use servo_util::atom::Atom;
use servo_util::namespace::Namespace;
use servo_util::str::{DOMString, split_html_space_chars};

use serialize::{Encoder, Encodable};

pub trait CollectionFilter {
    fn filter(&self, elem: &JSRef<Element>, root: &JSRef<Node>) -> bool;
}

impl<S: Encoder<E>, E> Encodable<S, E> for Box<CollectionFilter> {
    fn encode(&self, _s: &mut S) -> Result<(), E> {
        Ok(())
    }
}

#[deriving(Encodable)]
pub enum CollectionTypeId {
    Static(Vec<JS<Element>>),
    Live(JS<Node>, Box<CollectionFilter>)
}

#[deriving(Encodable)]
pub struct HTMLCollection {
    collection: CollectionTypeId,
    reflector_: Reflector,
}

impl HTMLCollection {
    pub fn new_inherited(collection: CollectionTypeId) -> HTMLCollection {
        HTMLCollection {
            collection: collection,
            reflector_: Reflector::new(),
        }
    }

    pub fn new(window: &JSRef<Window>, collection: CollectionTypeId) -> Temporary<HTMLCollection> {
        reflect_dom_object(box HTMLCollection::new_inherited(collection),
                           &Window(*window), HTMLCollectionBinding::Wrap)
    }
}

impl HTMLCollection {
    pub fn create(window: &JSRef<Window>, root: &JSRef<Node>,
                  filter: Box<CollectionFilter>) -> Temporary<HTMLCollection> {
        HTMLCollection::new(window, Live(JS::from_rooted(root), filter))
    }

    pub fn by_tag_name(window: &JSRef<Window>, root: &JSRef<Node>, tag: DOMString)
                       -> Temporary<HTMLCollection> {
        struct TagNameFilter {
            tag: Atom
        }
        impl CollectionFilter for TagNameFilter {
            fn filter(&self, elem: &JSRef<Element>, _root: &JSRef<Node>) -> bool {
                elem.deref().local_name == self.tag
            }
        }
        let filter = TagNameFilter {
            tag: Atom::from_slice(tag.as_slice())
        };
        HTMLCollection::create(window, root, box filter)
    }

    pub fn by_tag_name_ns(window: &JSRef<Window>, root: &JSRef<Node>, tag: DOMString,
                          namespace: Namespace) -> Temporary<HTMLCollection> {
        struct TagNameNSFilter {
            tag: Atom,
            namespace: Namespace
        }
        impl CollectionFilter for TagNameNSFilter {
            fn filter(&self, elem: &JSRef<Element>, _root: &JSRef<Node>) -> bool {
                elem.deref().namespace == self.namespace && elem.deref().local_name == self.tag
            }
        }
        let filter = TagNameNSFilter {
            tag: Atom::from_slice(tag.as_slice()),
            namespace: namespace
        };
        HTMLCollection::create(window, root, box filter)
    }

    pub fn by_class_name(window: &JSRef<Window>, root: &JSRef<Node>, classes: DOMString)
                         -> Temporary<HTMLCollection> {
        struct ClassNameFilter {
            classes: Vec<DOMString>
        }
        impl CollectionFilter for ClassNameFilter {
            fn filter(&self, elem: &JSRef<Element>, _root: &JSRef<Node>) -> bool {
                self.classes.iter().all(|class| elem.has_class(class.as_slice()))
            }
        }
        let filter = ClassNameFilter {
            classes: split_html_space_chars(classes.as_slice()).map(|class| class.to_string()).collect()
        };
        HTMLCollection::create(window, root, box filter)
    }

    pub fn children(window: &JSRef<Window>, root: &JSRef<Node>) -> Temporary<HTMLCollection> {
        struct ElementChildFilter;
        impl CollectionFilter for ElementChildFilter {
            fn filter(&self, elem: &JSRef<Element>, root: &JSRef<Node>) -> bool {
                root.is_parent_of(NodeCast::from_ref(elem))
            }
        }
        HTMLCollection::create(window, root, box ElementChildFilter)
    }
}

pub trait HTMLCollectionMethods {
    fn Length(&self) -> u32;
    fn Item(&self, index: u32) -> Option<Temporary<Element>>;
    fn NamedItem(&self, key: DOMString) -> Option<Temporary<Element>>;
    fn IndexedGetter(&self, index: u32, found: &mut bool) -> Option<Temporary<Element>>;
    fn NamedGetter(&self, maybe_name: Option<DOMString>, found: &mut bool) -> Option<Temporary<Element>>;
}

impl<'a> HTMLCollectionMethods for JSRef<'a, HTMLCollection> {
    // http://dom.spec.whatwg.org/#dom-htmlcollection-length
    fn Length(&self) -> u32 {
        match self.collection {
            Static(ref elems) => elems.len() as u32,
            Live(ref root, ref filter) => {
                let root = root.root();
                root.deref().traverse_preorder()
                    .filter(|&child| {
                        let elem: Option<&JSRef<Element>> = ElementCast::to_ref(&child);
                        elem.map_or(false, |elem| filter.filter(elem, &*root))
                    }).count() as u32
            }
        }
    }

    // http://dom.spec.whatwg.org/#dom-htmlcollection-item
    fn Item(&self, index: u32) -> Option<Temporary<Element>> {
        match self.collection {
            Static(ref elems) => elems
                .as_slice()
                .get(index as uint)
                .map(|elem| Temporary::new(elem.clone())),
            Live(ref root, ref filter) => {
                let root = root.root();
                root.deref().traverse_preorder()
                    .filter_map(|node| {
                        let elem: Option<&JSRef<Element>> = ElementCast::to_ref(&node);
                        elem.filtered(|&elem| filter.filter(elem, &*root))
                            .map(|elem| elem.clone())
                    })
                    .nth(index as uint)
                    .clone()
                    .map(|elem| Temporary::from_rooted(&elem))
            }
        }
    }

    // http://dom.spec.whatwg.org/#dom-htmlcollection-nameditem
    fn NamedItem(&self, key: DOMString) -> Option<Temporary<Element>> {
        // Step 1.
        if key.is_empty() {
            return None;
        }

        // Step 2.
        match self.collection {
            Static(ref elems) => elems.iter()
                .map(|elem| elem.root())
                .find(|elem| {
                    elem.get_string_attribute("name") == key ||
                    elem.get_string_attribute("id") == key })
                .map(|maybe_elem| Temporary::from_rooted(&*maybe_elem)),
            Live(ref root, ref filter) => {
                let root = root.root();
                root.deref().traverse_preorder()
                    .filter_map(|node| {
                        let elem: Option<&JSRef<Element>> = ElementCast::to_ref(&node);
                        elem.filtered(|&elem| filter.filter(elem, &*root))
                            .map(|elem| elem.clone())
                    })
                    .find(|elem| {
                        elem.get_string_attribute("name") == key ||
                        elem.get_string_attribute("id") == key })
                    .map(|maybe_elem| Temporary::from_rooted(&maybe_elem))
            }
        }
    }

    fn IndexedGetter(&self, index: u32, found: &mut bool) -> Option<Temporary<Element>> {
        let maybe_elem = self.Item(index);
        *found = maybe_elem.is_some();
        maybe_elem
    }

    fn NamedGetter(&self, maybe_name: Option<DOMString>, found: &mut bool) -> Option<Temporary<Element>> {
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
}
