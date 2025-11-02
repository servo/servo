/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::rust::HandleObject;
use script_bindings::codegen::GenericBindings::NodeBinding::NodeMethods;
use script_bindings::inheritance::Castable;
use stylo_atoms::Atom;

use crate::dom::bindings::codegen::Bindings::EventBinding::EventMethods;
use crate::dom::bindings::codegen::Bindings::ToggleEventBinding;
use crate::dom::bindings::codegen::Bindings::ToggleEventBinding::ToggleEventMethods;
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::reflector::reflect_dom_object_with_proto;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::element::Element;
use crate::dom::event::{Event, EventBubbles, EventCancelable};
use crate::dom::eventtarget::EventTarget;
use crate::dom::node::Node;
use crate::dom::types::Window;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct ToggleEvent {
    event: Event,
    /// <https://html.spec.whatwg.org/multipage/#dom-toggleevent-oldstate>
    old_state: DOMString,
    /// <https://html.spec.whatwg.org/multipage/#dom-toggleevent-newstate>
    new_state: DOMString,
    /// <https://html.spec.whatwg.org/multipage/#dom-toggleevent-source>
    source: Option<DomRoot<Element>>,
}

impl ToggleEvent {
    pub(crate) fn new_inherited(
        old_state: DOMString,
        new_state: DOMString,
        source: Option<DomRoot<Element>>,
    ) -> ToggleEvent {
        ToggleEvent {
            event: Event::new_inherited(),
            old_state,
            new_state,
            source,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        window: &Window,
        type_: Atom,
        bubbles: EventBubbles,
        cancelable: EventCancelable,
        old_state: DOMString,
        new_state: DOMString,
        source: Option<DomRoot<Element>>,
        can_gc: CanGc,
    ) -> DomRoot<ToggleEvent> {
        Self::new_with_proto(
            window, None, type_, bubbles, cancelable, old_state, new_state, source, can_gc,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn new_with_proto(
        window: &Window,
        proto: Option<HandleObject>,
        type_: Atom,
        bubbles: EventBubbles,
        cancelable: EventCancelable,
        old_state: DOMString,
        new_state: DOMString,
        source: Option<DomRoot<Element>>,
        can_gc: CanGc,
    ) -> DomRoot<ToggleEvent> {
        let event = Box::new(ToggleEvent::new_inherited(old_state, new_state, source));
        let event = reflect_dom_object_with_proto(event, window, proto, can_gc);
        {
            let event = event.upcast::<Event>();
            event.init_event(type_, bool::from(bubbles), bool::from(cancelable));
        }
        event
    }
}

impl ToggleEventMethods<crate::DomTypeHolder> for ToggleEvent {
    /// <https://html.spec.whatwg.org/multipage/#toggleevent>
    fn Constructor(
        window: &Window,
        proto: Option<HandleObject>,
        can_gc: CanGc,
        type_: DOMString,
        init: &ToggleEventBinding::ToggleEventInit,
    ) -> Fallible<DomRoot<ToggleEvent>> {
        let bubbles = EventBubbles::from(init.parent.bubbles);
        let cancelable = EventCancelable::from(init.parent.cancelable);
        Ok(ToggleEvent::new_with_proto(
            window,
            proto,
            Atom::from(type_),
            bubbles,
            cancelable,
            init.oldState.clone(),
            init.newState.clone(),
            init.source.as_ref().map(|s| DomRoot::from_ref(&**s)),
            can_gc,
        ))
    }

    /// <https://dom.spec.whatwg.org/#dom-event-istrusted>
    fn IsTrusted(&self) -> bool {
        self.event.IsTrusted()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-toggleevent-oldstate>
    fn OldState(&self) -> DOMString {
        self.old_state.clone()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-toggleevent-newstate>
    fn NewState(&self) -> DOMString {
        self.new_state.clone()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-toggleevent-source>
    fn GetSource(&self) -> Option<DomRoot<Element>> {
        // The source getter steps are to return the result of retargeting source against this's currentTarget.
        let source = self.source.as_ref()?;

        if let Some(current_target) = self.event.GetCurrentTarget() {
            let retargeted = source.upcast::<EventTarget>().retarget(&current_target);
            return retargeted.downcast::<Element>().map(DomRoot::from_ref);
        }

        let document = source.upcast::<Node>().GetOwnerDocument().unwrap();
        let retargeted = source
            .upcast::<EventTarget>()
            .retarget(document.upcast::<EventTarget>());
        retargeted.downcast::<Element>().map(DomRoot::from_ref)
    }
}
