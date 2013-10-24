/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::MouseEventBinding;
use dom::bindings::utils::{ErrorResult, Fallible, DOMString};
use dom::bindings::utils::{Reflectable, Reflector, reflect_dom_object};
use dom::eventtarget::EventTarget;
use dom::uievent::UIEvent;
use dom::window::Window;
use dom::windowproxy::WindowProxy;

use js::jsapi::{JSObject, JSContext};

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
    pub fn new(window: @mut Window, type_: &DOMString, can_bubble: bool, cancelable: bool,
               view: Option<@mut WindowProxy>, detail: i32, screen_x: i32,
               screen_y: i32, client_x: i32, client_y: i32, ctrl_key: bool,
               shift_key: bool, alt_key: bool, meta_key: bool, button: u16,
               _buttons: u16, related_target: Option<@mut EventTarget>) -> @mut MouseEvent {
        let ev = @mut MouseEvent {
            parent: UIEvent::new_inherited(type_, can_bubble, cancelable, view, detail),
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
        };
        reflect_dom_object(ev, window, MouseEventBinding::Wrap)
    }

    pub fn Constructor(owner: @mut Window,
                       type_: &DOMString,
                       init: &MouseEventBinding::MouseEventInit) -> Fallible<@mut MouseEvent> {
        Ok(MouseEvent::new(owner, type_, init.bubbles, init.cancelable, init.view, init.detail,
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

impl Reflectable for MouseEvent {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.parent.reflector()
    }

    fn mut_reflector<'a>(&'a mut self) -> &'a mut Reflector {
        self.parent.mut_reflector()
    }

    fn wrap_object_shared(@mut self, _cx: *JSContext, _scope: *JSObject) -> *JSObject {
        unreachable!()
    }

    fn GetParentObject(&self, cx: *JSContext) -> Option<@mut Reflectable> {
        self.parent.GetParentObject(cx)
    }
}
