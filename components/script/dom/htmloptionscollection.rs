/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::ElementBinding::ElementMethods;
use dom::bindings::codegen::Bindings::HTMLCollectionBinding::HTMLCollectionMethods;
use dom::bindings::codegen::Bindings::HTMLOptionsCollectionBinding;
use dom::bindings::codegen::Bindings::HTMLOptionsCollectionBinding::HTMLOptionsCollectionMethods;
use dom::bindings::codegen::Bindings::HTMLSelectElementBinding::HTMLSelectElementMethods;
use dom::bindings::codegen::Bindings::NodeBinding::NodeBinding::NodeMethods;
use dom::bindings::codegen::UnionTypes::{HTMLOptionElementOrHTMLOptGroupElement, HTMLElementOrLong};
use dom::bindings::error::{Error, ErrorResult};
use dom::bindings::inheritance::Castable;
use dom::bindings::js::{Root, RootedReference};
use dom::bindings::reflector::reflect_dom_object;
use dom::bindings::str::DOMString;
use dom::element::Element;
use dom::htmlcollection::{CollectionFilter, HTMLCollection};
use dom::htmloptionelement::HTMLOptionElement;
use dom::htmlselectelement::HTMLSelectElement;
use dom::node::{document_from_node, Node};
use dom::window::Window;
use dom_struct::dom_struct;

#[dom_struct]
pub struct HTMLOptionsCollection {
    collection: HTMLCollection,
}

impl HTMLOptionsCollection {
    fn new_inherited(select: &HTMLSelectElement, filter: Box<CollectionFilter + 'static>) -> HTMLOptionsCollection {
        HTMLOptionsCollection {
            collection: HTMLCollection::new_inherited(select.upcast(), filter),
        }
    }

    pub fn new(window: &Window, select: &HTMLSelectElement, filter: Box<CollectionFilter + 'static>)
        -> Root<HTMLOptionsCollection>
    {
        reflect_dom_object(box HTMLOptionsCollection::new_inherited(select, filter),
                           window,
                           HTMLOptionsCollectionBinding::Wrap)
    }

    fn add_new_elements(&self, count: u32) -> ErrorResult {
        let root = self.upcast().root_node();
        let document = document_from_node(&*root);

        for _ in 0..count {
            let element = HTMLOptionElement::new(local_name!("option"), None, &document);
            let node = element.upcast::<Node>();
            root.AppendChild(node)?;
        };
        Ok(())
    }
}

impl HTMLOptionsCollectionMethods for HTMLOptionsCollection {
    // FIXME: This shouldn't need to be implemented here since HTMLCollection (the parent of
    // HTMLOptionsCollection) implements NamedGetter.
    // https://github.com/servo/servo/issues/5875
    //
    // https://dom.spec.whatwg.org/#dom-htmlcollection-nameditem
    fn NamedGetter(&self, name: DOMString) -> Option<Root<Element>> {
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
    fn IndexedGetter(&self, index: u32) -> Option<Root<Element>> {
        self.upcast().IndexedGetter(index)
    }

    // https://html.spec.whatwg.org/multipage/#dom-htmloptionscollection-setter
    fn IndexedSetter(&self, index: u32, value: Option<&HTMLOptionElement>) -> ErrorResult {
        if let Some(value) = value {
            // Step 2
            let length = self.upcast().Length();

            // Step 3
            let n = index as i32 - length as i32;

            // Step 4
            if n > 0 {
                self.add_new_elements(n as u32)?;
            }

            // Step 5
            let node = value.upcast::<Node>();
            let root = self.upcast().root_node();
            if n >= 0 {
                Node::pre_insert(node, &root, None).map(|_| ())
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

    // https://html.spec.whatwg.org/multipage/#dom-htmloptionscollection-length
    fn Length(&self) -> u32 {
        self.upcast().Length()
    }

    // https://html.spec.whatwg.org/multipage/#dom-htmloptionscollection-length
    fn SetLength(&self, length: u32) {
        let current_length = self.upcast().Length();
        let delta = length as i32 - current_length as i32;
        if delta < 0 {
            // new length is lower - deleting last option elements
            for index in (length..current_length).rev() {
                self.Remove(index as i32)
            }
        } else if delta > 0 {
            // new length is higher - adding new option elements
            self.add_new_elements(delta as u32).unwrap();
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-htmloptionscollection-add
    fn Add(&self, element: HTMLOptionElementOrHTMLOptGroupElement, before: Option<HTMLElementOrLong>) -> ErrorResult {
        let root = self.upcast().root_node();

        let node: &Node = match element {
            HTMLOptionElementOrHTMLOptGroupElement::HTMLOptionElement(ref element) => element.upcast(),
            HTMLOptionElementOrHTMLOptGroupElement::HTMLOptGroupElement(ref element) => element.upcast(),
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
        let reference_node = before.and_then(|before| {
            match before {
                HTMLElementOrLong::HTMLElement(element) => Some(Root::upcast::<Node>(element)),
                HTMLElementOrLong::Long(index) => {
                    self.upcast().IndexedGetter(index as u32).map(Root::upcast::<Node>)
                }
            }
        });

        // Step 5
        let parent = if let Some(reference_node) = reference_node.r() {
            reference_node.GetParentNode().unwrap()
        } else {
            root
        };

        // Step 6
        Node::pre_insert(node, &parent, reference_node.r()).map(|_| ())
    }

    // https://html.spec.whatwg.org/multipage/#dom-htmloptionscollection-remove
    fn Remove(&self, index: i32) {
        if let Some(element) = self.upcast().IndexedGetter(index as u32) {
            element.Remove();
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-htmloptionscollection-selectedindex
    fn SelectedIndex(&self) -> i32 {
        self.upcast()
            .root_node()
            .downcast::<HTMLSelectElement>()
            .expect("HTMLOptionsCollection not rooted on a HTMLSelectElement")
            .SelectedIndex()
    }

    // https://html.spec.whatwg.org/multipage/#dom-htmloptionscollection-selectedindex
    fn SetSelectedIndex(&self, index: i32) {
        self.upcast()
            .root_node()
            .downcast::<HTMLSelectElement>()
            .expect("HTMLOptionsCollection not rooted on a HTMLSelectElement")
            .SetSelectedIndex(index)
    }
}
