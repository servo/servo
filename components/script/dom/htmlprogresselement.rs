/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLProgressElementBinding::{self, HTMLProgressElementMethods};
use dom::bindings::inheritance::Castable;
use dom::bindings::root::DomRoot;
use dom::document::Document;
use dom::htmlelement::HTMLElement;
use dom::node::Node;
use dom::nodelist::NodeList;
use dom_struct::dom_struct;
use html5ever::{LocalName, Prefix};
use typeholder::TypeHolderTrait;

#[dom_struct]
pub struct HTMLProgressElement<TH: TypeHolderTrait> {
    htmlelement: HTMLElement<TH>,
}

impl<TH: TypeHolderTrait> HTMLProgressElement<TH> {
    fn new_inherited(local_name: LocalName,
                     prefix: Option<Prefix>,
                     document: &Document<TH>) -> HTMLProgressElement<TH> {
        HTMLProgressElement {
            htmlelement:
                HTMLElement::new_inherited(local_name, prefix, document)
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(local_name: LocalName,
               prefix: Option<Prefix>,
               document: &Document<TH>) -> DomRoot<HTMLProgressElement<TH>> {
        Node::<TH>::reflect_node(Box::new(HTMLProgressElement::new_inherited(local_name, prefix, document)),
                           document,
                           HTMLProgressElementBinding::Wrap)
    }
}

impl<TH: TypeHolderTrait> HTMLProgressElementMethods<TH> for HTMLProgressElement<TH> {
    // https://html.spec.whatwg.org/multipage/#dom-lfe-labels
    fn Labels(&self) -> DomRoot<NodeList<TH>> {
        self.upcast::<HTMLElement<TH>>().labels()
    }
}
