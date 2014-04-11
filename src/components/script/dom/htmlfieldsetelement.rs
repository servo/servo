/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::BindingDeclarations::HTMLFieldSetElementBinding;
use dom::bindings::codegen::InheritTypes::{ElementCast, HTMLFieldSetElementDerived, NodeCast};
use dom::bindings::js::{JS, JSRef, RootCollection, Unrooted};
use dom::bindings::error::ErrorResult;
use dom::document::Document;
use dom::element::{Element, HTMLFieldSetElementTypeId};
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlformelement::HTMLFormElement;
use dom::htmlcollection::{HTMLCollection, CollectionFilter};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, ElementNodeTypeId, window_from_node};
use dom::validitystate::ValidityState;
use servo_util::str::{DOMString, StaticStringVec};

#[deriving(Encodable)]
pub struct HTMLFieldSetElement {
    pub htmlelement: HTMLElement
}

impl HTMLFieldSetElementDerived for EventTarget {
    fn is_htmlfieldsetelement(&self) -> bool {
        match self.type_id {
            NodeTargetTypeId(ElementNodeTypeId(HTMLFieldSetElementTypeId)) => true,
            _ => false
        }
    }
}

impl HTMLFieldSetElement {
    pub fn new_inherited(localName: DOMString, document: JS<Document>) -> HTMLFieldSetElement {
        HTMLFieldSetElement {
            htmlelement: HTMLElement::new_inherited(HTMLFieldSetElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: &JSRef<Document>) -> Unrooted<HTMLFieldSetElement> {
        let element = HTMLFieldSetElement::new_inherited(localName, document.unrooted());
        Node::reflect_node(~element, document, HTMLFieldSetElementBinding::Wrap)
    }
}

pub trait HTMLFieldSetElementMethods {
    fn Disabled(&self) -> bool;
    fn SetDisabled(&mut self, _disabled: bool) -> ErrorResult;
    fn GetForm(&self) -> Option<Unrooted<HTMLFormElement>>;
    fn Name(&self) -> DOMString;
    fn SetName(&mut self, _name: DOMString) -> ErrorResult;
    fn Type(&self) -> DOMString;
    fn Elements(&self) -> Unrooted<HTMLCollection>;
    fn WillValidate(&self) -> bool;
    fn Validity(&self) -> Unrooted<ValidityState>;
    fn ValidationMessage(&self) -> DOMString;
    fn CheckValidity(&self) -> bool;
    fn SetCustomValidity(&mut self, _error: DOMString);
}

impl<'a> HTMLFieldSetElementMethods for JSRef<'a, HTMLFieldSetElement> {
    fn Disabled(&self) -> bool {
        false
    }

    fn SetDisabled(&mut self, _disabled: bool) -> ErrorResult {
        Ok(())
    }

    fn GetForm(&self) -> Option<Unrooted<HTMLFormElement>> {
        None
    }

    fn Name(&self) -> DOMString {
        ~""
    }

    fn SetName(&mut self, _name: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Type(&self) -> DOMString {
        ~""
    }

    // http://www.whatwg.org/html/#dom-fieldset-elements
    fn Elements(&self) -> Unrooted<HTMLCollection> {
        struct ElementsFilter;
        impl CollectionFilter for ElementsFilter {
            fn filter(&self, elem: &JSRef<Element>, root: &JSRef<Node>) -> bool {
                static tag_names: StaticStringVec = &["button", "fieldset", "input",
                    "keygen", "object", "output", "select", "textarea"];
                let root: &JSRef<Element> = ElementCast::to_ref(root).unwrap();
                elem != root && tag_names.iter().any(|&tag_name| tag_name == elem.get().local_name)
            }
        }
        let roots = RootCollection::new();
        let node: &JSRef<Node> = NodeCast::from_ref(self);
        let filter = ~ElementsFilter;
        let window = window_from_node(node).root(&roots);
        HTMLCollection::create(&*window, node, filter)
    }

    fn WillValidate(&self) -> bool {
        false
    }

    fn Validity(&self) -> Unrooted<ValidityState> {
        let roots = RootCollection::new();
        let doc = self.htmlelement.element.node.owner_doc().root(&roots);
        let window = doc.deref().window.root(&roots);
        ValidityState::new(&*window)
    }

    fn ValidationMessage(&self) -> DOMString {
        ~""
    }

    fn CheckValidity(&self) -> bool {
        false
    }

    fn SetCustomValidity(&mut self, _error: DOMString) {
    }
}

