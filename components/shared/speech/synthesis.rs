/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use malloc_size_of_derive::MallocSizeOf;

#[derive(Clone, MallocSizeOf)]
pub struct Voice {
    pub name: String,
    pub lang: String,
    pub uri: String,
    pub local_service: bool,
    pub default: bool,
}

#[derive(Clone, MallocSizeOf)]
pub struct Utterance {
    pub text: String,
    pub voice: Option<Voice>,
    pub lang: Option<String>,
    pub volume: f32,
    pub rate: f32,
    pub pitch: f32,
}

impl Default for Utterance {
    fn default() -> Self {
        Utterance {
            text: String::new(),
            voice: None,
            lang: None,
            volume: 1.0,
            rate: 1.0,
            pitch: 1.0,
        }
    }
}
