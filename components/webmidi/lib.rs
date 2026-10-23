/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use log::warn;
use servo_base::generic_channel::{self, GenericReceiver, GenericSender};
use servo_config::pref;
use webmidi_traits::{MIDIMsg, MIDIRequest};

mod webmidi_thread;

use crate::webmidi_thread::MIDI;

pub fn start_webmidi_thread() -> Option<(GenericSender<MIDIRequest>, GenericReceiver<MIDIMsg>)> {
    if !pref!(dom_webmidi_enabled) {
        return None;
    }

    let (sender, receiver) = match generic_channel::channel() {
        Some(sender_and_receiver) => sender_and_receiver,
        None => {
            warn!("Failed to create sender and receiver for WebMIDI thread",);
            return None;
        },
    };

    let (script_sender, script_recv) = match generic_channel::channel() {
        Some(sender_and_receiver) => sender_and_receiver,
        None => {
            warn!("Failed to create receiver and sender for WebMIDI thread",);
            return None;
        },
    };

    if let Err(e) = std::thread::Builder::new()
        .name("WebMIDI".to_owned())
        .spawn(move || {
            MIDI::new(receiver, script_sender).run();
        })
    {
        warn!("Failed to spawn WebMIDI thread ({})", e);
        return None;
    }
    Some((sender, script_recv))
}
