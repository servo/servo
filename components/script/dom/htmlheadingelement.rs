/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLHeadingElementBinding;
use dom::bindings::js::Root;
use dom::document::Document;
use dom::htmlelement::HTMLElement;
use dom::node::Node;
use dom_struct::dom_struct;
use html5ever::{LocalName, Prefix};

#[derive(JSTraceable, HeapSizeOf)]
pub enum HeadingLevel {
    Heading1,
    Heading2,
    Heading3,
    Heading4,
    Heading5,
    Heading6,
}

#[dom_struct]
pub struct HTMLHeadingElement {
    htmlelement: HTMLElement,
    level: HeadingLevel,
}

impl HTMLHeadingElement {
    fn new_inherited(local_name: LocalName,
                     prefix: Option<Prefix>,
                     document: &Document,
                     level: HeadingLevel) -> HTMLHeadingElement {
        HTMLHeadingElement {
            htmlelement:
                HTMLElement::new_inherited(local_name, prefix, document),
            level: level,
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(local_name: LocalName,
               prefix: Option<Prefix>,
               document: &Document,
               level: HeadingLevel) -> Root<HTMLHeadingElement> {
        Node::reflect_node(box HTMLHeadingElement::new_inherited(local_name, prefix, document, level),
                           document,
                           HTMLHeadingElementBinding::Wrap)
    }
}
