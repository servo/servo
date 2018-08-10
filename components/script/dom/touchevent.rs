/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::TouchEventBinding;
use dom::bindings::codegen::Bindings::TouchEventBinding::TouchEventMethods;
use dom::bindings::codegen::Bindings::UIEventBinding::UIEventMethods;
use dom::bindings::inheritance::Castable;
use dom::bindings::reflector::reflect_dom_object;
use dom::bindings::root::{DomRoot, MutDom};
use dom::bindings::str::DOMString;
use dom::event::{EventBubbles, EventCancelable};
use dom::touchlist::TouchList;
use dom::uievent::UIEvent;
use dom::window::Window;
use dom_struct::dom_struct;
use std::cell::Cell;
use typeholder::TypeHolderTrait;

#[dom_struct]
pub struct TouchEvent<TH: TypeHolderTrait> {
    uievent: UIEvent<TH>,
    touches: MutDom<TouchList<TH>>,
    target_touches: MutDom<TouchList<TH>>,
    changed_touches: MutDom<TouchList<TH>>,
    alt_key: Cell<bool>,
    meta_key: Cell<bool>,
    ctrl_key: Cell<bool>,
    shift_key: Cell<bool>,
}

impl<TH: TypeHolderTrait> TouchEvent<TH> {
    fn new_inherited(touches: &TouchList<TH>,
                     changed_touches: &TouchList<TH>,
                     target_touches: &TouchList<TH>) -> TouchEvent<TH> {
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

    pub fn new_uninitialized(window: &Window<TH>,
                     touches: &TouchList<TH>,
                     changed_touches: &TouchList<TH>,
                     target_touches: &TouchList<TH>) -> DomRoot<TouchEvent<TH>> {
        reflect_dom_object(Box::new(TouchEvent::new_inherited(touches, changed_touches, target_touches)),
                           window,
                           TouchEventBinding::Wrap)
    }

    pub fn new(window: &Window<TH>,
               type_: DOMString,
               can_bubble: EventBubbles,
               cancelable: EventCancelable,
               view: Option<&Window<TH>>,
               detail: i32,
               touches: &TouchList<TH>,
               changed_touches: &TouchList<TH>,
               target_touches: &TouchList<TH>,
               ctrl_key: bool,
               alt_key: bool,
               shift_key: bool,
               meta_key: bool) -> DomRoot<TouchEvent<TH>> {
        let ev = TouchEvent::new_uninitialized(window, touches, changed_touches, target_touches);
        ev.upcast::<UIEvent<TH>>().InitUIEvent(type_,
                                           bool::from(can_bubble),
                                           bool::from(cancelable),
                                           view, detail);
        ev.ctrl_key.set(ctrl_key);
        ev.alt_key.set(alt_key);
        ev.shift_key.set(shift_key);
        ev.meta_key.set(meta_key);
        ev
    }
}

impl<'a, TH: TypeHolderTrait> TouchEventMethods<TH> for &'a TouchEvent<TH> {
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
    fn Touches(&self) -> DomRoot<TouchList<TH>> {
        self.touches.get()
    }

    /// <https://w3c.github.io/touch-events/#widl-TouchEvent-targetTouches>
    fn TargetTouches(&self) -> DomRoot<TouchList<TH>> {
        self.target_touches.get()
    }

    /// <https://w3c.github.io/touch-events/#widl-TouchEvent-changedTouches>
    fn ChangedTouches(&self) -> DomRoot<TouchList<TH>> {
        self.changed_touches.get()
    }

    /// <https://dom.spec.whatwg.org/#dom-event-istrusted>
    fn IsTrusted(&self) -> bool {
        self.uievent.IsTrusted()
    }
}
