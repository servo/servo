/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::BindingDeclarations::MouseEventBinding;
use dom::bindings::codegen::InheritTypes::{UIEventCast, MouseEventDerived};
use dom::bindings::js::{JS, JSRef, RootedReference, Temporary, OptionalSettable};
use dom::bindings::error::Fallible;
use dom::bindings::utils::{Reflectable, Reflector, reflect_dom_object};
use dom::event::{Event, MouseEventTypeId};
use dom::eventtarget::EventTarget;
use dom::uievent::{UIEvent, UIEventMethods};
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

    pub fn new(window: &JSRef<Window>) -> Temporary<MouseEvent> {
        reflect_dom_object(~MouseEvent::new_inherited(),
                           window,
                           MouseEventBinding::Wrap)
    }

    pub fn Constructor(owner: &JSRef<Window>,
                       type_: DOMString,
                       init: &MouseEventBinding::MouseEventInit) -> Fallible<Temporary<MouseEvent>> {
        let mut ev = MouseEvent::new(owner).root();
        ev.InitMouseEvent(type_, init.bubbles, init.cancelable, init.view.root_ref(),
                          init.detail, init.screenX, init.screenY,
                          init.clientX, init.clientY, init.ctrlKey,
                          init.altKey, init.shiftKey, init.metaKey,
                          init.button, init.relatedTarget.root_ref());
        Ok(Temporary::from_rooted(&*ev))
    }
}

pub trait MouseEventMethods {
    fn ScreenX(&self) -> i32;
    fn ScreenY(&self) -> i32;
    fn ClientX(&self) -> i32;
    fn ClientY(&self) -> i32;
    fn CtrlKey(&self) -> bool;
    fn ShiftKey(&self) -> bool;
    fn AltKey(&self) -> bool;
    fn MetaKey(&self) -> bool;
    fn Button(&self) -> u16;
    fn Buttons(&self)-> u16;
    fn GetRelatedTarget(&self) -> Option<Temporary<EventTarget>>;
    fn GetModifierState(&self, _keyArg: DOMString) -> bool;
    fn InitMouseEvent(&mut self,
                      typeArg: DOMString,
                      canBubbleArg: bool,
                      cancelableArg: bool,
                      viewArg: Option<JSRef<Window>>,
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
                      relatedTargetArg: Option<JSRef<EventTarget>>);
}

impl<'a> MouseEventMethods for JSRef<'a, MouseEvent> {
    fn ScreenX(&self) -> i32 {
        self.screen_x
    }

    fn ScreenY(&self) -> i32 {
        self.screen_y
    }

    fn ClientX(&self) -> i32 {
        self.client_x
    }

    fn ClientY(&self) -> i32 {
        self.client_y
    }

    fn CtrlKey(&self) -> bool {
        self.ctrl_key
    }

    fn ShiftKey(&self) -> bool {
        self.shift_key
    }

    fn AltKey(&self) -> bool {
        self.alt_key
    }

    fn MetaKey(&self) -> bool {
        self.meta_key
    }

    fn Button(&self) -> u16 {
        self.button
    }

    fn Buttons(&self)-> u16 {
        //TODO
        0
    }

    fn GetRelatedTarget(&self) -> Option<Temporary<EventTarget>> {
        self.related_target.clone().map(|target| Temporary::new(target))
    }

    fn GetModifierState(&self, _keyArg: DOMString) -> bool {
        //TODO
        false
    }

    fn InitMouseEvent(&mut self,
                      typeArg: DOMString,
                      canBubbleArg: bool,
                      cancelableArg: bool,
                      viewArg: Option<JSRef<Window>>,
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
                      relatedTargetArg: Option<JSRef<EventTarget>>) {
        {
            let uievent: &mut JSRef<UIEvent> = UIEventCast::from_mut_ref(self);
            uievent.InitUIEvent(typeArg, canBubbleArg, cancelableArg, viewArg, detailArg);
        }
        self.screen_x = screenXArg;
        self.screen_y = screenYArg;
        self.client_x = clientXArg;
        self.client_y = clientYArg;
        self.ctrl_key = ctrlKeyArg;
        self.alt_key = altKeyArg;
        self.shift_key = shiftKeyArg;
        self.meta_key = metaKeyArg;
        self.button = buttonArg;
        self.related_target.assign(relatedTargetArg);
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
