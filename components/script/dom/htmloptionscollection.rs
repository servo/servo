/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cmp::Ordering;

use dom_struct::dom_struct;
use html5ever::local_name;

use crate::dom::bindings::codegen::Bindings::ElementBinding::ElementMethods;
use crate::dom::bindings::codegen::Bindings::HTMLCollectionBinding::HTMLCollectionMethods;
use crate::dom::bindings::codegen::Bindings::HTMLOptionsCollectionBinding::HTMLOptionsCollectionMethods;
use crate::dom::bindings::codegen::Bindings::HTMLSelectElementBinding::HTMLSelectElementMethods;
use crate::dom::bindings::codegen::Bindings::NodeBinding::Node_Binding::NodeMethods;
use crate::dom::bindings::codegen::UnionTypes::{
    HTMLElementOrLong, HTMLOptionElementOrHTMLOptGroupElement,
};
use crate::dom::bindings::error::{Error, ErrorResult};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::reflect_dom_object;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::element::Element;
use crate::dom::htmlcollection::{CollectionFilter, HTMLCollection};
use crate::dom::htmloptionelement::HTMLOptionElement;
use crate::dom::htmlselectelement::HTMLSelectElement;
use crate::dom::node::{Node, NodeTraits};
use crate::dom::window::Window;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct HTMLOptionsCollection {
    collection: HTMLCollection,
}

impl HTMLOptionsCollection {
    fn new_inherited(
        select: &HTMLSelectElement,
        filter: Box<dyn CollectionFilter + 'static>,
    ) -> HTMLOptionsCollection {
        HTMLOptionsCollection {
            collection: HTMLCollection::new_inherited(select.upcast(), filter),
        }
    }

    pub(crate) fn new(
        window: &Window,
        select: &HTMLSelectElement,
        filter: Box<dyn CollectionFilter + 'static>,
        can_gc: CanGc,
    ) -> DomRoot<HTMLOptionsCollection> {
        reflect_dom_object(
            Box::new(HTMLOptionsCollection::new_inherited(select, filter)),
            window,
            can_gc,
        )
    }

    fn add_new_elements(&self, count: u32, can_gc: CanGc) -> ErrorResult {
        let root = self.upcast().root_node();
        let document = root.owner_document();

        for _ in 0..count {
            let element =
                HTMLOptionElement::new(local_name!("option"), None, &document, None, can_gc);
            let node = element.upcast::<Node>();
            root.AppendChild(node)?;
        }
        Ok(())
    }
}

impl HTMLOptionsCollectionMethods<crate::DomTypeHolder> for HTMLOptionsCollection {
    // FIXME: This shouldn't need to be implemented here since HTMLCollection (the parent of
    // HTMLOptionsCollection) implements NamedGetter.
    // https://github.com/servo/servo/issues/5875
    //
    // https://dom.spec.whatwg.org/#dom-htmlcollection-nameditem
    fn NamedGetter(&self, name: DOMString) -> Option<DomRoot<Element>> {
        self.upcast().NamedItem(name)
    }

    // https://heycam.github.io/webidl/#dfn-supported-property-names
    fn SupportedPropertyNames(&self) -> Vec<DOMString> {
        self.upcast().SupportedPropertyNames()
    }

    // FIXME: This shouldn't need to be implemented here since HTMLCollection (the parent of
    // HTMLOptionsCollection) implements IndexedGetter.
    // https://github.com/servo/servo/issues/5875
    //
    // https://dom.spec.whatwg.org/#dom-htmlcollection-item
    fn IndexedGetter(&self, index: u32) -> Option<DomRoot<Element>> {
        self.upcast().IndexedGetter(index)
    }

    // https://html.spec.whatwg.org/multipage/#dom-htmloptionscollection-setter
    fn IndexedSetter(
        &self,
        index: u32,
        value: Option<&HTMLOptionElement>,
        can_gc: CanGc,
    ) -> ErrorResult {
        if let Some(value) = value {
            // Step 2
            let length = self.upcast().Length();

            // Step 3
            let n = index as i32 - length as i32;

            // Step 4
            if n > 0 {
                self.add_new_elements(n as u32, can_gc)?;
            }

            // Step 5
            let node = value.upcast::<Node>();
            let root = self.upcast().root_node();
            if n >= 0 {
                Node::pre_insert(node, &root, None, can_gc).map(|_| ())
            } else {
                let child = self.upcast().IndexedGetter(index).unwrap();
                let child_node = child.upcast::<Node>();

                root.ReplaceChild(node, child_node).map(|_| ())
            }
        } else {
            // Step 1
            self.Remove(index as i32);
            Ok(())
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-htmloptionscollection-length>
    fn Length(&self) -> u32 {
        self.upcast().Length()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-htmloptionscollection-length>
    fn SetLength(&self, length: u32, can_gc: CanGc) {
        // Step 1. Let current be the number of nodes represented by the collection.
        let current = self.upcast().Length();

        match length.cmp(&current) {
            // Step 2. If the given value is greater than current, then:
            Ordering::Greater => {
                // Step 2.1 If the given value is greater than 100,000, then return.
                if length > 100_000 {
                    return;
                }

                // Step 2.2 Let n be value − current.
                let n = length - current;

                // Step 2.3 Append n new option elements with no attributes and no child
                // nodes to the select element on which this is rooted.
                self.add_new_elements(n, can_gc).unwrap();
            },
            // Step 3. If the given value is less than current, then:
            Ordering::Less => {
                // Step 3.1. Let n be current − value.
                // Step 3.2 Remove the last n nodes in the collection from their parent nodes.
                for index in (length..current).rev() {
                    self.Remove(index as i32)
                }
            },
            _ => {},
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-htmloptionscollection-add>
    fn Add(
        &self,
        element: HTMLOptionElementOrHTMLOptGroupElement,
        before: Option<HTMLElementOrLong>,
    ) -> ErrorResult {
        let root = self.upcast().root_node();

        let node: &Node = match element {
            HTMLOptionElementOrHTMLOptGroupElement::HTMLOptionElement(ref element) => {
                element.upcast()
            },
            HTMLOptionElementOrHTMLOptGroupElement::HTMLOptGroupElement(ref element) => {
                element.upcast()
            },
        };

        // Step 1
        if node.is_ancestor_of(&root) {
            return Err(Error::HierarchyRequest);
        }

        if let Some(HTMLElementOrLong::HTMLElement(ref before_element)) = before {
            // Step 2
            let before_node = before_element.upcast::<Node>();
            if !root.is_ancestor_of(before_node) {
                return Err(Error::NotFound);
            }

            // Step 3
            if node == before_node {
                return Ok(());
            }
        }

        // Step 4
        let reference_node = before.and_then(|before| match before {
            HTMLElementOrLong::HTMLElement(element) => Some(DomRoot::upcast::<Node>(element)),
            HTMLElementOrLong::Long(index) => self
                .upcast()
                .IndexedGetter(index as u32)
                .map(DomRoot::upcast::<Node>),
        });

        // Step 5
        let parent = if let Some(ref reference_node) = reference_node {
            reference_node.GetParentNode().unwrap()
        } else {
            root
        };

        // Step 6
        Node::pre_insert(node, &parent, reference_node.as_deref(), CanGc::note()).map(|_| ())
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-htmloptionscollection-remove>
    fn Remove(&self, index: i32) {
        if let Some(element) = self.upcast().IndexedGetter(index as u32) {
            element.Remove();
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-htmloptionscollection-selectedindex>
    fn SelectedIndex(&self) -> i32 {
        self.upcast()
            .root_node()
            .downcast::<HTMLSelectElement>()
            .expect("HTMLOptionsCollection not rooted on a HTMLSelectElement")
            .SelectedIndex()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-htmloptionscollection-selectedindex>
    fn SetSelectedIndex(&self, index: i32, can_gc: CanGc) {
        self.upcast()
            .root_node()
            .downcast::<HTMLSelectElement>()
            .expect("HTMLOptionsCollection not rooted on a HTMLSelectElement")
            .SetSelectedIndex(index, can_gc)
    }
}
