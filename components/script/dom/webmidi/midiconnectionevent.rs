/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::rust::HandleObject;
use script_bindings::reflector::reflect_dom_object_with_proto;
use stylo_atoms::Atom;

use crate::dom::bindings::codegen::Bindings::EventBinding::Event_Binding::EventMethods;
use crate::dom::bindings::codegen::Bindings::MIDIConnectionEventBinding::{
    MIDIConnectionEventInit, MIDIConnectionEventMethods,
};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::{DomRoot, MutNullableDom};
use crate::dom::bindings::str::DOMString;
use crate::dom::event::Event;
use crate::dom::globalscope::GlobalScope;
use crate::dom::webmidi::midiport::MIDIPort;
use crate::dom::window::Window;

// https://webaudio.github.io/web-midi-api/#MIDIConnectionEvent
#[dom_struct]
pub(crate) struct MIDIConnectionEvent {
    event: Event,
    port: MutNullableDom<MIDIPort>,
}

impl MIDIConnectionEvent {
    fn new_inherited(port: Option<&MIDIPort>) -> MIDIConnectionEvent {
        MIDIConnectionEvent {
            event: Event::new_inherited(),
            port: MutNullableDom::new(port),
        }
    }

    pub fn new_with_proto(
        cx: &mut js::context::JSContext,
        window: &Window,
        proto: Option<HandleObject>,
        type_: Atom,
        bubbles: bool,
        cancelable: bool,
        port: Option<&MIDIPort>,
    ) -> DomRoot<MIDIConnectionEvent> {
        let connectionevent = reflect_dom_object_with_proto(
            cx,
            Box::new(MIDIConnectionEvent::new_inherited(port)),
            window,
            proto,
        );
        {
            let event = connectionevent.upcast::<Event>();
            event.init_event(type_, bubbles, cancelable);
        }

        connectionevent
    }
}

impl MIDIConnectionEventMethods<crate::DomTypeHolder> for MIDIConnectionEvent {
    /// <https://webaudio.github.io/web-midi-api/#dom-midiconnectionevent-constructor>
    fn Constructor(
        cx: &mut js::context::JSContext,
        global: &GlobalScope,
        proto: Option<HandleObject>,
        type_: DOMString,
        init: &MIDIConnectionEventInit,
    ) -> DomRoot<MIDIConnectionEvent> {
        MIDIConnectionEvent::new_with_proto(
            cx,
            global.as_window(),
            proto,
            Atom::from(type_),
            init.parent.bubbles,
            init.parent.cancelable,
            init.port.as_deref(),
        )
    }

    /// <https://webaudio.github.io/web-midi-api/#dom-midiconnectionevent-port>
    fn GetPort(&self) -> Option<DomRoot<MIDIPort>> {
        self.port.get()
    }

    /// <https://dom.spec.whatwg.org/#dom-event-istrusted>
    fn IsTrusted(&self) -> bool {
        self.event.IsTrusted()
    }
}
