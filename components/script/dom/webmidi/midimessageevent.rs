/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::rust::HandleObject;
use js::typedarray::{HeapUint8Array, Uint8};
use script_bindings::cell::DomRefCell;
use script_bindings::reflector::reflect_dom_object_with_proto;
use stylo_atoms::Atom;

use crate::dom::bindings::buffer_source::HeapBufferSource;
use crate::dom::bindings::codegen::Bindings::EventBinding::Event_Binding::EventMethods;
use crate::dom::bindings::codegen::Bindings::MIDIMessageEventBinding::{
    MIDIMessageEventInit, MIDIMessageEventMethods,
};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::bindings::trace::RootedTraceableBox;
use crate::dom::event::Event;
use crate::dom::globalscope::GlobalScope;
use crate::dom::window::Window;

// https://webaudio.github.io/web-midi-api/#MIDIMessageEvent
#[dom_struct]
pub(crate) struct MIDIMessageEvent {
    event: Event,
    #[ignore_malloc_size_of = "mozjs"]
    data: DomRefCell<HeapBufferSource<Uint8>>,
}

impl MIDIMessageEvent {
    fn new_inherited() -> MIDIMessageEvent {
        MIDIMessageEvent {
            event: Event::new_inherited(),
            data: DomRefCell::new(HeapBufferSource::default()),
        }
    }

    pub(crate) fn new_with_proto(
        cx: &mut js::context::JSContext,
        window: &Window,
        proto: Option<HandleObject>,
        type_: Atom,
        bubbles: bool,
        cancelable: bool,
    ) -> DomRoot<MIDIMessageEvent> {
        let messageevent = reflect_dom_object_with_proto(
            cx,
            Box::new(MIDIMessageEvent::new_inherited()),
            window,
            proto,
        );
        {
            let event = messageevent.upcast::<Event>();
            event.init_event(type_, bubbles, cancelable);
        }

        messageevent
    }
}

impl MIDIMessageEventMethods<crate::DomTypeHolder> for MIDIMessageEvent {
    /// <https://webaudio.github.io/web-midi-api/#dom-midimessageevent-constructor>
    fn Constructor(
        cx: &mut js::context::JSContext,
        global: &GlobalScope,
        proto: Option<HandleObject>,
        type_: DOMString,
        init: &MIDIMessageEventInit,
    ) -> DomRoot<MIDIMessageEvent> {
        MIDIMessageEvent::new_with_proto(
            cx,
            global.as_window(),
            proto,
            Atom::from(type_),
            init.parent.bubbles,
            init.parent.cancelable,
        )
    }

    /// <https://webaudio.github.io/web-midi-api/#dom-midimessageevent-data>
    fn GetData(&self) -> Option<RootedTraceableBox<HeapUint8Array>> {
        self.data.borrow().get_typed_array().ok()
    }

    /// <https://dom.spec.whatwg.org/#dom-event-istrusted>
    fn IsTrusted(&self) -> bool {
        self.event.IsTrusted()
    }
}

impl MIDIMessageEvent {
    pub(crate) fn set_data(&self, cx: &mut js::context::JSContext, data: Vec<u8>) {
        let _ = self.data.borrow_mut().set_data(cx, &data);
    }
}
