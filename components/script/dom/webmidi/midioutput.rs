/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::time::Duration;

use dom_struct::dom_struct;
use js::realm::CurrentRealm;
use script_bindings::cell::DomRefCell;
use script_bindings::codegen::GenericBindings::MIDIPortBinding::MIDIPortMethods;
use script_bindings::error::{Error, Fallible};
use script_bindings::reflector::reflect_dom_object_with_cx;
use servo_base::cross_process_instant::CrossProcessInstant;
use servo_constellation_traits::ScriptToConstellationMessage;
use timers::{TimerEventRequest, TimerId};
use webmidi_traits::MIDIRequest;

use crate::dom::bindings::codegen::Bindings::MIDIPortBinding::{
    MIDIOutputMethods, MIDIPortConnectionState, MIDIPortDeviceState, MIDIPortType,
};
use crate::dom::bindings::codegen::Bindings::PerformanceBinding::DOMHighResTimeStamp;
use crate::dom::bindings::num::Finite;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::DomGlobal;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::webmidi::midiport::MIDIPort;
use crate::dom::window::Window;
use crate::event_loop::script_thread::with_script_thread;

// https://webaudio.github.io/web-midi-api/#MIDIOutput
#[dom_struct]
pub(crate) struct MIDIOutput {
    port: MIDIPort,
    #[no_trace]
    queued_messages: DomRefCell<Vec<TimerId>>,
}

impl MIDIOutput {
    fn new_inherited(
        id: DOMString,
        manufacturer: Option<DOMString>,
        name: Option<DOMString>,
        type_: MIDIPortType,
        version: Option<DOMString>,
        state: MIDIPortDeviceState,
        connection: MIDIPortConnectionState,
    ) -> MIDIOutput {
        MIDIOutput {
            port: MIDIPort::new_inherited(
                id,
                manufacturer,
                name,
                type_,
                version,
                state,
                connection,
            ),
            queued_messages: DomRefCell::new(Vec::new()),
        }
    }

    #[expect(clippy::too_many_arguments)]
    pub(crate) fn new(
        window: &Window,
        id: DOMString,
        manufacturer: Option<DOMString>,
        name: Option<DOMString>,
        type_: MIDIPortType,
        version: Option<DOMString>,
        state: MIDIPortDeviceState,
        connection: MIDIPortConnectionState,
        cx: &mut js::context::JSContext,
    ) -> DomRoot<MIDIOutput> {
        reflect_dom_object_with_cx(
            Box::new(MIDIOutput::new_inherited(
                id,
                manufacturer,
                name,
                type_,
                version,
                state,
                connection,
            )),
            window,
            cx,
        )
    }

    pub(crate) fn port(&self) -> DomRoot<MIDIPort> {
        DomRoot::from_ref(&self.port)
    }
}

impl MIDIOutputMethods<crate::DomTypeHolder> for MIDIOutput {
    /// <https://webaudio.github.io/web-midi-api/#dom-midioutput-send>
    fn Send(&self, cx: &mut CurrentRealm, data: Vec<u8>, timestamp: Finite<f64>) -> Fallible<()> {
        // If data is not a valid sequence or does not contain a valid MIDI message, throw a TypeError exception.
        self.validate_midi_message(&data)?;
        // If data is a System Exclusive message, and the MIDIAccess did not enable System Exclusive access, throw an NotAllowedError exception.
        if data[0] == 0xF0 {
            return Err(Error::NotAllowed(Some(
                "SysEx capabilities are not currently enabled.".to_string(),
            )));
        }
        // If the port is "disconnected", throw an InvalidStateError exception.
        if self.port.State() == MIDIPortDeviceState::Disconnected {
            return Err(Error::InvalidState(Some(
                "MIDI device is currently disconnected.".to_string(),
            )));
        }
        // If the port is "connected" but the connection is "closed", asynchronously try to open the port.
        if self.port.Connection() != MIDIPortConnectionState::Open {
            let _ = self.port.Open(cx);
        }

        let global = self.global();
        let trusted_global = Trusted::new(&*global);

        let midi_request = MIDIRequest::SendData(self.port.Id().to_string(), data);

        let end = *DOMHighResTimeStamp::from(timestamp) as u64;
        let start = *self
            .global()
            .performance(cx)
            .to_dom_high_res_time_stamp(CrossProcessInstant::now()) as u64;
        if *timestamp > 0. {
            let request = TimerEventRequest {
                callback: Box::new(move || {
                    let msg = ScriptToConstellationMessage::MIDIRequest(midi_request.clone());
                    let global = trusted_global.root();
                    global
                        .script_to_constellation_chan()
                        .send(msg)
                        .expect("Failed to send MIDIRequest::SendData to constellation");
                }),
                duration: Duration::from_millis(end - start),
            };
            let timer_id = self
                .global()
                .schedule_timer(request)
                .expect("Failed to schedule timer for MIDI message");
            self.queued_messages.borrow_mut().push(timer_id);
        } else {
            let msg = ScriptToConstellationMessage::MIDIRequest(midi_request);
            self.global()
                .script_to_constellation_chan()
                .send(msg)
                .expect("Failed to send MIDIRequest::SendData to constellation");
        }

        Ok(())
    }

    /// <https://webaudio.github.io/web-midi-api/#dom-midioutput-clear>
    fn Clear(&self) {
        self.queued_messages
            .take()
            .iter()
            .for_each(|timer_id| with_script_thread(|s| s.cancel_timer(*timer_id)));
    }
}

impl MIDIOutput {
    /// <https://webaudio.github.io/web-midi-api/#dfn-valid-midi-message>
    fn validate_midi_message(&self, data: &[u8]) -> Result<(), Error> {
        let len = data.len();

        // Reject empty messages
        if len == 0 {
            return Err(Error::Type(c"MIDI message cannot be empty.".to_owned()));
        }
        // The first byte (the status byte) should have the high bit set
        if data[0] & 0x80 == 0 {
            return Err(Error::Type(
                c"Status byte should have high bit set.".to_owned(),
            ));
        }
        // Any following bytes should not unless they are part of a System Exclusive message
        if data[0] != 0xF0 && data[1..].iter().any(|byte| (byte & 0x80) > 0) {
            return Err(Error::Type(
                c"Non-sysex messages should not have high bit set after status byte.".to_owned(),
            ));
        }
        // If the high nibble of the status byte in hex is 8, 9, A, B, or E then the total message length should be 3 bytes
        if matches!(data[0] & 0xF0, 0x80 | 0x90 | 0xA0 | 0xB0 | 0xE0) && len != 3 {
            return Err(Error::Type(c"Message length should be 3 bytes.".to_owned()));
        }
        // If the high nibble of the status byte is C or D then the total message length should be 2 bytes
        if matches!(data[0] & 0xF0, 0xC0 | 0xD0) && len != 2 {
            return Err(Error::Type(c"Message length should be 2 bytes.".to_owned()));
        }
        // If the status byte is F1 or F3 then the total message length should be 2 bytes
        if matches!(data[0], 0xF1 | 0xF3) && len != 2 {
            return Err(Error::Type(c"Message length should be 2 bytes.".to_owned()));
        }
        // If the status byte is F2 then the total message length should be 3 bytes
        if data[0] == 0xF2 && len != 3 {
            return Err(Error::Type(c"Message length should be 3 bytes.".to_owned()));
        }
        // If the status byte is F6, F8, FA, FB, FC, FE, or FF then the total message length should be 1 byte (only the status byte)
        if matches!(data[0], 0xF6 | 0xF8 | 0xFA | 0xFB | 0xFC | 0xFE | 0xFF) && len > 1 {
            return Err(Error::Type(c"Message length should be 1 byte.".to_owned()));
        }
        // If the status byte is F0 then this is a System Exclusive message with no length restriction, and the last byte should be F7
        if data[0] == 0xF0 && data[len - 1] != 0xF7 {
            return Err(Error::Type(
                c"SysEx message does not terminate with SysEx off byte.".to_owned(),
            ));
        }
        // If the status byte is F4, F5, F7, F9, or FD then the message is not valid
        if matches!(data[0], 0xF4 | 0xF5 | 0xF7 | 0xF9 | 0xFD) {
            return Err(Error::Type(c"Invalid status byte.".to_owned()));
        }

        Ok(())
    }
}
