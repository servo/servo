/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::audiobuffer::{AudioBuffer, MAX_SAMPLE_RATE, MIN_SAMPLE_RATE};
use dom::audionode::MAX_CHANNEL_COUNT;
use dom::baseaudiocontext::{BaseAudioContext, BaseAudioContextOptions};
use dom::bindings::cell::DomRefCell;
use dom::bindings::codegen::Bindings::BaseAudioContextBinding::BaseAudioContextBinding::BaseAudioContextMethods;
use dom::bindings::codegen::Bindings::OfflineAudioContextBinding;
use dom::bindings::codegen::Bindings::OfflineAudioContextBinding::OfflineAudioContextMethods;
use dom::bindings::codegen::Bindings::OfflineAudioContextBinding::OfflineAudioContextOptions;
use dom::bindings::error::{Error, Fallible};
use dom::bindings::inheritance::Castable;
use dom::bindings::num::Finite;
use dom::bindings::refcounted::Trusted;
use dom::bindings::reflector::{DomObject, reflect_dom_object};
use dom::bindings::root::DomRoot;
use dom::event::{Event, EventBubbles, EventCancelable};
use dom::offlineaudiocompletionevent::OfflineAudioCompletionEvent;
use dom::promise::Promise;
use dom::window::Window;
use dom_struct::dom_struct;
use servo_media::audio::context::OfflineAudioContextOptions as ServoMediaOfflineAudioContextOptions;
use std::cell::Cell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::sync::mpsc;
use std::thread::Builder;
use task_source::{TaskSource, TaskSourceName};

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
    fn new_inherited(options: &OfflineAudioContextOptions) -> OfflineAudioContext {
        let context = BaseAudioContext::new_inherited(
            BaseAudioContextOptions::OfflineAudioContext(options.into()),
        );
        OfflineAudioContext {
            context,
            channel_count: options.numberOfChannels,
            length: options.length,
            rendering_started: Cell::new(false),
            pending_rendering_promise: Default::default(),
        }
    }

    #[allow(unrooted_must_root)]
    fn new(window: &Window, options: &OfflineAudioContextOptions) -> DomRoot<OfflineAudioContext> {
        let context = OfflineAudioContext::new_inherited(options);
        reflect_dom_object(Box::new(context), window, OfflineAudioContextBinding::Wrap)
    }

    pub fn Constructor(
        window: &Window,
        options: &OfflineAudioContextOptions,
    ) -> Fallible<DomRoot<OfflineAudioContext>> {
        Ok(OfflineAudioContext::new(window, options))
    }

    pub fn Constructor_(
        window: &Window,
        number_of_channels: u32,
        length: u32,
        sample_rate: Finite<f32>,
    ) -> Fallible<DomRoot<OfflineAudioContext>> {
        if number_of_channels > MAX_CHANNEL_COUNT ||
            number_of_channels <= 0 ||
            length <= 0 ||
            *sample_rate < MIN_SAMPLE_RATE ||
            *sample_rate > MAX_SAMPLE_RATE
        {
            return Err(Error::NotSupported);
        }

        let mut options = OfflineAudioContextOptions::empty();
        options.numberOfChannels = number_of_channels;
        options.length = length;
        options.sampleRate = sample_rate;
        Ok(OfflineAudioContext::new(window, &options))
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
    #[allow(unrooted_must_root)]
    fn StartRendering(&self) -> Rc<Promise> {
        let promise = Promise::new(&self.global());
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
        let task_source = window.dom_manipulation_task_source();
        let canceller = window.task_canceller(TaskSourceName::DOMManipulation);
        Builder::new()
            .name("OfflineAudioContextResolver".to_owned())
            .spawn(move || {
                let _ = receiver.recv();
                let _ = task_source.queue_with_canceller(
                    task!(resolve: move || {
                        let this = this.root();
                        let processed_audio = processed_audio.lock().unwrap();
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

impl<'a> From<&'a OfflineAudioContextOptions> for ServoMediaOfflineAudioContextOptions {
    fn from(options: &OfflineAudioContextOptions) -> Self {
        Self {
            channels: options.numberOfChannels as u8,
            length: options.length as usize,
            sample_rate: *options.sampleRate,
        }
    }
}
