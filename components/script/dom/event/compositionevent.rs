/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::context::JSContext;
use js::rust::HandleObject;
use script_bindings::reflector::{
    reflect_dom_object_with_cx, reflect_dom_object_with_proto_and_cx,
};
use style::Atom;

use crate::dom::bindings::codegen::Bindings::CompositionEventBinding::{
    self, CompositionEventMethods,
};
use crate::dom::bindings::codegen::Bindings::UIEventBinding::UIEvent_Binding::UIEventMethods;
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::uievent::UIEvent;
use crate::dom::window::Window;

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

    pub(crate) fn new_uninitialized(
        cx: &mut JSContext,
        window: &Window,
    ) -> DomRoot<CompositionEvent> {
        reflect_dom_object_with_cx(Box::new(CompositionEvent::new_inherited()), window, cx)
    }

    #[expect(clippy::too_many_arguments)]
    pub(crate) fn new(
        cx: &mut JSContext,
        window: &Window,
        event_type: Atom,
        can_bubble: bool,
        cancelable: bool,
        view: Option<&Window>,
        detail: i32,
        data: DOMString,
    ) -> DomRoot<CompositionEvent> {
        Self::new_with_proto(
            cx, window, None, event_type, can_bubble, cancelable, view, detail, data,
        )
    }

    #[expect(clippy::too_many_arguments)]
    fn new_with_proto(
        cx: &mut JSContext,
        window: &Window,
        proto: Option<HandleObject>,
        event_type: Atom,
        can_bubble: bool,
        cancelable: bool,
        view: Option<&Window>,
        detail: i32,
        data: DOMString,
    ) -> DomRoot<CompositionEvent> {
        let ev = reflect_dom_object_with_proto_and_cx(
            Box::new(CompositionEvent {
                uievent: UIEvent::new_inherited(),
                data,
            }),
            window,
            proto,
            cx,
        );
        ev.uievent
            .init_event(event_type, can_bubble, cancelable, view, detail);
        ev
    }

    pub(crate) fn data(&self) -> &DOMString {
        &self.data
    }
}

impl CompositionEventMethods<crate::DomTypeHolder> for CompositionEvent {
    /// <https://w3c.github.io/uievents/#dom-compositionevent-compositionevent>
    fn Constructor(
        cx: &mut JSContext,
        window: &Window,
        proto: Option<HandleObject>,
        event_type: DOMString,
        init: &CompositionEventBinding::CompositionEventInit,
    ) -> Fallible<DomRoot<CompositionEvent>> {
        let event = CompositionEvent::new_with_proto(
            cx,
            window,
            proto,
            event_type.into(),
            init.parent.parent.bubbles,
            init.parent.parent.cancelable,
            init.parent.view.as_deref(),
            init.parent.detail,
            init.data.clone(),
        );
        Ok(event)
    }

    /// <https://w3c.github.io/uievents/#dom-compositionevent-data>
    fn Data(&self) -> DOMString {
        self.data.clone()
    }

    /// <https://dom.spec.whatwg.org/#dom-event-istrusted>
    fn IsTrusted(&self) -> bool {
        self.uievent.IsTrusted()
    }
}
