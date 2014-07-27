/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLFieldSetElementBinding;
use dom::bindings::codegen::Bindings::HTMLFieldSetElementBinding::HTMLFieldSetElementMethods;
use dom::bindings::codegen::InheritTypes::{ElementCast, HTMLFieldSetElementDerived, NodeCast};
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::utils::{Reflectable, Reflector};
use dom::document::Document;
use dom::element::{Element, HTMLFieldSetElementTypeId};
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
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
        self.type_id == NodeTargetTypeId(ElementNodeTypeId(HTMLFieldSetElementTypeId))
    }
}

impl HTMLFieldSetElement {
    pub fn new_inherited(localName: DOMString, document: &JSRef<Document>) -> HTMLFieldSetElement {
        HTMLFieldSetElement {
            htmlelement: HTMLElement::new_inherited(HTMLFieldSetElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: &JSRef<Document>) -> Temporary<HTMLFieldSetElement> {
        let element = HTMLFieldSetElement::new_inherited(localName, document);
        Node::reflect_node(box element, document, HTMLFieldSetElementBinding::Wrap)
    }
}

impl<'a> HTMLFieldSetElementMethods for JSRef<'a, HTMLFieldSetElement> {
    // http://www.whatwg.org/html/#dom-fieldset-elements
    fn Elements(&self) -> Temporary<HTMLCollection> {
        struct ElementsFilter;
        impl CollectionFilter for ElementsFilter {
            fn filter(&self, elem: &JSRef<Element>, root: &JSRef<Node>) -> bool {
                static tag_names: StaticStringVec = &["button", "fieldset", "input",
                    "keygen", "object", "output", "select", "textarea"];
                let root: &JSRef<Element> = ElementCast::to_ref(root).unwrap();
                elem != root && tag_names.iter().any(|&tag_name| tag_name == elem.deref().local_name.as_slice())
            }
        }
        let node: &JSRef<Node> = NodeCast::from_ref(self);
        let filter = box ElementsFilter;
        let window = window_from_node(node).root();
        HTMLCollection::create(&*window, node, filter)
    }

    fn Validity(&self) -> Temporary<ValidityState> {
        let window = window_from_node(self).root();
        ValidityState::new(&*window)
    }
}

impl Reflectable for HTMLFieldSetElement {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.htmlelement.reflector()
    }
}
