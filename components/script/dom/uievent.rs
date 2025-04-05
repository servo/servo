/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::default::Default;

use dom_struct::dom_struct;
use js::rust::HandleObject;
use stylo_atoms::Atom;

use super::node::NodeTraits;
use crate::dom::bindings::codegen::Bindings::EventBinding::EventMethods;
use crate::dom::bindings::codegen::Bindings::UIEventBinding;
use crate::dom::bindings::codegen::Bindings::UIEventBinding::UIEventMethods;
use crate::dom::bindings::codegen::Bindings::WindowBinding::Window_Binding::WindowMethods;
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::reflect_dom_object_with_proto;
use crate::dom::bindings::root::{DomRoot, MutNullableDom};
use crate::dom::bindings::str::DOMString;
use crate::dom::element::Element;
use crate::dom::event::{Event, EventBubbles, EventCancelable};
use crate::dom::eventtarget::EventTarget;
use crate::dom::window::Window;
use crate::script_runtime::CanGc;

// https://w3c.github.io/uievents/#interface-uievent
#[dom_struct]
pub(crate) struct UIEvent {
    event: Event,
    view: MutNullableDom<Window>,
    detail: Cell<i32>,
}

impl UIEvent {
    pub(crate) fn new_inherited() -> UIEvent {
        UIEvent {
            event: Event::new_inherited(),
            view: Default::default(),
            detail: Cell::new(0),
        }
    }

    pub(crate) fn new_uninitialized(window: &Window, can_gc: CanGc) -> DomRoot<UIEvent> {
        Self::new_uninitialized_with_proto(window, None, can_gc)
    }

    fn new_uninitialized_with_proto(
        window: &Window,
        proto: Option<HandleObject>,
        can_gc: CanGc,
    ) -> DomRoot<UIEvent> {
        reflect_dom_object_with_proto(Box::new(UIEvent::new_inherited()), window, proto, can_gc)
    }

    pub(crate) fn new(
        window: &Window,
        type_: DOMString,
        can_bubble: EventBubbles,
        cancelable: EventCancelable,
        view: Option<&Window>,
        detail: i32,
        can_gc: CanGc,
    ) -> DomRoot<UIEvent> {
        Self::new_with_proto(
            window, None, type_, can_bubble, cancelable, view, detail, can_gc,
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
        can_gc: CanGc,
    ) -> DomRoot<UIEvent> {
        let ev = UIEvent::new_uninitialized_with_proto(window, proto, can_gc);
        ev.initialize_ui_event(
            type_,
            view.map(|window| window.upcast::<EventTarget>()),
            can_bubble,
            cancelable,
        );
        ev.detail.set(detail);
        ev
    }

    /// <https://w3c.github.io/uievents/#initialize-a-uievent>
    pub(crate) fn initialize_ui_event(
        &self,
        type_: DOMString,
        target_: Option<&EventTarget>,
        bubbles: EventBubbles,
        cancelable: EventCancelable,
    ) {
        // 1. Initialize the base Event attributes:
        self.event
            .init_event(type_.into(), bool::from(bubbles), bool::from(cancelable));
        self.event.set_target(target_);
        // 2. Initialize view/detail:
        if let Some(target_) = target_ {
            let element = target_.downcast::<Element>();
            let document = match element {
                Some(element) => element.owner_document(),
                None => target_.downcast::<Window>().unwrap().Document(),
            };
            self.view.set(Some(document.window()));
        }
        self.detail.set(0_i32);
    }

    pub(crate) fn set_detail(&self, detail_: i32) {
        self.detail.set(detail_);
    }
}

impl UIEventMethods<crate::DomTypeHolder> for UIEvent {
    /// <https://w3c.github.io/uievents/#dom-uievent-uievent>
    fn Constructor(
        window: &Window,
        proto: Option<HandleObject>,
        can_gc: CanGc,
        type_: DOMString,
        init: &UIEventBinding::UIEventInit,
    ) -> Fallible<DomRoot<UIEvent>> {
        let bubbles = EventBubbles::from(init.parent.bubbles);
        let cancelable = EventCancelable::from(init.parent.cancelable);
        let event = UIEvent::new_with_proto(
            window,
            proto,
            type_,
            bubbles,
            cancelable,
            init.view.as_deref(),
            init.detail,
            can_gc,
        );
        Ok(event)
    }

    // https://w3c.github.io/uievents/#widl-UIEvent-view
    fn GetView(&self) -> Option<DomRoot<Window>> {
        self.view.get()
    }

    // https://w3c.github.io/uievents/#widl-UIEvent-detail
    fn Detail(&self) -> i32 {
        self.detail.get()
    }

    // https://w3c.github.io/uievents/#widl-UIEvent-initUIEvent
    fn InitUIEvent(
        &self,
        type_: DOMString,
        can_bubble: bool,
        cancelable: bool,
        view: Option<&Window>,
        detail: i32,
    ) {
        let event = self.upcast::<Event>();
        if event.dispatching() {
            return;
        }

        event.init_event(Atom::from(type_), can_bubble, cancelable);
        self.view.set(view);
        self.detail.set(detail);
    }

    // https://dom.spec.whatwg.org/#dom-event-istrusted
    fn IsTrusted(&self) -> bool {
        self.event.IsTrusted()
    }
}
