/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::HTMLMapElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLMapElementDerived;
use dom::bindings::js::JS;
use dom::bindings::error::ErrorResult;
use dom::document::Document;
use dom::element::HTMLMapElementTypeId;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlcollection::{HTMLCollection, Static};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, ElementNodeTypeId};
use servo_util::str::DOMString;

#[deriving(Encodable)]
pub struct HTMLMapElement {
    pub htmlelement: HTMLElement
}

impl HTMLMapElementDerived for EventTarget {
    fn is_htmlmapelement(&self) -> bool {
        match self.type_id {
            NodeTargetTypeId(ElementNodeTypeId(HTMLMapElementTypeId)) => true,
            _ => false
        }
    }
}

impl HTMLMapElement {
    pub fn new_inherited(localName: DOMString, document: JS<Document>) -> HTMLMapElement {
        HTMLMapElement {
            htmlelement: HTMLElement::new_inherited(HTMLMapElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: &JS<Document>) -> JS<HTMLMapElement> {
        let element = HTMLMapElement::new_inherited(localName, document.clone());
        Node::reflect_node(~element, document, HTMLMapElementBinding::Wrap)
    }
}

impl HTMLMapElement {
    pub fn Name(&self) -> DOMString {
        ~""
    }

    pub fn SetName(&mut self, _name: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Areas(&self) -> JS<HTMLCollection> {
        // FIXME: https://github.com/mozilla/servo/issues/1845
        let doc = self.htmlelement.element.node.owner_doc();
        let doc = doc.get();
        HTMLCollection::new(&doc.window, Static(~[]))
    }
}
