/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::HTMLLabelElementBinding;
use dom::bindings::utils::DOMString;
use dom::document::AbstractDocument;
use dom::element::HTMLLabelElementTypeId;
use dom::htmlelement::HTMLElement;
use dom::node::{AbstractNode, Node, ScriptView};

pub struct HTMLLabelElement {
    htmlelement: HTMLElement,
}

impl HTMLLabelElement {
    pub fn new_inherited(localName: ~str, document: AbstractDocument) -> HTMLLabelElement {
        HTMLLabelElement {
            htmlelement: HTMLElement::new_inherited(HTMLLabelElementTypeId, localName, document)
        }
    }

    pub fn new(localName: ~str, document: AbstractDocument) -> AbstractNode<ScriptView> {
        let element = HTMLLabelElement::new_inherited(localName, document);
        Node::reflect_node(@mut element, document, HTMLLabelElementBinding::Wrap)
    }
}

impl HTMLLabelElement {
    pub fn HtmlFor(&self) -> DOMString {
        ~""
    }

    pub fn SetHtmlFor(&mut self, _html_for: DOMString) {
    }
}
