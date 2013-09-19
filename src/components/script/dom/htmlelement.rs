/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::HTMLElementBinding;
use dom::bindings::utils::{DOMString, ErrorResult, Fallible};
use dom::bindings::utils::{CacheableWrapper, BindingObject, WrapperCache};
use dom::element::{Element, ElementTypeId};
use dom::node::{AbstractNode, ScriptView};
use js::jsapi::{JSObject, JSContext, JSVal};
use js::JSVAL_NULL;

pub struct HTMLElement {
    element: Element
}

impl HTMLElement {
    pub fn new(type_id: ElementTypeId, tag_name: ~str) -> HTMLElement {
        HTMLElement {
            element: Element::new(type_id, tag_name)
        }
    }
}

impl HTMLElement {
    pub fn Title(&self) -> DOMString {
        None
    }

    pub fn SetTitle(&mut self, _title: &DOMString) {
    }

    pub fn Lang(&self) -> DOMString {
        None
    }

    pub fn SetLang(&mut self, _lang: &DOMString) {
    }

    pub fn Dir(&self) -> DOMString {
        None
    }

    pub fn SetDir(&mut self, _dir: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn GetItemValue(&self, _cx: *JSContext) -> Fallible<JSVal> {
        Ok(JSVAL_NULL)
    }

    pub fn SetItemValue(&mut self, _cx: *JSContext, _val: JSVal) -> ErrorResult {
        Ok(())
    }

    pub fn Hidden(&self) -> bool {
        false
    }

    pub fn SetHidden(&mut self, _hidden: bool) -> ErrorResult {
        Ok(())
    }

    pub fn Click(&self) {
    }

    pub fn TabIndex(&self) -> i32 {
        0
    }

    pub fn SetTabIndex(&mut self, _index: i32) -> ErrorResult {
        Ok(())
    }

    pub fn Focus(&self) -> ErrorResult {
        Ok(())
    }

    pub fn Blur(&self) -> ErrorResult {
        Ok(())
    }

    pub fn AccessKey(&self) -> DOMString {
        None
    }

    pub fn SetAccessKey(&self, _key: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn AccessKeyLabel(&self) -> DOMString {
        None
    }

    pub fn Draggable(&self) -> bool {
        false
    }

    pub fn SetDraggable(&mut self, _draggable: bool) -> ErrorResult {
        Ok(())
    }

    pub fn ContentEditable(&self) -> DOMString {
        None
    }

    pub fn SetContentEditable(&mut self, _val: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn IsContentEditable(&self) -> bool {
        false
    }

    pub fn Spellcheck(&self) -> bool {
        false
    }

    pub fn SetSpellcheck(&self, _val: bool) -> ErrorResult {
        Ok(())
    }

    pub fn ClassName(&self) -> DOMString {
        None
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
        self.element.get_wrappercache()
    }

    fn wrap_object_shared(@mut self, cx: *JSContext, scope: *JSObject) -> *JSObject {
        let mut unused = false;
        HTMLElementBinding::Wrap(cx, scope, self, &mut unused)
    }
}

impl BindingObject for HTMLElement {
    fn GetParentObject(&self, cx: *JSContext) -> Option<@mut CacheableWrapper> {
        self.element.GetParentObject(cx)
    }
}
