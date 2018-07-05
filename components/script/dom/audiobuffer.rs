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
use js::jsapi::{Heap, JSContext, JSObject, JS_StealArrayBufferContents};
use js::rust::CustomAutoRooterGuard;
use js::typedarray::{CreateWith, Float32Array};
use servo_media::audio::buffer_source_node::AudioBuffer as ServoMediaAudioBuffer;
use std::ptr::{self, NonNull};
use std::sync::{Arc, Mutex};

type JSAudioChannel = Heap<*mut JSObject>;

#[dom_struct]
pub struct AudioBuffer {
    reflector_: Reflector,
    js_channels: DomRefCell<Vec<JSAudioChannel>>,
    #[ignore_malloc_size_of = "Arc"]
    shared_channels: Arc<Mutex<ServoMediaAudioBuffer>>,
    sample_rate: f32,
    length: u32,
    duration: f64,
    number_of_channels: u32,
}

impl AudioBuffer {
    #[allow(unrooted_must_root)]
    #[allow(unsafe_code)]
    pub fn new_inherited(cx: *mut JSContext,
                         number_of_channels: u32,
                         length: u32,
                         sample_rate: f32) -> AudioBuffer {
        let initial_data = vec![0.; length as usize];
        let mut js_channels: Vec<JSAudioChannel> = Vec::with_capacity(number_of_channels as usize);
        for _ in 0..number_of_channels {
            rooted!(in (cx) let mut array = ptr::null_mut::<JSObject>());
            let _ = unsafe {
                Float32Array::create(cx, CreateWith::Slice(initial_data.as_slice()), array.handle_mut())
            };
            let js_channel = Heap::default();
            js_channel.set(array.get());
            js_channels.push(js_channel);
        }
        AudioBuffer {
            reflector_: Reflector::new(),
            js_channels: DomRefCell::new(js_channels),
            shared_channels: Arc::new(Mutex::new(
                    ServoMediaAudioBuffer::new(number_of_channels as u8, length as usize))),
                    sample_rate: sample_rate,
                    length: length,
                    duration: length as f64 / sample_rate as f64,
                    number_of_channels: number_of_channels,
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(global: &Window,
               number_of_channels: u32,
               length: u32,
               sample_rate: f32) -> DomRoot<AudioBuffer> {
        let buffer = AudioBuffer::new_inherited(global.get_cx(), number_of_channels, length, sample_rate);
        reflect_dom_object(Box::new(buffer), global, AudioBufferBinding::Wrap)
    }

    pub fn Constructor(window: &Window,
                       options: &AudioBufferOptions) -> Fallible<DomRoot<AudioBuffer>> {
        if options.numberOfChannels > MAX_CHANNEL_COUNT {
            return Err(Error::NotSupported);
        }
        Ok(AudioBuffer::new(window, options.numberOfChannels, options.length, *options.sampleRate))
    }

    #[allow(unsafe_code)]
    fn restore_js_channel_data(&self, cx: *mut JSContext) -> bool {
        for (i, channel) in self.js_channels.borrow_mut().iter().enumerate() {
            if !channel.get().is_null() {
                // Already have data in JS array.
                continue;
            }

            // Move the channel data from shared_channels to js_channels.
            rooted!(in (cx) let mut array = ptr::null_mut::<JSObject>());
            let shared_channel = (*self.shared_channels.lock().unwrap()).buffers.remove(i);
            if unsafe {
                Float32Array::create(cx, CreateWith::Slice(&shared_channel), array.handle_mut())
            }.is_err() {
                return false;
            }
            channel.set(array.get());
        }

        true
    }

    /// https://webaudio.github.io/web-audio-api/#acquire-the-content
    #[allow(unsafe_code)]
    pub fn acquire_contents(&self) -> Option<Arc<Mutex<ServoMediaAudioBuffer>>> {
        let cx = self.global().get_cx();
        for (i, channel) in self.js_channels.borrow_mut().iter().enumerate() {
            // Step 1.
            if channel.get().is_null() {
                return None;
            }

            // Step 2.
            let channel_data = unsafe {
                typedarray!(in(cx) let array: Float32Array = channel.get());
                if let Ok(array) = array {
                    // XXX TypedArrays API does not expose a way to steal the buffer's
                    //     content.
                    let data = array.to_vec();
                    let _ = JS_StealArrayBufferContents(cx, channel.handle());
                    data
                } else {
                    return None;
                }
            };

            channel.set(ptr::null_mut());

            // Step 3.
            (*self.shared_channels.lock().unwrap()).buffers[i] = channel_data;

            // Step 4 will complete turning shared_channels
            // data into js_channels ArrayBuffers in restore_js_channel_data.
        }

        self.js_channels.borrow_mut().clear();

        Some(self.shared_channels.clone())
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

        Ok(NonNull::new_unchecked(self.js_channels.borrow()[channel as usize].get()))
    }

    fn CopyFromChannel(&self,
                       _destination: CustomAutoRooterGuard<Float32Array>,
                       _channel_number: u32,
                       _start_in_channel: u32) -> Fallible<()> {
        // XXX
        Ok(())
    }

    fn CopyToChannel(&self,
                     _source: CustomAutoRooterGuard<Float32Array>,
                     _channel_number: u32,
                     _start_in_channel: u32) -> Fallible<()> {
        // XXX
        Ok(())
    }
}
