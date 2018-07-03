/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::audionode::MAX_CHANNEL_COUNT;
use dom::bindings::cell::DomRefCell;
use dom::bindings::codegen::Bindings::AudioBufferBinding::{self, AudioBufferMethods, AudioBufferOptions};
use dom::bindings::error::{Error, Fallible};
use dom::bindings::num::Finite;
use dom::bindings::reflector::{DomObject, Reflector, reflect_dom_object};
use dom::bindings::root::DomRoot;
use dom::window::Window;
use dom_struct::dom_struct;
use js::conversions::ToJSValConvertible;
use js::jsapi::{Heap, JSContext, JSObject, JS_StealArrayBufferContents};
use js::typedarray::{CreateWith, Float32Array};
use std::ptr::{self, NonNull};
use std::slice;

type JSAudioChannel = Heap<*mut JSObject>;

#[dom_struct]
pub struct AudioBuffer {
    reflector_: Reflector,
    js_channels: Vec<JSAudioChannel>,
    shared_channels: DomRefCell<Option<Vec<Vec<f32>>>>,
    sample_rate: f32,
    length: u32,
    duration: f64,
    number_of_channels: u32,
}

impl AudioBuffer {
    #[allow(unrooted_must_root)]
    #[allow(unsafe_code)]
    pub fn new_inherited(options: &AudioBufferOptions) -> AudioBuffer {
        AudioBuffer {
            reflector_: Reflector::new(),
            js_channels: Vec::with_capacity(options.numberOfChannels as usize),
            shared_channels: DomRefCell::new(None),
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

    #[allow(unsafe_code)]
    fn restore_js_channel_data(&self, cx: *mut JSContext) -> bool {
        for (i, channel) in self.js_channels.iter().enumerate() {
            if !channel.get().is_null() {
                // Already have data in JS array.
                continue;
            }

            match *self.shared_channels.borrow_mut() {
                Some(ref mut shared_channels) => {
                    // Step 4 of https://webaudio.github.io/web-audio-api/#acquire-the-content
                    // "Attach ArrayBuffers containing copies of the data of the AudioBuffer, to
                    // be returned by the next call to getChannelData()".
                    rooted!(in (cx) let mut array = ptr::null_mut::<JSObject>());
                    let shared_channel = shared_channels.remove(i);
                    if unsafe {
                        Float32Array::create(cx, CreateWith::Slice(&shared_channel), array.handle_mut())
                    }.is_err() {
                        return false;
                    }
                    channel.set(array.get());
                },
                None => return false,
            }
        }

        *self.shared_channels.borrow_mut() = None;

        true
    }

    /// https://webaudio.github.io/web-audio-api/#acquire-the-content
    #[allow(unsafe_code)]
    pub fn acquire_contents(&self) {
        let cx = self.global().get_cx();
        for (i, channel) in self.js_channels.iter().enumerate() {
            // Step 1.
            if channel.get().is_null() {
                return;
            }

            // Step 2.
            let channel_data = unsafe {
                slice::from_raw_parts(
                    JS_StealArrayBufferContents(cx, channel.handle()) as *mut f32,
                    self.length as usize
                ).to_vec()
            };

            // Step 3.
            let mut shared_channels = self.shared_channels.borrow_mut();
            if shared_channels.is_none() {
                *shared_channels = Some(Vec::with_capacity(self.number_of_channels as usize));
            }
            (*shared_channels).as_mut().unwrap()[i] = channel_data;
        }
    }
}

impl AudioBufferMethods for AudioBuffer {
    /// https://webaudio.github.io/web-audio-api/#dom-audiobuffer-samplerate
    fn SampleRate(&self) -> Finite<f32> {
        Finite::wrap(self.sample_rate)
    }

    /// https://webaudio.github.io/web-audio-api/#dom-audiobuffer-length
    fn Length(&self) -> u32 {
        self.length
    }

    /// https://webaudio.github.io/web-audio-api/#dom-audiobuffer-duration
    fn Duration(&self) -> Finite<f64> {
        Finite::wrap(self.duration)
    }

    /// https://webaudio.github.io/web-audio-api/#dom-audiobuffer-numberofchannels
    fn NumberOfChannels(&self) -> u32 {
        self.number_of_channels
    }

    /// https://webaudio.github.io/web-audio-api/#dom-audiobuffer-getchanneldata
    #[allow(unsafe_code)]
    unsafe fn GetChannelData(&self, cx: *mut JSContext, channel: u32) -> Fallible<NonNull<JSObject>> {
        if channel >= self.number_of_channels {
            return Err(Error::IndexSize);
        }

        if !self.restore_js_channel_data(cx) {
            return Err(Error::JSFailed);
        }

        Ok(NonNull::new_unchecked(self.js_channels[channel as usize].get()))
    }
}
