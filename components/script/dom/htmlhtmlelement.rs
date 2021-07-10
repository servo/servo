/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::DomRoot;
use crate::dom::document::Document;
use crate::dom::htmlelement::HTMLElement;
use crate::dom::node::Node;
use dom_struct::dom_struct;
use html5ever::{LocalName, Prefix};

#[dom_struct]
pub struct HTMLHtmlElement {
    htmlelement: HTMLElement,
}

#[allow(non_snake_case)]
impl HTMLHtmlElement {
    fn new_inherited(
        localName: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
    ) -> HTMLHtmlElement {
        HTMLHtmlElement {
            htmlelement: HTMLElement::new_inherited(localName, prefix, document),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(
        localName: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
    ) -> DomRoot<HTMLHtmlElement> {
        let n = Node::reflect_node(
            Box::new(HTMLHtmlElement::new_inherited(localName, prefix, document)),
            document,
        );

        n.upcast::<Node>().set_weird_parser_insertion_mode();
        n
    }
}
