/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLTableHeaderCellElementBinding;
use dom::bindings::root::DomRoot;
use dom::document::Document;
use dom::htmltablecellelement::HTMLTableCellElement;
use dom::node::Node;
use dom_struct::dom_struct;
use html5ever::{LocalName, Prefix};

#[dom_struct]
pub struct HTMLTableHeaderCellElement {
    htmltablecellelement: HTMLTableCellElement,
}

impl HTMLTableHeaderCellElement {
    fn new_inherited(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
    ) -> HTMLTableHeaderCellElement {
        HTMLTableHeaderCellElement {
            htmltablecellelement: HTMLTableCellElement::new_inherited(local_name, prefix, document),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
    ) -> DomRoot<HTMLTableHeaderCellElement> {
        Node::reflect_node(
            Box::new(HTMLTableHeaderCellElement::new_inherited(
                local_name, prefix, document,
            )),
            document,
            HTMLTableHeaderCellElementBinding::Wrap,
        )
    }
}
