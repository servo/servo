/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLHtmlElementBinding;
use dom::bindings::root::DomRoot;
use dom::document::Document;
use dom::htmlelement::HTMLElement;
use dom::node::Node;
use dom_struct::dom_struct;
use html5ever::{LocalName, Prefix};
use typeholder::TypeHolderTrait;

#[dom_struct]
pub struct HTMLHtmlElement<TH: TypeHolderTrait> {
    htmlelement: HTMLElement<TH>
}

impl<TH: TypeHolderTrait> HTMLHtmlElement<TH> {
    fn new_inherited(localName: LocalName, prefix: Option<Prefix>, document: &Document<TH>) -> HTMLHtmlElement<TH> {
        HTMLHtmlElement {
            htmlelement: HTMLElement::new_inherited(localName, prefix, document)
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: LocalName,
               prefix: Option<Prefix>,
               document: &Document<TH>) -> DomRoot<HTMLHtmlElement<TH>> {
        Node::<TH>::reflect_node(Box::new(HTMLHtmlElement::new_inherited(localName, prefix, document)),
                           document,
                           HTMLHtmlElementBinding::Wrap)
    }
}
