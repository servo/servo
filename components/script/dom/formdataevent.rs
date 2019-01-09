/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::EventBinding::EventMethods;
use crate::dom::bindings::codegen::Bindings::FormDataEventBinding;
use crate::dom::bindings::codegen::Bindings::FormDataEventBinding::FormDataEventMethods;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::event::Event;
use crate::dom::event::{EventBubbles, EventCancelable};
use crate::dom::formdata::FormData;
use crate::dom::globalscope::GlobalScope;
use crate::dom::window::Window;
use dom_struct::dom_struct;
use servo_atoms::Atom;

#[dom_struct]
pub struct FormDataEvent {
    event: Event,
    form_data: Dom<FormData>,
}

impl FormDataEvent {
    fn new_inherited(form_data: &FormData) -> FormDataEvent {
        FormDataEvent {
            event: Event::new_inherited(),
            form_data: Dom::from_ref(form_data),
        }
    }

    pub fn new_uninitialized(global: &GlobalScope, form_data: &FormData) -> DomRoot<FormDataEvent> {
        reflect_dom_object(
            Box::new(FormDataEvent::new_inherited(form_data)),
            global,
            FormDataEventBinding::Wrap,
        )
    }

    pub fn new(
        global: &GlobalScope,
        type_: Atom,
        can_bubble: EventBubbles,
        cancelable: EventCancelable,
        form_data: &FormData,
    ) -> DomRoot<FormDataEvent> {
        let ev = FormDataEvent::new_uninitialized(global, form_data);
        {
            let event = ev.upcast::<Event>();
            event.init_event(type_, bool::from(can_bubble), bool::from(cancelable));
        }
        ev
    }

    pub fn Constructor(
        window: &Window,
        type_: DOMString,
        init: &FormDataEventBinding::FormDataEventInit,
    ) -> Fallible<DomRoot<FormDataEvent>> {
        let bubbles = EventBubbles::from(init.parent.bubbles);
        let cancelable = EventCancelable::from(init.parent.cancelable);

        let form_data = match init.formData {
            Some(ref form_data) => form_data.clone(),
            None => {
                return Err(Error::Type(
                    "required member formdata is undefined".to_string(),
                ));
            },
        };

        let event = FormDataEvent::new(
            &window.global(),
            Atom::from(type_),
            bubbles,
            cancelable,
            &*form_data,
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
