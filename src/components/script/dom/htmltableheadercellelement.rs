/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::HTMLTableHeaderCellElementBinding;
use dom::document::AbstractDocument;
use dom::element::HTMLTableHeaderCellElementTypeId;
use dom::htmltablecellelement::HTMLTableCellElement;
use dom::node::{AbstractNode, Node};

pub struct HTMLTableHeaderCellElement {
    htmltablecellelement: HTMLTableCellElement,
}

impl HTMLTableHeaderCellElement {
    pub fn new_inherited(localName: ~str, document: AbstractDocument) -> HTMLTableHeaderCellElement {
        HTMLTableHeaderCellElement {
            htmltablecellelement: HTMLTableCellElement::new_inherited(HTMLTableHeaderCellElementTypeId, localName, document)
        }
    }

    pub fn new(localName: ~str, document: AbstractDocument) -> AbstractNode {
        let element = HTMLTableHeaderCellElement::new_inherited(localName, document);
        Node::reflect_node(@mut element, document, HTMLTableHeaderCellElementBinding::Wrap)
    }
}
