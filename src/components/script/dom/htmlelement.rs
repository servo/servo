/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::HTMLElementBinding;
use dom::bindings::utils::{DOMString, null_string, ErrorResult};
use dom::bindings::utils::{CacheableWrapper, BindingObject, WrapperCache};
use dom::element::{Element, ElementTypeId};
use dom::node::{AbstractNode, ScriptView};
use js::jsapi::{JSObject, JSContext, JSVal};
use js::JSVAL_NULL;

pub struct HTMLElement {
    parent: Element
}

impl HTMLElement {
    pub fn new(type_id: ElementTypeId, tag_name: ~str) -> HTMLElement {
        HTMLElement {
            parent: Element::new(type_id, tag_name)
        }
    }
}

impl HTMLElement {
    pub fn Title(&self) -> DOMString {
        null_string
    }

    pub fn SetTitle(&mut self, _title: &DOMString) {
    }

    pub fn Lang(&self) -> DOMString {
        null_string
    }

    pub fn SetLang(&mut self, _lang: &DOMString) {
    }

    pub fn Dir(&self) -> DOMString {
        null_string
    }

    pub fn SetDir(&mut self, _dir: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn GetItemValue(&self, _cx: *JSContext, _rv: &mut ErrorResult) -> JSVal {
        JSVAL_NULL
    }

    pub fn SetItemValue(&mut self, _cx: *JSContext, _val: JSVal, _rv: &mut ErrorResult) {
    }

    pub fn Hidden(&self) -> bool {
        false
    }

    pub fn SetHidden(&mut self, _hidden: bool, _rv: &mut ErrorResult) {
    }

    pub fn Click(&self) {
    }

    pub fn TabIndex(&self) -> i32 {
        0
    }

    pub fn SetTabIndex(&mut self, _index: i32, _rv: &mut ErrorResult) {
    }

    pub fn Focus(&self, _rv: &mut ErrorResult) {
    }

    pub fn Blur(&self, _rv: &mut ErrorResult) {
    }

    pub fn AccessKey(&self) -> DOMString {
        null_string
    }

    pub fn SetAccessKey(&self, _key: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn AccessKeyLabel(&self) -> DOMString {
        null_string
    }

    pub fn Draggable(&self) -> bool {
        false
    }

    pub fn SetDraggable(&mut self, _draggable: bool, _rv: &mut ErrorResult) {
    }

    pub fn ContentEditable(&self) -> DOMString {
        null_string
    }

    pub fn SetContentEditable(&mut self, _val: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn IsContentEditable(&self) -> bool {
        false
    }

    pub fn Spellcheck(&self) -> bool {
        false
    }

    pub fn SetSpellcheck(&self, _val: bool, _rv: &mut ErrorResult) {
    }

    pub fn ClassName(&self) -> DOMString {
        null_string
    }

    pub fn SetClassName(&self, _class: &DOMString) {
    }

    pub fn GetOffsetParent(&self) -> Option<AbstractNode<ScriptView>> {
        None
    }

    pub fn OffsetTop(&self) -> i32 {
        0
    }

    pub fn OffsetLeft(&self) -> i32 {
        0
    }

    pub fn OffsetWidth(&self) -> i32 {
        0
    }

    pub fn OffsetHeight(&self) -> i32 {
        0
    }
}

impl CacheableWrapper for HTMLElement {
    fn get_wrappercache(&mut self) -> &mut WrapperCache {
        self.parent.get_wrappercache()
    }

    fn wrap_object_shared(@mut self, cx: *JSContext, scope: *JSObject) -> *JSObject {
        let mut unused = false;
        HTMLElementBinding::Wrap(cx, scope, self, &mut unused)
    }
}

impl BindingObject for HTMLElement {
    fn GetParentObject(&self, cx: *JSContext) -> Option<@mut CacheableWrapper> {
        self.parent.GetParentObject(cx)
    }
}
