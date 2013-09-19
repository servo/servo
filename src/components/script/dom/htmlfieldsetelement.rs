/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::utils::{DOMString, ErrorResult, CacheableWrapper};
use dom::htmlcollection::HTMLCollection;
use dom::htmlelement::HTMLElement;
use dom::node::{AbstractNode, ScriptView};
use dom::validitystate::ValidityState;

use js::jsapi::{JSContext, JSObject};

pub struct HTMLFieldSetElement {
    htmlelement: HTMLElement
}

impl HTMLFieldSetElement {
    pub fn Disabled(&self) -> bool {
        false
    }

    pub fn SetDisabled(&mut self, _disabled: bool) -> ErrorResult {
        Ok(())
    }

    pub fn GetForm(&self) -> Option<AbstractNode<ScriptView>> {
        None
    }

    pub fn Name(&self) -> DOMString {
        None
    }

    pub fn SetName(&mut self, _name: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Type(&self) -> DOMString {
        None
    }

    fn get_scope_and_cx(&self) -> (*JSObject, *JSContext) {
        let doc = self.htmlelement.element.node.owner_doc.unwrap();
        let win = doc.with_base(|doc| doc.window.unwrap());
        let cx = win.page.js_info.get_ref().js_compartment.cx.ptr;
        let cache = win.get_wrappercache();
        let scope = cache.get_wrapper();
        (scope, cx)
    }

    pub fn Elements(&self) -> @mut HTMLCollection {
        let (scope, cx) = self.get_scope_and_cx();
        HTMLCollection::new(~[], cx, scope)
    }

    pub fn WillValidate(&self) -> bool {
        false
    }

    pub fn Validity(&self) -> @mut ValidityState {
        @mut ValidityState::valid()
    }

    pub fn ValidationMessage(&self) -> DOMString {
        None
    }

    pub fn CheckValidity(&self) -> bool {
        false
    }

    pub fn SetCustomValidity(&mut self, _error: &DOMString) {
    }
}
