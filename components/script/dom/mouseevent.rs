/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::MouseEventBinding;
use dom::bindings::codegen::Bindings::MouseEventBinding::MouseEventMethods;
use dom::bindings::codegen::Bindings::UIEventBinding::UIEventMethods;
use dom::bindings::error::Fallible;
use dom::bindings::inheritance::Castable;
use dom::bindings::reflector::reflect_dom_object;
use dom::bindings::root::{DomRoot, MutNullableDom, RootedReference};
use dom::bindings::str::DOMString;
use dom::event::{Event, EventBubbles, EventCancelable};
use dom::eventtarget::EventTarget;
use dom::uievent::UIEvent;
use dom::window::Window;
use dom_struct::dom_struct;
use servo_config::prefs::PREFS;
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
    related_target: MutNullableDom<EventTarget>,
}

impl MouseEvent {
    fn new_inherited() -> MouseEvent {
        MouseEvent {
            uievent: UIEvent::new_inherited(),
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

    pub fn new_uninitialized(window: &Window) -> DomRoot<MouseEvent> {
        reflect_dom_object(Box::new(MouseEvent::new_inherited()),
                           window,
                           MouseEventBinding::Wrap)
    }

    pub fn new(window: &Window,
               type_: DOMString,
               can_bubble: EventBubbles,
               cancelable: EventCancelable,
               view: Option<&Window>,
               detail: i32,
               screen_x: i32,
               screen_y: i32,
               client_x: i32,
               client_y: i32,
               ctrl_key: bool,
               alt_key: bool,
               shift_key: bool,
               meta_key: bool,
               button: i16,
               related_target: Option<&EventTarget>) -> DomRoot<MouseEvent> {
        let ev = MouseEvent::new_uninitialized(window);
        ev.InitMouseEvent(type_, bool::from(can_bubble), bool::from(cancelable),
                          view, detail,
                          screen_x, screen_y, client_x, client_y,
                          ctrl_key, alt_key, shift_key, meta_key,
                          button, related_target);
        ev
    }

    pub fn Constructor(window: &Window,
                       type_: DOMString,
                       init: &MouseEventBinding::MouseEventInit) -> Fallible<DomRoot<MouseEvent>> {
        let bubbles = EventBubbles::from(init.parent.parent.parent.bubbles);
        let cancelable = EventCancelable::from(init.parent.parent.parent.cancelable);
        let event = MouseEvent::new(window,
                                    type_,
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
    fn GetRelatedTarget(&self) -> Option<DomRoot<EventTarget>> {
        self.related_target.get()
    }

    // See discussion at:
    //  - https://github.com/servo/servo/issues/6643
    //  - https://bugzilla.mozilla.org/show_bug.cgi?id=1186125
    // This returns the same result as current gecko.
    // https://developer.mozilla.org/en-US/docs/Web/API/MouseEvent/which
    fn Which(&self) -> i32 {
        if PREFS.get("dom.mouseevent.which.enabled").as_boolean().unwrap_or(false) {
            (self.button.get() + 1) as i32
        } else {
            0
        }
    }

    // https://w3c.github.io/uievents/#widl-MouseEvent-initMouseEvent
    fn InitMouseEvent(&self,
                      type_arg: DOMString,
                      can_bubble_arg: bool,
                      cancelable_arg: bool,
                      view_arg: Option<&Window>,
                      detail_arg: i32,
                      screen_x_arg: i32,
                      screen_y_arg: i32,
                      client_x_arg: i32,
                      client_y_arg: i32,
                      ctrl_key_arg: bool,
                      alt_key_arg: bool,
                      shift_key_arg: bool,
                      meta_key_arg: bool,
                      button_arg: i16,
                      related_target_arg: Option<&EventTarget>) {
        if self.upcast::<Event>().dispatching() {
            return;
        }

        self.upcast::<UIEvent>()
            .InitUIEvent(type_arg, can_bubble_arg, cancelable_arg, view_arg, detail_arg);
        self.screen_x.set(screen_x_arg);
        self.screen_y.set(screen_y_arg);
        self.client_x.set(client_x_arg);
        self.client_y.set(client_y_arg);
        self.ctrl_key.set(ctrl_key_arg);
        self.alt_key.set(alt_key_arg);
        self.shift_key.set(shift_key_arg);
        self.meta_key.set(meta_key_arg);
        self.button.set(button_arg);
        self.related_target.set(related_target_arg);
    }

    // https://dom.spec.whatwg.org/#dom-event-istrusted
    fn IsTrusted(&self) -> bool {
        self.uievent.IsTrusted()
    }
}
