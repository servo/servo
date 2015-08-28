/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLCollectionBinding;
use dom::bindings::codegen::Bindings::HTMLCollectionBinding::HTMLCollectionMethods;
use dom::bindings::codegen::InheritTypes::{ElementCast, NodeCast};
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JS, Root};
use dom::bindings::trace::JSTraceable;
use dom::bindings::utils::{namespace_from_domstring, Reflector, reflect_dom_object};
use dom::element::Element;
use dom::node::{Node, TreeIterator};
use dom::window::Window;
use util::str::{DOMString, split_html_space_chars};

use std::ascii::AsciiExt;
use string_cache::{Atom, Namespace};

pub trait CollectionFilter : JSTraceable {
    fn filter<'a>(&self, elem: &'a Element, root: &'a Node) -> bool;
}

#[derive(JSTraceable)]
#[must_root]
pub struct Collection(JS<Node>, Box<CollectionFilter + 'static>);

#[dom_struct]
pub struct HTMLCollection {
    reflector_: Reflector,
    #[ignore_heap_size_of = "Contains a trait object; can't measure due to #6870"]
    collection: Collection,
}

impl HTMLCollection {
    fn new_inherited(collection: Collection) -> HTMLCollection {
        HTMLCollection {
            reflector_: Reflector::new(),
            collection: collection,
        }
    }

    pub fn new(window: &Window, collection: Collection) -> Root<HTMLCollection> {
        reflect_dom_object(box HTMLCollection::new_inherited(collection),
                           GlobalRef::Window(window), HTMLCollectionBinding::Wrap)
    }

    pub fn create(window: &Window, root: &Node,
                  filter: Box<CollectionFilter + 'static>) -> Root<HTMLCollection> {
        HTMLCollection::new(window, Collection(JS::from_ref(root), filter))
    }

    fn all_elements(window: &Window, root: &Node,
                    namespace_filter: Option<Namespace>) -> Root<HTMLCollection> {
        #[derive(JSTraceable, HeapSizeOf)]
        struct AllElementFilter {
            namespace_filter: Option<Namespace>
        }
        impl CollectionFilter for AllElementFilter {
            fn filter(&self, elem: &Element, _root: &Node) -> bool {
                match self.namespace_filter {
                    None => true,
                    Some(ref namespace) => *elem.namespace() == *namespace
                }
            }
        }
        let filter = AllElementFilter {namespace_filter: namespace_filter};
        HTMLCollection::create(window, root, box filter)
    }

    pub fn by_tag_name(window: &Window, root: &Node, tag: DOMString)
                       -> Root<HTMLCollection> {
        if tag == "*" {
            return HTMLCollection::all_elements(window, root, None);
        }

        #[derive(JSTraceable, HeapSizeOf)]
        struct TagNameFilter {
            tag: Atom,
            ascii_lower_tag: Atom,
        }
        impl CollectionFilter for TagNameFilter {
            fn filter(&self, elem: &Element, _root: &Node) -> bool {
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

    pub fn by_tag_name_ns(window: &Window, root: &Node, tag: DOMString,
                          maybe_ns: Option<DOMString>) -> Root<HTMLCollection> {
        let namespace_filter = match maybe_ns {
            Some(ref namespace) if namespace == &"*" => None,
            ns => Some(namespace_from_domstring(ns)),
        };

        if tag == "*" {
            return HTMLCollection::all_elements(window, root, namespace_filter);
        }
        #[derive(JSTraceable, HeapSizeOf)]
        struct TagNameNSFilter {
            tag: Atom,
            namespace_filter: Option<Namespace>
        }
        impl CollectionFilter for TagNameNSFilter {
            fn filter(&self, elem: &Element, _root: &Node) -> bool {
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

    pub fn by_class_name(window: &Window, root: &Node, classes: DOMString)
                         -> Root<HTMLCollection> {
        #[derive(JSTraceable, HeapSizeOf)]
        struct ClassNameFilter {
            classes: Vec<Atom>
        }
        impl CollectionFilter for ClassNameFilter {
            fn filter(&self, elem: &Element, _root: &Node) -> bool {
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

    pub fn children(window: &Window, root: &Node) -> Root<HTMLCollection> {
        #[derive(JSTraceable, HeapSizeOf)]
        struct ElementChildFilter;
        impl CollectionFilter for ElementChildFilter {
            fn filter(&self, elem: &Element, root: &Node) -> bool {
                root.is_parent_of(NodeCast::from_ref(elem))
            }
        }
        HTMLCollection::create(window, root, box ElementChildFilter)
    }

    fn elements_iter(&self) -> HTMLCollectionElementsIter {
        let ref filter = self.collection.1;
        let root = self.collection.0.root();
        let mut node_iter = root.traverse_preorder();
        let _ = node_iter.next();  // skip the root node
        HTMLCollectionElementsIter {
            node_iter: node_iter,
            root: root,
            filter: filter,
        }
    }
}

struct HTMLCollectionElementsIter<'a> {
    node_iter: TreeIterator,
    root: Root<Node>,
    filter: &'a Box<CollectionFilter>,
}

impl<'a> Iterator for HTMLCollectionElementsIter<'a> {
    type Item = Root<Element>;

    fn next(&mut self) -> Option<Self::Item> {
        let filter = self.filter;
        let root = self.root.r();
        self.node_iter.by_ref()
                      .filter_map(ElementCast::to_root)
                      .filter(|element| filter.filter(element.r(), root))
                      .next()
    }
}

impl HTMLCollectionMethods for HTMLCollection {
    // https://dom.spec.whatwg.org/#dom-htmlcollection-length
    fn Length(&self) -> u32 {
        self.elements_iter().count() as u32
    }

    // https://dom.spec.whatwg.org/#dom-htmlcollection-item
    fn Item(&self, index: u32) -> Option<Root<Element>> {
        self.elements_iter().nth(index as usize)
    }

    // https://dom.spec.whatwg.org/#dom-htmlcollection-nameditem
    fn NamedItem(&self, key: DOMString) -> Option<Root<Element>> {
        // Step 1.
        if key.is_empty() {
            return None;
        }

        // Step 2.
        self.elements_iter().find(|elem| {
            elem.r().get_string_attribute(&atom!("name")) == key ||
            elem.r().get_string_attribute(&atom!("id")) == key
        })
    }

    // https://dom.spec.whatwg.org/#dom-htmlcollection-item
    fn IndexedGetter(&self, index: u32, found: &mut bool) -> Option<Root<Element>> {
        let maybe_elem = self.Item(index);
        *found = maybe_elem.is_some();
        maybe_elem
    }

    // check-tidy: no specs after this line
    fn NamedGetter(&self, name: DOMString, found: &mut bool) -> Option<Root<Element>> {
        let maybe_elem = self.NamedItem(name);
        *found = maybe_elem.is_some();
        maybe_elem
    }

    // https://dom.spec.whatwg.org/#interface-htmlcollection
    fn SupportedPropertyNames(&self) -> Vec<DOMString> {
        // Step 1
        let mut result = vec![];

        // Step 2
        for elem in self.elements_iter() {
            // Step 2.1
            let id_attr = elem.get_string_attribute(&atom!("id"));
            if !id_attr.is_empty() && !result.contains(&id_attr) {
                result.push(id_attr)
            }
            // Step 2.2
            let name_attr = elem.get_string_attribute(&atom!("name"));
            if !name_attr.is_empty() && !result.contains(&name_attr) && *elem.namespace() == ns!(HTML) {
                result.push(name_attr)
            }
        }

        // Step 3
        result
    }
}
