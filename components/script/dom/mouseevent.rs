/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::MouseEventBinding;
use dom::bindings::codegen::Bindings::MouseEventBinding::MouseEventMethods;
use dom::bindings::codegen::Bindings::UIEventBinding::UIEventMethods;
use dom::bindings::codegen::InheritTypes::{EventCast, UIEventCast, MouseEventDerived};
use dom::bindings::error::Fallible;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JS, MutNullableHeap, Root, RootedReference};
use dom::bindings::utils::reflect_dom_object;
use dom::event::{Event, EventTypeId, EventBubbles, EventCancelable};
use dom::eventtarget::EventTarget;
use dom::uievent::{UIEvent, UIEventTypeId};
use dom::window::Window;
use std::cell::Cell;
use std::default::Default;
use util::prefs;
use util::str::DOMString;

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
    related_target: MutNullableHeap<JS<EventTarget>>,
}

impl MouseEventDerived for Event {
    fn is_mouseevent(&self) -> bool {
        *self.type_id() == EventTypeId::UIEvent(UIEventTypeId::MouseEvent)
    }
}

impl MouseEvent {
    fn new_inherited() -> MouseEvent {
        MouseEvent {
            uievent: UIEvent::new_inherited(UIEventTypeId::MouseEvent),
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

    pub fn new_uninitialized(window: &Window) -> Root<MouseEvent> {
        reflect_dom_object(box MouseEvent::new_inherited(),
                           GlobalRef::Window(window),
                           MouseEventBinding::Wrap)
    }

    pub fn new(window: &Window,
               type_: DOMString,
               canBubble: EventBubbles,
               cancelable: EventCancelable,
               view: Option<&Window>,
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
               relatedTarget: Option<&EventTarget>) -> Root<MouseEvent> {
        let ev = MouseEvent::new_uninitialized(window);
        ev.r().InitMouseEvent(type_, canBubble == EventBubbles::Bubbles, cancelable == EventCancelable::Cancelable,
                              view, detail,
                              screenX, screenY, clientX, clientY,
                              ctrlKey, altKey, shiftKey, metaKey,
                              button, relatedTarget);
        ev
    }

    pub fn Constructor(global: GlobalRef,
                       type_: DOMString,
                       init: &MouseEventBinding::MouseEventInit) -> Fallible<Root<MouseEvent>> {
        let bubbles = if init.parent.parent.parent.bubbles {
            EventBubbles::Bubbles
        } else {
            EventBubbles::DoesNotBubble
        };
        let cancelable = if init.parent.parent.parent.cancelable {
            EventCancelable::Cancelable
        } else {
        EventCancelable::NotCancelable
        };
        let event = MouseEvent::new(global.as_window(), type_,
                                    bubbles,
                                    cancelable,
                                    init.parent.parent.view.r(),
                                    init.parent.parent.detail,
                                    init.screenX, init.screenY,
                                    init.clientX, init.clientY, init.parent.ctrlKey,
                                    init.parent.altKey, init.parent.shiftKey, init.parent.metaKey,
                                    init.button, init.relatedTarget.r());
        Ok(event)
    }
}

impl MouseEventMethods for MouseEvent {
    // https://w3c.github.io/uievents/#widl-MouseEvent-screenX
    fn ScreenX(&self) -> i32 {
        self.screen_x.get()
    }

    // https://w3c.github.io/uievents/#widl-MouseEvent-screenY
    fn ScreenY(&self) -> i32 {
        self.screen_y.get()
    }

    // https://w3c.github.io/uievents/#widl-MouseEvent-clientX
    fn ClientX(&self) -> i32 {
        self.client_x.get()
    }

    // https://w3c.github.io/uievents/#widl-MouseEvent-clientY
    fn ClientY(&self) -> i32 {
        self.client_y.get()
    }

    // https://w3c.github.io/uievents/#widl-MouseEvent-ctrlKey
    fn CtrlKey(&self) -> bool {
        self.ctrl_key.get()
    }

    // https://w3c.github.io/uievents/#widl-MouseEvent-shiftKey
    fn ShiftKey(&self) -> bool {
        self.shift_key.get()
    }

    // https://w3c.github.io/uievents/#widl-MouseEvent-altKey
    fn AltKey(&self) -> bool {
        self.alt_key.get()
    }

    // https://w3c.github.io/uievents/#widl-MouseEvent-metaKey
    fn MetaKey(&self) -> bool {
        self.meta_key.get()
    }

    // https://w3c.github.io/uievents/#widl-MouseEvent-button
    fn Button(&self) -> i16 {
        self.button.get()
    }

    // https://w3c.github.io/uievents/#widl-MouseEvent-relatedTarget
    fn GetRelatedTarget(&self) -> Option<Root<EventTarget>> {
        self.related_target.get().map(Root::from_rooted)
    }

    // See discussion at:
    //  - https://github.com/servo/servo/issues/6643
    //  - https://bugzilla.mozilla.org/show_bug.cgi?id=1186125
    // This returns the same result as current gecko.
    // https://developer.mozilla.org/en-US/docs/Web/API/MouseEvent/which
    fn Which(&self) -> i32 {
        if prefs::get_pref("dom.mouseevent.which.enabled").unwrap_or(false) {
            (self.button.get() + 1) as i32
        } else {
            0
        }
    }

    // https://w3c.github.io/uievents/#widl-MouseEvent-initMouseEvent
    fn InitMouseEvent(&self,
                      typeArg: DOMString,
                      canBubbleArg: bool,
                      cancelableArg: bool,
                      viewArg: Option<&Window>,
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
                      relatedTargetArg: Option<&EventTarget>) {
        let event: &Event = EventCast::from_ref(self);
        if event.dispatching() {
            return;
        }

        let uievent: &UIEvent = UIEventCast::from_ref(self);
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
        self.related_target.set(relatedTargetArg.map(JS::from_ref));
    }
}
