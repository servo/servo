/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::audiobuffer::AudioBuffer;
use dom::audiobuffersourcenode::AudioBufferSourceNode;
use dom::audiodestinationnode::AudioDestinationNode;
use dom::audionode::MAX_CHANNEL_COUNT;
use dom::bindings::callback::ExceptionHandling;
use dom::bindings::cell::DomRefCell;
use dom::bindings::codegen::Bindings::AudioBufferSourceNodeBinding::AudioBufferSourceOptions;
use dom::bindings::codegen::Bindings::AudioNodeBinding::{ChannelCountMode, ChannelInterpretation};
use dom::bindings::codegen::Bindings::AudioNodeBinding::AudioNodeOptions;
use dom::bindings::codegen::Bindings::BaseAudioContextBinding::AudioContextState;
use dom::bindings::codegen::Bindings::BaseAudioContextBinding::BaseAudioContextMethods;
use dom::bindings::codegen::Bindings::BaseAudioContextBinding::DecodeErrorCallback;
use dom::bindings::codegen::Bindings::BaseAudioContextBinding::DecodeSuccessCallback;
use dom::bindings::codegen::Bindings::GainNodeBinding::GainOptions;
use dom::bindings::codegen::Bindings::OscillatorNodeBinding::OscillatorOptions;
use dom::bindings::error::{Error, ErrorResult, Fallible};
use dom::bindings::inheritance::Castable;
use dom::bindings::num::Finite;
use dom::bindings::refcounted::Trusted;
use dom::bindings::reflector::DomObject;
use dom::bindings::root::{DomRoot, MutNullableDom};
use dom::domexception::{DOMErrorName, DOMException};
use dom::eventtarget::EventTarget;
use dom::gainnode::GainNode;
use dom::oscillatornode::OscillatorNode;
use dom::promise::Promise;
use dom::window::Window;
use dom_struct::dom_struct;
use js::rust::CustomAutoRooterGuard;
use js::typedarray::ArrayBuffer;
use servo_media::{Backend, ServoMedia};
use servo_media::audio::context::{AudioContext, AudioContextOptions, ProcessingState};
use servo_media::audio::context::{OfflineAudioContextOptions, RealTimeAudioContextOptions};
use servo_media::audio::decoder::AudioDecoderCallbacks;
use servo_media::audio::graph::NodeId;
use std::cell::Cell;
use std::collections::{HashMap, VecDeque};
use std::mem;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use task_source::{TaskSource, TaskSourceName};
use typeholder::TypeHolderTrait;
use uuid::Uuid;

#[allow(dead_code)]
pub enum BaseAudioContextOptions {
    AudioContext(RealTimeAudioContextOptions),
    OfflineAudioContext(OfflineAudioContextOptions),
}

#[must_root]
#[derive(JSTraceable)]
struct DecodeResolver<TH: TypeHolderTrait> {
    pub promise: Rc<Promise<TH>>,
    pub success_callback: Option<Rc<DecodeSuccessCallback<TH>>>,
    pub error_callback: Option<Rc<DecodeErrorCallback<TH>>>,
}

#[dom_struct]
pub struct BaseAudioContext<TH: TypeHolderTrait> {
    eventtarget: EventTarget<TH>,
    #[ignore_malloc_size_of = "servo_media"]
    audio_context_impl: Rc<AudioContext<Backend>>,
    /// https://webaudio.github.io/web-audio-api/#dom-baseaudiocontext-destination
    destination: MutNullableDom<AudioDestinationNode<TH>>,
    /// Resume promises which are soon to be fulfilled by a queued task.
    #[ignore_malloc_size_of = "promises are hard"]
    in_flight_resume_promises_queue: DomRefCell<VecDeque<(Box<[Rc<Promise<TH>>]>, ErrorResult)>>,
    /// https://webaudio.github.io/web-audio-api/#pendingresumepromises
    #[ignore_malloc_size_of = "promises are hard"]
    pending_resume_promises: DomRefCell<Vec<Rc<Promise<TH>>>>,
    #[ignore_malloc_size_of = "promises are hard"]
    decode_resolvers: DomRefCell<HashMap<String, DecodeResolver<TH>>>,
    /// https://webaudio.github.io/web-audio-api/#dom-baseaudiocontext-samplerate
    sample_rate: f32,
    /// https://webaudio.github.io/web-audio-api/#dom-baseaudiocontext-state
    /// Although servo-media already keeps track of the control thread state,
    /// we keep a state flag here as well. This is so that we can synchronously
    /// throw when trying to do things on the context when the context has just
    /// been "closed()".
    state: Cell<AudioContextState>,
    channel_count: u32,
}

impl<TH: TypeHolderTrait> BaseAudioContext<TH> {
    #[allow(unrooted_must_root)]
    pub fn new_inherited(options: BaseAudioContextOptions) -> BaseAudioContext<TH> {
        let (sample_rate, channel_count) = match options {
            BaseAudioContextOptions::AudioContext(ref opt) => (opt.sample_rate, 2),
            BaseAudioContextOptions::OfflineAudioContext(ref opt) => (opt.sample_rate, opt.channels),
        };

        let context = BaseAudioContext {
            eventtarget: EventTarget::new_inherited(),
            audio_context_impl: Rc::new(
                ServoMedia::get()
                    .unwrap()
                    .create_audio_context(options.into()),
            ),
            destination: Default::default(),
            in_flight_resume_promises_queue: Default::default(),
            pending_resume_promises: Default::default(),
            decode_resolvers: Default::default(),
            sample_rate,
            state: Cell::new(AudioContextState::Suspended),
            channel_count: channel_count.into(),
        };

        context
    }

    /// Tells whether this is an OfflineAudioContext or not.
    pub fn is_offline(&self) -> bool {
        false
    }

    pub fn audio_context_impl(&self) -> Rc<AudioContext<Backend>> {
        self.audio_context_impl.clone()
    }

    pub fn destination_node(&self) -> NodeId {
        self.audio_context_impl.dest_node()
    }

    // https://webaudio.github.io/web-audio-api/#allowed-to-start
    pub fn is_allowed_to_start(&self) -> bool {
        self.state.get() == AudioContextState::Suspended
    }

    #[allow(unrooted_must_root)]
    fn push_pending_resume_promise(&self, promise: &Rc<Promise<TH>>) {
        self.pending_resume_promises
            .borrow_mut()
            .push(promise.clone());
    }

    /// Takes the pending resume promises.
    ///
    /// The result with which these promises will be fulfilled is passed here
    /// and this method returns nothing because we actually just move the
    /// current list of pending resume promises to the
    /// `in_flight_resume_promises_queue` field.
    ///
    /// Each call to this method must be followed by a call to
    /// `fulfill_in_flight_resume_promises`, to actually fulfill the promises
    /// which were taken and moved to the in-flight queue.
    #[allow(unrooted_must_root)]
    fn take_pending_resume_promises(&self, result: ErrorResult) {
        let pending_resume_promises =
            mem::replace(&mut *self.pending_resume_promises.borrow_mut(), vec![]);
        self.in_flight_resume_promises_queue
            .borrow_mut()
            .push_back((pending_resume_promises.into(), result));
    }

    /// Fulfills the next in-flight resume promises queue after running a closure.
    ///
    /// See the comment on `take_pending_resume_promises` for why this method
    /// does not take a list of promises to fulfill. Callers cannot just pop
    /// the front list off of `in_flight_resume_promises_queue` and later fulfill
    /// the promises because that would mean putting
    /// `#[allow(unrooted_must_root)]` on even more functions, potentially
    /// hiding actual safety bugs.
    #[allow(unrooted_must_root)]
    fn fulfill_in_flight_resume_promises<F>(&self, f: F)
    where
        F: FnOnce(),
    {
        let (promises, result) = self
            .in_flight_resume_promises_queue
            .borrow_mut()
            .pop_front()
            .expect("there should be at least one list of in flight resume promises");
        f();
        for promise in &*promises {
            match result {
                Ok(ref value) => promise.resolve_native(value),
                Err(ref error) => promise.reject_error(error.clone()),
            }
        }
    }

    /// Control thread processing state
    pub fn control_thread_state(&self) -> ProcessingState {
        self.audio_context_impl.state()
    }

    /// Set audio context state
    pub fn set_state_attribute(&self, state: AudioContextState) {
        self.state.set(state);
    }

    pub fn resume(&self) {
        let global = self.global();
        let window = global.as_window();
        let task_source = window.dom_manipulation_task_source();
        let this = Trusted::new(self);
        // Set the rendering thread state to 'running' and start
        // rendering the audio graph.
        match self.audio_context_impl.resume() {
            Ok(()) => {
                self.take_pending_resume_promises(Ok(()));
                let _ = task_source.queue(
                    task!(resume_success: move || {
                        let this = this.root();
                        this.fulfill_in_flight_resume_promises(|| {
                            if this.state.get() != AudioContextState::Running {
                                this.state.set(AudioContextState::Running);
                                let window = DomRoot::downcast::<Window<TH>>(this.global()).unwrap();
                                window.dom_manipulation_task_source().queue_simple_event(
                                    this.upcast(),
                                    atom!("statechange"),
                                    &window
                                    );
                            }
                        });
                    }),
                    window.upcast(),
                );
            },
            Err(()) => {
                self.take_pending_resume_promises(Err(Error::Type(
                    "Something went wrong".to_owned(),
                )));
                let _ = task_source.queue(
                    task!(resume_error: move || {
                    this.root().fulfill_in_flight_resume_promises(|| {})
                }),
                    window.upcast(),
                );
            },
        }
    }
}

impl<TH: TypeHolderTrait> BaseAudioContextMethods<TH> for BaseAudioContext<TH> {
    /// https://webaudio.github.io/web-audio-api/#dom-baseaudiocontext-samplerate
    fn SampleRate(&self) -> Finite<f32> {
        Finite::wrap(self.sample_rate)
    }

    /// https://webaudio.github.io/web-audio-api/#dom-baseaudiocontext-currenttime
    fn CurrentTime(&self) -> Finite<f64> {
        let current_time = self.audio_context_impl.current_time();
        Finite::wrap(current_time)
    }

    /// https://webaudio.github.io/web-audio-api/#dom-baseaudiocontext-state
    fn State(&self) -> AudioContextState {
        self.state.get()
    }

    /// https://webaudio.github.io/web-audio-api/#dom-baseaudiocontext-resume
    #[allow(unrooted_must_root)]
    fn Resume(&self) -> Rc<Promise<TH>> {
        // Step 1.
        let promise = Promise::new(&self.global());

        // Step 2.
        if self.audio_context_impl.state() == ProcessingState::Closed {
            promise.reject_error(Error::InvalidState);
            return promise;
        }

        // Step 3.
        if self.state.get() == AudioContextState::Running {
            promise.resolve_native(&());
            return promise;
        }

        self.push_pending_resume_promise(&promise);

        // Step 4.
        if !self.is_allowed_to_start() {
            return promise;
        }

        // Steps 5 and 6.
        self.resume();

        // Step 7.
        promise
    }

    /// https://webaudio.github.io/web-audio-api/#dom-baseaudiocontext-destination
    fn Destination(&self) -> DomRoot<AudioDestinationNode<TH>> {
        let global = self.global();
        self.destination.or_init(|| {
            let mut options = AudioNodeOptions::empty();
            options.channelCount = Some(self.channel_count);
            options.channelCountMode = Some(ChannelCountMode::Explicit);
            options.channelInterpretation = Some(ChannelInterpretation::Speakers);
            AudioDestinationNode::new(&global, self, &options)
        })
    }

    /// https://webaudio.github.io/web-audio-api/#dom-baseaudiocontext-onstatechange
    event_handler!(statechange, GetOnstatechange, SetOnstatechange);

    /// https://webaudio.github.io/web-audio-api/#dom-baseaudiocontext-createoscillator
    fn CreateOscillator(&self) -> DomRoot<OscillatorNode<TH>> {
        OscillatorNode::new(
            &self.global().as_window(),
            &self,
            &OscillatorOptions::empty(),
        )
    }

    /// https://webaudio.github.io/web-audio-api/#dom-baseaudiocontext-creategain
    fn CreateGain(&self) -> DomRoot<GainNode<TH>> {
        GainNode::new(&self.global().as_window(), &self, &GainOptions::empty())
    }

    /// https://webaudio.github.io/web-audio-api/#dom-baseaudiocontext-createbuffer
    fn CreateBuffer(
        &self,
        number_of_channels: u32,
        length: u32,
        sample_rate: Finite<f32>,
    ) -> Fallible<DomRoot<AudioBuffer<TH>>> {
        if number_of_channels <= 0 ||
            number_of_channels > MAX_CHANNEL_COUNT ||
            length <= 0 ||
            *sample_rate <= 0.
        {
            return Err(Error::NotSupported);
        }
        Ok(AudioBuffer::new(
            &self.global().as_window(),
            number_of_channels,
            length,
            *sample_rate,
            None,
        ))
    }

    // https://webaudio.github.io/web-audio-api/#dom-baseaudiocontext-createbuffersource
    fn CreateBufferSource(&self) -> DomRoot<AudioBufferSourceNode<TH>> {
        AudioBufferSourceNode::new(
            &self.global().as_window(),
            &self,
            &AudioBufferSourceOptions::empty(),
        )
    }

    // https://webaudio.github.io/web-audio-api/#dom-baseaudiocontext-decodeaudiodata
    #[allow(unrooted_must_root)]
    fn DecodeAudioData(
        &self,
        audio_data: CustomAutoRooterGuard<ArrayBuffer>,
        decode_success_callback: Option<Rc<DecodeSuccessCallback<TH>>>,
        decode_error_callback: Option<Rc<DecodeErrorCallback<TH>>>,
    ) -> Rc<Promise<TH>> {
        // Step 1.
        let promise = Promise::new(&self.global());
        let global = self.global();
        let window = global.as_window();

        if audio_data.len() > 0 {
            // Step 2.
            // XXX detach array buffer.
            let uuid = Uuid::new_v4().simple().to_string();
            let uuid_ = uuid.clone();
            self.decode_resolvers.borrow_mut().insert(
                uuid.clone(),
                DecodeResolver {
                    promise: promise.clone(),
                    success_callback: decode_success_callback,
                    error_callback: decode_error_callback,
                },
            );
            let audio_data = audio_data.to_vec();
            let decoded_audio = Arc::new(Mutex::new(Vec::new()));
            let decoded_audio_ = decoded_audio.clone();
            let this = Trusted::new(self);
            let this_ = this.clone();
            let task_source = window.dom_manipulation_task_source();
            let task_source_ = window.dom_manipulation_task_source();
            let canceller = window.task_canceller(TaskSourceName::DOMManipulation);
            let canceller_ = window.task_canceller(TaskSourceName::DOMManipulation);
            let callbacks = AudioDecoderCallbacks::new()
                .eos(move || {
                    let _ = task_source.queue_with_canceller(
                        task!(audio_decode_eos: move || {
                            let this = this.root();
                            let decoded_audio = decoded_audio.lock().unwrap();
                            let buffer = AudioBuffer::new(
                                &this.global().as_window(),
                                1, // XXX servo-media should provide this info
                                decoded_audio.len() as u32,
                                this.sample_rate,
                                Some(decoded_audio.as_slice()));
                            let mut resolvers = this.decode_resolvers.borrow_mut();
                            assert!(resolvers.contains_key(&uuid_));
                            let resolver = resolvers.remove(&uuid_).unwrap();
                            if let Some(callback) = resolver.success_callback {
                                let _ = callback.Call__(&buffer, ExceptionHandling::Report);
                            }
                            resolver.promise.resolve_native(&buffer);
                        }),
                        &canceller,
                    );
                })
                .error(move || {
                    let _ = task_source_.queue_with_canceller(
                        task!(audio_decode_eos: move || {
                        let this = this_.root();
                        let mut resolvers = this.decode_resolvers.borrow_mut();
                        assert!(resolvers.contains_key(&uuid));
                        let resolver = resolvers.remove(&uuid).unwrap();
                        if let Some(callback) = resolver.error_callback {
                            let _ = callback.Call__(
                                &DOMException::new(&this.global(), DOMErrorName::DataCloneError),
                                ExceptionHandling::Report);
                        }
                        resolver.promise.reject_error(Error::Type("Audio decode error".to_owned()));
                    }),
                        &canceller_,
                    );
                })
                .progress(move |buffer| {
                    decoded_audio_
                        .lock()
                        .unwrap()
                        .extend_from_slice((*buffer).as_ref());
                })
                .build();
            self.audio_context_impl
                .decode_audio_data(audio_data, callbacks);
        } else {
            // Step 3.
            promise.reject_error(Error::DataClone);
            return promise;
        }

        // Step 4.
        promise
    }
}

impl From<BaseAudioContextOptions> for AudioContextOptions {
    fn from(options: BaseAudioContextOptions) -> Self {
        match options {
            BaseAudioContextOptions::AudioContext(options) =>
                AudioContextOptions::RealTimeAudioContext(options),
            BaseAudioContextOptions::OfflineAudioContext(options) =>
                AudioContextOptions::OfflineAudioContext(options),
        }
    }
}

impl From<ProcessingState> for AudioContextState {
    fn from(state: ProcessingState) -> Self {
        match state {
            ProcessingState::Suspended => AudioContextState::Suspended,
            ProcessingState::Running => AudioContextState::Running,
            ProcessingState::Closed => AudioContextState::Closed,
        }
    }
}
