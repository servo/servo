/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::realm::CurrentRealm;
use script_bindings::cell::DomRefCell;
use servo_constellation_traits::ScriptToConstellationMessage;
use webmidi_traits::{MIDIRequest, MidiPort, MidiPortInfo};

use crate::dom::bindings::codegen::Bindings::MIDIPortBinding::{
    MIDIOutput, MIDIOutputMethods, MIDIPortConnectionState, MIDIPortDeviceState, MIDIPortMethods,
    MIDIPortType,
};
use crate::dom::bindings::reflector::DomGlobal;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::eventtarget::EventTarget;
use crate::dom::promise::Promise;
use crate::dom::promise::promise::{RootedPromise, TracedPromise};
use crate::script_bindings::inheritance::Castable;

// https://webaudio.github.io/web-midi-api/#MIDIPort
#[dom_struct]
pub(crate) struct MIDIPort {
    event_target: EventTarget,
    id: DOMString,
    manufacturer: Option<DOMString>,
    name: Option<DOMString>,
    type_: MIDIPortType,
    version: Option<DOMString>,
    state: DomRefCell<MIDIPortDeviceState>,
    connection: DomRefCell<MIDIPortConnectionState>,
    port_open_promise: DomRefCell<Option<TracedPromise>>,
    port_close_promise: DomRefCell<Option<TracedPromise>>,
}

impl MIDIPort {
    pub(crate) fn new_inherited(
        id: DOMString,
        manufacturer: Option<DOMString>,
        name: Option<DOMString>,
        type_: MIDIPortType,
        version: Option<DOMString>,
        state: MIDIPortDeviceState,
        connection: MIDIPortConnectionState,
    ) -> MIDIPort {
        MIDIPort {
            event_target: EventTarget::new_inherited(),
            id,
            manufacturer,
            name,
            type_,
            version,
            state: DomRefCell::new(state),
            connection: DomRefCell::new(connection),
            port_open_promise: DomRefCell::new(None),
            port_close_promise: DomRefCell::new(None),
        }
    }
}

impl MIDIPortMethods<crate::DomTypeHolder> for MIDIPort {
    /// <https://webaudio.github.io/web-midi-api/#dom-midiport-id>
    fn Id(&self) -> DOMString {
        self.id.clone()
    }

    /// <https://webaudio.github.io/web-midi-api/#dom-midiport-manufacturer>
    fn GetManufacturer(&self) -> Option<DOMString> {
        self.manufacturer.clone()
    }

    /// <https://webaudio.github.io/web-midi-api/#dom-midiport-name>
    fn GetName(&self) -> Option<DOMString> {
        self.name.clone()
    }

    /// <https://webaudio.github.io/web-midi-api/#dom-midiport-type>
    fn Type(&self) -> MIDIPortType {
        self.type_
    }

    /// <https://webaudio.github.io/web-midi-api/#dom-midiport-version
    fn GetVersion(&self) -> Option<DOMString> {
        self.version.clone()
    }

    /// <https://webaudio.github.io/web-midi-api/#dom-midiport-state>
    fn State(&self) -> MIDIPortDeviceState {
        *self.state.borrow()
    }

    /// <https://webaudio.github.io/web-midi-api/#dom-midiport-connection>
    fn Connection(&self) -> MIDIPortConnectionState {
        *self.connection.borrow()
    }

    // <https://webaudio.github.io/web-midi-api/#dom-midiport-onstatechange>
    event_handler!(statechange, GetOnstatechange, SetOnstatechange);

    /// <https://webaudio.github.io/web-midi-api/#dom-midiport-open>
    fn Open(&self, cx: &mut CurrentRealm) -> RootedPromise {
        let global = self.global();
        let pipeline_id = global.pipeline_id();
        let p = Promise::new_in_realm(cx);

        // If the device's connection is already "open"/"pending"
        // resolve promise immediately with current port
        if *self.connection.borrow() != MIDIPortConnectionState::Closed {
            p.resolve_native(cx, self);
            return p;
        }

        *self.port_open_promise.borrow_mut() = Some(p.to_traced());

        // 6. If the device's state is "disconnected",
        // change the connection attribute of the MIDIPort to "pending",
        if *self.state.borrow() == MIDIPortDeviceState::Disconnected {
            *self.connection.borrow_mut() = MIDIPortConnectionState::Pending;
        }

        let id = self.Id().to_string();
        let name = self.name.as_ref().unwrap().to_string();
        let port_info = match self.type_ {
            MIDIPortType::Input => MidiPort::Input(MidiPortInfo { id, name }),
            MIDIPortType::Output => MidiPort::Output(MidiPortInfo { id, name }),
        };
        let request = MIDIRequest::OpenPort(pipeline_id, port_info);
        let msg = ScriptToConstellationMessage::MIDIRequest(request);
        self.global()
            .script_to_constellation_chan()
            .send(msg)
            .expect("Failed to send MIDIRequest::OpenPort to Constellation");

        p
    }

    /// <https://webaudio.github.io/web-midi-api/#dom-midiport-close>
    fn Close(&self, cx: &mut CurrentRealm) -> RootedPromise {
        let global = self.global();
        let pipeline_id = global.pipeline_id();
        let p = Promise::new_in_realm(cx);

        // If the device's connection is already "closed",
        // resolve promise immediately with current port
        if *self.connection.borrow() == MIDIPortConnectionState::Closed {
            p.resolve_native(cx, self);
            return p;
        }

        *self.port_close_promise.borrow_mut() = Some(p.to_traced());

        // If the output port's .state is not "connected"
        // or if its .connection is "pending", clear all enqueued send data.
        if let Some(output) = self.downcast::<MIDIOutput>() {
            output.Clear();
        }

        let id = self.Id().to_string();
        let name = self.name.as_ref().unwrap().to_string();
        let port_info = match self.type_ {
            MIDIPortType::Input => MidiPort::Input(MidiPortInfo { id, name }),
            MIDIPortType::Output => MidiPort::Output(MidiPortInfo { id, name }),
        };
        let request = MIDIRequest::ClosePort(pipeline_id, port_info);
        let msg = ScriptToConstellationMessage::MIDIRequest(request);
        self.global()
            .script_to_constellation_chan()
            .send(msg)
            .expect("Failed to send MIDIRequest::ClosePort to Constellation");

        p
    }
}

impl MIDIPort {
    pub fn set_state(&self, state: MIDIPortDeviceState) {
        *self.state.borrow_mut() = state;
        self.fire_statechange();
    }

    pub fn handle_open_completed(&self, cx: &mut js::context::JSContext) {
        if let Some(ref promise) = self.port_open_promise.borrow_mut().take() {
            let promise = promise.root(cx);
            promise.resolve_native(cx, &DomRoot::from_ref(self));
            *self.connection.borrow_mut() = MIDIPortConnectionState::Open;
            self.fire_statechange();
        }
    }

    pub fn handle_close_completed(&self, cx: &mut js::context::JSContext) {
        if let Some(ref promise) = self.port_close_promise.borrow_mut().take() {
            let promise = promise.root(cx);
            promise.resolve_native(cx, &DomRoot::from_ref(self));
            *self.connection.borrow_mut() = MIDIPortConnectionState::Closed;
            self.fire_statechange();
        }
    }

    fn fire_statechange(&self) {
        self.global()
            .task_manager()
            .dom_manipulation_task_source()
            .queue_simple_event(self.upcast(), atom!("statechange"));
    }
}
