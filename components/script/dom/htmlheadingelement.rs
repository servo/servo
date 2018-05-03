/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLHeadingElementBinding;
use dom::bindings::root::DomRoot;
use dom::document::Document;
use dom::htmlelement::HTMLElement;
use dom::node::Node;
use dom_struct::dom_struct;
use html5ever::{LocalName, Prefix};
use typeholder::TypeHolderTrait;

#[derive(JSTraceable, MallocSizeOf)]
pub enum HeadingLevel {
    Heading1,
    Heading2,
    Heading3,
    Heading4,
    Heading5,
    Heading6,
}

#[dom_struct]
pub struct HTMLHeadingElement<TH: TypeHolderTrait> {
    htmlelement: HTMLElement<TH>,
    level: HeadingLevel,
}

impl<TH: TypeHolderTrait> HTMLHeadingElement<TH> {
    fn new_inherited(local_name: LocalName,
                     prefix: Option<Prefix>,
                     document: &Document<TH>,
                     level: HeadingLevel) -> HTMLHeadingElement<TH> {
        HTMLHeadingElement {
            htmlelement:
                HTMLElement::new_inherited(local_name, prefix, document),
            level: level,
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(local_name: LocalName,
               prefix: Option<Prefix>,
               document: &Document<TH>,
               level: HeadingLevel) -> DomRoot<HTMLHeadingElement<TH>> {
        Node::<TH>::reflect_node(Box::new(HTMLHeadingElement::new_inherited(local_name, prefix, document, level)),
                           document,
                           HTMLHeadingElementBinding::Wrap)
    }
}
