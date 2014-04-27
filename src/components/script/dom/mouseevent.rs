/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::MouseEventBinding;
use dom::bindings::codegen::InheritTypes::MouseEventDerived;
use dom::bindings::js::JS;
use dom::bindings::error::Fallible;
use dom::bindings::utils::{Reflectable, Reflector, reflect_dom_object};
use dom::event::{Event, MouseEventTypeId};
use dom::eventtarget::EventTarget;
use dom::uievent::UIEvent;
use dom::window::Window;
use servo_util::str::DOMString;

#[deriving(Encodable)]
pub struct MouseEvent {
    pub mouseevent: UIEvent,
    pub screen_x: i32,
    pub screen_y: i32,
    pub client_x: i32,
    pub client_y: i32,
    pub ctrl_key: bool,
    pub shift_key: bool,
    pub alt_key: bool,
    pub meta_key: bool,
    pub button: u16,
    pub related_target: Option<JS<EventTarget>>
}

impl MouseEventDerived for Event {
    fn is_mouseevent(&self) -> bool {
        self.type_id == MouseEventTypeId
    }
}

impl MouseEvent {
    pub fn new_inherited() -> MouseEvent {
        MouseEvent {
            mouseevent: UIEvent::new_inherited(MouseEventTypeId),
            screen_x: 0,
            screen_y: 0,
            client_x: 0,
            client_y: 0,
            ctrl_key: false,
            shift_key: false,
            alt_key: false,
            meta_key: false,
            button: 0,
            related_target: None
        }
    }

    pub fn new(window: &JS<Window>) -> JS<MouseEvent> {
        reflect_dom_object(~MouseEvent::new_inherited(),
                           window,
                           MouseEventBinding::Wrap)
    }

    pub fn Constructor(owner: &JS<Window>,
                       type_: DOMString,
                       init: &MouseEventBinding::MouseEventInit) -> Fallible<JS<MouseEvent>> {
        let mut ev = MouseEvent::new(owner);
        ev.get_mut().InitMouseEvent(type_, init.bubbles, init.cancelable, init.view.clone(),
                                      init.detail, init.screenX, init.screenY,
                                      init.clientX, init.clientY, init.ctrlKey,
                                      init.altKey, init.shiftKey, init.metaKey,
                                      init.button, init.relatedTarget.clone());
        Ok(ev)
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

    pub fn GetRelatedTarget(&self) -> Option<JS<EventTarget>> {
        self.related_target.clone()
    }

    pub fn GetModifierState(&self, _keyArg: DOMString) -> bool {
        //TODO
        false
    }

    pub fn InitMouseEvent(&mut self,
                          typeArg: DOMString,
                          canBubbleArg: bool,
                          cancelableArg: bool,
                          viewArg: Option<JS<Window>>,
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
                          relatedTargetArg: Option<JS<EventTarget>>) {
        self.mouseevent.InitUIEvent(typeArg, canBubbleArg, cancelableArg, viewArg, detailArg);
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
    }
}

impl Reflectable for MouseEvent {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.mouseevent.reflector()
    }

    fn mut_reflector<'a>(&'a mut self) -> &'a mut Reflector {
        self.mouseevent.mut_reflector()
    }
}
