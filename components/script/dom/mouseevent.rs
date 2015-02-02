/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::MouseEventBinding;
use dom::bindings::codegen::Bindings::MouseEventBinding::MouseEventMethods;
use dom::bindings::codegen::Bindings::UIEventBinding::UIEventMethods;
use dom::bindings::codegen::InheritTypes::{EventCast, UIEventCast, MouseEventDerived};
use dom::bindings::error::Fallible;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{MutNullableJS, JSRef, RootedReference, Temporary};
use dom::bindings::utils::reflect_dom_object;
use dom::event::{Event, EventTypeId};
use dom::eventtarget::EventTarget;
use dom::uievent::UIEvent;
use dom::window::Window;
use util::str::DOMString;
use std::cell::Cell;
use std::default::Default;

#[dom_struct]
pub struct MouseEvent {
    uievent: UIEvent,
    screen_x: Cell<i32>,
    screen_y: Cell<i32>,
    client_x: Cell<i32>,
    client_y: Cell<i32>,
    ctrl_key: Cell<bool>,
    shift_key: Cell<bool>,
    alt_key: Cell<bool>,
    meta_key: Cell<bool>,
    button: Cell<i16>,
    related_target: MutNullableJS<EventTarget>
}

impl MouseEventDerived for Event {
    fn is_mouseevent(&self) -> bool {
        *self.type_id() == EventTypeId::MouseEvent
    }
}

impl MouseEvent {
    fn new_inherited() -> MouseEvent {
        MouseEvent {
            uievent: UIEvent::new_inherited(EventTypeId::MouseEvent),
            screen_x: Cell::new(0),
            screen_y: Cell::new(0),
            client_x: Cell::new(0),
            client_y: Cell::new(0),
            ctrl_key: Cell::new(false),
            shift_key: Cell::new(false),
            alt_key: Cell::new(false),
            meta_key: Cell::new(false),
            button: Cell::new(0),
            related_target: Default::default(),
        }
    }

    pub fn new_uninitialized(window: JSRef<Window>) -> Temporary<MouseEvent> {
        reflect_dom_object(box MouseEvent::new_inherited(),
                           GlobalRef::Window(window),
                           MouseEventBinding::Wrap)
    }

    pub fn new(window: JSRef<Window>,
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
        let ev = MouseEvent::new_uninitialized(window).root();
        ev.r().InitMouseEvent(type_, canBubble, cancelable, view, detail,
                              screenX, screenY, clientX, clientY,
                              ctrlKey, altKey, shiftKey, metaKey,
                              button, relatedTarget);
        Temporary::from_rooted(ev.r())
    }

    pub fn Constructor(global: GlobalRef,
                       type_: DOMString,
                       init: &MouseEventBinding::MouseEventInit) -> Fallible<Temporary<MouseEvent>> {
        let event = MouseEvent::new(global.as_window(), type_,
                                    init.parent.parent.parent.bubbles,
                                    init.parent.parent.parent.cancelable,
                                    init.parent.parent.view.r(),
                                    init.parent.parent.detail,
                                    init.screenX, init.screenY,
                                    init.clientX, init.clientY, init.parent.ctrlKey,
                                    init.parent.altKey, init.parent.shiftKey, init.parent.metaKey,
                                    init.button, init.relatedTarget.r());
        Ok(event)
    }
}

impl<'a> MouseEventMethods for JSRef<'a, MouseEvent> {
    fn ScreenX(self) -> i32 {
        self.screen_x.get()
    }

    fn ScreenY(self) -> i32 {
        self.screen_y.get()
    }

    fn ClientX(self) -> i32 {
        self.client_x.get()
    }

    fn ClientY(self) -> i32 {
        self.client_y.get()
    }

    fn CtrlKey(self) -> bool {
        self.ctrl_key.get()
    }

    fn ShiftKey(self) -> bool {
        self.shift_key.get()
    }

    fn AltKey(self) -> bool {
        self.alt_key.get()
    }

    fn MetaKey(self) -> bool {
        self.meta_key.get()
    }

    fn Button(self) -> i16 {
        self.button.get()
    }

    fn GetRelatedTarget(self) -> Option<Temporary<EventTarget>> {
        self.related_target.get()
    }

    fn InitMouseEvent(self,
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
        let event: JSRef<Event> = EventCast::from_ref(self);
        if event.dispatching() {
            return;
        }

        let uievent: JSRef<UIEvent> = UIEventCast::from_ref(self);
        uievent.InitUIEvent(typeArg, canBubbleArg, cancelableArg, viewArg, detailArg);
        self.screen_x.set(screenXArg);
        self.screen_y.set(screenYArg);
        self.client_x.set(clientXArg);
        self.client_y.set(clientYArg);
        self.ctrl_key.set(ctrlKeyArg);
        self.alt_key.set(altKeyArg);
        self.shift_key.set(shiftKeyArg);
        self.meta_key.set(metaKeyArg);
        self.button.set(buttonArg);
        self.related_target.assign(relatedTargetArg);
    }
}

