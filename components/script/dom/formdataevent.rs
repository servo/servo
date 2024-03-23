/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::rust::HandleObject;
use servo_atoms::Atom;

use crate::dom::bindings::codegen::Bindings::EventBinding::EventMethods;
use crate::dom::bindings::codegen::Bindings::FormDataEventBinding;
use crate::dom::bindings::codegen::Bindings::FormDataEventBinding::FormDataEventMethods;
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::{reflect_dom_object_with_proto, DomObject};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::event::{Event, EventBubbles, EventCancelable};
use crate::dom::formdata::FormData;
use crate::dom::globalscope::GlobalScope;
use crate::dom::window::Window;

#[dom_struct]
pub struct FormDataEvent {
    event: Event,
    form_data: Dom<FormData>,
}

impl FormDataEvent {
    pub fn new(
        global: &GlobalScope,
        type_: Atom,
        can_bubble: EventBubbles,
        cancelable: EventCancelable,
        form_data: &FormData,
    ) -> DomRoot<FormDataEvent> {
        Self::new_with_proto(global, None, type_, can_bubble, cancelable, form_data)
    }

    fn new_with_proto(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        type_: Atom,
        can_bubble: EventBubbles,
        cancelable: EventCancelable,
        form_data: &FormData,
    ) -> DomRoot<FormDataEvent> {
        let ev = reflect_dom_object_with_proto(
            Box::new(FormDataEvent {
                event: Event::new_inherited(),
                form_data: Dom::from_ref(form_data),
            }),
            global,
            proto,
        );

        {
            let event = ev.upcast::<Event>();
            event.init_event(type_, bool::from(can_bubble), bool::from(cancelable));
        }
        ev
    }

    #[allow(non_snake_case)]
    pub fn Constructor(
        window: &Window,
        proto: Option<HandleObject>,
        type_: DOMString,
        init: &FormDataEventBinding::FormDataEventInit,
    ) -> Fallible<DomRoot<FormDataEvent>> {
        let bubbles = EventBubbles::from(init.parent.bubbles);
        let cancelable = EventCancelable::from(init.parent.cancelable);

        let event = FormDataEvent::new_with_proto(
            &window.global(),
            proto,
            Atom::from(type_),
            bubbles,
            cancelable,
            &init.formData.clone(),
        );

        Ok(event)
    }
}

impl FormDataEventMethods for FormDataEvent {
    // https://html.spec.whatwg.org/multipage/#dom-formdataevent-formdata
    fn FormData(&self) -> DomRoot<FormData> {
        DomRoot::from_ref(&*self.form_data)
    }

    // https://dom.spec.whatwg.org/#dom-event-istrusted
    fn IsTrusted(&self) -> bool {
        self.event.IsTrusted()
    }
}
