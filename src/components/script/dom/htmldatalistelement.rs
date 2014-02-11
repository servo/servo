/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::HTMLDataListElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLDataListElementDerived;
use dom::bindings::js::JS;
use dom::document::Document;
use dom::element::HTMLDataListElementTypeId;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlcollection::HTMLCollection;
use dom::htmlelement::HTMLElement;
use dom::node::{Node, ElementNodeTypeId};
use servo_util::str::DOMString;

#[deriving(Encodable)]
pub struct HTMLDataListElement {
    htmlelement: HTMLElement
}

impl HTMLDataListElementDerived for EventTarget {
    fn is_htmldatalistelement(&self) -> bool {
        match self.type_id {
            NodeTargetTypeId(ElementNodeTypeId(HTMLDataListElementTypeId)) => true,
            _ => false
        }
    }
}

impl HTMLDataListElement {
    pub fn new_inherited(localName: DOMString, document: JS<Document>) -> HTMLDataListElement {
        HTMLDataListElement {
            htmlelement: HTMLElement::new_inherited(HTMLDataListElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: &JS<Document>) -> JS<HTMLDataListElement> {
        let element = HTMLDataListElement::new_inherited(localName, document.clone());
        Node::reflect_node(~element, document, HTMLDataListElementBinding::Wrap)
    }
}

impl HTMLDataListElement {
    pub fn Options(&self) -> JS<HTMLCollection> {
        // FIXME: https://github.com/mozilla/servo/issues/1842
        let doc = self.htmlelement.element.node.owner_doc();
        let doc = doc.get();
        HTMLCollection::new(&doc.window, ~[])
    }
}
