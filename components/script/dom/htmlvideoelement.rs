/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLVideoElementBinding;
use dom::bindings::root::DomRoot;
use dom::document::Document;
use dom::htmlmediaelement::HTMLMediaElement;
use dom::node::Node;
use dom_struct::dom_struct;
use html5ever::{LocalName, Prefix};
use typeholder::TypeHolderTrait;

#[dom_struct]
pub struct HTMLVideoElement<TH: TypeHolderTrait> {
    htmlmediaelement: HTMLMediaElement<TH>
}

impl<TH: TypeHolderTrait> HTMLVideoElement<TH> {
    fn new_inherited(local_name: LocalName, prefix: Option<Prefix>, document: &Document<TH>) -> HTMLVideoElement<TH> {
        HTMLVideoElement {
            htmlmediaelement:
                HTMLMediaElement::new_inherited(local_name, prefix, document)
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(local_name: LocalName,
               prefix: Option<Prefix>,
               document: &Document<TH>) -> DomRoot<HTMLVideoElement<TH>> {
        Node::<TH>::reflect_node(Box::new(HTMLVideoElement::new_inherited(local_name, prefix, document)),
                           document,
                           HTMLVideoElementBinding::Wrap)
    }
}
