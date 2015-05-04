/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLCollectionBinding;
use dom::bindings::codegen::Bindings::HTMLCollectionBinding::HTMLCollectionMethods;
use dom::bindings::codegen::InheritTypes::{ElementCast, NodeCast};
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JS, JSRef, Rootable, Temporary};
use dom::bindings::trace::JSTraceable;
use dom::bindings::utils::{Reflector, reflect_dom_object};
use dom::element::{Element, AttributeHandlers, ElementHelpers};
use dom::node::{Node, NodeHelpers, TreeIterator};
use dom::window::Window;
use util::namespace;
use util::str::{DOMString, split_html_space_chars};

use std::ascii::AsciiExt;
use std::iter::{FilterMap, Skip};
use string_cache::{Atom, Namespace};

pub trait CollectionFilter : JSTraceable {
    fn filter<'a>(&self, elem: JSRef<'a, Element>, root: JSRef<'a, Node>) -> bool;
}

#[jstraceable]
#[must_root]
pub enum CollectionTypeId {
    Static(Vec<JS<Element>>),
    Live(JS<Node>, Box<CollectionFilter+'static>)
}

#[dom_struct]
pub struct HTMLCollection {
    reflector_: Reflector,
    collection: CollectionTypeId,
}

impl HTMLCollection {
    fn new_inherited(collection: CollectionTypeId) -> HTMLCollection {
        HTMLCollection {
            reflector_: Reflector::new(),
            collection: collection,
        }
    }

    pub fn new(window: JSRef<Window>, collection: CollectionTypeId) -> Temporary<HTMLCollection> {
        reflect_dom_object(box HTMLCollection::new_inherited(collection),
                           GlobalRef::Window(window), HTMLCollectionBinding::Wrap)
    }
}

impl HTMLCollection {
    pub fn create(window: JSRef<Window>, root: JSRef<Node>,
                  filter: Box<CollectionFilter+'static>) -> Temporary<HTMLCollection> {
        HTMLCollection::new(window, CollectionTypeId::Live(JS::from_rooted(root), filter))
    }

    fn all_elements(window: JSRef<Window>, root: JSRef<Node>,
                    namespace_filter: Option<Namespace>) -> Temporary<HTMLCollection> {
        #[jstraceable]
        struct AllElementFilter {
            namespace_filter: Option<Namespace>
        }
        impl CollectionFilter for AllElementFilter {
            fn filter(&self, elem: JSRef<Element>, _root: JSRef<Node>) -> bool {
                match self.namespace_filter {
                    None => true,
                    Some(ref namespace) => *elem.namespace() == *namespace
                }
            }
        }
        let filter = AllElementFilter {namespace_filter: namespace_filter};
        HTMLCollection::create(window, root, box filter)
    }

    pub fn by_tag_name(window: JSRef<Window>, root: JSRef<Node>, tag: DOMString)
                       -> Temporary<HTMLCollection> {
        if tag == "*" {
            return HTMLCollection::all_elements(window, root, None);
        }

        #[jstraceable]
        struct TagNameFilter {
            tag: Atom,
            ascii_lower_tag: Atom,
        }
        impl CollectionFilter for TagNameFilter {
            fn filter(&self, elem: JSRef<Element>, _root: JSRef<Node>) -> bool {
                if elem.html_element_in_html_document() {
                    *elem.local_name() == self.ascii_lower_tag
                } else {
                    *elem.local_name() == self.tag
                }
            }
        }
        let filter = TagNameFilter {
            tag: Atom::from_slice(&tag),
            ascii_lower_tag: Atom::from_slice(&tag.to_ascii_lowercase()),
        };
        HTMLCollection::create(window, root, box filter)
    }

    pub fn by_tag_name_ns(window: JSRef<Window>, root: JSRef<Node>, tag: DOMString,
                          maybe_ns: Option<DOMString>) -> Temporary<HTMLCollection> {
        let namespace_filter = match maybe_ns {
            Some(ref namespace) if namespace == &"*" => None,
            ns => Some(namespace::from_domstring(ns)),
        };

        if tag == "*" {
            return HTMLCollection::all_elements(window, root, namespace_filter);
        }
        #[jstraceable]
        struct TagNameNSFilter {
            tag: Atom,
            namespace_filter: Option<Namespace>
        }
        impl CollectionFilter for TagNameNSFilter {
            fn filter(&self, elem: JSRef<Element>, _root: JSRef<Node>) -> bool {
                let ns_match = match self.namespace_filter {
                    Some(ref namespace) => {
                        *elem.namespace() == *namespace
                    },
                    None => true
                };
                ns_match && *elem.local_name() == self.tag
            }
        }
        let filter = TagNameNSFilter {
            tag: Atom::from_slice(&tag),
            namespace_filter: namespace_filter
        };
        HTMLCollection::create(window, root, box filter)
    }

    pub fn by_class_name(window: JSRef<Window>, root: JSRef<Node>, classes: DOMString)
                         -> Temporary<HTMLCollection> {
        #[jstraceable]
        struct ClassNameFilter {
            classes: Vec<Atom>
        }
        impl CollectionFilter for ClassNameFilter {
            fn filter(&self, elem: JSRef<Element>, _root: JSRef<Node>) -> bool {
                self.classes.iter().all(|class| elem.has_class(class))
            }
        }
        let filter = ClassNameFilter {
            classes: split_html_space_chars(&classes).map(|class| {
                         Atom::from_slice(class)
                     }).collect()
        };
        HTMLCollection::create(window, root, box filter)
    }

    pub fn children(window: JSRef<Window>, root: JSRef<Node>) -> Temporary<HTMLCollection> {
        #[jstraceable]
        struct ElementChildFilter;
        impl CollectionFilter for ElementChildFilter {
            fn filter(&self, elem: JSRef<Element>, root: JSRef<Node>) -> bool {
                root.is_parent_of(NodeCast::from_ref(elem))
            }
        }
        HTMLCollection::create(window, root, box ElementChildFilter)
    }

    fn traverse(root: JSRef<Node>)
                -> FilterMap<Skip<TreeIterator>,
                             fn(Temporary<Node>) -> Option<Temporary<Element>>> {
        fn to_temporary(node: Temporary<Node>) -> Option<Temporary<Element>> {
            ElementCast::to_temporary(node)
        }
        root.traverse_preorder()
            .skip(1)
            .filter_map(to_temporary as fn(Temporary<Node>) -> Option<Temporary<Element>>)
    }
}

impl<'a> HTMLCollectionMethods for JSRef<'a, HTMLCollection> {
    // https://dom.spec.whatwg.org/#dom-htmlcollection-length
    fn Length(self) -> u32 {
        match self.collection {
            CollectionTypeId::Static(ref elems) => elems.len() as u32,
            CollectionTypeId::Live(ref root, ref filter) => {
                let root = root.root();
                HTMLCollection::traverse(root.r())
                    .filter(|element| filter.filter(element.root().r(), root.r()))
                    .count() as u32
            }
        }
    }

    // https://dom.spec.whatwg.org/#dom-htmlcollection-item
    fn Item(self, index: u32) -> Option<Temporary<Element>> {
        let index = index as usize;
        match self.collection {
            CollectionTypeId::Static(ref elems) => elems
                .get(index)
                .map(|elem| Temporary::from_rooted(elem.clone())),
            CollectionTypeId::Live(ref root, ref filter) => {
                let root = root.root();
                HTMLCollection::traverse(root.r())
                    .filter(|element| filter.filter(element.root().r(), root.r()))
                    .nth(index)
            }
        }
    }

    // https://dom.spec.whatwg.org/#dom-htmlcollection-nameditem
    fn NamedItem(self, key: DOMString) -> Option<Temporary<Element>> {
        // Step 1.
        if key.is_empty() {
            return None;
        }

        // Step 2.
        match self.collection {
            CollectionTypeId::Static(ref elems) => elems.iter()
                .map(|elem| elem.root())
                .find(|elem| {
                    elem.r().get_string_attribute(&atom!("name")) == key ||
                    elem.r().get_string_attribute(&atom!("id")) == key })
                .map(|maybe_elem| Temporary::from_rooted(maybe_elem.r())),
            CollectionTypeId::Live(ref root, ref filter) => {
                let root = root.root();
                HTMLCollection::traverse(root.r())
                    .map(|element| element.root())
                    .filter(|element| filter.filter(element.r(), root.r()))
                    .find(|elem| {
                        elem.r().get_string_attribute(&atom!("name")) == key ||
                        elem.r().get_string_attribute(&atom!("id")) == key })
                    .map(|elem| Temporary::from_rooted(elem.r()))
            }
        }
    }

    fn IndexedGetter(self, index: u32, found: &mut bool) -> Option<Temporary<Element>> {
        let maybe_elem = self.Item(index);
        *found = maybe_elem.is_some();
        maybe_elem
    }

    fn NamedGetter(self, name: DOMString, found: &mut bool) -> Option<Temporary<Element>> {
        let maybe_elem = self.NamedItem(name);
        *found = maybe_elem.is_some();
        maybe_elem
    }
}

