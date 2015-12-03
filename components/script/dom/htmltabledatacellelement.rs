/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLTableDataCellElementBinding;
use dom::bindings::js::Root;
use dom::document::Document;
use dom::htmltablecellelement::HTMLTableCellElement;
use dom::node::Node;
use string_cache::Atom;
use util::str::DOMString;

#[dom_struct]
pub struct HTMLTableDataCellElement {
    htmltablecellelement: HTMLTableCellElement,
}

impl HTMLTableDataCellElement {
    fn new_inherited(localName: Atom,
                     prefix: Option<DOMString>,
                     document: &Document) -> HTMLTableDataCellElement {
        HTMLTableDataCellElement {
            htmltablecellelement:
                HTMLTableCellElement::new_inherited(localName, prefix, document)
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: Atom, prefix: Option<DOMString>, document: &Document)
               -> Root<HTMLTableDataCellElement> {
        Node::reflect_node(box HTMLTableDataCellElement::new_inherited(localName,
                                                                       prefix,
                                                                       document),
                           document,
                           HTMLTableDataCellElementBinding::Wrap)
    }
}
