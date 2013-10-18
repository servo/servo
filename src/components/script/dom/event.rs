/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::eventtarget::EventTarget;
use dom::window::Window;
use dom::bindings::codegen::EventBinding;
use dom::bindings::utils::{Reflectable, Reflector};
use dom::bindings::utils::{DOMString, ErrorResult, Fallible};

use geom::point::Point2D;
use js::jsapi::{JSObject, JSContext};

use script_task::page_from_context;

pub enum Event_ {
    ResizeEvent(uint, uint), 
    ReflowEvent,
    ClickEvent(uint, Point2D<f32>),
    MouseDownEvent(uint, Point2D<f32>),
    MouseUpEvent(uint, Point2D<f32>),
}

pub struct Event {
    reflector_: Reflector,
    type_: DOMString,
    default_prevented: bool,
    cancelable: bool,
    bubbles: bool,
    trusted: bool,
}

impl Event {
    pub fn new(type_: &DOMString) -> Event {
        Event {
            reflector_: Reflector::new(),
            type_: (*type_).clone(),
            default_prevented: false,
            cancelable: true,
            bubbles: true,
            trusted: false
        }
    }

    pub fn init_wrapper(@mut self, cx: *JSContext, scope: *JSObject) {
        self.wrap_object_shared(cx, scope);
    }

    pub fn EventPhase(&self) -> u16 {
        0
    }

    pub fn Type(&self) -> DOMString {
        self.type_.clone()
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
                     type_: &DOMString,
                     bubbles: bool,
                     cancelable: bool) -> ErrorResult {
        self.type_ = (*type_).clone();
        self.cancelable = cancelable;
        self.bubbles = bubbles;
        Ok(())
    }

    pub fn IsTrusted(&self) -> bool {
        self.trusted
    }

    pub fn Constructor(_global: @mut Window,
                   type_: &DOMString,
                   _init: &EventBinding::EventInit) -> Fallible<@mut Event> {
        Ok(@mut Event::new(type_))
    }
}

impl Reflectable for Event {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        &self.reflector_
    }

    fn mut_reflector<'a>(&'a mut self) -> &'a mut Reflector {
        &mut self.reflector_
    }

    fn wrap_object_shared(@mut self, cx: *JSContext, scope: *JSObject) -> *JSObject {
        EventBinding::Wrap(cx, scope, self)
    }

    fn GetParentObject(&self, cx: *JSContext) -> Option<@mut Reflectable> {
        let page = page_from_context(cx);
        unsafe {
            Some((*page).frame.get_ref().window as @mut Reflectable)
        }
    }
}
