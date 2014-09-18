/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLDataListElementBinding;
use dom::bindings::codegen::Bindings::HTMLDataListElementBinding::HTMLDataListElementMethods;
use dom::bindings::codegen::InheritTypes::{HTMLDataListElementDerived, HTMLOptionElementDerived};
use dom::bindings::codegen::InheritTypes::NodeCast;
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::utils::{Reflectable, Reflector};
use dom::document::Document;
use dom::element::{Element, HTMLDataListElementTypeId};
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlcollection::{HTMLCollection, CollectionFilter};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, ElementNodeTypeId, window_from_node};
use servo_util::str::DOMString;

#[deriving(Encodable)]
#[must_root]
pub struct HTMLDataListElement {
    pub htmlelement: HTMLElement
}

impl HTMLDataListElementDerived for EventTarget {
    fn is_htmldatalistelement(&self) -> bool {
        self.type_id == NodeTargetTypeId(ElementNodeTypeId(HTMLDataListElementTypeId))
    }
}

impl HTMLDataListElement {
    pub fn new_inherited(localName: DOMString, document: JSRef<Document>) -> HTMLDataListElement {
        HTMLDataListElement {
            htmlelement: HTMLElement::new_inherited(HTMLDataListElementTypeId, localName, document)
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: DOMString, document: JSRef<Document>) -> Temporary<HTMLDataListElement> {
        let element = HTMLDataListElement::new_inherited(localName, document);
        Node::reflect_node(box element, document, HTMLDataListElementBinding::Wrap)
    }
}

impl<'a> HTMLDataListElementMethods for JSRef<'a, HTMLDataListElement> {
    fn Options(&self) -> Temporary<HTMLCollection> {
        struct HTMLDataListOptionsFilter;
        impl CollectionFilter for HTMLDataListOptionsFilter {
            fn filter(&self, elem: JSRef<Element>, _root: JSRef<Node>) -> bool {
                elem.is_htmloptionelement()
            }
        }
        let node: JSRef<Node> = NodeCast::from_ref(*self);
        let filter = box HTMLDataListOptionsFilter;
        let window = window_from_node(node).root();
        HTMLCollection::create(*window, node, filter)
    }
}

impl Reflectable for HTMLDataListElement {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.htmlelement.reflector()
    }
}
