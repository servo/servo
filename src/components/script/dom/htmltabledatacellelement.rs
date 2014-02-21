/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::HTMLTableDataCellElementBinding;
use dom::document::AbstractDocument;
use dom::element::HTMLTableDataCellElementTypeId;
use dom::htmltablecellelement::HTMLTableCellElement;
use dom::node::{AbstractNode, Node};
use servo_util::str::DOMString;

pub struct HTMLTableDataCellElement {
    htmltablecellelement: HTMLTableCellElement,
}

impl HTMLTableDataCellElement {
    pub fn new_inherited(localName: DOMString, document: AbstractDocument) -> HTMLTableDataCellElement {
        HTMLTableDataCellElement {
            htmltablecellelement: HTMLTableCellElement::new_inherited(HTMLTableDataCellElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: AbstractDocument) -> AbstractNode {
        let element = HTMLTableDataCellElement::new_inherited(localName, document);
        Node::reflect_node(@mut element, document, HTMLTableDataCellElementBinding::Wrap)
    }
}
