/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::compartments::{AlreadyInCompartment, InCompartment};
use crate::dom::audiobuffer::{AudioBuffer, MAX_SAMPLE_RATE, MIN_SAMPLE_RATE};
use crate::dom::audionode::MAX_CHANNEL_COUNT;
use crate::dom::baseaudiocontext::{BaseAudioContext, BaseAudioContextOptions};
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::BaseAudioContextBinding::BaseAudioContextBinding::BaseAudioContextMethods;
use crate::dom::bindings::codegen::Bindings::OfflineAudioContextBinding;
use crate::dom::bindings::codegen::Bindings::OfflineAudioContextBinding::OfflineAudioContextMethods;
use crate::dom::bindings::codegen::Bindings::OfflineAudioContextBinding::OfflineAudioContextOptions;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::num::Finite;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject};
use crate::dom::bindings::root::DomRoot;
use crate::dom::event::{Event, EventBubbles, EventCancelable};
use crate::dom::offlineaudiocompletionevent::OfflineAudioCompletionEvent;
use crate::dom::promise::Promise;
use crate::dom::window::Window;
use crate::task_source::TaskSource;
use dom_struct::dom_struct;
use servo_media::audio::context::OfflineAudioContextOptions as ServoMediaOfflineAudioContextOptions;
use std::cell::Cell;
use std::rc::Rc;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread::Builder;

#[dom_struct]
pub struct OfflineAudioContext {
    context: BaseAudioContext,
    channel_count: u32,
    length: u32,
    rendering_started: Cell<bool>,
    #[ignore_malloc_size_of = "promises are hard"]
    pending_rendering_promise: DomRefCell<Option<Rc<Promise>>>,
}

impl OfflineAudioContext {
    #[allow(unrooted_must_root)]
    fn new_inherited(channel_count: u32, length: u32, sample_rate: f32) -> OfflineAudioContext {
        let options = ServoMediaOfflineAudioContextOptions {
            channels: channel_count as u8,
            length: length as usize,
            sample_rate,
        };
        let context =
            BaseAudioContext::new_inherited(BaseAudioContextOptions::OfflineAudioContext(options));
        OfflineAudioContext {
            context,
            channel_count,
            length,
            rendering_started: Cell::new(false),
            pending_rendering_promise: Default::default(),
        }
    }

    #[allow(unrooted_must_root)]
    fn new(
        window: &Window,
        channel_count: u32,
        length: u32,
        sample_rate: f32,
    ) -> Fallible<DomRoot<OfflineAudioContext>> {
        if channel_count > MAX_CHANNEL_COUNT ||
            channel_count <= 0 ||
            length <= 0 ||
            sample_rate < MIN_SAMPLE_RATE ||
            sample_rate > MAX_SAMPLE_RATE
        {
            return Err(Error::NotSupported);
        }
        let context = OfflineAudioContext::new_inherited(channel_count, length, sample_rate);
        Ok(reflect_dom_object(
            Box::new(context),
            window,
            OfflineAudioContextBinding::Wrap,
        ))
    }

    pub fn Constructor(
        window: &Window,
        options: &OfflineAudioContextOptions,
    ) -> Fallible<DomRoot<OfflineAudioContext>> {
        OfflineAudioContext::new(
            window,
            options.numberOfChannels,
            options.length,
            *options.sampleRate,
        )
    }

    pub fn Constructor_(
        window: &Window,
        number_of_channels: u32,
        length: u32,
        sample_rate: Finite<f32>,
    ) -> Fallible<DomRoot<OfflineAudioContext>> {
        OfflineAudioContext::new(window, number_of_channels, length, *sample_rate)
    }
}

impl OfflineAudioContextMethods for OfflineAudioContext {
    // https://webaudio.github.io/web-audio-api/#dom-offlineaudiocontext-oncomplete
    event_handler!(complete, GetOncomplete, SetOncomplete);

    // https://webaudio.github.io/web-audio-api/#dom-offlineaudiocontext-length
    fn Length(&self) -> u32 {
        self.length
    }

    // https://webaudio.github.io/web-audio-api/#dom-offlineaudiocontext-startrendering
    fn StartRendering(&self) -> Rc<Promise> {
        let in_compartment_proof = AlreadyInCompartment::assert(&self.global());
        let promise = Promise::new_in_current_compartment(
            &self.global(),
            InCompartment::Already(&in_compartment_proof),
        );
        if self.rendering_started.get() {
            promise.reject_error(Error::InvalidState);
            return promise;
        }
        self.rendering_started.set(true);

        *self.pending_rendering_promise.borrow_mut() = Some(promise.clone());

        let processed_audio = Arc::new(Mutex::new(Vec::new()));
        let processed_audio_ = processed_audio.clone();
        let (sender, receiver) = mpsc::channel();
        let sender = Mutex::new(sender);
        self.context
            .audio_context_impl()
            .set_eos_callback(Box::new(move |buffer| {
                processed_audio_
                    .lock()
                    .unwrap()
                    .extend_from_slice((*buffer).as_ref());
                let _ = sender.lock().unwrap().send(());
            }));

        let this = Trusted::new(self);
        let global = self.global();
        let window = global.as_window();
        let (task_source, canceller) = window
            .task_manager()
            .media_element_task_source_with_canceller();
        Builder::new()
            .name("OfflineAudioContextResolver".to_owned())
            .spawn(move || {
                let _ = receiver.recv();
                let _ = task_source.queue_with_canceller(
                    task!(resolve: move || {
                        let this = this.root();
                        let processed_audio = processed_audio.lock().unwrap();
                        let mut processed_audio: Vec<_> = processed_audio
                            .chunks(this.length as usize)
                            .map(|channel| channel.to_vec())
                            .collect();
                        // it can end up being empty if the task failed
                        if processed_audio.len() != this.length as usize {
                            processed_audio.resize(this.length as usize, Vec::new())
                        }
                        let buffer = AudioBuffer::new(
                            &this.global().as_window(),
                            this.channel_count,
                            this.length,
                            *this.context.SampleRate(),
                            Some(processed_audio.as_slice()));
                        (*this.pending_rendering_promise.borrow_mut()).take().unwrap().resolve_native(&buffer);
                        let global = &this.global();
                        let window = global.as_window();
                        let event = OfflineAudioCompletionEvent::new(&window,
                                                                     atom!("complete"),
                                                                     EventBubbles::DoesNotBubble,
                                                                     EventCancelable::NotCancelable,
                                                                     &buffer);
                        event.upcast::<Event>().fire(this.upcast());
                    }),
                    &canceller,
                );
            })
            .unwrap();

        if self.context.audio_context_impl().resume().is_err() {
            promise.reject_error(Error::Type("Could not start offline rendering".to_owned()));
        }

        promise
    }
}
