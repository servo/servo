/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use std::default::Default;

use ipc_channel::ipc::IpcSender;
use malloc_size_of_derive::MallocSizeOf;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, MallocSizeOf, Serialize)]
pub struct Utterance {
    pub text: String,
    pub lang: Option<String>,
    pub volume: f32,
    pub rate: f32,
    pub pitch: f32,
    pub voice: Option<Voice>,
}

impl Default for Utterance {
    fn default() -> Self {
        Self {
            text: String::new(),
            lang: None,
            // https://webaudio.github.io/web-speech-api/#dom-speechsynthesisutterance-volume
            volume: 1.0,
            // https://webaudio.github.io/web-speech-api/#dom-speechsynthesisutterance-rate
            rate: 1.0,
            // https://webaudio.github.io/web-speech-api/#dom-speechsynthesisutterance-pitch
            pitch: 1.0,
            voice: None
        }
    }
}

#[derive(Clone, Debug, Deserialize, MallocSizeOf, Serialize)]
pub struct Voice {
    pub uri: String,
    pub name: String,
    pub lang: String,
    pub local_service: bool,
    pub default: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum SpeechSynthesisThreadMsg {
    Speak(Utterance),
    Pause,
    Resume,
    Cancel,
    GetVoices(IpcSender<Vec<Voice>>),
}
