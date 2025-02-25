/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::rc::Rc;

use base::id::PipelineId;
use dom_struct::dom_struct;
use js::rust::HandleObject;
use servo_media::audio::context::{LatencyCategory, ProcessingState, RealTimeAudioContextOptions};

use crate::conversions::Convert;
use crate::dom::baseaudiocontext::{BaseAudioContext, BaseAudioContextOptions};
use crate::dom::bindings::codegen::Bindings::AudioContextBinding::{
    AudioContextLatencyCategory, AudioContextMethods, AudioContextOptions, AudioTimestamp,
};
use crate::dom::bindings::codegen::Bindings::AudioNodeBinding::AudioNodeOptions;
use crate::dom::bindings::codegen::Bindings::BaseAudioContextBinding::AudioContextState;
use crate::dom::bindings::codegen::Bindings::BaseAudioContextBinding::BaseAudioContext_Binding::BaseAudioContextMethods;
use crate::dom::bindings::codegen::UnionTypes::AudioContextLatencyCategoryOrDouble;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::num::Finite;
use crate::dom::bindings::refcounted::{Trusted, TrustedPromise};
use crate::dom::bindings::reflector::{reflect_dom_object_with_proto, DomGlobal};
use crate::dom::bindings::root::DomRoot;
use crate::dom::htmlmediaelement::HTMLMediaElement;
use crate::dom::mediaelementaudiosourcenode::MediaElementAudioSourceNode;
use crate::dom::mediastream::MediaStream;
use crate::dom::mediastreamaudiodestinationnode::MediaStreamAudioDestinationNode;
use crate::dom::mediastreamaudiosourcenode::MediaStreamAudioSourceNode;
use crate::dom::mediastreamtrack::MediaStreamTrack;
use crate::dom::mediastreamtrackaudiosourcenode::MediaStreamTrackAudioSourceNode;
use crate::dom::promise::Promise;
use crate::dom::window::Window;
use crate::realms::InRealm;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct AudioContext {
    context: BaseAudioContext,
    latency_hint: AudioContextLatencyCategory,
    /// <https://webaudio.github.io/web-audio-api/#dom-audiocontext-baselatency>
    base_latency: f64,
    /// <https://webaudio.github.io/web-audio-api/#dom-audiocontext-outputlatency>
    output_latency: f64,
}

impl AudioContext {
    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    // https://webaudio.github.io/web-audio-api/#AudioContext-constructors
    fn new_inherited(
        options: &AudioContextOptions,
        pipeline_id: PipelineId,
    ) -> Fallible<AudioContext> {
        // Steps 1-3.
        let context = BaseAudioContext::new_inherited(
            BaseAudioContextOptions::AudioContext(options.convert()),
            pipeline_id,
        )?;

        // Step 4.1.
        let latency_hint = match options.latencyHint {
            AudioContextLatencyCategoryOrDouble::AudioContextLatencyCategory(category) => category,
            AudioContextLatencyCategoryOrDouble::Double(_) => {
                AudioContextLatencyCategory::Interactive
            }, // TODO
        };

        // Step 4.2. The sample rate is set during the creation of the BaseAudioContext.
        // servo-media takes care of setting the default sample rate of the output device
        // and of resampling the audio output if needed.

        // Steps 5 and 6 of the construction algorithm will happen in `resume`,
        // after reflecting dom object.

        Ok(AudioContext {
            context,
            latency_hint,
            base_latency: 0.,   // TODO
            output_latency: 0., // TODO
        })
    }

    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    fn new(
        window: &Window,
        proto: Option<HandleObject>,
        options: &AudioContextOptions,
        can_gc: CanGc,
    ) -> Fallible<DomRoot<AudioContext>> {
        let pipeline_id = window.pipeline_id();
        let context = AudioContext::new_inherited(options, pipeline_id)?;
        let context = reflect_dom_object_with_proto(Box::new(context), window, proto, can_gc);
        context.resume();
        Ok(context)
    }

    fn resume(&self) {
        // Step 5.
        if self.context.is_allowed_to_start() {
            // Step 6.
            self.context.resume();
        }
    }

    pub(crate) fn base(&self) -> DomRoot<BaseAudioContext> {
        DomRoot::from_ref(&self.context)
    }
}

impl AudioContextMethods<crate::DomTypeHolder> for AudioContext {
    // https://webaudio.github.io/web-audio-api/#AudioContext-constructors
    fn Constructor(
        window: &Window,
        proto: Option<HandleObject>,
        can_gc: CanGc,
        options: &AudioContextOptions,
    ) -> Fallible<DomRoot<AudioContext>> {
        AudioContext::new(window, proto, options, can_gc)
    }

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
    fn Suspend(&self, comp: InRealm, can_gc: CanGc) -> Rc<Promise> {
        // Step 1.
        let promise = Promise::new_in_current_realm(comp, can_gc);

        // Step 2.
        if self.context.control_thread_state() == ProcessingState::Closed {
            promise.reject_error(Error::InvalidState, can_gc);
            return promise;
        }

        // Step 3.
        if self.context.State() == AudioContextState::Suspended {
            promise.resolve_native(&(), can_gc);
            return promise;
        }

        // Steps 4 and 5.
        let trusted_promise = TrustedPromise::new(promise.clone());
        match self.context.audio_context_impl().lock().unwrap().suspend() {
            Ok(_) => {
                let base_context = Trusted::new(&self.context);
                let context = Trusted::new(self);
                self.global().task_manager().dom_manipulation_task_source().queue(
                    task!(suspend_ok: move || {
                        let base_context = base_context.root();
                        let context = context.root();
                        let promise = trusted_promise.root();
                        promise.resolve_native(&(), CanGc::note());
                        if base_context.State() != AudioContextState::Suspended {
                            base_context.set_state_attribute(AudioContextState::Suspended);
                            context.global().task_manager().dom_manipulation_task_source().queue_simple_event(
                                context.upcast(),
                                atom!("statechange"),
                            );
                        }
                    })
                );
            },
            Err(_) => {
                // The spec does not define the error case and `suspend` should
                // never fail, but we handle the case here for completion.
                self.global()
                    .task_manager()
                    .dom_manipulation_task_source()
                    .queue(task!(suspend_error: move || {
                        let promise = trusted_promise.root();
                        promise.reject_error(Error::Type("Something went wrong".to_owned()), CanGc::note());
                    }));
            },
        };

        // Step 6.
        promise
    }

    // https://webaudio.github.io/web-audio-api/#dom-audiocontext-close
    fn Close(&self, comp: InRealm, can_gc: CanGc) -> Rc<Promise> {
        // Step 1.
        let promise = Promise::new_in_current_realm(comp, can_gc);

        // Step 2.
        if self.context.control_thread_state() == ProcessingState::Closed {
            promise.reject_error(Error::InvalidState, can_gc);
            return promise;
        }

        // Step 3.
        if self.context.State() == AudioContextState::Closed {
            promise.resolve_native(&(), can_gc);
            return promise;
        }

        // Steps 4 and 5.
        let trusted_promise = TrustedPromise::new(promise.clone());
        match self.context.audio_context_impl().lock().unwrap().close() {
            Ok(_) => {
                let base_context = Trusted::new(&self.context);
                let context = Trusted::new(self);
                self.global().task_manager().dom_manipulation_task_source().queue(
                    task!(suspend_ok: move || {
                        let base_context = base_context.root();
                        let context = context.root();
                        let promise = trusted_promise.root();
                        promise.resolve_native(&(), CanGc::note());
                        if base_context.State() != AudioContextState::Closed {
                            base_context.set_state_attribute(AudioContextState::Closed);
                            context.global().task_manager().dom_manipulation_task_source().queue_simple_event(
                                context.upcast(),
                                atom!("statechange"),
                            );
                        }
                    })
                );
            },
            Err(_) => {
                // The spec does not define the error case and `suspend` should
                // never fail, but we handle the case here for completion.
                self.global()
                    .task_manager()
                    .dom_manipulation_task_source()
                    .queue(task!(suspend_error: move || {
                        let promise = trusted_promise.root();
                        promise.reject_error(Error::Type("Something went wrong".to_owned()), CanGc::note());
                    }));
            },
        };

        // Step 6.
        promise
    }

    /// <https://webaudio.github.io/web-audio-api/#dom-audiocontext-createmediaelementsource>
    fn CreateMediaElementSource(
        &self,
        media_element: &HTMLMediaElement,
        can_gc: CanGc,
    ) -> Fallible<DomRoot<MediaElementAudioSourceNode>> {
        let global = self.global();
        let window = global.as_window();
        MediaElementAudioSourceNode::new(window, self, media_element, can_gc)
    }

    /// <https://webaudio.github.io/web-audio-api/#dom-audiocontext-createmediastreamsource>
    fn CreateMediaStreamSource(
        &self,
        stream: &MediaStream,
        can_gc: CanGc,
    ) -> Fallible<DomRoot<MediaStreamAudioSourceNode>> {
        let global = self.global();
        let window = global.as_window();
        MediaStreamAudioSourceNode::new(window, self, stream, can_gc)
    }

    /// <https://webaudio.github.io/web-audio-api/#dom-audiocontext-createmediastreamtracksource>
    fn CreateMediaStreamTrackSource(
        &self,
        track: &MediaStreamTrack,
        can_gc: CanGc,
    ) -> Fallible<DomRoot<MediaStreamTrackAudioSourceNode>> {
        let global = self.global();
        let window = global.as_window();
        MediaStreamTrackAudioSourceNode::new(window, self, track, can_gc)
    }

    /// <https://webaudio.github.io/web-audio-api/#dom-audiocontext-createmediastreamdestination>
    fn CreateMediaStreamDestination(
        &self,
        can_gc: CanGc,
    ) -> Fallible<DomRoot<MediaStreamAudioDestinationNode>> {
        let global = self.global();
        let window = global.as_window();
        MediaStreamAudioDestinationNode::new(window, self, &AudioNodeOptions::empty(), can_gc)
    }
}

impl Convert<LatencyCategory> for AudioContextLatencyCategory {
    fn convert(self) -> LatencyCategory {
        match self {
            AudioContextLatencyCategory::Balanced => LatencyCategory::Balanced,
            AudioContextLatencyCategory::Interactive => LatencyCategory::Interactive,
            AudioContextLatencyCategory::Playback => LatencyCategory::Playback,
        }
    }
}

impl Convert<RealTimeAudioContextOptions> for &AudioContextOptions {
    fn convert(self) -> RealTimeAudioContextOptions {
        RealTimeAudioContextOptions {
            sample_rate: *self.sampleRate.unwrap_or(Finite::wrap(44100.)),
            latency_hint: match self.latencyHint {
                AudioContextLatencyCategoryOrDouble::AudioContextLatencyCategory(category) => {
                    category.convert()
                },
                AudioContextLatencyCategoryOrDouble::Double(_) => LatencyCategory::Interactive, // TODO
            },
        }
    }
}
