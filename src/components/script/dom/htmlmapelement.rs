/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::utils::{DOMString, ErrorResult};
use dom::htmlcollection::HTMLCollection;
use dom::htmlelement::HTMLElement;

pub struct HTMLMapElement {
    htmlelement: HTMLElement
}

impl HTMLMapElement {
    pub fn Name(&self) -> DOMString {
        None
    }

    pub fn SetName(&mut self, _name: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Areas(&self) -> @mut HTMLCollection {
        let (scope, cx) = self.htmlelement.element.node.get_scope_and_cx();
        HTMLCollection::new(~[], cx, scope)
    }
}
