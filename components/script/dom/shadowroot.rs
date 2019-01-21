/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::documentfragment::DocumentFragment;
use crate::dom::bindings::codegen::Bindings::ShadowRootBinding::ShadowRootBinding::ShadowRootMethods;
use crate::dom::bindings::codegen::Bindings::ShadowRootBinding::ShadowRootMode;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::document::Document;
use crate::dom::element::Element;
use dom_struct::dom_struct;

// https://dom.spec.whatwg.org/#interface-shadowroot
#[dom_struct]
pub struct ShadowRoot {
    document_fragment: DocumentFragment,
    host: Dom<Element>,
}

impl ShadowRoot {
    #[allow(dead_code)]
    pub fn new_inherited(host: &Element, document: &Document) -> ShadowRoot {
        ShadowRoot {
            document_fragment: DocumentFragment::new_inherited(document),
            host: Dom::from_ref(host),
        }
    }
}

impl ShadowRootMethods for ShadowRoot {
    /// https://dom.spec.whatwg.org/#dom-shadowroot-mode
    fn Mode(&self) -> ShadowRootMode {
        ShadowRootMode::Closed
    }

    /// https://dom.spec.whatwg.org/#dom-shadowroot-host
    fn Host(&self) -> DomRoot<Element> {
        DomRoot::from_ref(&self.host)
    }
}
