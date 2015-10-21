/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLCollectionBinding;
use dom::bindings::codegen::Bindings::HTMLCollectionBinding::HTMLCollectionMethods;
use dom::bindings::global::GlobalRef;
use dom::bindings::inheritance::Castable;
use dom::bindings::js::{JS, Root, MutNullableHeap};
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::bindings::trace::JSTraceable;
use dom::bindings::xmlname::namespace_from_domstring;
use dom::element::Element;
use dom::node::{Node, FollowingNodeIterator};
use dom::window::Window;
use std::ascii::AsciiExt;
use std::cell::Cell;
use string_cache::{Atom, Namespace};
use util::str::{DOMString, split_html_space_chars};

pub trait CollectionFilter : JSTraceable {
    fn filter<'a>(&self, elem: &'a Element, root: &'a Node) -> bool;
}

#[dom_struct]
pub struct HTMLCollection {
    reflector_: Reflector,
    root: JS<Node>,
    #[ignore_heap_size_of = "Contains a trait object; can't measure due to #6870"]
    filter: Box<CollectionFilter + 'static>,
    cached_version: Cell<u32>,
    cached_length: Cell<u32>,
    cached_cursor_element: MutNullableHeap<JS<Element>>,
    cached_cursor_index: Cell<u32>,
}

impl HTMLCollection {
    #[allow(unrooted_must_root)]
    fn new_inherited(root: &Node, filter: Box<CollectionFilter + 'static>) -> HTMLCollection {
        HTMLCollection {
            reflector_: Reflector::new(),
            root: JS::from_ref(root),
	    filter: filter,
            cached_version: Cell::new(root.get_descendents_version()),
            cached_length: Cell::new(u32::max_value()),
            cached_cursor_element: MutNullableHeap::new(None),
            cached_cursor_index: Cell::new(u32::max_value()),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(window: &Window, root: &Node, filter: Box<CollectionFilter + 'static>) -> Root<HTMLCollection> {
        reflect_dom_object(box HTMLCollection::new_inherited(root, filter),
                           GlobalRef::Window(window), HTMLCollectionBinding::Wrap)
    }

    pub fn create(window: &Window, root: &Node,
                  filter: Box<CollectionFilter + 'static>) -> Root<HTMLCollection> {
        HTMLCollection::new(window, root, filter)
    }

    fn validate_cache(&self) {
       let cached_version = self.cached_version.get();
       let curr_version = self.root.get_descendents_version();
       if curr_version != cached_version {
           self.cached_version.set(curr_version);
           self.cached_length.set(u32::max_value());
           self.cached_cursor_element.set(None);
           self.cached_cursor_index.set(u32::max_value());
        }
    }

    fn get_length(&self) -> u32 {
        let cached_length = self.cached_length.get();
	if cached_length == u32::max_value() {
	    let length = self.elements_iter().count() as u32;
	    self.cached_length.set(length);
	    length
	} else {
	    cached_length
	}
    }

    fn set_cached_cursor(&self, index: u32, element: Option<Root<Element>>) -> Option<Root<Element>> {
        if let Some(element) = element {
	    self.cached_cursor_index.set(index);
	    self.cached_cursor_element.set(Some(element.r()));
	    Some(element)
	} else {
	    None
	}
    }
    
    fn get_item(&self, index: u32) -> Option<Root<Element>> {
        if let Some(element) = self.cached_cursor_element.get() {
	    let cached_index = self.cached_cursor_index.get();
	    if cached_index == index {
	        Some(element)
	    } else if cached_index < index {
	        let offset = index - (cached_index + 1);
		let node = NodeCast::from_root(element);
		self.set_cached_cursor(index, self.elements_iter_after(&*node).nth(offset as usize))
	    } else {
		self.set_cached_cursor(index, self.elements_iter().nth(index as usize))
	    }
	} else {
            self.set_cached_cursor(index, self.elements_iter().nth(index as usize))
	}
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
        let filter = AllElementFilter { namespace_filter: namespace_filter };
        HTMLCollection::create(window, root, box filter)
    }

    pub fn by_tag_name(window: &Window, root: &Node, mut tag: DOMString)
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
        let tag_atom = Atom::from_slice(&tag);
        tag.make_ascii_lowercase();
        let ascii_lower_tag = Atom::from_slice(&tag);
        let filter = TagNameFilter {
            tag: tag_atom,
            ascii_lower_tag: ascii_lower_tag,
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
                root.is_parent_of(elem.upcast())
            }
        }
        HTMLCollection::create(window, root, box ElementChildFilter)
    }

    pub fn elements_iter_after(&self, after: &Node) -> HTMLCollectionElementsIter {
        HTMLCollectionElementsIter {
            node_iter: after.following_nodes(&self.root),
	    root: Root::from_ref(&self.root),
            filter: &self.filter,
        }
    }
    
    pub fn elements_iter(&self) -> HTMLCollectionElementsIter {
        self.elements_iter_after(&*self.root)
    }
    
}

pub struct HTMLCollectionElementsIter<'a> {
    node_iter: FollowingNodeIterator,
    root: Root<Node>,
    filter: &'a Box<CollectionFilter>,
}

impl<'a> Iterator for HTMLCollectionElementsIter<'a> {
    type Item = Root<Element>;

    fn next(&mut self) -> Option<Self::Item> {
        let ref filter = self.filter;
	let ref root = self.root;
        self.node_iter.by_ref()
                      .filter_map(Root::downcast)
                      .filter(|element| filter.filter(&element, root))
                      .next()
   }
}

impl HTMLCollectionMethods for HTMLCollection {
    // https://dom.spec.whatwg.org/#dom-htmlcollection-length
    fn Length(&self) -> u32 {
        self.validate_cache();
	self.get_length()
    }

    // https://dom.spec.whatwg.org/#dom-htmlcollection-item
    fn Item(&self, index: u32) -> Option<Root<Element>> {
        self.validate_cache();
	self.get_item(index)
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
