/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use script_bindings::inheritance::Castable;
use script_bindings::reflector::reflect_dom_object;
use stylo_atoms::Atom;

use crate::dom::bindings::codegen::Bindings::MIDIPortBinding::{
    MIDIInputMethods, MIDIPortConnectionState, MIDIPortDeviceState, MIDIPortType,
};
use crate::dom::bindings::reflector::DomGlobal;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::event::Event;
use crate::dom::eventtarget::EventTarget;
use crate::dom::webmidi::midimessageevent::MIDIMessageEvent;
use crate::dom::webmidi::midiport::MIDIPort;
use crate::dom::window::Window;

// https://webaudio.github.io/web-midi-api/#MIDIInput
#[dom_struct]
pub(crate) struct MIDIInput {
    port: MIDIPort,
}

impl MIDIInput {
    fn new_inherited(
        id: DOMString,
        manufacturer: Option<DOMString>,
        name: Option<DOMString>,
        type_: MIDIPortType,
        version: Option<DOMString>,
        state: MIDIPortDeviceState,
        connection: MIDIPortConnectionState,
    ) -> MIDIInput {
        MIDIInput {
            port: MIDIPort::new_inherited(
                id,
                manufacturer,
                name,
                type_,
                version,
                state,
                connection,
            ),
        }
    }

    #[expect(clippy::too_many_arguments)]
    pub(crate) fn new(
        cx: &mut js::context::JSContext,
        window: &Window,
        id: DOMString,
        manufacturer: Option<DOMString>,
        name: Option<DOMString>,
        type_: MIDIPortType,
        version: Option<DOMString>,
        state: MIDIPortDeviceState,
        connection: MIDIPortConnectionState,
    ) -> DomRoot<MIDIInput> {
        reflect_dom_object(
            cx,
            Box::new(MIDIInput::new_inherited(
                id,
                manufacturer,
                name,
                type_,
                version,
                state,
                connection,
            )),
            window,
        )
    }

    pub(crate) fn port(&self) -> DomRoot<MIDIPort> {
        DomRoot::from_ref(&self.port)
    }
}

impl MIDIInputMethods<crate::DomTypeHolder> for MIDIInput {
    // <https://webaudio.github.io/web-midi-api/#dom-midiinput-onmidimessage>
    event_handler!(midimessage, GetOnmidimessage, SetOnmidimessage);
}

impl MIDIInput {
    pub(crate) fn handle_midi_message(&self, cx: &mut js::context::JSContext, data: Vec<u8>) {
        if data[0] == 0xF0 {
            // SysEx messages are currently not allowed, abort process
            return;
        }
        let event = MIDIMessageEvent::new_with_proto(
            cx,
            self.global().as_window(),
            None,
            Atom::from("midimessage"),
            false,
            false,
        );
        event.set_data(cx, data);
        event
            .upcast::<Event>()
            .fire(cx, self.port.upcast::<EventTarget>());
    }
}
