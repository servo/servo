/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLCollectionBinding;
use dom::bindings::codegen::Bindings::HTMLCollectionBinding::HTMLCollectionMethods;
use dom::bindings::inheritance::Castable;
use dom::bindings::js::{JS, Root, MutNullableJS};
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::bindings::str::DOMString;
use dom::bindings::trace::JSTraceable;
use dom::bindings::xmlname::namespace_from_domstring;
use dom::element::Element;
use dom::node::Node;
use dom::window::Window;
use dom_struct::dom_struct;
use html5ever::{LocalName, QualName};
use servo_atoms::Atom;
use std::cell::Cell;
use style::str::split_html_space_chars;

pub trait CollectionFilter : JSTraceable {
    fn filter<'a>(&self, elem: &'a Element, root: &'a Node) -> bool;
}

// An optional u32, using maxint to represent None.
// It would be nicer just to use Option<u32> for this, but that would produce word
// alignment issues since Option<u32> uses 33 bits.
#[derive(Clone, Copy, JSTraceable, HeapSizeOf)]
struct OptionU32 {
    bits: u32,
}

impl OptionU32 {
    fn to_option(self) -> Option<u32> {
        if self.bits == u32::max_value() {
            None
        } else {
            Some(self.bits)
        }
    }

    fn some(bits: u32) -> OptionU32 {
        assert!(bits != u32::max_value());
        OptionU32 { bits: bits }
    }

    fn none() -> OptionU32 {
        OptionU32 { bits: u32::max_value() }
    }
}

#[dom_struct]
pub struct HTMLCollection {
    reflector_: Reflector,
    root: JS<Node>,
    #[ignore_heap_size_of = "Contains a trait object; can't measure due to #6870"]
    filter: Box<CollectionFilter + 'static>,
    // We cache the version of the root node and all its decendents,
    // the length of the collection, and a cursor into the collection.
    // FIXME: make the cached cursor element a weak pointer
    cached_version: Cell<u64>,
    cached_cursor_element: MutNullableJS<Element>,
    cached_cursor_index: Cell<OptionU32>,
    cached_length: Cell<OptionU32>,
}

impl HTMLCollection {
    #[allow(unrooted_must_root)]
    pub fn new_inherited(root: &Node, filter: Box<CollectionFilter + 'static>) -> HTMLCollection {
        HTMLCollection {
            reflector_: Reflector::new(),
            root: JS::from_ref(root),
            filter: filter,
            // Default values for the cache
            cached_version: Cell::new(root.inclusive_descendants_version()),
            cached_cursor_element: MutNullableJS::new(None),
            cached_cursor_index: Cell::new(OptionU32::none()),
            cached_length: Cell::new(OptionU32::none()),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(window: &Window, root: &Node, filter: Box<CollectionFilter + 'static>) -> Root<HTMLCollection> {
        reflect_dom_object(box HTMLCollection::new_inherited(root, filter),
                           window, HTMLCollectionBinding::Wrap)
    }

    pub fn create(window: &Window, root: &Node,
                  filter: Box<CollectionFilter + 'static>) -> Root<HTMLCollection> {
        HTMLCollection::new(window, root, filter)
    }

    fn validate_cache(&self) {
        // Clear the cache if the root version is different from our cached version
        let cached_version = self.cached_version.get();
        let curr_version = self.root.inclusive_descendants_version();
        if curr_version != cached_version {
            // Default values for the cache
            self.cached_version.set(curr_version);
            self.cached_cursor_element.set(None);
            self.cached_length.set(OptionU32::none());
            self.cached_cursor_index.set(OptionU32::none());
        }
    }

    fn set_cached_cursor(&self, index: u32, element: Option<Root<Element>>) -> Option<Root<Element>> {
        if let Some(element) = element {
            self.cached_cursor_index.set(OptionU32::some(index));
            self.cached_cursor_element.set(Some(&element));
            Some(element)
        } else {
            None
        }
    }

    // https://dom.spec.whatwg.org/#concept-getelementsbytagname
    pub fn by_qualified_name(window: &Window, root: &Node, qualified_name: LocalName)
                             -> Root<HTMLCollection> {
        // case 1
        if qualified_name == local_name!("*") {
            #[derive(JSTraceable, HeapSizeOf)]
            struct AllFilter;
            impl CollectionFilter for AllFilter {
                fn filter(&self, _elem: &Element, _root: &Node) -> bool {
                    true
                }
            }
            return HTMLCollection::create(window, root, box AllFilter);
        }

        #[derive(JSTraceable, HeapSizeOf)]
        struct HtmlDocumentFilter {
            qualified_name: LocalName,
            ascii_lower_qualified_name: LocalName,
        }
        impl CollectionFilter for HtmlDocumentFilter {
            fn filter(&self, elem: &Element, root: &Node) -> bool {
                if root.is_in_html_doc() && elem.namespace() == &ns!(html) {    // case 2
                    HTMLCollection::match_element(elem, &self.ascii_lower_qualified_name)
                } else {    // case 2 and 3
                    HTMLCollection::match_element(elem, &self.qualified_name)
                }
            }
        }

        let filter = HtmlDocumentFilter {
            ascii_lower_qualified_name: qualified_name.to_ascii_lowercase(),
            qualified_name: qualified_name,
        };
        HTMLCollection::create(window, root, box filter)
    }

    fn match_element(elem: &Element, qualified_name: &LocalName) -> bool {
        match elem.prefix() {
            None => elem.local_name() == qualified_name,
            Some(prefix) => qualified_name.starts_with(&**prefix) &&
                qualified_name.find(":") == Some(prefix.len()) &&
                qualified_name.ends_with(&**elem.local_name()),
        }
    }

    pub fn by_tag_name_ns(window: &Window, root: &Node, tag: DOMString,
                          maybe_ns: Option<DOMString>) -> Root<HTMLCollection> {
        let local = LocalName::from(tag);
        let ns = namespace_from_domstring(maybe_ns);
        let qname = QualName::new(None, ns, local);
        HTMLCollection::by_qual_tag_name(window, root, qname)
    }

    pub fn by_qual_tag_name(window: &Window, root: &Node, qname: QualName) -> Root<HTMLCollection> {
        #[derive(JSTraceable, HeapSizeOf)]
        struct TagNameNSFilter {
            qname: QualName
        }
        impl CollectionFilter for TagNameNSFilter {
            fn filter(&self, elem: &Element, _root: &Node) -> bool {
                    ((self.qname.ns == namespace_url!("*")) || (self.qname.ns == *elem.namespace())) &&
                    ((self.qname.local == local_name!("*")) || (self.qname.local == *elem.local_name()))
            }
        }
        let filter = TagNameNSFilter {
            qname: qname
        };
        HTMLCollection::create(window, root, box filter)
    }

    pub fn by_class_name(window: &Window, root: &Node, classes: DOMString)
                         -> Root<HTMLCollection> {
        let class_atoms = split_html_space_chars(&classes).map(Atom::from).collect();
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

    pub fn elements_iter_after<'a>(&'a self, after: &'a Node) -> impl Iterator<Item=Root<Element>> + 'a {
        // Iterate forwards from a node.
        HTMLCollectionElementsIter {
            node_iter: after.following_nodes(&self.root),
            root: Root::from_ref(&self.root),
            filter: &self.filter,
        }
    }

    pub fn elements_iter<'a>(&'a self) -> impl Iterator<Item=Root<Element>> + 'a {
        // Iterate forwards from the root.
        self.elements_iter_after(&*self.root)
    }

    pub fn elements_iter_before<'a>(&'a self, before: &'a Node) -> impl Iterator<Item=Root<Element>> + 'a {
        // Iterate backwards from a node.
        HTMLCollectionElementsIter {
            node_iter: before.preceding_nodes(&self.root),
            root: Root::from_ref(&self.root),
            filter: &self.filter,
        }
    }

    pub fn root_node(&self) -> Root<Node> {
        Root::from_ref(&self.root)
    }
}

// TODO: Make this generic, and avoid code duplication
struct HTMLCollectionElementsIter<'a, I> {
    node_iter: I,
    root: Root<Node>,
    filter: &'a Box<CollectionFilter>,
}

impl<'a, I: Iterator<Item=Root<Node>>> Iterator for HTMLCollectionElementsIter<'a, I> {
    type Item = Root<Element>;

    fn next(&mut self) -> Option<Self::Item> {
        let filter = &self.filter;
        let root = &self.root;
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

        if let Some(cached_length) = self.cached_length.get().to_option() {
            // Cache hit
            cached_length
        } else {
            // Cache miss, calculate the length
            let length = self.elements_iter().count() as u32;
            self.cached_length.set(OptionU32::some(length));
            length
        }
    }

    // https://dom.spec.whatwg.org/#dom-htmlcollection-item
    fn Item(&self, index: u32) -> Option<Root<Element>> {
        self.validate_cache();

        if let Some(element) = self.cached_cursor_element.get() {
            // Cache hit, the cursor element is set
            if let Some(cached_index) = self.cached_cursor_index.get().to_option() {
                if cached_index == index {
                    // The cursor is the element we're looking for
                    Some(element)
                } else if cached_index < index {
                    // The cursor is before the element we're looking for
                    // Iterate forwards, starting at the cursor.
                    let offset = index - (cached_index + 1);
                    let node: Root<Node> = Root::upcast(element);
                    let mut iter = self.elements_iter_after(&node);
                    self.set_cached_cursor(index, iter.nth(offset as usize))
                } else {
                    // The cursor is after the element we're looking for
                    // Iterate backwards, starting at the cursor.
                    let offset = cached_index - (index + 1);
                    let node: Root<Node> = Root::upcast(element);
                    let mut iter = self.elements_iter_before(&node);
                    self.set_cached_cursor(index, iter.nth(offset as usize))
                }
            } else {
                // Cache miss
                // Iterate forwards through all the nodes
                self.set_cached_cursor(index, self.elements_iter().nth(index as usize))
            }
        } else {
            // Cache miss
            // Iterate forwards through all the nodes
            self.set_cached_cursor(index, self.elements_iter().nth(index as usize))
        }
    }

    // https://dom.spec.whatwg.org/#dom-htmlcollection-nameditem
    fn NamedItem(&self, key: DOMString) -> Option<Root<Element>> {
        // Step 1.
        if key.is_empty() {
            return None;
        }

        // Step 2.
        self.elements_iter().find(|elem| {
            elem.get_string_attribute(&local_name!("id")) == key ||
            (elem.namespace() == &ns!(html) && elem.get_string_attribute(&local_name!("name")) == key)
        })
    }

    // https://dom.spec.whatwg.org/#dom-htmlcollection-item
    fn IndexedGetter(&self, index: u32) -> Option<Root<Element>> {
        self.Item(index)
    }

    // check-tidy: no specs after this line
    fn NamedGetter(&self, name: DOMString) -> Option<Root<Element>> {
        self.NamedItem(name)
    }

    // https://dom.spec.whatwg.org/#interface-htmlcollection
    fn SupportedPropertyNames(&self) -> Vec<DOMString> {
        // Step 1
        let mut result = vec![];

        // Step 2
        for elem in self.elements_iter() {
            // Step 2.1
            let id_attr = elem.get_string_attribute(&local_name!("id"));
            if !id_attr.is_empty() && !result.contains(&id_attr) {
                result.push(id_attr)
            }
            // Step 2.2
            let name_attr = elem.get_string_attribute(&local_name!("name"));
            if !name_attr.is_empty() && !result.contains(&name_attr) && *elem.namespace() == ns!(html) {
                result.push(name_attr)
            }
        }

        // Step 3
        result
    }
}
