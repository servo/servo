/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::BindingDeclarations::HTMLMapElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLMapElementDerived;
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::error::ErrorResult;
use dom::document::Document;
use dom::element::HTMLMapElementTypeId;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlcollection::{HTMLCollection, Static};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, ElementNodeTypeId, window_from_node};
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
    pub fn new_inherited(localName: DOMString, document: &JSRef<Document>) -> HTMLMapElement {
        HTMLMapElement {
            htmlelement: HTMLElement::new_inherited(HTMLMapElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: &JSRef<Document>) -> Temporary<HTMLMapElement> {
        let element = HTMLMapElement::new_inherited(localName, document);
        Node::reflect_node(~element, document, HTMLMapElementBinding::Wrap)
    }
}

pub trait HTMLMapElementMethods {
    fn Name(&self) -> DOMString;
    fn SetName(&mut self, _name: DOMString) -> ErrorResult;
    fn Areas(&self) -> Temporary<HTMLCollection>;
}

impl<'a> HTMLMapElementMethods for JSRef<'a, HTMLMapElement> {
    fn Name(&self) -> DOMString {
        "".to_owned()
    }

    fn SetName(&mut self, _name: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Areas(&self) -> Temporary<HTMLCollection> {
        // FIXME: https://github.com/mozilla/servo/issues/1845
        let window = window_from_node(self).root();
        HTMLCollection::new(&*window, Static(vec!()))
    }
}

