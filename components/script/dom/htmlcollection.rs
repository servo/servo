/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLCollectionBinding;
use dom::bindings::codegen::Bindings::HTMLCollectionBinding::HTMLCollectionMethods;
use dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use dom::bindings::codegen::InheritTypes::{ElementCast, NodeCast};
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JS, Root};
use dom::bindings::trace::JSTraceable;
use dom::bindings::utils::{namespace_from_domstring, Reflector, reflect_dom_object};
use dom::element::{Element, AttributeHandlers, ElementHelpers};
use dom::document::{Document, DocumentHelpers};
use dom::node::{Node, NodeHelpers, TreeIterator};
use dom::window::Window;
use util::str::{DOMString, StaticStringVec, split_html_space_chars};

use std::ascii::AsciiExt;
use std::iter::{FilterMap, Skip};
use std::cell::{Cell, RefCell};
use string_cache::{Atom, Namespace};

use dom::bindings::codegen::InheritTypes::{HTMLAnchorElementDerived, HTMLAppletElementDerived};
use dom::bindings::codegen::InheritTypes::{HTMLAreaElementDerived, HTMLEmbedElementDerived};
use dom::bindings::codegen::InheritTypes::{HTMLFormElementDerived, HTMLImageElementDerived};
use dom::bindings::codegen::InheritTypes::{HTMLScriptElementDerived};
use dom::bindings::codegen::InheritTypes::HTMLOptionElementDerived;

#[derive(Clone, Hash, PartialEq, Eq, JSTraceable)]
pub enum CollectionFilter {
    AllElements(Option<Namespace>),
    TagName(Atom, Atom),
    TagNameNS(Atom, Option<Namespace>),
    ClassName(Vec<Atom>),
    ElementChild,
    Images,
    Embeds,
    Links,
    Forms,
    Scripts,
    Anchors,
    Applets,
    NamedElement(Atom),
    DataListOptions,
    FieldSetElements,
}

impl CollectionFilter {
    fn filter(&self, elem: &Element, root: &Node) -> bool {
        match self {
            &CollectionFilter::AllElements(ref namespace_filter) => {
                match namespace_filter {
                    &None => true,
                    &Some(ref namespace) => elem.namespace() == namespace
                }
            },
            &CollectionFilter::TagName(ref tag, ref ascii_lower_tag) => {
                if elem.html_element_in_html_document() {
                    elem.local_name() == ascii_lower_tag
                } else {
                    elem.local_name() == tag
                }
            },
            &CollectionFilter::TagNameNS(ref tag, ref namespace_filter) => {
                let ns_match = match namespace_filter {
                    &Some(ref namespace) => {
                        elem.namespace() == namespace
                    },
                    &None => true
                };
                ns_match && elem.local_name() == tag
             },
             &CollectionFilter::ClassName(ref classes) =>
                 classes.iter().all(|class| elem.has_class(class)),
             &CollectionFilter::ElementChild => root.is_parent_of(NodeCast::from_ref(elem)),
             &CollectionFilter::Images => elem.is_htmlimageelement(),
             &CollectionFilter::Embeds => elem.is_htmlembedelement(),
             &CollectionFilter::Links =>
                (elem.is_htmlanchorelement() || elem.is_htmlareaelement()) &&
                    elem.has_attribute(&atom!("href")),
             &CollectionFilter::Forms => elem.is_htmlformelement(),
             &CollectionFilter::Scripts => elem.is_htmlscriptelement(),
             &CollectionFilter::Anchors =>
                 elem.is_htmlanchorelement() && elem.has_attribute(&atom!("href")),
             &CollectionFilter::Applets => elem.is_htmlappletelement(),
             &CollectionFilter::NamedElement(ref name) =>
                 Document::filter_by_name(&name, NodeCast::from_ref(elem)),
             &CollectionFilter::DataListOptions => elem.is_htmloptionelement(),
             &CollectionFilter::FieldSetElements => {
                static TAG_NAMES: StaticStringVec = &["button", "fieldset", "input",
                    "keygen", "object", "output", "select", "textarea"];
                TAG_NAMES.iter().any(|&tag_name| tag_name == &**elem.local_name())
             }
        }
    }
}

#[derive(Copy, Clone, JSTraceable, PartialEq)]
pub enum CollectionState {
    Dirty,
    Updated,
}

#[derive(JSTraceable)]
#[must_root]
pub struct LiveCollection {
    root: JS<Node>,
    filter: CollectionFilter,
    state: Cell<CollectionState>,
    elements: RefCell<Option<Vec<JS<Element>>>>,
}

#[derive(JSTraceable)]
#[must_root]
pub enum CollectionTypeId {
    Static(Vec<JS<Element>>),
    Live(LiveCollection)
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

    pub fn new(window: &Window, collection: CollectionTypeId) -> Root<HTMLCollection> {
        reflect_dom_object(box HTMLCollection::new_inherited(collection),
                           GlobalRef::Window(window), HTMLCollectionBinding::Wrap)
    }
}

impl HTMLCollection {
    pub fn create(window: &Window, root: &Node,
                  filter: CollectionFilter) -> Root<HTMLCollection> {
        let key = (root as *const Node as usize, filter.clone());
        match window.Document().get_collection(&key) {
            Some(existing) => existing,
            _ => {
                let collection =
                    HTMLCollection::new(window,
                                        CollectionTypeId::Live(
                                            LiveCollection {
                                                root: JS::from_ref(root),
                                                filter: filter,
                                                state: Cell::new(CollectionState::Dirty),
                                                elements: RefCell::new(None),
                                            }
                                         ));
                window.Document().add_collection(key, collection.r());
                collection
            }
        }
    }

    fn all_elements(window: &Window, root: &Node,
                    namespace_filter: Option<Namespace>) -> Root<HTMLCollection> {
        let filter = CollectionFilter::AllElements(namespace_filter);
        HTMLCollection::create(window, root, filter)
    }

    pub fn by_tag_name(window: &Window, root: &Node, tag: DOMString)
                       -> Root<HTMLCollection> {
        if tag == "*" {
            return HTMLCollection::all_elements(window, root, None);
        }

        let filter = CollectionFilter::TagName(Atom::from_slice(&tag),
                                               Atom::from_slice(&tag.to_ascii_lowercase()));
        HTMLCollection::create(window, root, filter)
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
        let filter = CollectionFilter::TagNameNS(Atom::from_slice(&tag), namespace_filter);
        HTMLCollection::create(window, root, filter)
    }

    pub fn by_class_name(window: &Window, root: &Node, classes: DOMString)
                         -> Root<HTMLCollection> {
        let filter = CollectionFilter::ClassName(
            split_html_space_chars(&classes).map(|class| {
                Atom::from_slice(class)
            }).collect()
        );

        HTMLCollection::create(window, root, filter)
    }

    pub fn children(window: &Window, root: &Node) -> Root<HTMLCollection> {
        HTMLCollection::create(window, root, CollectionFilter::ElementChild)
    }

    fn traverse(root: &Node)
                -> FilterMap<Skip<TreeIterator>,
                             fn(Root<Node>) -> Option<Root<Element>>> {
        fn to_temporary(node: Root<Node>) -> Option<Root<Element>> {
            ElementCast::to_root(node)
        }
        root.traverse_preorder()
            .skip(1)
            .filter_map(to_temporary)
    }
}

impl<'a> HTMLCollectionMethods for &'a HTMLCollection {
    // https://dom.spec.whatwg.org/#dom-htmlcollection-length
    fn Length(self) -> u32 {
        match self.collection {
            CollectionTypeId::Static(ref elems) => elems.len() as u32,
            CollectionTypeId::Live(ref collection) => {
                if collection.state.get() == CollectionState::Dirty {
                    let root = collection.root.root();
                    *collection.elements.borrow_mut() = Some(HTMLCollection::traverse(root.r())
                        .filter(|element| collection.filter.filter(element.r(), root.r()))
                        .map(|e| JS::from_ref(e.r()))
                        .collect());
                    collection.state.set(CollectionState::Updated);
                }
                match *collection.elements.borrow() {
                    Some(ref elements) => elements.len() as u32,
                    _ => unreachable!("Should have elements")
                }
            }
        }
    }

    // https://dom.spec.whatwg.org/#dom-htmlcollection-item
    fn Item(self, index: u32) -> Option<Root<Element>> {
        let index = index as usize;
        match self.collection {
            CollectionTypeId::Static(ref elems) => elems
                .get(index).map(|t| t.root()),
            CollectionTypeId::Live(ref collection) => {
                if collection.state.get() == CollectionState::Dirty {
                    let root = collection.root.root();
                    *collection.elements.borrow_mut() = Some(HTMLCollection::traverse(root.r())
                        .filter(|element| collection.filter.filter(element.r(), root.r()))
                        .map(|e| JS::from_ref(e.r()))
                        .collect());
                    collection.state.set(CollectionState::Updated);
                }
                match *collection.elements.borrow() {
                    Some(ref elements) => elements.get(index).map(|e| e.root()),
                    _ => unreachable!("Should have elements")
                }
            }
        }
    }

    // https://dom.spec.whatwg.org/#dom-htmlcollection-nameditem
    fn NamedItem(self, key: DOMString) -> Option<Root<Element>> {
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
                    elem.r().get_string_attribute(&atom!("id")) == key }),
            CollectionTypeId::Live(ref collection) => {
                if collection.state.get() == CollectionState::Dirty {
                    let root = collection.root.root();
                    *collection.elements.borrow_mut() = Some(HTMLCollection::traverse(root.r())
                        .filter(|element| collection.filter.filter(element.r(), root.r()))
                        .map(|e| JS::from_ref(e.r()))
                        .collect());
                    collection.state.set(CollectionState::Updated);
                }
                match *collection.elements.borrow() {
                    Some(ref elements) => {
                        elements.iter()
                                .find(|elem| {
                                    let elem = elem.root();
                                    elem.r().get_string_attribute(&atom!("name")) == key ||
                                    elem.r().get_string_attribute(&atom!("id")) == key })
                                .map(|e| e.root())
                    },
                    _ => unreachable!("Should have elements")
                }
            }
        }
    }

    // https://dom.spec.whatwg.org/#dom-htmlcollection-item
    fn IndexedGetter(self, index: u32, found: &mut bool) -> Option<Root<Element>> {
        let maybe_elem = self.Item(index);
        *found = maybe_elem.is_some();
        maybe_elem
    }

    // check-tidy: no specs after this line
    fn NamedGetter(self, name: DOMString, found: &mut bool) -> Option<Root<Element>> {
        let maybe_elem = self.NamedItem(name);
        *found = maybe_elem.is_some();
        maybe_elem
    }
}

