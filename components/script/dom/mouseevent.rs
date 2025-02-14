/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::default::Default;

use dom_struct::dom_struct;
use euclid::default::Point2D;
use js::rust::HandleObject;
use servo_config::pref;

use crate::dom::bindings::codegen::Bindings::EventBinding::Event_Binding::EventMethods;
use crate::dom::bindings::codegen::Bindings::MouseEventBinding;
use crate::dom::bindings::codegen::Bindings::MouseEventBinding::MouseEventMethods;
use crate::dom::bindings::codegen::Bindings::UIEventBinding::UIEventMethods;
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::{reflect_dom_object_with_proto, DomGlobal};
use crate::dom::bindings::root::{DomRoot, MutNullableDom};
use crate::dom::bindings::str::DOMString;
use crate::dom::event::{Event, EventBubbles, EventCancelable};
use crate::dom::eventtarget::EventTarget;
use crate::dom::node::Node;
use crate::dom::uievent::UIEvent;
use crate::dom::window::Window;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct MouseEvent {
    uievent: UIEvent,
    screen_x: Cell<i32>,
    screen_y: Cell<i32>,
    client_x: Cell<i32>,
    client_y: Cell<i32>,
    page_x: Cell<i32>,
    page_y: Cell<i32>,
    x: Cell<i32>,
    y: Cell<i32>,
    offset_x: Cell<i32>,
    offset_y: Cell<i32>,
    ctrl_key: Cell<bool>,
    shift_key: Cell<bool>,
    alt_key: Cell<bool>,
    meta_key: Cell<bool>,
    button: Cell<i16>,
    buttons: Cell<u16>,
    related_target: MutNullableDom<EventTarget>,
    #[no_trace]
    point_in_target: Cell<Option<Point2D<f32>>>,
}

impl MouseEvent {
    pub(crate) fn new_inherited() -> MouseEvent {
        MouseEvent {
            uievent: UIEvent::new_inherited(),
            screen_x: Cell::new(0),
            screen_y: Cell::new(0),
            client_x: Cell::new(0),
            client_y: Cell::new(0),
            page_x: Cell::new(0),
            page_y: Cell::new(0),
            x: Cell::new(0),
            y: Cell::new(0),
            offset_x: Cell::new(0),
            offset_y: Cell::new(0),
            ctrl_key: Cell::new(false),
            shift_key: Cell::new(false),
            alt_key: Cell::new(false),
            meta_key: Cell::new(false),
            button: Cell::new(0),
            buttons: Cell::new(0),
            related_target: Default::default(),
            point_in_target: Cell::new(None),
        }
    }

    pub(crate) fn new_uninitialized(window: &Window, can_gc: CanGc) -> DomRoot<MouseEvent> {
        Self::new_uninitialized_with_proto(window, None, can_gc)
    }

    fn new_uninitialized_with_proto(
        window: &Window,
        proto: Option<HandleObject>,
        can_gc: CanGc,
    ) -> DomRoot<MouseEvent> {
        reflect_dom_object_with_proto(Box::new(MouseEvent::new_inherited()), window, proto, can_gc)
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        window: &Window,
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
        buttons: u16,
        related_target: Option<&EventTarget>,
        point_in_target: Option<Point2D<f32>>,
        can_gc: CanGc,
    ) -> DomRoot<MouseEvent> {
        Self::new_with_proto(
            window,
            None,
            type_,
            can_bubble,
            cancelable,
            view,
            detail,
            screen_x,
            screen_y,
            client_x,
            client_y,
            ctrl_key,
            alt_key,
            shift_key,
            meta_key,
            button,
            buttons,
            related_target,
            point_in_target,
            can_gc,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn new_with_proto(
        window: &Window,
        proto: Option<HandleObject>,
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
        buttons: u16,
        related_target: Option<&EventTarget>,
        point_in_target: Option<Point2D<f32>>,
        can_gc: CanGc,
    ) -> DomRoot<MouseEvent> {
        let ev = MouseEvent::new_uninitialized_with_proto(window, proto, can_gc);
        ev.initialize_mouse_event(
            type_,
            can_bubble,
            cancelable,
            view,
            detail,
            screen_x,
            screen_y,
            client_x,
            client_y,
            ctrl_key,
            alt_key,
            shift_key,
            meta_key,
            button,
            buttons,
            related_target,
            point_in_target,
        );
        ev
    }

    /// <https://w3c.github.io/uievents/#initialize-a-mouseevent>
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn initialize_mouse_event(
        &self,
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
        buttons: u16,
        related_target: Option<&EventTarget>,
        point_in_target: Option<Point2D<f32>>,
    ) {
        self.uievent.initialize_ui_event(
            type_,
            view.map(|window| window.upcast::<EventTarget>()),
            can_bubble,
            cancelable,
        );

        self.uievent.set_detail(detail);

        self.screen_x.set(screen_x);
        self.screen_y.set(screen_y);
        self.client_x.set(client_x);
        self.client_y.set(client_y);
        self.page_x.set(self.PageX());
        self.page_y.set(self.PageY());

        // skip setting flags as they are absent
        self.shift_key.set(shift_key);
        self.ctrl_key.set(ctrl_key);
        self.alt_key.set(alt_key);
        self.meta_key.set(meta_key);

        self.button.set(button);
        self.buttons.set(buttons);
        // skip step 3: Initialize PointerLock attributes for MouseEvent with event,
        // as movementX, movementY is absent

        self.related_target.set(related_target);

        // below is not in the spec
        self.point_in_target.set(point_in_target);
    }

    pub(crate) fn point_in_target(&self) -> Option<Point2D<f32>> {
        self.point_in_target.get()
    }
}

impl MouseEventMethods<crate::DomTypeHolder> for MouseEvent {
    // https://w3c.github.io/uievents/#dom-mouseevent-mouseevent
    fn Constructor(
        window: &Window,
        proto: Option<HandleObject>,
        can_gc: CanGc,
        type_: DOMString,
        init: &MouseEventBinding::MouseEventInit,
    ) -> Fallible<DomRoot<MouseEvent>> {
        let bubbles = EventBubbles::from(init.parent.parent.parent.bubbles);
        let cancelable = EventCancelable::from(init.parent.parent.parent.cancelable);
        let event = MouseEvent::new_with_proto(
            window,
            proto,
            type_,
            bubbles,
            cancelable,
            init.parent.parent.view.as_deref(),
            init.parent.parent.detail,
            init.screenX,
            init.screenY,
            init.clientX,
            init.clientY,
            init.parent.ctrlKey,
            init.parent.altKey,
            init.parent.shiftKey,
            init.parent.metaKey,
            init.button,
            init.buttons,
            init.relatedTarget.as_deref(),
            None,
            can_gc,
        );
        Ok(event)
    }

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

    // https://drafts.csswg.org/cssom-view/#dom-mouseevent-pagex
    fn PageX(&self) -> i32 {
        if self.upcast::<Event>().dispatching() {
            self.page_x.get()
        } else {
            let global = self.global();
            let window = global.as_window();
            window.current_viewport().origin.x.to_px() + self.client_x.get()
        }
    }

    // https://drafts.csswg.org/cssom-view/#dom-mouseevent-pagey
    fn PageY(&self) -> i32 {
        if self.upcast::<Event>().dispatching() {
            self.page_y.get()
        } else {
            let global = self.global();
            let window = global.as_window();
            window.current_viewport().origin.y.to_px() + self.client_y.get()
        }
    }

    // https://drafts.csswg.org/cssom-view/#dom-mouseevent-x
    fn X(&self) -> i32 {
        self.client_x.get()
    }

    // https://drafts.csswg.org/cssom-view/#dom-mouseevent-y
    fn Y(&self) -> i32 {
        self.client_y.get()
    }

    // https://drafts.csswg.org/cssom-view/#dom-mouseevent-offsetx
    fn OffsetX(&self, can_gc: CanGc) -> i32 {
        let event = self.upcast::<Event>();
        if event.dispatching() {
            match event.GetTarget() {
                Some(target) => {
                    if let Some(node) = target.downcast::<Node>() {
                        let rect = node.client_rect(can_gc);
                        self.client_x.get() - rect.origin.x
                    } else {
                        self.offset_x.get()
                    }
                },
                None => self.offset_x.get(),
            }
        } else {
            self.PageX()
        }
    }

    // https://drafts.csswg.org/cssom-view/#dom-mouseevent-offsety
    fn OffsetY(&self, can_gc: CanGc) -> i32 {
        let event = self.upcast::<Event>();
        if event.dispatching() {
            match event.GetTarget() {
                Some(target) => {
                    if let Some(node) = target.downcast::<Node>() {
                        let rect = node.client_rect(can_gc);
                        self.client_y.get() - rect.origin.y
                    } else {
                        self.offset_y.get()
                    }
                },
                None => self.offset_y.get(),
            }
        } else {
            self.PageY()
        }
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

    // https://w3c.github.io/uievents/#dom-mouseevent-buttons
    fn Buttons(&self) -> u16 {
        self.buttons.get()
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
        if pref!(dom_mouse_event_which_enabled) {
            (self.button.get() + 1) as i32
        } else {
            0
        }
    }

    // https://w3c.github.io/uievents/#widl-MouseEvent-initMouseEvent
    fn InitMouseEvent(
        &self,
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
        related_target_arg: Option<&EventTarget>,
    ) {
        if self.upcast::<Event>().dispatching() {
            return;
        }

        self.upcast::<UIEvent>().InitUIEvent(
            type_arg,
            can_bubble_arg,
            cancelable_arg,
            view_arg,
            detail_arg,
        );
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
