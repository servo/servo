/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::utils::Reflectable;
use dom::htmlcollection::HTMLCollection;
use dom::htmlelement::HTMLElement;

use js::jsapi::{JSObject, JSContext};

pub struct HTMLDataListElement {
    htmlelement: HTMLElement
}

impl HTMLDataListElement {
    fn get_scope_and_cx(&self) -> (*JSObject, *JSContext) {
        let doc = self.htmlelement.element.node.owner_doc;
        let win = doc.with_base(|doc| doc.window.unwrap());
        let cx = win.page.js_info.get_ref().js_compartment.cx.ptr;
        let scope = win.reflector().get_jsobject();
        (scope, cx)
    }

    pub fn Options(&self) -> @mut HTMLCollection {
        let (scope, cx) = self.get_scope_and_cx();
        HTMLCollection::new(~[], cx, scope)
    }
}
