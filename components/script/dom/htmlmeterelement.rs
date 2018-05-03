/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLMeterElementBinding::{self, HTMLMeterElementMethods};
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
pub struct HTMLMeterElement<TH: TypeHolderTrait> {
    htmlelement: HTMLElement<TH>
}

impl<TH: TypeHolderTrait> HTMLMeterElement<TH> {
    fn new_inherited(local_name: LocalName,
                     prefix: Option<Prefix>,
                     document: &Document<TH>) -> HTMLMeterElement<TH> {
        HTMLMeterElement {
            htmlelement: HTMLElement::new_inherited(local_name, prefix, document)
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(local_name: LocalName,
               prefix: Option<Prefix>,
               document: &Document<TH>) -> DomRoot<HTMLMeterElement<TH>> {
        Node::<TH>::reflect_node(Box::new(HTMLMeterElement::new_inherited(local_name, prefix, document)),
                           document,
                           HTMLMeterElementBinding::Wrap)
    }
}

impl<TH: TypeHolderTrait> HTMLMeterElementMethods<TH> for HTMLMeterElement<TH> {
    // https://html.spec.whatwg.org/multipage/#dom-lfe-labels
    fn Labels(&self) -> DomRoot<NodeList<TH>> {
        self.upcast::<HTMLElement<TH>>().labels()
    }
}
