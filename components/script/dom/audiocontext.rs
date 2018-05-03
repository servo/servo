/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::baseaudiocontext::{BaseAudioContext, BaseAudioContextOptions};
use dom::bindings::codegen::Bindings::AudioContextBinding;
use dom::bindings::codegen::Bindings::AudioContextBinding::{AudioContextLatencyCategory, AudioContextMethods};
use dom::bindings::codegen::Bindings::AudioContextBinding::{AudioContextOptions, AudioTimestamp};
use dom::bindings::codegen::Bindings::BaseAudioContextBinding::AudioContextState;
use dom::bindings::codegen::Bindings::BaseAudioContextBinding::BaseAudioContextBinding::BaseAudioContextMethods;
use dom::bindings::error::{Error, Fallible};
use dom::bindings::inheritance::Castable;
use dom::bindings::num::Finite;
use dom::bindings::refcounted::{Trusted, TrustedPromise};
use dom::bindings::reflector::{DomObject, reflect_dom_object};
use dom::bindings::root::DomRoot;
use dom::promise::Promise;
use dom::window::Window;
use dom_struct::dom_struct;
use servo_media::audio::context::{LatencyCategory, ProcessingState, RealTimeAudioContextOptions};
use std::rc::Rc;
use task_source::TaskSource;
use typeholder::TypeHolderTrait;

#[dom_struct]
pub struct AudioContext<TH: TypeHolderTrait> {
    context: BaseAudioContext<TH>,
    latency_hint: AudioContextLatencyCategory,
    /// https://webaudio.github.io/web-audio-api/#dom-audiocontext-baselatency
    base_latency: f64,
    /// https://webaudio.github.io/web-audio-api/#dom-audiocontext-outputlatency
    output_latency: f64,
}

impl<TH: TypeHolderTrait> AudioContext<TH> {
    #[allow(unrooted_must_root)]
    // https://webaudio.github.io/web-audio-api/#AudioContext-constructors
    fn new_inherited(options: &AudioContextOptions) -> AudioContext<TH> {
        // Steps 1-3.
        let context = BaseAudioContext::new_inherited(
            BaseAudioContextOptions::AudioContext(options.into()),
        );

        // Step 4.1.
        let latency_hint = options.latencyHint;

        // Step 4.2. The sample rate is set during the creation of the BaseAudioContext.
        // servo-media takes care of setting the default sample rate of the output device
        // and of resampling the audio output if needed.

        // Steps 5 and 6 of the construction algorithm will happen in `resume`,
        // after reflecting dom object.

        AudioContext {
            context,
            latency_hint,
            base_latency: 0.,   // TODO
            output_latency: 0., // TODO
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(window: &Window<TH>, options: &AudioContextOptions) -> DomRoot<AudioContext<TH>> {
        let context = AudioContext::new_inherited(options);
        let context = reflect_dom_object(Box::new(context), window, AudioContextBinding::Wrap);
        context.resume();
        context
    }

    // https://webaudio.github.io/web-audio-api/#AudioContext-constructors
    pub fn Constructor(
        window: &Window<TH>,
        options: &AudioContextOptions,
    ) -> Fallible<DomRoot<AudioContext<TH>>> {
        Ok(AudioContext::new(window, options))
    }

    fn resume(&self) {
        // Step 5.
        if self.context.is_allowed_to_start() {
            // Step 6.
            self.context.resume();
        }
    }
}

impl<TH: TypeHolderTrait> AudioContextMethods<TH> for AudioContext<TH> {
    // https://webaudio.github.io/web-audio-api/#dom-audiocontext-baselatency
    fn BaseLatency(&self) -> Finite<f64> {
        Finite::wrap(self.base_latency)
    }

    // https://webaudio.github.io/web-audio-api/#dom-audiocontext-outputlatency
    fn OutputLatency(&self) -> Finite<f64> {
        Finite::wrap(self.output_latency)
    }

    // https://webaudio.github.io/web-audio-api/#dom-audiocontext-outputlatency
    fn GetOutputTimestamp(&self) -> AudioTimestamp {
        // TODO
        AudioTimestamp {
            contextTime: Some(Finite::wrap(0.)),
            performanceTime: Some(Finite::wrap(0.)),
        }
    }

    // https://webaudio.github.io/web-audio-api/#dom-audiocontext-suspend
    #[allow(unrooted_must_root)]
    fn Suspend(&self) -> Rc<Promise<TH>> {
        // Step 1.
        let promise = Promise::new(&self.global());

        // Step 2.
        if self.context.control_thread_state() == ProcessingState::Closed {
            promise.reject_error(Error::InvalidState);
            return promise;
        }

        // Step 3.
        if self.context.State() == AudioContextState::Suspended {
            promise.resolve_native(&());
            return promise;
        }

        // Steps 4 and 5.
        let window = DomRoot::downcast::<Window<TH>>(self.global()).unwrap();
        let task_source = window.dom_manipulation_task_source();
        let trusted_promise = TrustedPromise::new(promise.clone());
        match self.context.audio_context_impl().suspend() {
            Ok(_) => {
                let base_context = Trusted::new(&self.context);
                let context = Trusted::new(self);
                let _ = task_source.queue(
                    task!(suspend_ok: move || {
                    let base_context = base_context.root();
                    let context = context.root();
                    let promise = trusted_promise.root();
                    promise.resolve_native(&());
                    if base_context.State() != AudioContextState::Suspended {
                        base_context.set_state_attribute(AudioContextState::Suspended);
                        let window = DomRoot::downcast::<Window<TH>>(context.global()).unwrap();
                        window.dom_manipulation_task_source().queue_simple_event(
                            context.upcast(),
                            atom!("statechange"),
                            &window
                            );
                    }
                }),
                    window.upcast(),
                );
            },
            Err(_) => {
                // The spec does not define the error case and `suspend` should
                // never fail, but we handle the case here for completion.
                let _ = task_source.queue(
                    task!(suspend_error: move || {
                    let promise = trusted_promise.root();
                    promise.reject_error(Error::Type("Something went wrong".to_owned()));
                }),
                    window.upcast(),
                );
            },
        };

        // Step 6.
        promise
    }

    // https://webaudio.github.io/web-audio-api/#dom-audiocontext-close
    #[allow(unrooted_must_root)]
    fn Close(&self) -> Rc<Promise<TH>> {
        // Step 1.
        let promise = Promise::new(&self.global());

        // Step 2.
        if self.context.control_thread_state() == ProcessingState::Closed {
            promise.reject_error(Error::InvalidState);
            return promise;
        }

        // Step 3.
        if self.context.State() == AudioContextState::Closed {
            promise.resolve_native(&());
            return promise;
        }

        // Steps 4 and 5.
        let window = DomRoot::downcast::<Window<TH>>(self.global()).unwrap();
        let task_source = window.dom_manipulation_task_source();
        let trusted_promise = TrustedPromise::new(promise.clone());
        match self.context.audio_context_impl().close() {
            Ok(_) => {
                let base_context = Trusted::new(&self.context);
                let context = Trusted::new(self);
                let _ = task_source.queue(
                    task!(suspend_ok: move || {
                    let base_context = base_context.root();
                    let context = context.root();
                    let promise = trusted_promise.root();
                    promise.resolve_native(&());
                    if base_context.State() != AudioContextState::Closed {
                        base_context.set_state_attribute(AudioContextState::Closed);
                        let window = DomRoot::downcast::<Window<TH>>(context.global()).unwrap();
                        window.dom_manipulation_task_source().queue_simple_event(
                            context.upcast(),
                            atom!("statechange"),
                            &window
                            );
                    }
                }),
                    window.upcast(),
                );
            },
            Err(_) => {
                // The spec does not define the error case and `suspend` should
                // never fail, but we handle the case here for completion.
                let _ = task_source.queue(
                    task!(suspend_error: move || {
                    let promise = trusted_promise.root();
                    promise.reject_error(Error::Type("Something went wrong".to_owned()));
                }),
                    window.upcast(),
                );
            },
        };

        // Step 6.
        promise
    }
}

impl From<AudioContextLatencyCategory> for LatencyCategory {
    fn from(category: AudioContextLatencyCategory) -> Self {
        match category {
            AudioContextLatencyCategory::Balanced => LatencyCategory::Balanced,
            AudioContextLatencyCategory::Interactive => LatencyCategory::Interactive,
            AudioContextLatencyCategory::Playback => LatencyCategory::Playback,
        }
    }
}

impl<'a> From<&'a AudioContextOptions> for RealTimeAudioContextOptions {
    fn from(options: &AudioContextOptions) -> Self {
        Self {
            sample_rate: *options.sampleRate.unwrap_or(Finite::wrap(44100.)),
            latency_hint: options.latencyHint.into(),
        }
    }
}
