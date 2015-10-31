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
use dom::node::{Node, FollowingNodeIterator, FollowingNodeReverseIterator};
use dom::window::Window;
use std::ascii::AsciiExt;
use std::cell::Cell;
use string_cache::{Atom, Namespace, QualName};
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
    // We cache the version of the root node and all its decendents,
    // the length of the collection, and a cursor into the collection.
    // We use maxint in case the length or cursor index are unset:
    // it would be nicer to use Option<u32> for this, but that would produce word
    // alignment issues.
    cached_version: Cell<u64>,
    cached_cursor_element: MutNullableHeap<JS<Element>>,
    cached_cursor_index: Cell<u32>,
    cached_length: Cell<u32>,
}

impl HTMLCollection {
    #[allow(unrooted_must_root)]
    fn new_inherited(root: &Node, filter: Box<CollectionFilter + 'static>) -> HTMLCollection {
        HTMLCollection {
            reflector_: Reflector::new(),
            root: JS::from_ref(root),
            filter: filter,
            // Default values for the cache
            cached_version: Cell::new(root.get_inclusive_descendents_version()),
            cached_cursor_element: MutNullableHeap::new(None),
            cached_cursor_index: Cell::new(u32::max_value()),
            cached_length: Cell::new(u32::max_value()),
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
        // Clear the cache if the root version is different from our cached version
        let cached_version = self.cached_version.get();
        let curr_version = self.root.get_inclusive_descendents_version();
        if curr_version != cached_version {
            // Default values for the cache
            self.cached_version.set(curr_version);
            self.cached_length.set(u32::max_value());
            self.cached_cursor_element.set(None);
            self.cached_cursor_index.set(u32::max_value());
        }
    }

    fn get_length(&self) -> u32 {
        // Call validate_cache before calling this method!
        let cached_length = self.cached_length.get();
        if cached_length == u32::max_value() {
            // Cache miss, calculate the length
            let length = self.elements_iter().count() as u32;
            self.cached_length.set(length);
            length
        } else {
            // Cache hit
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
        // Call validate_cache before calling this method!
        let cached_length = self.cached_length.get();
        if let Some(element) = self.cached_cursor_element.get() {
            // Cache hit, the cursor element is set
            let cached_index = self.cached_cursor_index.get();
            if cached_index == index {
                // The cursor is the element we're looking for
                Some(element)
            } else if cached_index < index {
                // The cursor is before the element we're looking for
                // Iterate forwards, starting at the cursor.
                let offset = index - (cached_index + 1);
                let node: Root<Node> = Root::upcast(element);
                self.set_cached_cursor(index, self.elements_iter_after(node.r()).nth(offset as usize))
            } else {
                // The cursor is after the element we're looking for
                // Iterate backwards, starting at the cursor.
                let offset = cached_index - (index + 1);
                let node: Root<Node> = Root::upcast(element);
                self.set_cached_cursor(index, self.elements_iter_reverse_before(node.r()).nth(offset as usize))
            }
        } else if (index < (cached_length - index)) {
            // Cache miss, the index is close to the beginning of the collection
            // Note that this is the case that will fire if the cached length is unset
            // so cached_length is maxint.
            self.set_cached_cursor(index, self.elements_iter().nth(index as usize))
        } else {
            // Cache miss, the index is close to the end of the collection
            let offset = cached_length - (index + 1);
            self.set_cached_cursor(index, self.elements_iter_reverse().nth(offset as usize))
        }
    }

    pub fn by_tag_name(window: &Window, root: &Node, mut tag: DOMString)
                       -> Root<HTMLCollection> {
        let tag_atom = Atom::from_slice(&tag);
        tag.make_ascii_lowercase();
        let ascii_lower_tag = Atom::from_slice(&tag);
        HTMLCollection::by_atomic_tag_name(window, root, tag_atom, ascii_lower_tag)
    }

    pub fn by_atomic_tag_name(window: &Window, root: &Node, tag_atom: Atom, ascii_lower_tag: Atom)
                       -> Root<HTMLCollection> {
        #[derive(JSTraceable, HeapSizeOf)]
        struct TagNameFilter {
            tag: Atom,
            ascii_lower_tag: Atom,
        }
        impl CollectionFilter for TagNameFilter {
            fn filter(&self, elem: &Element, _root: &Node) -> bool {
                if self.tag == atom!("*") {
                    true
                } else if elem.html_element_in_html_document() {
                    *elem.local_name() == self.ascii_lower_tag
                } else {
                    *elem.local_name() == self.tag
                }
            }
        }
        let filter = TagNameFilter {
            tag: tag_atom,
            ascii_lower_tag: ascii_lower_tag,
        };
        HTMLCollection::create(window, root, box filter)
    }

    pub fn by_tag_name_ns(window: &Window, root: &Node, tag: DOMString,
                          maybe_ns: Option<DOMString>) -> Root<HTMLCollection> {
        let local = Atom::from_slice(&tag);
        let ns = namespace_from_domstring(maybe_ns);
        let qname = QualName::new(ns, local);
        HTMLCollection::by_qual_tag_name(window, root, qname)
    }

    pub fn by_qual_tag_name(window: &Window, root: &Node, qname: QualName) -> Root<HTMLCollection> {
        #[derive(JSTraceable, HeapSizeOf)]
        struct TagNameNSFilter {
            qname: QualName
        }
        impl CollectionFilter for TagNameNSFilter {
            fn filter(&self, elem: &Element, _root: &Node) -> bool {
                    ((self.qname.ns == Namespace(atom!("*"))) || (self.qname.ns == *elem.namespace()))
                &&  ((self.qname.local == atom!("*")) || (self.qname.local == *elem.local_name()))
            }
        }
        let filter = TagNameNSFilter {
            qname: qname
        };
        HTMLCollection::create(window, root, box filter)
    }

    pub fn by_class_name(window: &Window, root: &Node, classes: DOMString)
                         -> Root<HTMLCollection> {
        let class_atoms = split_html_space_chars(&classes).map(Atom::from_slice).collect();
        HTMLCollection::by_atomic_class_name(window, root, class_atoms)
    }

    pub fn by_atomic_class_name(window: &Window, root: &Node, classes: Vec<Atom>)
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
            classes: classes
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
        // Iterate forwards from a node.
        HTMLCollectionElementsIter {
            node_iter: after.following_nodes(&self.root),
            root: Root::from_ref(&self.root),
            filter: &self.filter,
        }
    }

    pub fn elements_iter(&self) -> HTMLCollectionElementsIter {
        // Iterate forwards from the root.
        self.elements_iter_after(&*self.root)
    }

    pub fn elements_iter_reverse_before(&self, before: &Node) -> HTMLCollectionElementsRevIter {
        // Iterate backwards from a node.
        HTMLCollectionElementsRevIter {
            node_iter: before.following_nodes_reverse(&self.root),
            root: Root::from_ref(&self.root),
            filter: &self.filter,
        }
    }

    pub fn elements_iter_reverse(&self) -> HTMLCollectionElementsRevIter {
        // Iterate backwards from the root
        self.elements_iter_reverse_before(&*self.root)
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

pub struct HTMLCollectionElementsRevIter<'a> {
    node_iter: FollowingNodeReverseIterator,
    root: Root<Node>,
    filter: &'a Box<CollectionFilter>,
}

impl<'a> Iterator for HTMLCollectionElementsRevIter<'a> {
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
