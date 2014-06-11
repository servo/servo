/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::MouseEventBinding;
use dom::bindings::codegen::InheritTypes::{UIEventCast, MouseEventDerived};
use dom::bindings::js::{JS, JSRef, RootedReference, Temporary, OptionalSettable};
use dom::bindings::error::Fallible;
use dom::bindings::utils::reflect_dom_object;
use dom::event::{Event, MouseEventTypeId};
use dom::eventtarget::EventTarget;
use dom::uievent::{UIEvent, UIEventMethods};
use dom::window::Window;
use servo_util::str::DOMString;
use std::cell::Cell;

#[deriving(Encodable)]
pub struct MouseEvent {
    pub uievent: UIEvent,
    pub screen_x: i32,
    pub screen_y: i32,
    pub client_x: i32,
    pub client_y: i32,
    pub ctrl_key: bool,
    pub shift_key: bool,
    pub alt_key: bool,
    pub meta_key: bool,
    pub button: i16,
    pub related_target: Cell<Option<JS<EventTarget>>>
}

impl MouseEventDerived for Event {
    fn is_mouseevent(&self) -> bool {
        self.type_id == MouseEventTypeId
    }
}

impl MouseEvent {
    pub fn new_inherited() -> MouseEvent {
        MouseEvent {
            uievent: UIEvent::new_inherited(MouseEventTypeId),
            screen_x: 0,
            screen_y: 0,
            client_x: 0,
            client_y: 0,
            ctrl_key: false,
            shift_key: false,
            alt_key: false,
            meta_key: false,
            button: 0,
            related_target: Cell::new(None)
        }
    }

    pub fn new_uninitialized(window: &JSRef<Window>) -> Temporary<MouseEvent> {
        reflect_dom_object(box MouseEvent::new_inherited(),
                           window,
                           MouseEventBinding::Wrap)
    }

    pub fn new(window: &JSRef<Window>,
               type_: DOMString,
               canBubble: bool,
               cancelable: bool,
               view: Option<JSRef<Window>>,
               detail: i32,
               screenX: i32,
               screenY: i32,
               clientX: i32,
               clientY: i32,
               ctrlKey: bool,
               altKey: bool,
               shiftKey: bool,
               metaKey: bool,
               button: i16,
               relatedTarget: Option<JSRef<EventTarget>>) -> Temporary<MouseEvent> {
        let mut ev = MouseEvent::new_uninitialized(window).root();
        ev.InitMouseEvent(type_, canBubble, cancelable, view, detail,
                          screenX, screenY, clientX, clientY,
                          ctrlKey, altKey, shiftKey, metaKey,
                          button, relatedTarget);
        Temporary::from_rooted(&*ev)
    }

    pub fn Constructor(owner: &JSRef<Window>,
                       type_: DOMString,
                       init: &MouseEventBinding::MouseEventInit) -> Fallible<Temporary<MouseEvent>> {
        let event = MouseEvent::new(owner, type_, init.parent.parent.bubbles,
                                    init.parent.parent.cancelable,
                                    init.parent.view.root_ref(),
                                    init.parent.detail,
                                    init.screenX, init.screenY,
                                    init.clientX, init.clientY, init.ctrlKey,
                                    init.altKey, init.shiftKey, init.metaKey,
                                    init.button, init.relatedTarget.root_ref());
        Ok(event)
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
    fn Button(&self) -> i16;
    fn GetRelatedTarget(&self) -> Option<Temporary<EventTarget>>;
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
                      buttonArg: i16,
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

    fn Button(&self) -> i16 {
        self.button
    }

    fn GetRelatedTarget(&self) -> Option<Temporary<EventTarget>> {
        self.related_target.get().clone().map(|target| Temporary::new(target))
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
                      buttonArg: i16,
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
