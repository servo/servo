/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::eventtarget::EventTarget;
use dom::window::Window;
use dom::bindings::codegen::EventBinding;
use dom::bindings::utils::{CacheableWrapper, BindingObject, DerivedWrapper};
use dom::bindings::utils::{DOMString, ErrorResult, WrapperCache};
use script_task::{task_from_context, global_script_context};

use geom::point::Point2D;
use js::glue::RUST_OBJECT_TO_JSVAL;
use js::jsapi::{JSObject, JSContext, JSVal};

use std::cast;


pub enum Event {
    ResizeEvent(uint, uint), 
    ReflowEvent,
    ClickEvent(uint, Point2D<f32>),
    MouseDownEvent(uint, Point2D<f32>),
    MouseUpEvent(uint, Point2D<f32>),
}

pub struct Event_ {
    wrapper: WrapperCache,
    type_: DOMString,
    default_prevented: bool,
    cancelable: bool,
    bubbles: bool,
    trusted: bool,
}

impl Event_ {
    pub fn new(type_: DOMString) -> Event_ {
        Event_ {
            wrapper: WrapperCache::new(),
            type_: type_,
            default_prevented: false,
            cancelable: true,
            bubbles: true,
            trusted: false
        }
    }

    pub fn init_wrapper(@mut self) {
        let script_context = global_script_context();
        let cx = script_context.js_compartment.cx.ptr;
        let owner = script_context.root_frame.get_ref().window;
        let cache = owner.get_wrappercache();
        let scope = cache.get_wrapper();
        self.wrap_object_shared(cx, scope);
    }

    pub fn EventPhase(&self) -> u16 {
        0
    }

    pub fn Type(&self) -> DOMString {
        copy self.type_
    }

    pub fn GetTarget(&self) -> Option<@mut EventTarget> {
        None
    }

    pub fn GetCurrentTarget(&self) -> Option<@mut EventTarget> {
        None
    }

    pub fn DefaultPrevented(&self) -> bool {
        self.default_prevented
    }

    pub fn PreventDefault(&mut self) {
        self.default_prevented = true
    }

    pub fn StopPropagation(&mut self) {
    }

    pub fn StopImmediatePropagation(&mut self) {
    }

    pub fn Bubbles(&self) -> bool {
        self.bubbles
    }

    pub fn Cancelable(&self) -> bool {
        self.cancelable
    }

    pub fn TimeStamp(&self) -> u64 {
        0
    }

    pub fn InitEvent(&mut self,
                     type_: DOMString,
                     bubbles: bool,
                     cancelable: bool,
                     _rv: &mut ErrorResult) {
        self.type_ = type_;
        self.cancelable = cancelable;
        self.bubbles = bubbles;
    }

    pub fn IsTrusted(&self) -> bool {
        self.trusted
    }

    pub fn Constructor(_global: @mut Window,
                   type_: DOMString,
                   _init: &EventBinding::EventInit,
                   _rv: &mut ErrorResult) -> @mut Event_ {
        @mut Event_::new(type_)
    }
}

impl CacheableWrapper for Event_ {
    fn get_wrappercache(&mut self) -> &mut WrapperCache {
        unsafe { cast::transmute(&self.wrapper) }
    }

    fn wrap_object_shared(@mut self, cx: *JSContext, scope: *JSObject) -> *JSObject {
        let mut unused = false;
        EventBinding::Wrap(cx, scope, self, &mut unused)
    }
}

impl BindingObject for Event_ {
    fn GetParentObject(&self, cx: *JSContext) -> @mut CacheableWrapper {
        let script_context = task_from_context(cx);
        unsafe {
            (*script_context).root_frame.get_ref().window as @mut CacheableWrapper
        }
    }
}

impl DerivedWrapper for Event_ {
    fn wrap(&mut self, _cx: *JSContext, _scope: *JSObject, _vp: *mut JSVal) -> i32 {
        fail!(~"nyi")
    }

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
