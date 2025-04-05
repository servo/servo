/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::rust::HandleObject;

use crate::dom::bindings::codegen::Bindings::CompositionEventBinding::{
    self, CompositionEventMethods,
};
use crate::dom::bindings::codegen::Bindings::UIEventBinding::UIEvent_Binding::UIEventMethods;
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::reflector::{reflect_dom_object, reflect_dom_object_with_proto};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::uievent::UIEvent;
use crate::dom::window::Window;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct CompositionEvent {
    uievent: UIEvent,
    data: DOMString,
}

impl CompositionEvent {
    pub(crate) fn new_inherited() -> CompositionEvent {
        CompositionEvent {
            uievent: UIEvent::new_inherited(),
            data: DOMString::new(),
        }
    }

    pub(crate) fn new_uninitialized(window: &Window, can_gc: CanGc) -> DomRoot<CompositionEvent> {
        reflect_dom_object(Box::new(CompositionEvent::new_inherited()), window, can_gc)
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        window: &Window,
        type_: DOMString,
        can_bubble: bool,
        cancelable: bool,
        view: Option<&Window>,
        detail: i32,
        data: DOMString,
        can_gc: CanGc,
    ) -> DomRoot<CompositionEvent> {
        Self::new_with_proto(
            window, None, type_, can_bubble, cancelable, view, detail, data, can_gc,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn new_with_proto(
        window: &Window,
        proto: Option<HandleObject>,
        type_: DOMString,
        can_bubble: bool,
        cancelable: bool,
        view: Option<&Window>,
        detail: i32,
        data: DOMString,
        can_gc: CanGc,
    ) -> DomRoot<CompositionEvent> {
        let ev = reflect_dom_object_with_proto(
            Box::new(CompositionEvent {
                uievent: UIEvent::new_inherited(),
                data,
            }),
            window,
            proto,
            can_gc,
        );
        ev.uievent
            .InitUIEvent(type_, can_bubble, cancelable, view, detail);
        ev
    }

    pub(crate) fn data(&self) -> &str {
        &self.data
    }
}

impl CompositionEventMethods<crate::DomTypeHolder> for CompositionEvent {
    // https://w3c.github.io/uievents/#dom-compositionevent-compositionevent
    fn Constructor(
        window: &Window,
        proto: Option<HandleObject>,
        can_gc: CanGc,
        type_: DOMString,
        init: &CompositionEventBinding::CompositionEventInit,
    ) -> Fallible<DomRoot<CompositionEvent>> {
        let event = CompositionEvent::new_with_proto(
            window,
            proto,
            type_,
            init.parent.parent.bubbles,
            init.parent.parent.cancelable,
            init.parent.view.as_deref(),
            init.parent.detail,
            init.data.clone(),
            can_gc,
        );
        Ok(event)
    }

    // https://w3c.github.io/uievents/#dom-compositionevent-data
    fn Data(&self) -> DOMString {
        self.data.clone()
    }

    // https://dom.spec.whatwg.org/#dom-event-istrusted
    fn IsTrusted(&self) -> bool {
        self.uievent.IsTrusted()
    }
}
