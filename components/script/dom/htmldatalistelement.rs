/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLDataListElementBinding;
use dom::bindings::codegen::Bindings::HTMLDataListElementBinding::HTMLDataListElementMethods;
use dom::bindings::codegen::InheritTypes::NodeCast;
use dom::bindings::codegen::InheritTypes::{HTMLDataListElementDerived, HTMLOptionElementDerived};
use dom::bindings::js::Root;
use dom::document::Document;
use dom::element::Element;
use dom::element::ElementTypeId;
use dom::eventtarget::{EventTarget, EventTargetTypeId};
use dom::htmlcollection::{HTMLCollection, CollectionFilter};
use dom::htmlelement::{HTMLElement, HTMLElementTypeId};
use dom::node::{Node, NodeTypeId, window_from_node};
use util::str::DOMString;

#[dom_struct]
#[derive(HeapSizeOf)]
pub struct HTMLDataListElement {
    htmlelement: HTMLElement
}

impl HTMLDataListElementDerived for EventTarget {
    fn is_htmldatalistelement(&self) -> bool {
        *self.type_id() ==
            EventTargetTypeId::Node(
                NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLDataListElement)))
    }
}

impl HTMLDataListElement {
    fn new_inherited(localName: DOMString,
                     prefix: Option<DOMString>,
                     document: &Document) -> HTMLDataListElement {
        HTMLDataListElement {
            htmlelement:
                HTMLElement::new_inherited(HTMLElementTypeId::HTMLDataListElement, localName, prefix, document)
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: DOMString,
               prefix: Option<DOMString>,
               document: &Document) -> Root<HTMLDataListElement> {
        let element = HTMLDataListElement::new_inherited(localName, prefix, document);
        Node::reflect_node(box element, document, HTMLDataListElementBinding::Wrap)
    }
}

impl<'a> HTMLDataListElementMethods for &'a HTMLDataListElement {
    // https://html.spec.whatwg.org/multipage/#dom-datalist-options
    fn Options(self) -> Root<HTMLCollection> {
        #[derive(JSTraceable, HeapSizeOf)]
        struct HTMLDataListOptionsFilter;
        impl CollectionFilter for HTMLDataListOptionsFilter {
            fn filter(&self, elem: &Element, _root: &Node) -> bool {
                elem.is_htmloptionelement()
            }
        }
        let node = NodeCast::from_ref(self);
        let filter = box HTMLDataListOptionsFilter;
        let window = window_from_node(node);
        HTMLCollection::create(window.r(), node, filter)
    }
}

