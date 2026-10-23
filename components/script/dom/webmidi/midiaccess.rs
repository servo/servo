/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use script_bindings::like::Maplike;
use script_bindings::reflector::reflect_dom_object;
use servo_base::generic_channel;
use servo_constellation_traits::ScriptToConstellationMessage;
use stylo_atoms::Atom;
use webmidi_traits::{MIDIRequest, MidiPort};

use crate::dom::bindings::codegen::Bindings::MIDIAccessBinding::MIDIAccessMethods;
use crate::dom::bindings::codegen::Bindings::MIDIPortBinding::{
    MIDIPortConnectionState, MIDIPortDeviceState, MIDIPortType,
};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::DomGlobal;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::event::Event;
use crate::dom::eventtarget::EventTarget;
use crate::dom::webmidi::midiconnectionevent::MIDIConnectionEvent;
use crate::dom::webmidi::midiinput::MIDIInput;
use crate::dom::webmidi::midiinputmap::MIDIInputMap;
use crate::dom::webmidi::midioutput::MIDIOutput;
use crate::dom::webmidi::midioutputmap::MIDIOutputMap;
use crate::dom::webmidi::midiport::MIDIPort;
use crate::dom::window::Window;

// https://webaudio.github.io/web-midi-api/#MIDIAccess
#[dom_struct]
pub(crate) struct MIDIAccess {
    event_target: EventTarget,
    input_map: Dom<MIDIInputMap>,
    output_map: Dom<MIDIOutputMap>,
    use_software_synths: bool,
}

impl MIDIAccess {
    fn new_inherited(
        input_map: &MIDIInputMap,
        output_map: &MIDIOutputMap,
        use_software_synths: bool,
    ) -> MIDIAccess {
        MIDIAccess {
            event_target: EventTarget::new_inherited(),
            input_map: Dom::from_ref(input_map),
            output_map: Dom::from_ref(output_map),
            use_software_synths,
        }
    }

    pub(crate) fn new(
        window: &Window,
        cx: &mut js::context::JSContext,
        use_software_synths: bool,
    ) -> DomRoot<MIDIAccess> {
        let (sender, receiver) = generic_channel::channel().expect("ipc channel failure");
        let request = MIDIRequest::GetPorts(window.pipeline_id(), sender);
        let msg = ScriptToConstellationMessage::MIDIRequest(request);
        window
            .as_global_scope()
            .script_to_constellation_chan()
            .send(msg)
            .expect("Failed sending MIDIRequest::GetPorts to constellation");
        let ports = receiver.recv().expect("Failed to reach midir backend");
        let input_map = MIDIInputMap::new(window.as_global_scope(), cx);
        let output_map = MIDIOutputMap::new(window.as_global_scope(), cx);
        ports.iter().for_each(|port| match port {
            MidiPort::Input(midi_port_info) => {
                let id = DOMString::from(midi_port_info.id.clone());
                let name = DOMString::from(midi_port_info.name.clone());
                let input = MIDIInput::new(
                    window,
                    id.clone(),
                    None,
                    Some(name),
                    MIDIPortType::Input,
                    None,
                    MIDIPortDeviceState::Connected,
                    MIDIPortConnectionState::Closed,
                    cx,
                );
                input_map.set(cx, id, DomRoot::from_ref(&input));
            },
            MidiPort::Output(midi_port_info) => {
                let id = DOMString::from(midi_port_info.id.clone());
                let name = DOMString::from(midi_port_info.name.clone());
                if name == "Microsoft GS Wavetable Synth" {
                    // Disabled for security concerns, see:
                    // Chromium: <https://issues.chromium.org/issues/40082258>
                    // Gecko: <https://bugzilla.mozilla.org/show_bug.cgi?id=1798097>
                    return;
                }
                let output = MIDIOutput::new(
                    window,
                    id.clone(),
                    None,
                    Some(name),
                    MIDIPortType::Output,
                    None,
                    MIDIPortDeviceState::Connected,
                    MIDIPortConnectionState::Closed,
                    cx,
                );
                output_map.set(cx, id, DomRoot::from_ref(&output));
            },
        });

        reflect_dom_object(
            cx,
            Box::new(MIDIAccess::new_inherited(
                &input_map,
                &output_map,
                use_software_synths,
            )),
            window,
        )
    }

    pub(crate) fn get_port(
        &self,
        port: MidiPort,
        cx: &mut js::context::JSContext,
    ) -> Result<DomRoot<MIDIPort>, String> {
        let id = DOMString::from(port.id());

        match port {
            MidiPort::Input(_) => {
                let input_map = self.input_map.as_rooted();
                if let Some(port) = input_map.get(cx, id.clone()) {
                    return Ok(port.port());
                }
            },
            MidiPort::Output(_) => {
                let output_map = self.output_map.as_rooted();
                if let Some(port) = output_map.get(cx, id.clone()) {
                    return Ok(port.port());
                }
            },
        }

        Err(format!("Unable to retrieve port with id {}", id))
    }
}

impl MIDIAccessMethods<crate::DomTypeHolder> for MIDIAccess {
    /// <https://webaudio.github.io/web-midi-api/#dom-midiaccess-inputs>
    fn Inputs(&self) -> DomRoot<MIDIInputMap> {
        DomRoot::from_ref(&*self.input_map)
    }

    /// <https://webaudio.github.io/web-midi-api/#dom-midiaccess-outputs>
    fn Outputs(&self) -> DomRoot<MIDIOutputMap> {
        DomRoot::from_ref(&*self.output_map)
    }

    // <https://webaudio.github.io/web-midi-api/#dom-midiaccess-onstatechange>
    event_handler!(statechange, GetOnstatechange, SetOnstatechange);

    /// <https://webaudio.github.io/web-midi-api/#dom-midiaccess-sysexenabled>
    fn SysexEnabled(&self) -> bool {
        // SysEx will initially ship disabled by default pending further TSC discussion
        false
    }
}

impl MIDIAccess {
    pub fn handle_midi_port_available(&self, port: MidiPort, cx: &mut js::context::JSContext) {
        let midi_port = match port {
            MidiPort::Input(midi_port_info) => {
                match self
                    .input_map
                    .get(cx, DOMString::from(midi_port_info.id.clone()))
                {
                    Some(input) => {
                        input.port().set_state(MIDIPortDeviceState::Connected);
                        input.port()
                    },
                    None => {
                        let port = MIDIInput::new(
                            self.global().as_window(),
                            DOMString::from(midi_port_info.id.clone()),
                            None,
                            Some(DOMString::from(midi_port_info.name)),
                            MIDIPortType::Input,
                            None,
                            MIDIPortDeviceState::Connected,
                            MIDIPortConnectionState::Closed,
                            cx,
                        );
                        self.input_map.set(
                            cx,
                            DOMString::from(midi_port_info.id),
                            DomRoot::from_ref(&port),
                        );
                        port.port()
                    },
                }
            },
            MidiPort::Output(midi_port_info) => {
                match self
                    .output_map
                    .get(cx, DOMString::from(midi_port_info.id.clone()))
                {
                    Some(output) => {
                        output.port().set_state(MIDIPortDeviceState::Connected);
                        output.port()
                    },
                    None => {
                        let port = MIDIOutput::new(
                            self.global().as_window(),
                            DOMString::from(midi_port_info.id.clone()),
                            None,
                            Some(DOMString::from(midi_port_info.name)),
                            MIDIPortType::Output,
                            None,
                            MIDIPortDeviceState::Connected,
                            MIDIPortConnectionState::Closed,
                            cx,
                        );
                        self.output_map.set(
                            cx,
                            DOMString::from(midi_port_info.id),
                            DomRoot::from_ref(&port),
                        );
                        port.port()
                    },
                }
            },
        };
        let event = MIDIConnectionEvent::new_with_proto(
            self.global().as_window(),
            None,
            Atom::from("statechange"),
            false,
            false,
            Some(&midi_port),
            cx,
        );
        event
            .upcast::<Event>()
            .fire(cx, self.upcast::<EventTarget>());
    }

    pub fn handle_midi_port_unavailable(&self, port: MidiPort, cx: &mut js::context::JSContext) {
        let midi_port = match port {
            MidiPort::Input(info) => {
                if let Some(input) = self.input_map.get(cx, DOMString::from(info.id.clone())) {
                    input.port().set_state(MIDIPortDeviceState::Disconnected);
                    input.port()
                } else {
                    log::warn!("Could not locate port in input map with id: {}", info.id);
                    return;
                }
            },
            MidiPort::Output(info) => {
                if let Some(output) = self.output_map.get(cx, DOMString::from(info.id.clone())) {
                    output.port().set_state(MIDIPortDeviceState::Disconnected);
                    output.port()
                } else {
                    log::warn!("Could not locate port in output map with id: {}", info.id);
                    return;
                }
            },
        };
        let connection_event = MIDIConnectionEvent::new_with_proto(
            self.global().as_window(),
            None,
            Atom::from("statechange"),
            false,
            false,
            Some(&midi_port),
            cx,
        );
        connection_event
            .upcast::<Event>()
            .fire(cx, self.upcast::<EventTarget>());
    }
}
