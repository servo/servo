/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::MouseEventBinding;
use dom::bindings::utils::{ErrorResult, Fallible, DOMString};
use dom::bindings::utils::{CacheableWrapper, WrapperCache, BindingObject, DerivedWrapper};
use dom::eventtarget::EventTarget;
use dom::uievent::UIEvent;
use dom::window::Window;
use dom::windowproxy::WindowProxy;

use js::glue::RUST_OBJECT_TO_JSVAL;
use js::jsapi::{JSObject, JSContext, JSVal};

pub struct MouseEvent {
    parent: UIEvent,
    screen_x: i32,
    screen_y: i32,
    client_x: i32,
    client_y: i32,
    ctrl_key: bool,
    shift_key: bool,
    alt_key: bool,
    meta_key: bool,
    button: u16,
    related_target: Option<@mut EventTarget>
}

impl MouseEvent {
    pub fn new(type_: &DOMString, can_bubble: bool, cancelable: bool,
               view: Option<@mut WindowProxy>, detail: i32, screen_x: i32,
               screen_y: i32, client_x: i32, client_y: i32, ctrl_key: bool,
               shift_key: bool, alt_key: bool, meta_key: bool, button: u16,
               _buttons: u16, related_target: Option<@mut EventTarget>) -> MouseEvent {
        MouseEvent {
            parent: UIEvent::new(type_, can_bubble, cancelable, view, detail),
            screen_x: screen_x,
            screen_y: screen_y,
            client_x: client_x,
            client_y: client_y,
            ctrl_key: ctrl_key,
            shift_key: shift_key,
            alt_key: alt_key,
            meta_key: meta_key,
            button: button,
            related_target: related_target
        }
    }

    pub fn init_wrapper(@mut self, cx: *JSContext, scope: *JSObject) {
        self.wrap_object_shared(cx, scope);
    }

    pub fn Constructor(_owner: @mut Window,
                       type_: &DOMString,
                       init: &MouseEventBinding::MouseEventInit) -> Fallible<@mut MouseEvent> {
        Ok(@mut MouseEvent::new(type_, init.bubbles, init.cancelable, init.view, init.detail,
                                init.screenX, init.screenY, init.clientX, init.clientY,
                                init.ctrlKey, init.shiftKey, init.altKey, init.metaKey,
                                init.button, init.buttons, init.relatedTarget))
    }

    pub fn ScreenX(&self) -> i32 {
        self.screen_x
    }

    pub fn ScreenY(&self) -> i32 {
        self.screen_y
    }

    pub fn ClientX(&self) -> i32 {
        self.client_x
    }

    pub fn ClientY(&self) -> i32 {
        self.client_y
    }

    pub fn CtrlKey(&self) -> bool {
        self.ctrl_key
    }

    pub fn ShiftKey(&self) -> bool {
        self.shift_key
    }

    pub fn AltKey(&self) -> bool {
        self.alt_key
    }

    pub fn MetaKey(&self) -> bool {
        self.meta_key
    }

    pub fn Button(&self) -> u16 {
        self.button
    }

    pub fn Buttons(&self)-> u16 {
        //TODO
        0
    }

    pub fn GetRelatedTarget(&self) -> Option<@mut EventTarget> {
        self.related_target
    }

    pub fn GetModifierState(&self, _keyArg: &DOMString) -> bool {
        //TODO
        false
    }

    pub fn InitMouseEvent(&mut self,
                          typeArg: &DOMString,
                          canBubbleArg: bool,
                          cancelableArg: bool,
                          viewArg: Option<@mut WindowProxy>,
                          detailArg: i32,
                          screenXArg: i32,
                          screenYArg: i32,
                          clientXArg: i32,
                          clientYArg: i32,
                          ctrlKeyArg: bool,
                          altKeyArg: bool,
                          shiftKeyArg: bool,
                          metaKeyArg: bool,
                          buttonArg: u16,
                          relatedTargetArg: Option<@mut EventTarget>) -> ErrorResult {
        self.parent.InitUIEvent(typeArg, canBubbleArg, cancelableArg, viewArg, detailArg);
        self.screen_x = screenXArg;
        self.screen_y = screenYArg;
        self.client_x = clientXArg;
        self.client_y = clientYArg;
        self.ctrl_key = ctrlKeyArg;
        self.alt_key = altKeyArg;
        self.shift_key = shiftKeyArg;
        self.meta_key = metaKeyArg;
        self.button = buttonArg;
        self.related_target = relatedTargetArg;
        Ok(())
    }
}

impl CacheableWrapper for MouseEvent {
    fn get_wrappercache(&mut self) -> &mut WrapperCache {
        return self.parent.get_wrappercache()
    }

    fn wrap_object_shared(@mut self, cx: *JSContext, scope: *JSObject) -> *JSObject {
        let mut unused = false;
        MouseEventBinding::Wrap(cx, scope, self, &mut unused)
    }
}

impl BindingObject for MouseEvent {
    fn GetParentObject(&self, cx: *JSContext) -> Option<@mut CacheableWrapper> {
        self.parent.GetParentObject(cx)
    }
}

impl DerivedWrapper for MouseEvent {
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
