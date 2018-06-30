/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::audionode::MAX_CHANNEL_COUNT;
use dom::bindings::codegen::Bindings::AudioBufferBinding::{self, AudioBufferMethods, AudioBufferOptions};
use dom::bindings::error::{Error, Fallible};
use dom::bindings::num::Finite;
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::bindings::root::DomRoot;
use dom::window::Window;
use dom_struct::dom_struct;
use smallvec::SmallVec;

#[derive(JSTraceable, MallocSizeOf)]
struct AudioChannel(pub Vec<f32>);

impl AudioChannel {
    pub fn new(capacity: usize) -> AudioChannel {
        AudioChannel(Vec::with_capacity(capacity))
    }
}

#[dom_struct]
pub struct AudioBuffer {
    reflector_: Reflector,
    internal_data: SmallVec<[AudioChannel; MAX_CHANNEL_COUNT as usize]>,
    sample_rate: f32,
    length: u32,
    duration: f64,
    number_of_channels: u32,
}

impl AudioBuffer {
    #[allow(unrooted_must_root)]
    #[allow(unsafe_code)]
    pub fn new_inherited(options: &AudioBufferOptions) -> AudioBuffer {
        let mut internal_data = SmallVec::new();
        unsafe { internal_data.set_len(options.numberOfChannels as usize); }
        AudioBuffer {
            reflector_: Reflector::new(),
            internal_data,
            sample_rate: *options.sampleRate,
            length: options.length,
            duration: options.length as f64 / *options.sampleRate as f64,
            number_of_channels: options.numberOfChannels,
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(global: &Window,
               options: &AudioBufferOptions) -> DomRoot<AudioBuffer> {
        let buffer = AudioBuffer::new_inherited(options);
        reflect_dom_object(Box::new(buffer), global, AudioBufferBinding::Wrap)
    }

    pub fn Constructor(window: &Window,
                       options: &AudioBufferOptions) -> Fallible<DomRoot<AudioBuffer>> {
        if options.numberOfChannels > MAX_CHANNEL_COUNT {
            return Err(Error::NotSupported);
        }
        Ok(AudioBuffer::new(window, options))
    }
}

impl AudioBufferMethods for AudioBuffer {
    fn SampleRate(&self) -> Finite<f32> {
        Finite::wrap(self.sample_rate)
    }

    fn Length(&self) -> u32 {
        self.length
    }

    fn Duration(&self) -> Finite<f64> {
        Finite::wrap(self.duration)
    }

    fn NumberOfChannels(&self) -> u32 {
        self.number_of_channels
    }
}
