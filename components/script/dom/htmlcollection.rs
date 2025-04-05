/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::cmp::Ordering;

use dom_struct::dom_struct;
use html5ever::{LocalName, QualName, local_name, namespace_url, ns};
use style::str::split_html_space_chars;
use stylo_atoms::Atom;

use crate::dom::bindings::codegen::Bindings::HTMLCollectionBinding::HTMLCollectionMethods;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::{Reflector, reflect_dom_object};
use crate::dom::bindings::root::{Dom, DomRoot, MutNullableDom};
use crate::dom::bindings::str::DOMString;
use crate::dom::bindings::trace::JSTraceable;
use crate::dom::bindings::xmlname::namespace_from_domstring;
use crate::dom::element::Element;
use crate::dom::node::{Node, NodeTraits};
use crate::dom::window::Window;
use crate::script_runtime::CanGc;

pub(crate) trait CollectionFilter: JSTraceable {
    fn filter<'a>(&self, elem: &'a Element, root: &'a Node) -> bool;
}

/// An optional `u32`, using `u32::MAX` to represent None.  It would be nicer
/// just to use `Option<u32>` for this, but that would produce word alignment
/// issues since `Option<u32>` uses 33 bits.
#[derive(Clone, Copy, JSTraceable, MallocSizeOf)]
struct OptionU32 {
    bits: u32,
}

impl OptionU32 {
    fn to_option(self) -> Option<u32> {
        if self.bits == u32::MAX {
            None
        } else {
            Some(self.bits)
        }
    }

    fn some(bits: u32) -> OptionU32 {
        assert_ne!(bits, u32::MAX);
        OptionU32 { bits }
    }

    fn none() -> OptionU32 {
        OptionU32 { bits: u32::MAX }
    }
}

#[dom_struct]
pub(crate) struct HTMLCollection {
    reflector_: Reflector,
    root: Dom<Node>,
    #[ignore_malloc_size_of = "Trait object (Box<dyn CollectionFilter>) cannot be sized"]
    filter: Box<dyn CollectionFilter + 'static>,
    // We cache the version of the root node and all its decendents,
    // the length of the collection, and a cursor into the collection.
    // FIXME: make the cached cursor element a weak pointer
    cached_version: Cell<u64>,
    cached_cursor_element: MutNullableDom<Element>,
    cached_cursor_index: Cell<OptionU32>,
    cached_length: Cell<OptionU32>,
}

impl HTMLCollection {
    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    pub(crate) fn new_inherited(
        root: &Node,
        filter: Box<dyn CollectionFilter + 'static>,
    ) -> HTMLCollection {
        HTMLCollection {
            reflector_: Reflector::new(),
            root: Dom::from_ref(root),
            filter,
            // Default values for the cache
            cached_version: Cell::new(root.inclusive_descendants_version()),
            cached_cursor_element: MutNullableDom::new(None),
            cached_cursor_index: Cell::new(OptionU32::none()),
            cached_length: Cell::new(OptionU32::none()),
        }
    }

    /// Returns a collection which is always empty.
    pub(crate) fn always_empty(window: &Window, root: &Node, can_gc: CanGc) -> DomRoot<Self> {
        #[derive(JSTraceable)]
        struct NoFilter;
        impl CollectionFilter for NoFilter {
            fn filter<'a>(&self, _: &'a Element, _: &'a Node) -> bool {
                false
            }
        }

        Self::new(window, root, Box::new(NoFilter), can_gc)
    }

    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    pub(crate) fn new(
        window: &Window,
        root: &Node,
        filter: Box<dyn CollectionFilter + 'static>,
        can_gc: CanGc,
    ) -> DomRoot<Self> {
        reflect_dom_object(Box::new(Self::new_inherited(root, filter)), window, can_gc)
    }

    /// Create a new  [`HTMLCollection`] that just filters element using a static function.
    pub(crate) fn new_with_filter_fn(
        window: &Window,
        root: &Node,
        filter_function: fn(&Element, &Node) -> bool,
        can_gc: CanGc,
    ) -> DomRoot<Self> {
        #[derive(JSTraceable, MallocSizeOf)]
        pub(crate) struct StaticFunctionFilter(
            // The function *must* be static so that it never holds references to DOM objects, which
            // would cause issues with garbage collection -- since it isn't traced.
            #[no_trace]
            #[ignore_malloc_size_of = "Static function pointer"]
            fn(&Element, &Node) -> bool,
        );
        impl CollectionFilter for StaticFunctionFilter {
            fn filter(&self, element: &Element, root: &Node) -> bool {
                (self.0)(element, root)
            }
        }
        Self::new(
            window,
            root,
            Box::new(StaticFunctionFilter(filter_function)),
            can_gc,
        )
    }

    pub(crate) fn create(
        window: &Window,
        root: &Node,
        filter: Box<dyn CollectionFilter + 'static>,
    ) -> DomRoot<Self> {
        Self::new(window, root, filter, CanGc::note())
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

    fn set_cached_cursor(
        &self,
        index: u32,
        element: Option<DomRoot<Element>>,
    ) -> Option<DomRoot<Element>> {
        if let Some(element) = element {
            self.cached_cursor_index.set(OptionU32::some(index));
            self.cached_cursor_element.set(Some(&element));
            Some(element)
        } else {
            None
        }
    }

    /// <https://dom.spec.whatwg.org/#concept-getelementsbytagname>
    pub(crate) fn by_qualified_name(
        window: &Window,
        root: &Node,
        qualified_name: LocalName,
        _can_gc: CanGc,
    ) -> DomRoot<HTMLCollection> {
        // case 1
        if qualified_name == local_name!("*") {
            #[derive(JSTraceable, MallocSizeOf)]
            struct AllFilter;
            impl CollectionFilter for AllFilter {
                fn filter(&self, _elem: &Element, _root: &Node) -> bool {
                    true
                }
            }
            return HTMLCollection::create(window, root, Box::new(AllFilter));
        }

        #[derive(JSTraceable, MallocSizeOf)]
        struct HtmlDocumentFilter {
            #[no_trace]
            qualified_name: LocalName,
            #[no_trace]
            ascii_lower_qualified_name: LocalName,
        }
        impl CollectionFilter for HtmlDocumentFilter {
            fn filter(&self, elem: &Element, root: &Node) -> bool {
                if root.is_in_html_doc() && elem.namespace() == &ns!(html) {
                    // case 2
                    HTMLCollection::match_element(elem, &self.ascii_lower_qualified_name)
                } else {
                    // case 2 and 3
                    HTMLCollection::match_element(elem, &self.qualified_name)
                }
            }
        }

        let filter = HtmlDocumentFilter {
            ascii_lower_qualified_name: qualified_name.to_ascii_lowercase(),
            qualified_name,
        };
        HTMLCollection::create(window, root, Box::new(filter))
    }

    fn match_element(elem: &Element, qualified_name: &LocalName) -> bool {
        match elem.prefix().as_ref() {
            None => elem.local_name() == qualified_name,
            Some(prefix) => {
                qualified_name.starts_with(&**prefix) &&
                    qualified_name.find(':') == Some(prefix.len()) &&
                    qualified_name.ends_with(&**elem.local_name())
            },
        }
    }

    pub(crate) fn by_tag_name_ns(
        window: &Window,
        root: &Node,
        tag: DOMString,
        maybe_ns: Option<DOMString>,
        can_gc: CanGc,
    ) -> DomRoot<HTMLCollection> {
        let local = LocalName::from(tag);
        let ns = namespace_from_domstring(maybe_ns);
        let qname = QualName::new(None, ns, local);
        HTMLCollection::by_qual_tag_name(window, root, qname, can_gc)
    }

    pub(crate) fn by_qual_tag_name(
        window: &Window,
        root: &Node,
        qname: QualName,
        _can_gc: CanGc,
    ) -> DomRoot<HTMLCollection> {
        #[derive(JSTraceable, MallocSizeOf)]
        struct TagNameNSFilter {
            #[no_trace]
            qname: QualName,
        }
        impl CollectionFilter for TagNameNSFilter {
            fn filter(&self, elem: &Element, _root: &Node) -> bool {
                ((self.qname.ns == namespace_url!("*")) || (self.qname.ns == *elem.namespace())) &&
                    ((self.qname.local == local_name!("*")) ||
                        (self.qname.local == *elem.local_name()))
            }
        }
        let filter = TagNameNSFilter { qname };
        HTMLCollection::create(window, root, Box::new(filter))
    }

    pub(crate) fn by_class_name(
        window: &Window,
        root: &Node,
        classes: DOMString,
        can_gc: CanGc,
    ) -> DomRoot<HTMLCollection> {
        let class_atoms = split_html_space_chars(&classes).map(Atom::from).collect();
        HTMLCollection::by_atomic_class_name(window, root, class_atoms, can_gc)
    }

    pub(crate) fn by_atomic_class_name(
        window: &Window,
        root: &Node,
        classes: Vec<Atom>,
        can_gc: CanGc,
    ) -> DomRoot<HTMLCollection> {
        #[derive(JSTraceable, MallocSizeOf)]
        struct ClassNameFilter {
            #[no_trace]
            classes: Vec<Atom>,
        }
        impl CollectionFilter for ClassNameFilter {
            fn filter(&self, elem: &Element, _root: &Node) -> bool {
                let case_sensitivity = elem
                    .owner_document()
                    .quirks_mode()
                    .classes_and_ids_case_sensitivity();

                self.classes
                    .iter()
                    .all(|class| elem.has_class(class, case_sensitivity))
            }
        }

        if classes.is_empty() {
            return HTMLCollection::always_empty(window, root, can_gc);
        }

        let filter = ClassNameFilter { classes };
        HTMLCollection::create(window, root, Box::new(filter))
    }

    pub(crate) fn children(window: &Window, root: &Node, can_gc: CanGc) -> DomRoot<HTMLCollection> {
        HTMLCollection::new_with_filter_fn(
            window,
            root,
            |element, root| root.is_parent_of(element.upcast()),
            can_gc,
        )
    }

    pub(crate) fn elements_iter_after<'a>(
        &'a self,
        after: &'a Node,
    ) -> impl Iterator<Item = DomRoot<Element>> + 'a {
        // Iterate forwards from a node.
        after
            .following_nodes(&self.root)
            .filter_map(DomRoot::downcast)
            .filter(move |element| self.filter.filter(element, &self.root))
    }

    pub(crate) fn elements_iter(&self) -> impl Iterator<Item = DomRoot<Element>> + '_ {
        // Iterate forwards from the root.
        self.elements_iter_after(&self.root)
    }

    pub(crate) fn elements_iter_before<'a>(
        &'a self,
        before: &'a Node,
    ) -> impl Iterator<Item = DomRoot<Element>> + 'a {
        // Iterate backwards from a node.
        before
            .preceding_nodes(&self.root)
            .filter_map(DomRoot::downcast)
            .filter(move |element| self.filter.filter(element, &self.root))
    }

    pub(crate) fn root_node(&self) -> DomRoot<Node> {
        DomRoot::from_ref(&self.root)
    }
}

impl HTMLCollectionMethods<crate::DomTypeHolder> for HTMLCollection {
    /// <https://dom.spec.whatwg.org/#dom-htmlcollection-length>
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

    /// <https://dom.spec.whatwg.org/#dom-htmlcollection-item>
    fn Item(&self, index: u32) -> Option<DomRoot<Element>> {
        self.validate_cache();

        if let Some(element) = self.cached_cursor_element.get() {
            // Cache hit, the cursor element is set
            if let Some(cached_index) = self.cached_cursor_index.get().to_option() {
                match cached_index.cmp(&index) {
                    Ordering::Equal => {
                        // The cursor is the element we're looking for
                        Some(element)
                    },
                    Ordering::Less => {
                        // The cursor is before the element we're looking for
                        // Iterate forwards, starting at the cursor.
                        let offset = index - (cached_index + 1);
                        let node: DomRoot<Node> = DomRoot::upcast(element);
                        let mut iter = self.elements_iter_after(&node);
                        self.set_cached_cursor(index, iter.nth(offset as usize))
                    },
                    Ordering::Greater => {
                        // The cursor is after the element we're looking for
                        // Iterate backwards, starting at the cursor.
                        let offset = cached_index - (index + 1);
                        let node: DomRoot<Node> = DomRoot::upcast(element);
                        let mut iter = self.elements_iter_before(&node);
                        self.set_cached_cursor(index, iter.nth(offset as usize))
                    },
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

    /// <https://dom.spec.whatwg.org/#dom-htmlcollection-nameditem>
    fn NamedItem(&self, key: DOMString) -> Option<DomRoot<Element>> {
        // Step 1.
        if key.is_empty() {
            return None;
        }

        let key = Atom::from(key);

        // Step 2.
        self.elements_iter().find(|elem| {
            elem.get_id().is_some_and(|id| id == key) ||
                (elem.namespace() == &ns!(html) && elem.get_name().is_some_and(|id| id == key))
        })
    }

    // https://dom.spec.whatwg.org/#dom-htmlcollection-item
    fn IndexedGetter(&self, index: u32) -> Option<DomRoot<Element>> {
        self.Item(index)
    }

    // check-tidy: no specs after this line
    fn NamedGetter(&self, name: DOMString) -> Option<DomRoot<Element>> {
        self.NamedItem(name)
    }

    // https://dom.spec.whatwg.org/#interface-htmlcollection
    fn SupportedPropertyNames(&self) -> Vec<DOMString> {
        // Step 1
        let mut result = vec![];

        // Step 2
        for elem in self.elements_iter() {
            // Step 2.1
            if let Some(id_atom) = elem.get_id() {
                let id_str = DOMString::from(&*id_atom);
                if !result.contains(&id_str) {
                    result.push(id_str);
                }
            }
            // Step 2.2
            if *elem.namespace() == ns!(html) {
                if let Some(name_atom) = elem.get_name() {
                    let name_str = DOMString::from(&*name_atom);
                    if !result.contains(&name_str) {
                        result.push(name_str)
                    }
                }
            }
        }

        // Step 3
        result
    }
}
