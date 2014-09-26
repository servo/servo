/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLTitleElementBinding;
use dom::bindings::codegen::Bindings::HTMLTitleElementBinding::HTMLTitleElementMethods;
use dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use dom::bindings::codegen::InheritTypes::{HTMLTitleElementDerived, NodeCast, TextCast};
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::utils::{Reflectable, Reflector};
use dom::document::Document;
use dom::element::HTMLTitleElementTypeId;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, NodeHelpers, ElementNodeTypeId};
use dom::text::Text;
use servo_util::str::DOMString;

#[jstraceable]
#[must_root]
pub struct HTMLTitleElement {
    pub htmlelement: HTMLElement,
}

impl HTMLTitleElementDerived for EventTarget {
    fn is_htmltitleelement(&self) -> bool {
        self.type_id == NodeTargetTypeId(ElementNodeTypeId(HTMLTitleElementTypeId))
    }
}

impl HTMLTitleElement {
    fn new_inherited(localName: DOMString, document: JSRef<Document>) -> HTMLTitleElement {
        HTMLTitleElement {
            htmlelement: HTMLElement::new_inherited(HTMLTitleElementTypeId, localName, document)
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: DOMString, document: JSRef<Document>) -> Temporary<HTMLTitleElement> {
        let element = HTMLTitleElement::new_inherited(localName, document);
        Node::reflect_node(box element, document, HTMLTitleElementBinding::Wrap)
    }
}

impl<'a> HTMLTitleElementMethods for JSRef<'a, HTMLTitleElement> {
    // http://www.whatwg.org/html/#dom-title-text
    fn Text(self) -> DOMString {
        let node: JSRef<Node> = NodeCast::from_ref(self);
        let mut content = String::new();
        for child in node.children() {
            let text: Option<JSRef<Text>> = TextCast::to_ref(child);
            match text {
                Some(text) => content.push_str(text.characterdata.data.borrow().as_slice()),
                None => (),
            }
        }
        content
    }

    // http://www.whatwg.org/html/#dom-title-text
    fn SetText(self, value: DOMString) {
        let node: JSRef<Node> = NodeCast::from_ref(self);
        node.SetTextContent(Some(value))
    }
}

impl Reflectable for HTMLTitleElement {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.htmlelement.reflector()
    }
}
