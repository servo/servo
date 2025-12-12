/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::rust::HandleObject;
use script_bindings::codegen::GenericBindings::NodeBinding::NodeMethods;
use script_bindings::inheritance::Castable;
use stylo_atoms::Atom;

use crate::dom::bindings::codegen::Bindings::CommandEventBinding;
use crate::dom::bindings::codegen::Bindings::CommandEventBinding::CommandEventMethods;
use crate::dom::bindings::codegen::Bindings::EventBinding::EventMethods;
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
pub(crate) struct CommandEvent {
    event: Event,
    /// <https://html.spec.whatwg.org/multipage/#dom-commandevent-source>
    source: Option<DomRoot<Element>>,
    /// <https://html.spec.whatwg.org/multipage/#dom-commandevent-command>
    command: DOMString,
}

impl CommandEvent {
    pub(crate) fn new_inherited(
        source: Option<DomRoot<Element>>,
        command: DOMString,
    ) -> CommandEvent {
        CommandEvent {
            event: Event::new_inherited(),
            source,
            command,
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn new_with_proto(
        window: &Window,
        proto: Option<HandleObject>,
        type_: Atom,
        bubbles: EventBubbles,
        cancelable: EventCancelable,
        source: Option<DomRoot<Element>>,
        command: DOMString,
        can_gc: CanGc,
    ) -> DomRoot<CommandEvent> {
        let event = Box::new(CommandEvent::new_inherited(source, command));
        let event = reflect_dom_object_with_proto(event, window, proto, can_gc);
        {
            let event = event.upcast::<Event>();
            event.init_event(type_, bool::from(bubbles), bool::from(cancelable));
        }
        event
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        window: &Window,
        type_: Atom,
        bubbles: EventBubbles,
        cancelable: EventCancelable,
        source: Option<DomRoot<Element>>,
        command: DOMString,
        can_gc: CanGc,
    ) -> DomRoot<CommandEvent> {
        Self::new_with_proto(
            window, None, type_, bubbles, cancelable, source, command, can_gc,
        )
    }
}

impl CommandEventMethods<crate::DomTypeHolder> for CommandEvent {
    /// <https://html.spec.whatwg.org/multipage/#commandevent>
    fn Constructor(
        window: &Window,
        proto: Option<HandleObject>,
        can_gc: CanGc,
        type_: DOMString,
        init: &CommandEventBinding::CommandEventInit,
    ) -> Fallible<DomRoot<CommandEvent>> {
        let bubbles = EventBubbles::from(init.parent.bubbles);
        let cancelable = EventCancelable::from(init.parent.cancelable);
        Ok(CommandEvent::new_with_proto(
            window,
            proto,
            Atom::from(type_),
            bubbles,
            cancelable,
            init.source.as_ref().map(|s| DomRoot::from_ref(&**s)),
            init.command.clone(),
            can_gc,
        ))
    }

    /// <https://dom.spec.whatwg.org/#dom-event-istrusted>
    fn IsTrusted(&self) -> bool {
        self.event.IsTrusted()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-commandevent-command>
    fn Command(&self) -> DOMString {
        // The command attribute must return the value it was initialized to.
        self.command.clone()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-commandevent-source>
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
