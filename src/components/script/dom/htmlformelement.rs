/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::utils::{CacheableWrapper, DOMString, ErrorResult, null_string};
use dom::element::HTMLFormElementTypeId;
use dom::htmlcollection::HTMLCollection;
use dom::htmlelement::HTMLElement;
use dom::node::{AbstractNode, ElementNodeTypeId, Node, ScriptView};

use js::jsapi::{JSObject, JSContext};

pub struct HTMLFormElement {
    parent: HTMLElement
}

impl HTMLFormElement {
    fn get_scope_and_cx(&self) -> (*JSObject, *JSContext) {
        let doc = self.parent.parent.parent.owner_doc.unwrap();
        let win = doc.with_base(|doc| doc.window.unwrap());
        let cx = unsafe {(*win.page).js_info.get_ref().js_compartment.cx.ptr};
        let cache = win.get_wrappercache();
        let scope = cache.get_wrapper();
        (scope, cx)
    }

    pub fn AcceptCharset(&self) -> DOMString {
        null_string
    }

    pub fn SetAcceptCharset(&mut self, _accept_charset: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn Action(&self) -> DOMString {
        null_string
    }

    pub fn SetAction(&mut self, _action: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn Autocomplete(&self) -> DOMString {
        null_string
    }

    pub fn SetAutocomplete(&mut self, _autocomplete: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn Enctype(&self) -> DOMString {
        null_string
    }

    pub fn SetEnctype(&mut self, _enctype: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn Encoding(&self) -> DOMString {
        null_string
    }

    pub fn SetEncoding(&mut self, _encoding: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn Method(&self) -> DOMString {
        null_string
    }

    pub fn SetMethod(&mut self, _method: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn Name(&self) -> DOMString {
        null_string
    }

    pub fn SetName(&mut self, _name: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn NoValidate(&self) -> bool {
        false
    }

    pub fn SetNoValidate(&mut self, _no_validate: bool, _rv: &mut ErrorResult) {
    }

    pub fn Target(&self) -> DOMString {
        null_string
    }

    pub fn SetTarget(&mut self, _target: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn Elements(&self) -> @mut HTMLCollection {
        let (scope, cx) = self.get_scope_and_cx();
        HTMLCollection::new(~[], cx, scope)
    }

    pub fn Length(&self) -> i32 {
        0
    }
    
    pub fn Submit(&self, _rv: &mut ErrorResult) {
    }

    pub fn Reset(&self) {
    }

    pub fn CheckValidity(&self) -> bool {
        false
    }

    pub fn IndexedGetter(&self, _index: u32, _found: &mut bool) -> AbstractNode<ScriptView> {
        let (_scope, cx) = self.get_scope_and_cx();
        // FIXME: This should be replaced with a proper value according to the index
        let node = @Node::new(ElementNodeTypeId(HTMLFormElementTypeId));
        unsafe { return Node::as_abstract_node(cx, node) }
    }
}
