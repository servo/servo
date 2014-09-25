/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::MouseEventBinding;
use dom::bindings::codegen::Bindings::MouseEventBinding::MouseEventMethods;
use dom::bindings::codegen::Bindings::UIEventBinding::UIEventMethods;
use dom::bindings::codegen::InheritTypes::{UIEventCast, MouseEventDerived};
use dom::bindings::error::Fallible;
use dom::bindings::global::GlobalRef;
use dom::bindings::global;
use dom::bindings::js::{JS, JSRef, RootedReference, Temporary, OptionalSettable};
use dom::bindings::trace::Traceable;
use dom::bindings::utils::{Reflectable, Reflector, reflect_dom_object};
use dom::event::{Event, MouseEventTypeId};
use dom::eventtarget::EventTarget;
use dom::uievent::UIEvent;
use dom::window::Window;
use servo_util::str::DOMString;
use std::cell::Cell;

#[jstraceable]
#[must_root]
pub struct MouseEvent {
    pub mouseevent: UIEvent,
    pub screen_x: Traceable<Cell<i32>>,
    pub screen_y: Traceable<Cell<i32>>,
    pub client_x: Traceable<Cell<i32>>,
    pub client_y: Traceable<Cell<i32>>,
    pub ctrl_key: Traceable<Cell<bool>>,
    pub shift_key: Traceable<Cell<bool>>,
    pub alt_key: Traceable<Cell<bool>>,
    pub meta_key: Traceable<Cell<bool>>,
    pub button: Traceable<Cell<i16>>,
    pub related_target: Cell<Option<JS<EventTarget>>>
}

impl MouseEventDerived for Event {
    fn is_mouseevent(&self) -> bool {
        self.type_id == MouseEventTypeId
    }
}

impl MouseEvent {
    fn new_inherited() -> MouseEvent {
        MouseEvent {
            mouseevent: UIEvent::new_inherited(MouseEventTypeId),
            screen_x: Traceable::new(Cell::new(0)),
            screen_y: Traceable::new(Cell::new(0)),
            client_x: Traceable::new(Cell::new(0)),
            client_y: Traceable::new(Cell::new(0)),
            ctrl_key: Traceable::new(Cell::new(false)),
            shift_key: Traceable::new(Cell::new(false)),
            alt_key: Traceable::new(Cell::new(false)),
            meta_key: Traceable::new(Cell::new(false)),
            button: Traceable::new(Cell::new(0)),
            related_target: Cell::new(None)
        }
    }

    pub fn new_uninitialized(window: JSRef<Window>) -> Temporary<MouseEvent> {
        reflect_dom_object(box MouseEvent::new_inherited(),
                           &global::Window(window),
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
        ev.deref().InitMouseEvent(type_, canBubble, cancelable, view, detail,
                                  screenX, screenY, clientX, clientY,
                                  ctrlKey, altKey, shiftKey, metaKey,
                                  button, relatedTarget);
        Temporary::from_rooted(*ev)
    }

    pub fn Constructor(global: &GlobalRef,
                       type_: DOMString,
                       init: &MouseEventBinding::MouseEventInit) -> Fallible<Temporary<MouseEvent>> {
        let event = MouseEvent::new(global.as_window(), type_,
                                    init.parent.parent.bubbles,
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

impl<'a> MouseEventMethods for JSRef<'a, MouseEvent> {
    fn ScreenX(self) -> i32 {
        self.screen_x.deref().get()
    }

    fn ScreenY(self) -> i32 {
        self.screen_y.deref().get()
    }

    fn ClientX(self) -> i32 {
        self.client_x.deref().get()
    }

    fn ClientY(self) -> i32 {
        self.client_y.deref().get()
    }

    fn CtrlKey(self) -> bool {
        self.ctrl_key.deref().get()
    }

    fn ShiftKey(self) -> bool {
        self.shift_key.deref().get()
    }

    fn AltKey(self) -> bool {
        self.alt_key.deref().get()
    }

    fn MetaKey(self) -> bool {
        self.meta_key.deref().get()
    }

    fn Button(self) -> i16 {
        self.button.deref().get()
    }

    fn GetRelatedTarget(self) -> Option<Temporary<EventTarget>> {
        self.related_target.get().clone().map(|target| Temporary::new(target))
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
        let uievent: JSRef<UIEvent> = UIEventCast::from_ref(self);
        uievent.InitUIEvent(typeArg, canBubbleArg, cancelableArg, viewArg, detailArg);
        self.screen_x.deref().set(screenXArg);
        self.screen_y.deref().set(screenYArg);
        self.client_x.deref().set(clientXArg);
        self.client_y.deref().set(clientYArg);
        self.ctrl_key.deref().set(ctrlKeyArg);
        self.alt_key.deref().set(altKeyArg);
        self.shift_key.deref().set(shiftKeyArg);
        self.meta_key.deref().set(metaKeyArg);
        self.button.deref().set(buttonArg);
        self.related_target.assign(relatedTargetArg);
    }
}


impl Reflectable for MouseEvent {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.mouseevent.reflector()
    }
}
