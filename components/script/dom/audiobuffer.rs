/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cmp::min;

use dom_struct::dom_struct;
use js::rust::{CustomAutoRooterGuard, HandleObject};
use js::typedarray::{Float32, Float32Array};
use servo_media::audio::buffer_source_node::AudioBuffer as ServoMediaAudioBuffer;

use super::bindings::buffer_source::HeapBufferSource;
use crate::dom::audionode::MAX_CHANNEL_COUNT;
use crate::dom::bindings::cell::{DomRefCell, Ref};
use crate::dom::bindings::codegen::Bindings::AudioBufferBinding::{
    AudioBufferMethods, AudioBufferOptions,
};
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::num::Finite;
use crate::dom::bindings::reflector::{reflect_dom_object_with_proto, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::globalscope::GlobalScope;
use crate::dom::window::Window;
use crate::realms::enter_realm;
use crate::script_runtime::JSContext;

// Spec mandates at least [8000, 96000], we use [8000, 192000] to match Firefox
// https://webaudio.github.io/web-audio-api/#dom-baseaudiocontext-createbuffer
pub const MIN_SAMPLE_RATE: f32 = 8000.;
pub const MAX_SAMPLE_RATE: f32 = 192000.;

/// The AudioBuffer keeps its data either in js_channels
/// or in shared_channels if js_channels buffers are detached.
///
/// js_channels buffers are (re)attached right before calling GetChannelData
/// and remain attached until its contents are needed by some other API
/// implementation. Follow <https://webaudio.github.io/web-audio-api/#acquire-the-content>
/// to know in which situations js_channels buffers must be detached.
///
#[dom_struct]
pub struct AudioBuffer {
    reflector_: Reflector,
    /// Float32Arrays returned by calls to GetChannelData.
    #[ignore_malloc_size_of = "mozjs"]
    js_channels: DomRefCell<Vec<HeapBufferSource<Float32>>>,
    /// Aggregates the data from js_channels.
    /// This is `Some<T>` iff the buffers in js_channels are detached.
    #[ignore_malloc_size_of = "servo_media"]
    #[no_trace]
    shared_channels: DomRefCell<Option<ServoMediaAudioBuffer>>,
    /// <https://webaudio.github.io/web-audio-api/#dom-audiobuffer-samplerate>
    sample_rate: f32,
    /// <https://webaudio.github.io/web-audio-api/#dom-audiobuffer-length>
    length: u32,
    /// <https://webaudio.github.io/web-audio-api/#dom-audiobuffer-duration>
    duration: f64,
    /// <https://webaudio.github.io/web-audio-api/#dom-audiobuffer-numberofchannels>
    number_of_channels: u32,
}

impl AudioBuffer {
    #[allow(crown::unrooted_must_root)]
    pub fn new_inherited(number_of_channels: u32, length: u32, sample_rate: f32) -> AudioBuffer {
        let vec = (0..number_of_channels)
            .map(|_| HeapBufferSource::default())
            .collect();
        AudioBuffer {
            reflector_: Reflector::new(),
            js_channels: DomRefCell::new(vec),
            shared_channels: DomRefCell::new(None),
            sample_rate,
            length,
            duration: length as f64 / sample_rate as f64,
            number_of_channels,
        }
    }

    pub fn new(
        global: &Window,
        number_of_channels: u32,
        length: u32,
        sample_rate: f32,
        initial_data: Option<&[Vec<f32>]>,
    ) -> DomRoot<AudioBuffer> {
        Self::new_with_proto(
            global,
            None,
            number_of_channels,
            length,
            sample_rate,
            initial_data,
        )
    }

    #[allow(crown::unrooted_must_root)]
    fn new_with_proto(
        global: &Window,
        proto: Option<HandleObject>,
        number_of_channels: u32,
        length: u32,
        sample_rate: f32,
        initial_data: Option<&[Vec<f32>]>,
    ) -> DomRoot<AudioBuffer> {
        let buffer = AudioBuffer::new_inherited(number_of_channels, length, sample_rate);
        let buffer = reflect_dom_object_with_proto(Box::new(buffer), global, proto);
        buffer.set_initial_data(initial_data);
        buffer
    }

    // https://webaudio.github.io/web-audio-api/#dom-audiobuffer-audiobuffer
    #[allow(non_snake_case)]
    pub fn Constructor(
        window: &Window,
        proto: Option<HandleObject>,
        options: &AudioBufferOptions,
    ) -> Fallible<DomRoot<AudioBuffer>> {
        if options.length == 0 ||
            options.numberOfChannels == 0 ||
            options.numberOfChannels > MAX_CHANNEL_COUNT ||
            *options.sampleRate < MIN_SAMPLE_RATE ||
            *options.sampleRate > MAX_SAMPLE_RATE
        {
            return Err(Error::NotSupported);
        }
        Ok(AudioBuffer::new_with_proto(
            window,
            proto,
            options.numberOfChannels,
            options.length,
            *options.sampleRate,
            None,
        ))
    }

    // Initialize the underlying channels data with initial data provided by
    // the user or silence otherwise.
    fn set_initial_data(&self, initial_data: Option<&[Vec<f32>]>) {
        let mut channels = ServoMediaAudioBuffer::new(
            self.number_of_channels as u8,
            self.length as usize,
            self.sample_rate,
        );
        for channel in 0..self.number_of_channels {
            channels.buffers[channel as usize] = match initial_data {
                Some(data) => data[channel as usize].clone(),
                None => vec![0.; self.length as usize],
            };
        }
        *self.shared_channels.borrow_mut() = Some(channels);
    }

    fn restore_js_channel_data(&self, cx: JSContext) -> bool {
        let _ac = enter_realm(self);
        for (i, channel) in self.js_channels.borrow_mut().iter().enumerate() {
            if channel.is_initialized() {
                // Already have data in JS array.
                continue;
            }

            if let Some(ref shared_channels) = *self.shared_channels.borrow() {
                // Step 4. of
                // https://webaudio.github.io/web-audio-api/#acquire-the-content
                // "Attach ArrayBuffers containing copies of the data to the AudioBuffer,
                // to be returned by the next call to getChannelData()".
                if channel.set_data(cx, &shared_channels.buffers[i]).is_err() {
                    return false;
                }
            }
        }

        *self.shared_channels.borrow_mut() = None;

        true
    }

    // https://webaudio.github.io/web-audio-api/#acquire-the-content
    fn acquire_contents(&self) -> Option<ServoMediaAudioBuffer> {
        let mut result = ServoMediaAudioBuffer::new(
            self.number_of_channels as u8,
            self.length as usize,
            self.sample_rate,
        );
        let cx = GlobalScope::get_cx();
        for (i, channel) in self.js_channels.borrow_mut().iter().enumerate() {
            // Step 1.
            if !channel.is_initialized() {
                return None;
            }

            // Step 3.
            result.buffers[i] = channel.acquire_data(cx).ok()?;
        }

        Some(result)
    }

    pub fn get_channels(&self) -> Ref<Option<ServoMediaAudioBuffer>> {
        if self.shared_channels.borrow().is_none() {
            let channels = self.acquire_contents();
            if channels.is_some() {
                *self.shared_channels.borrow_mut() = channels;
            }
        }
        return self.shared_channels.borrow();
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
    fn GetChannelData(&self, cx: JSContext, channel: u32) -> Fallible<Float32Array> {
        if channel >= self.number_of_channels {
            return Err(Error::IndexSize);
        }

        if !self.restore_js_channel_data(cx) {
            return Err(Error::JSFailed);
        }

        self.js_channels.borrow()[channel as usize]
            .get_buffer()
            .map_err(|_| Error::JSFailed)
    }

    // https://webaudio.github.io/web-audio-api/#dom-audiobuffer-copyfromchannel
    #[allow(unsafe_code)]
    fn CopyFromChannel(
        &self,
        mut destination: CustomAutoRooterGuard<Float32Array>,
        channel_number: u32,
        start_in_channel: u32,
    ) -> Fallible<()> {
        if destination.is_shared() {
            return Err(Error::Type("Cannot copy to shared buffer".to_owned()));
        }

        if channel_number >= self.number_of_channels || start_in_channel >= self.length {
            return Err(Error::IndexSize);
        }

        let bytes_to_copy = min(self.length - start_in_channel, destination.len() as u32) as usize;
        let cx = GlobalScope::get_cx();
        let channel_number = channel_number as usize;
        let offset = start_in_channel as usize;
        let mut dest = vec![0.0_f32; bytes_to_copy];

        // We either copy form js_channels or shared_channels.
        let js_channel = &self.js_channels.borrow()[channel_number];
        if js_channel.is_initialized() {
            if js_channel
                .copy_data_to(cx, &mut dest, offset, offset + bytes_to_copy)
                .is_err()
            {
                return Err(Error::IndexSize);
            }
        } else if let Some(ref shared_channels) = *self.shared_channels.borrow() {
            if let Some(shared_channel) = shared_channels.buffers.get(channel_number) {
                dest.extend_from_slice(&shared_channel.as_slice()[offset..offset + bytes_to_copy]);
            }
        }

        unsafe {
            destination.update(&dest);
        }

        Ok(())
    }

    // https://webaudio.github.io/web-audio-api/#dom-audiobuffer-copytochannel
    fn CopyToChannel(
        &self,
        source: CustomAutoRooterGuard<Float32Array>,
        channel_number: u32,
        start_in_channel: u32,
    ) -> Fallible<()> {
        if source.is_shared() {
            return Err(Error::Type("Cannot copy from shared buffer".to_owned()));
        }

        if channel_number >= self.number_of_channels || start_in_channel > (source.len() as u32) {
            return Err(Error::IndexSize);
        }

        let cx = GlobalScope::get_cx();
        if !self.restore_js_channel_data(cx) {
            return Err(Error::JSFailed);
        }

        let js_channel = &self.js_channels.borrow()[channel_number as usize];
        if !js_channel.is_initialized() {
            // The array buffer was detached.
            return Err(Error::IndexSize);
        }

        let bytes_to_copy = min(self.length - start_in_channel, source.len() as u32) as usize;
        js_channel
            .copy_data_from(cx, source, start_in_channel as usize, bytes_to_copy)
            .map_err(|_| Error::IndexSize)
    }
}
