/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;

use dom_struct::dom_struct;

use crate::dom::bindings::codegen::Bindings::TouchEventBinding::TouchEventMethods;
use crate::dom::bindings::codegen::Bindings::UIEventBinding::UIEventMethods;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::reflect_dom_object;
use crate::dom::bindings::root::{DomRoot, MutDom};
use crate::dom::bindings::str::DOMString;
use crate::dom::event::{EventBubbles, EventCancelable};
use crate::dom::touchlist::TouchList;
use crate::dom::uievent::UIEvent;
use crate::dom::window::Window;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct TouchEvent {
    uievent: UIEvent,
    touches: MutDom<TouchList>,
    target_touches: MutDom<TouchList>,
    changed_touches: MutDom<TouchList>,
    alt_key: Cell<bool>,
    meta_key: Cell<bool>,
    ctrl_key: Cell<bool>,
    shift_key: Cell<bool>,
}

impl TouchEvent {
    fn new_inherited(
        touches: &TouchList,
        changed_touches: &TouchList,
        target_touches: &TouchList,
    ) -> TouchEvent {
        TouchEvent {
            uievent: UIEvent::new_inherited(),
            touches: MutDom::new(touches),
            target_touches: MutDom::new(target_touches),
            changed_touches: MutDom::new(changed_touches),
            ctrl_key: Cell::new(false),
            shift_key: Cell::new(false),
            alt_key: Cell::new(false),
            meta_key: Cell::new(false),
        }
    }

    pub(crate) fn new_uninitialized(
        window: &Window,
        touches: &TouchList,
        changed_touches: &TouchList,
        target_touches: &TouchList,
        can_gc: CanGc,
    ) -> DomRoot<TouchEvent> {
        reflect_dom_object(
            Box::new(TouchEvent::new_inherited(
                touches,
                changed_touches,
                target_touches,
            )),
            window,
            can_gc,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        window: &Window,
        type_: DOMString,
        can_bubble: EventBubbles,
        cancelable: EventCancelable,
        view: Option<&Window>,
        detail: i32,
        touches: &TouchList,
        changed_touches: &TouchList,
        target_touches: &TouchList,
        ctrl_key: bool,
        alt_key: bool,
        shift_key: bool,
        meta_key: bool,
    ) -> DomRoot<TouchEvent> {
        let ev = TouchEvent::new_uninitialized(
            window,
            touches,
            changed_touches,
            target_touches,
            CanGc::note(),
        );
        ev.upcast::<UIEvent>().InitUIEvent(
            type_,
            bool::from(can_bubble),
            bool::from(cancelable),
            view,
            detail,
        );
        ev.ctrl_key.set(ctrl_key);
        ev.alt_key.set(alt_key);
        ev.shift_key.set(shift_key);
        ev.meta_key.set(meta_key);
        ev
    }
}

impl TouchEventMethods<crate::DomTypeHolder> for TouchEvent {
    /// <https://w3c.github.io/touch-events/#widl-TouchEvent-ctrlKey>
    fn CtrlKey(&self) -> bool {
        self.ctrl_key.get()
    }

    /// <https://w3c.github.io/touch-events/#widl-TouchEvent-shiftKey>
    fn ShiftKey(&self) -> bool {
        self.shift_key.get()
    }

    /// <https://w3c.github.io/touch-events/#widl-TouchEvent-altKey>
    fn AltKey(&self) -> bool {
        self.alt_key.get()
    }

    /// <https://w3c.github.io/touch-events/#widl-TouchEvent-metaKey>
    fn MetaKey(&self) -> bool {
        self.meta_key.get()
    }

    /// <https://w3c.github.io/touch-events/#widl-TouchEventInit-touches>
    fn Touches(&self) -> DomRoot<TouchList> {
        self.touches.get()
    }

    /// <https://w3c.github.io/touch-events/#widl-TouchEvent-targetTouches>
    fn TargetTouches(&self) -> DomRoot<TouchList> {
        self.target_touches.get()
    }

    /// <https://w3c.github.io/touch-events/#widl-TouchEvent-changedTouches>
    fn ChangedTouches(&self) -> DomRoot<TouchList> {
        self.changed_touches.get()
    }

    /// <https://dom.spec.whatwg.org/#dom-event-istrusted>
    fn IsTrusted(&self) -> bool {
        self.uievent.IsTrusted()
    }
}
