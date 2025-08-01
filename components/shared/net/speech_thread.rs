/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use ipc_channel::ipc::IpcSender;
use malloc_size_of_derive::MallocSizeOf;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, MallocSizeOf, Serialize)]
pub struct Utterance {
    pub text: String,
    pub lang: String,
    pub volume: f64,
    pub rate: f64,
    pub pitch: f64,
    pub voice_uri: String,
}

#[derive(Debug, Deserialize, MallocSizeOf, Serialize)]
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
