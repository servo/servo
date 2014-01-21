/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::HTMLTitleElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLTitleElementDerived;
use dom::bindings::jsmanaged::JSManaged;
use dom::bindings::utils::{DOMString, ErrorResult};
use dom::document::Document;
use dom::element::HTMLTitleElementTypeId;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, ElementNodeTypeId};

pub struct HTMLTitleElement {
    htmlelement: HTMLElement,
}

impl HTMLTitleElementDerived for EventTarget {
    fn is_htmltitleelement(&self) -> bool {
        match self.type_id {
            NodeTargetTypeId(ElementNodeTypeId(HTMLTitleElementTypeId)) => true,
            _ => false
        }
    }
}

impl HTMLTitleElement {
    pub fn new_inherited(localName: ~str, document: JSManaged<Document>) -> HTMLTitleElement {
        HTMLTitleElement {
            htmlelement: HTMLElement::new_inherited(HTMLTitleElementTypeId, localName, document)
        }
    }

    pub fn new(localName: ~str, document: JSManaged<Document>) -> JSManaged<HTMLTitleElement> {
        let element = HTMLTitleElement::new_inherited(localName, document);
        Node::reflect_node(~element, document, HTMLTitleElementBinding::Wrap)
    }
}

impl HTMLTitleElement {
    pub fn Text(&self) -> DOMString {
        ~""
    }

    pub fn SetText(&mut self, _text: DOMString) -> ErrorResult {
        Ok(())
    }
}
