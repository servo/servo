/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::UIEventBinding;
use dom::bindings::utils::{DOMString, Fallible};
use dom::bindings::utils::{CacheableWrapper, WrapperCache, BindingObject, DerivedWrapper};
use dom::node::{AbstractNode, ScriptView};
use dom::event::Event;
use dom::window::Window;
use dom::windowproxy::WindowProxy;

use js::glue::RUST_OBJECT_TO_JSVAL;
use js::jsapi::{JSObject, JSContext, JSVal};

pub struct UIEvent {
    parent: Event,
    can_bubble: bool,
    cancelable: bool,
    view: Option<@mut WindowProxy>,
    detail: i32
}

impl UIEvent {
    pub fn new(type_: &DOMString, can_bubble: bool, cancelable: bool,
               view: Option<@mut WindowProxy>, detail: i32) -> UIEvent {
        UIEvent {
            parent: Event::new(type_),
            can_bubble: can_bubble,
            cancelable: cancelable,
            view: view,
            detail: detail
        }
    }

    pub fn init_wrapper(@mut self, cx: *JSContext, scope: *JSObject) {
        self.wrap_object_shared(cx, scope);
    }

    pub fn Constructor(_owner: @mut Window,
                       type_: &DOMString,
                       init: &UIEventBinding::UIEventInit) -> Fallible<@mut UIEvent> {
        Ok(@mut UIEvent::new(type_, init.parent.bubbles, init.parent.cancelable,
                             init.view, init.detail))
    }

    pub fn GetView(&self) -> Option<@mut WindowProxy> {
        self.view
    }

    pub fn Detail(&self) -> i32 {
        self.detail
    }

    pub fn InitUIEvent(&mut self,
                       type_: &DOMString,
                       can_bubble: bool,
                       cancelable: bool,
                       view: Option<@mut WindowProxy>,
                       detail: i32) {
        self.parent.InitEvent(type_, can_bubble, cancelable);
        self.can_bubble = can_bubble;
        self.cancelable = cancelable;
        self.view = view;
        self.detail = detail;
    }

    pub fn LayerX(&self) -> i32 {
        //TODO
        0
    }

    pub fn LayerY(&self) -> i32 {
        //TODO
        0
    }

    pub fn PageX(&self) -> i32 {
        //TODO
        0
    }

    pub fn PageY(&self) -> i32 {
        //TODO
        0
    }

    pub fn Which(&self) -> u32 {
        //TODO
        0
    }

    pub fn GetRangeParent(&self) -> Option<AbstractNode<ScriptView>> {
        //TODO
        None
    }

    pub fn RangeOffset(&self) -> i32 {
        //TODO
        0
    }

    pub fn CancelBubble(&self) -> bool {
        //TODO
        false
    }

    pub fn SetCancelBubble(&mut self, _val: bool) {
        //TODO
    }

    pub fn IsChar(&self) -> bool {
        //TODO
        false
    }
}

impl CacheableWrapper for UIEvent {
    fn get_wrappercache(&mut self) -> &mut WrapperCache {
        return self.parent.get_wrappercache()
    }

    fn wrap_object_shared(@mut self, cx: *JSContext, scope: *JSObject) -> *JSObject {
        let mut unused = false;
        UIEventBinding::Wrap(cx, scope, self, &mut unused)
    }
}

impl BindingObject for UIEvent {
    fn GetParentObject(&self, cx: *JSContext) -> Option<@mut CacheableWrapper> {
        self.parent.GetParentObject(cx)
    }
}

impl DerivedWrapper for UIEvent {
    fn wrap(&mut self, _cx: *JSContext, _scope: *JSObject, _vp: *mut JSVal) -> i32 {
        fail!(~"nyi")
    }

    #[fixed_stack_segment]
    fn wrap_shared(@mut self, cx: *JSContext, scope: *JSObject, vp: *mut JSVal) -> i32 {
        let obj = self.wrap_object_shared(cx, scope);
        if obj.is_null() {
            return 0;
        } else {
            unsafe { *vp = RUST_OBJECT_TO_JSVAL(obj) };
            return 1;
        }
    }
}
