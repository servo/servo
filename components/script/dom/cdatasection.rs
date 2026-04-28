/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;

use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::document::Document;
use crate::dom::node::Node;
use crate::dom::text::Text;

#[dom_struct]
pub(crate) struct CDATASection {
    text: Text,
}

impl CDATASection {
    fn new_inherited(text: DOMString, document: &Document) -> CDATASection {
        CDATASection {
            text: Text::new_inherited(text, document),
        }
    }

    pub(crate) fn new(
        cx: &mut js::context::JSContext,
        text: DOMString,
        document: &Document,
    ) -> DomRoot<CDATASection> {
        Node::reflect_node(
            cx,
            Box::new(CDATASection::new_inherited(text, document)),
            document,
        )
    }
}
