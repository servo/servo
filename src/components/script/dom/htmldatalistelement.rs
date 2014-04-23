/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::HTMLDataListElementBinding;
use dom::bindings::codegen::InheritTypes::{HTMLDataListElementDerived, NodeCast};
use dom::bindings::js::JS;
use dom::document::Document;
use dom::element::{Element, HTMLDataListElementTypeId};
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlcollection::{HTMLCollection, CollectionFilter};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, ElementNodeTypeId, window_from_node};
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
    pub fn Options(&self, abstract_self: &JS<HTMLDataListElement>) -> JS<HTMLCollection> {
        struct HTMLDataListOptionsFilter;
        impl CollectionFilter for HTMLDataListOptionsFilter {
            fn filter(&self, elem: &JS<Element>, _root: &JS<Node>) -> bool {
                elem.get().local_name == ~"option"
            }
        }
        let node: JS<Node> = NodeCast::from(abstract_self);
        let filter = ~HTMLDataListOptionsFilter;
        HTMLCollection::create(&window_from_node(&node), &node, filter)
    }
}
