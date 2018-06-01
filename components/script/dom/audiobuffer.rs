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
use js::jsapi::{Heap, JSAutoCompartment, JSContext, JSObject};
use js::jsapi::JS_GetArrayBufferViewBuffer;
use js::rust::CustomAutoRooterGuard;
use js::rust::wrappers::JS_DetachArrayBuffer;
use js::typedarray::{CreateWith, Float32Array};
use servo_media::audio::buffer_source_node::AudioBuffer as ServoMediaAudioBuffer;
use std::cmp::min;
use std::ptr::{self, NonNull};

// This range is defined by the spec.
// https://webaudio.github.io/web-audio-api/#dom-baseaudiocontext-createbuffer
pub const MIN_SAMPLE_RATE: f32 = 8000.;
pub const MAX_SAMPLE_RATE: f32 = 96000.;

type JSAudioChannel = Heap<*mut JSObject>;

#[dom_struct]
pub struct AudioBuffer {
    reflector_: Reflector,
    js_channels: DomRefCell<Vec<JSAudioChannel>>,
    #[ignore_malloc_size_of = "servo_media"]
    shared_channels: DomRefCell<ServoMediaAudioBuffer>,
    sample_rate: f32,
    length: u32,
    duration: f64,
    number_of_channels: u32,
}

impl AudioBuffer {
    #[allow(unrooted_must_root)]
    #[allow(unsafe_code)]
    pub fn new_inherited(number_of_channels: u32, length: u32, sample_rate: f32) -> AudioBuffer {
        let vec = (0..number_of_channels).map(|_| Heap::default()).collect();
        AudioBuffer {
            reflector_: Reflector::new(),
            js_channels: DomRefCell::new(vec),
            shared_channels: DomRefCell::new(ServoMediaAudioBuffer::new(
                number_of_channels as u8,
                length as usize,
            )),
            sample_rate,
            length,
            duration: length as f64 / sample_rate as f64,
            number_of_channels,
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(
        global: &Window,
        number_of_channels: u32,
        length: u32,
        sample_rate: f32,
        initial_data: Option<&[f32]>,
    ) -> DomRoot<AudioBuffer> {
        let buffer = AudioBuffer::new_inherited(number_of_channels, length, sample_rate);
        let buffer = reflect_dom_object(Box::new(buffer), global, AudioBufferBinding::Wrap);
        buffer.set_channels(initial_data);
        buffer
    }

    // https://webaudio.github.io/web-audio-api/#dom-audiobuffer-audiobuffer
    pub fn Constructor(
        window: &Window,
        options: &AudioBufferOptions,
    ) -> Fallible<DomRoot<AudioBuffer>> {
        if options.numberOfChannels > MAX_CHANNEL_COUNT ||
            *options.sampleRate < MIN_SAMPLE_RATE ||
            *options.sampleRate > MAX_SAMPLE_RATE
        {
            return Err(Error::NotSupported);
        }
        Ok(AudioBuffer::new(
            window,
            options.numberOfChannels,
            options.length,
            *options.sampleRate,
            None,
        ))
    }

    #[allow(unsafe_code)]
    pub fn set_channels(&self, initial_data: Option<&[f32]>) {
        let global = self.global();
        let cx = global.get_cx();
        let _ac = JSAutoCompartment::new(cx, global.reflector().get_jsobject().get());
        let chans = self.js_channels.borrow_mut();
        for channel in 0..self.number_of_channels {
            rooted!(in (cx) let mut array = ptr::null_mut::<JSObject>());
            let offset = (channel * self.length) as usize;
            match initial_data {
                Some(data) => {
                    let _ = unsafe {
                        Float32Array::create(
                            cx,
                            CreateWith::Slice(&data[offset..offset + (self.length as usize) - 1]),
                            array.handle_mut(),
                        )
                    };
                },
                None => {
                    let _ = unsafe {
                        Float32Array::create(
                            cx,
                            CreateWith::Slice(&vec![0.; self.length as usize]),
                            array.handle_mut(),
                        )
                    };
                },
            }
            chans[channel as usize].set(array.get());
        }
    }

    #[allow(unsafe_code)]
    unsafe fn restore_js_channel_data(&self, cx: *mut JSContext) -> bool {
        for (i, channel) in self.js_channels.borrow_mut().iter().enumerate() {
            if !channel.get().is_null() {
                // Already have data in JS array.
                continue;
            }

            // Move the channel data from shared_channels to js_channels.
            rooted!(in (cx) let mut array = ptr::null_mut::<JSObject>());
            let shared_channel = (*self.shared_channels.borrow_mut()).buffers.remove(i);
            if Float32Array::create(cx, CreateWith::Slice(&shared_channel), array.handle_mut())
                .is_err()
            {
                return false;
            }
            channel.set(array.get());
        }

        true
    }

    // https://webaudio.github.io/web-audio-api/#acquire-the-content
    #[allow(unsafe_code)]
    pub fn acquire_contents(&self) -> Option<ServoMediaAudioBuffer> {
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
                    let data = array.to_vec();
                    let mut is_shared = false;
                    rooted!(in (cx) let view_buffer =
                        JS_GetArrayBufferViewBuffer(cx, channel.handle(), &mut is_shared));
                    // This buffer is always created unshared
                    debug_assert!(!is_shared);
                    let _ = JS_DetachArrayBuffer(cx, view_buffer.handle());
                    data
                } else {
                    return None;
                }
            };

            channel.set(ptr::null_mut());

            // Step 3.
            (*self.shared_channels.borrow_mut()).buffers[i] = channel_data;

            // Step 4 will complete turning shared_channels
            // data into js_channels ArrayBuffers in restore_js_channel_data.
        }

        self.js_channels.borrow_mut().clear();

        Some((*self.shared_channels.borrow()).clone())
    }
}

impl AudioBufferMethods for AudioBuffer {
    // https://webaudio.github.io/web-audio-api/#dom-audiobuffer-samplerate
    fn SampleRate(&self) -> Finite<f32> {
        Finite::wrap(self.sample_rate)
    }

    // https://webaudio.github.io/web-audio-api/#dom-audiobuffer-length
    fn Length(&self) -> u32 {
        self.length
    }

    // https://webaudio.github.io/web-audio-api/#dom-audiobuffer-duration
    fn Duration(&self) -> Finite<f64> {
        Finite::wrap(self.duration)
    }

    // https://webaudio.github.io/web-audio-api/#dom-audiobuffer-numberofchannels
    fn NumberOfChannels(&self) -> u32 {
        self.number_of_channels
    }

    // https://webaudio.github.io/web-audio-api/#dom-audiobuffer-getchanneldata
    #[allow(unsafe_code)]
    unsafe fn GetChannelData(
        &self,
        cx: *mut JSContext,
        channel: u32,
    ) -> Fallible<NonNull<JSObject>> {
        if channel >= self.number_of_channels {
            return Err(Error::IndexSize);
        }

        if !self.restore_js_channel_data(cx) {
            return Err(Error::JSFailed);
        }

        Ok(NonNull::new_unchecked(
            self.js_channels.borrow()[channel as usize].get(),
        ))
    }

    // https://webaudio.github.io/web-audio-api/#dom-audiobuffer-copyfromchannel
    #[allow(unsafe_code)]
    fn CopyFromChannel(
        &self,
        mut destination: CustomAutoRooterGuard<Float32Array>,
        channel_number: u32,
        start_in_channel: u32,
    ) -> Fallible<()> {
        if channel_number >= self.number_of_channels || start_in_channel > self.length {
            return Err(Error::IndexSize);
        }

        let bytes_to_copy = min(self.length - start_in_channel, destination.len() as u32) as usize;
        let cx = self.global().get_cx();
        let channel_number = channel_number as usize;
        let offset = start_in_channel as usize;
        let mut dest = Vec::with_capacity(destination.len());

        // We either copy form js_channels or shared_channels.

        let js_channel = self.js_channels.borrow()[channel_number].get();
        if !js_channel.is_null() {
            typedarray!(in(cx) let array: Float32Array = js_channel);
            if let Ok(array) = array {
                let data = unsafe { array.as_slice() };
                dest.extend_from_slice(&data[offset..offset + bytes_to_copy]);
                return Ok(());
            }
        }

        if let Some(shared_channel) = self.shared_channels.borrow().buffers.get(channel_number) {
            dest.extend_from_slice(&shared_channel.as_slice()[offset..offset + bytes_to_copy]);
        }

        unsafe {
            destination.update(&dest);
        }

        Ok(())
    }

    // https://webaudio.github.io/web-audio-api/#dom-audiobuffer-copytochannel
    #[allow(unsafe_code)]
    fn CopyToChannel(
        &self,
        source: CustomAutoRooterGuard<Float32Array>,
        channel_number: u32,
        start_in_channel: u32,
    ) -> Fallible<()> {
        if channel_number >= self.number_of_channels || start_in_channel > (source.len() as u32) {
            return Err(Error::IndexSize);
        }

        let cx = self.global().get_cx();
        if unsafe { !self.restore_js_channel_data(cx) } {
            return Err(Error::JSFailed);
        }

        let js_channel = self.js_channels.borrow()[channel_number as usize].get();
        if js_channel.is_null() {
            // The array buffer was detached.
            return Err(Error::IndexSize);
        }

        typedarray!(in(cx) let array: Float32Array = js_channel);
        if let Ok(mut array) = array {
            let bytes_to_copy = min(self.length - start_in_channel, source.len() as u32) as usize;
            let offset = start_in_channel as usize;
            unsafe {
                array.update(&source.as_slice()[offset..offset + bytes_to_copy]);
            }
        } else {
            return Err(Error::IndexSize);
        }

        Ok(())
    }
}
