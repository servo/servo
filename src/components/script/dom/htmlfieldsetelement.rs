/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::HTMLFieldSetElementBinding;
use dom::bindings::codegen::InheritTypes::{ElementCast, HTMLFieldSetElementDerived, NodeCast};
use dom::bindings::js::JS;
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

    pub fn new(localName: DOMString, document: &JS<Document>) -> JS<HTMLFieldSetElement> {
        let element = HTMLFieldSetElement::new_inherited(localName, document.clone());
        Node::reflect_node(~element, document, HTMLFieldSetElementBinding::Wrap)
    }
}

impl HTMLFieldSetElement {
    pub fn Disabled(&self) -> bool {
        false
    }

    pub fn SetDisabled(&mut self, _disabled: bool) -> ErrorResult {
        Ok(())
    }

    pub fn GetForm(&self) -> Option<JS<HTMLFormElement>> {
        None
    }

    pub fn Name(&self) -> DOMString {
        ~""
    }

    pub fn SetName(&mut self, _name: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Type(&self) -> DOMString {
        ~""
    }

    // http://www.whatwg.org/html/#dom-fieldset-elements
    pub fn Elements(&self, abstract_self: &JS<HTMLFieldSetElement>) -> JS<HTMLCollection> {
        struct ElementsFilter;
        impl CollectionFilter for ElementsFilter {
            fn filter(&self, elem: &JS<Element>, root: &JS<Node>) -> bool {
                static tag_names: StaticStringVec = &["button", "fieldset", "input",
                    "keygen", "object", "output", "select", "textarea"];
                let root: &JS<Element> = &ElementCast::to(root).unwrap();
                elem != root && tag_names.iter().any(|&tag_name| tag_name == elem.get().local_name)
            }
        }
        let node: JS<Node> = NodeCast::from(abstract_self);
        let filter = ~ElementsFilter;
        HTMLCollection::create(&window_from_node(&node), &node, filter)
    }

    pub fn WillValidate(&self) -> bool {
        false
    }

    pub fn Validity(&self) -> JS<ValidityState> {
        let doc = self.htmlelement.element.node.owner_doc();
        let doc = doc.get();
        ValidityState::new(&doc.window)
    }

    pub fn ValidationMessage(&self) -> DOMString {
        ~""
    }

    pub fn CheckValidity(&self) -> bool {
        false
    }

    pub fn SetCustomValidity(&mut self, _error: DOMString) {
    }
}
