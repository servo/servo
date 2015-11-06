/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLMediaElementBinding::HTMLMediaElementMethods;
use dom::bindings::codegen::Bindings::HTMLMediaElementBinding::HTMLMediaElementConstants;
use dom::document::Document;
use dom::htmlelement::HTMLElement;
use std::cell::Cell;
use util::str::DOMString;

#[dom_struct]
pub struct HTMLMediaElement {
    htmlelement: HTMLElement,
    network_state: Cell<u16>,
    ready_state: Cell<u16>,
}

impl HTMLMediaElement {
    pub fn new_inherited(tag_name: DOMString,
                         prefix: Option<DOMString>, document: &Document)
                         -> HTMLMediaElement {
        HTMLMediaElement {
            htmlelement:
                HTMLElement::new_inherited(tag_name, prefix, document),
            network_state: Cell::new(HTMLMediaElementConstants::NETWORK_EMPTY),
            ready_state: Cell::new(HTMLMediaElementConstants::HAVE_NOTHING),
        }
    }

    #[inline]
    pub fn htmlelement(&self) -> &HTMLElement {
        &self.htmlelement
    }
}

impl HTMLMediaElementMethods for HTMLMediaElement {
    fn NetworkState(&self) -> u16 {
        self.network_state.get()
    }

    fn ReadyState(&self) -> u16 {
        self.ready_state.get()
    }
}
