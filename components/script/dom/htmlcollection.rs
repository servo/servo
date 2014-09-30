/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLCollectionBinding;
use dom::bindings::codegen::Bindings::HTMLCollectionBinding::HTMLCollectionMethods;
use dom::bindings::codegen::InheritTypes::{ElementCast, NodeCast};
use dom::bindings::global;
use dom::bindings::js::{JS, JSRef, Temporary};
use dom::bindings::trace::JSTraceable;
use dom::bindings::utils::{Reflectable, Reflector, reflect_dom_object};
use dom::element::{Element, AttributeHandlers, ElementHelpers};
use dom::node::{Node, NodeHelpers};
use dom::window::Window;
use servo_util::namespace;
use servo_util::str::{DOMString, split_html_space_chars};

use std::ascii::StrAsciiExt;
use string_cache::{Atom, Namespace};

pub trait CollectionFilter : JSTraceable {
    fn filter(&self, elem: JSRef<Element>, root: JSRef<Node>) -> bool;
}

#[jstraceable]
#[must_root]
pub enum CollectionTypeId {
    Static(Vec<JS<Element>>),
    Live(JS<Node>, Box<CollectionFilter+'static>)
}

#[jstraceable]
#[must_root]
pub struct HTMLCollection {
    collection: CollectionTypeId,
    reflector_: Reflector,
}

impl HTMLCollection {
    fn new_inherited(collection: CollectionTypeId) -> HTMLCollection {
        HTMLCollection {
            collection: collection,
            reflector_: Reflector::new(),
        }
    }

    pub fn new(window: JSRef<Window>, collection: CollectionTypeId) -> Temporary<HTMLCollection> {
        reflect_dom_object(box HTMLCollection::new_inherited(collection),
                           &global::Window(window), HTMLCollectionBinding::Wrap)
    }
}

impl HTMLCollection {
    pub fn create(window: JSRef<Window>, root: JSRef<Node>,
                  filter: Box<CollectionFilter+'static>) -> Temporary<HTMLCollection> {
        HTMLCollection::new(window, Live(JS::from_rooted(root), filter))
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
                    Some(ref namespace) => elem.namespace == *namespace
                }
            }
        }
        let filter = AllElementFilter {namespace_filter: namespace_filter};
        HTMLCollection::create(window, root, box filter)
    }

    pub fn by_tag_name(window: JSRef<Window>, root: JSRef<Node>, tag: DOMString)
                       -> Temporary<HTMLCollection> {
        if tag.as_slice() == "*" {
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
                    elem.local_name == self.ascii_lower_tag
                } else {
                    elem.local_name == self.tag
                }
            }
        }
        let filter = TagNameFilter {
            tag: Atom::from_slice(tag.as_slice()),
            ascii_lower_tag: Atom::from_slice(tag.as_slice().to_ascii_lower().as_slice()),
        };
        HTMLCollection::create(window, root, box filter)
    }

    pub fn by_tag_name_ns(window: JSRef<Window>, root: JSRef<Node>, tag: DOMString,
                          maybe_ns: Option<DOMString>) -> Temporary<HTMLCollection> {
        let namespace_filter = match maybe_ns {
            Some(ref namespace) if namespace.as_slice() == "*" => None,
            ns => Some(namespace::from_domstring(ns)),
        };

        if tag.as_slice() == "*" {
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
                        elem.deref().namespace == *namespace
                    },
                    None => true
                };
                ns_match && elem.deref().local_name == self.tag
            }
        }
        let filter = TagNameNSFilter {
            tag: Atom::from_slice(tag.as_slice()),
            namespace_filter: namespace_filter
        };
        HTMLCollection::create(window, root, box filter)
    }

    pub fn by_class_name(window: JSRef<Window>, root: JSRef<Node>, classes: DOMString)
                         -> Temporary<HTMLCollection> {
        #[jstraceable]
        struct ClassNameFilter {
            classes: Vec<DOMString>
        }
        impl CollectionFilter for ClassNameFilter {
            fn filter(&self, elem: JSRef<Element>, _root: JSRef<Node>) -> bool {
                self.classes.iter().all(|class| elem.has_class(class.as_slice()))
            }
        }
        let filter = ClassNameFilter {
            classes: split_html_space_chars(classes.as_slice()).map(|class| class.to_string()).collect()
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
}

impl<'a> HTMLCollectionMethods for JSRef<'a, HTMLCollection> {
    // http://dom.spec.whatwg.org/#dom-htmlcollection-length
    fn Length(self) -> u32 {
        match self.collection {
            Static(ref elems) => elems.len() as u32,
            Live(ref root, ref filter) => {
                let root = root.root();
                root.deref().traverse_preorder()
                    .filter(|&child| {
                        let elem: Option<JSRef<Element>> = ElementCast::to_ref(child);
                        elem.map_or(false, |elem| filter.filter(elem, *root))
                    }).count() as u32
            }
        }
    }

    // http://dom.spec.whatwg.org/#dom-htmlcollection-item
    fn Item(self, index: u32) -> Option<Temporary<Element>> {
        match self.collection {
            Static(ref elems) => elems
                .as_slice()
                .get(index as uint)
                .map(|elem| Temporary::new(elem.clone())),
            Live(ref root, ref filter) => {
                let root = root.root();
                root.deref().traverse_preorder()
                    .filter_map(|node| {
                        let elem: Option<JSRef<Element>> = ElementCast::to_ref(node);
                        match elem {
                            Some(ref elem) if filter.filter(*elem, *root) => Some(elem.clone()),
                            _ => None
                        }
                    })
                    .nth(index as uint)
                    .clone()
                    .map(|elem| Temporary::from_rooted(elem))
            }
        }
    }

    // http://dom.spec.whatwg.org/#dom-htmlcollection-nameditem
    fn NamedItem(self, key: DOMString) -> Option<Temporary<Element>> {
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
                .map(|maybe_elem| Temporary::from_rooted(*maybe_elem)),
            Live(ref root, ref filter) => {
                let root = root.root();
                root.deref().traverse_preorder()
                    .filter_map(|node| {
                        let elem: Option<JSRef<Element>> = ElementCast::to_ref(node);
                        match elem {
                            Some(ref elem) if filter.filter(*elem, *root) => Some(elem.clone()),
                            _ => None
                        }
                    })
                    .find(|elem| {
                        elem.get_string_attribute("name") == key ||
                        elem.get_string_attribute("id") == key })
                    .map(|maybe_elem| Temporary::from_rooted(maybe_elem))
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

impl Reflectable for HTMLCollection {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        &self.reflector_
    }
}
