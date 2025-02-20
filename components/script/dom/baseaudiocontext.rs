/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::collections::hash_map::Entry;
use std::collections::{HashMap, VecDeque};
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use base::id::PipelineId;
use dom_struct::dom_struct;
use js::rust::CustomAutoRooterGuard;
use js::typedarray::ArrayBuffer;
use servo_media::audio::context::{
    AudioContext, AudioContextOptions, OfflineAudioContextOptions, ProcessingState,
    RealTimeAudioContextOptions,
};
use servo_media::audio::decoder::AudioDecoderCallbacks;
use servo_media::audio::graph::NodeId;
use servo_media::{ClientContextId, ServoMedia};
use uuid::Uuid;

use crate::conversions::Convert;
use crate::dom::analysernode::AnalyserNode;
use crate::dom::audiobuffer::AudioBuffer;
use crate::dom::audiobuffersourcenode::AudioBufferSourceNode;
use crate::dom::audiodestinationnode::AudioDestinationNode;
use crate::dom::audiolistener::AudioListener;
use crate::dom::audionode::MAX_CHANNEL_COUNT;
use crate::dom::bindings::callback::ExceptionHandling;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::AnalyserNodeBinding::AnalyserOptions;
use crate::dom::bindings::codegen::Bindings::AudioBufferSourceNodeBinding::AudioBufferSourceOptions;
use crate::dom::bindings::codegen::Bindings::AudioNodeBinding::{
    AudioNodeOptions, ChannelCountMode, ChannelInterpretation,
};
use crate::dom::bindings::codegen::Bindings::BaseAudioContextBinding::{
    AudioContextState, BaseAudioContextMethods, DecodeErrorCallback, DecodeSuccessCallback,
};
use crate::dom::bindings::codegen::Bindings::BiquadFilterNodeBinding::BiquadFilterOptions;
use crate::dom::bindings::codegen::Bindings::ChannelMergerNodeBinding::ChannelMergerOptions;
use crate::dom::bindings::codegen::Bindings::ChannelSplitterNodeBinding::ChannelSplitterOptions;
use crate::dom::bindings::codegen::Bindings::ConstantSourceNodeBinding::ConstantSourceOptions;
use crate::dom::bindings::codegen::Bindings::GainNodeBinding::GainOptions;
use crate::dom::bindings::codegen::Bindings::IIRFilterNodeBinding::IIRFilterOptions;
use crate::dom::bindings::codegen::Bindings::OscillatorNodeBinding::OscillatorOptions;
use crate::dom::bindings::codegen::Bindings::PannerNodeBinding::PannerOptions;
use crate::dom::bindings::codegen::Bindings::StereoPannerNodeBinding::StereoPannerOptions;
use crate::dom::bindings::error::{Error, ErrorResult, Fallible};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::num::Finite;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::DomGlobal;
use crate::dom::bindings::root::{DomRoot, MutNullableDom};
use crate::dom::biquadfilternode::BiquadFilterNode;
use crate::dom::channelmergernode::ChannelMergerNode;
use crate::dom::channelsplitternode::ChannelSplitterNode;
use crate::dom::constantsourcenode::ConstantSourceNode;
use crate::dom::domexception::{DOMErrorName, DOMException};
use crate::dom::eventtarget::EventTarget;
use crate::dom::gainnode::GainNode;
use crate::dom::iirfilternode::IIRFilterNode;
use crate::dom::oscillatornode::OscillatorNode;
use crate::dom::pannernode::PannerNode;
use crate::dom::promise::Promise;
use crate::dom::stereopannernode::StereoPannerNode;
use crate::realms::InRealm;
use crate::script_runtime::CanGc;

#[allow(dead_code)]
pub(crate) enum BaseAudioContextOptions {
    AudioContext(RealTimeAudioContextOptions),
    OfflineAudioContext(OfflineAudioContextOptions),
}

#[derive(JSTraceable)]
struct DecodeResolver {
    pub(crate) promise: Rc<Promise>,
    pub(crate) success_callback: Option<Rc<DecodeSuccessCallback>>,
    pub(crate) error_callback: Option<Rc<DecodeErrorCallback>>,
}

type BoxedSliceOfPromises = Box<[Rc<Promise>]>;

#[dom_struct]
pub(crate) struct BaseAudioContext {
    eventtarget: EventTarget,
    #[ignore_malloc_size_of = "servo_media"]
    #[no_trace]
    audio_context_impl: Arc<Mutex<AudioContext>>,
    /// <https://webaudio.github.io/web-audio-api/#dom-baseaudiocontext-destination>
    destination: MutNullableDom<AudioDestinationNode>,
    listener: MutNullableDom<AudioListener>,
    /// Resume promises which are soon to be fulfilled by a queued task.
    #[ignore_malloc_size_of = "promises are hard"]
    in_flight_resume_promises_queue: DomRefCell<VecDeque<(BoxedSliceOfPromises, ErrorResult)>>,
    /// <https://webaudio.github.io/web-audio-api/#pendingresumepromises>
    #[ignore_malloc_size_of = "promises are hard"]
    pending_resume_promises: DomRefCell<Vec<Rc<Promise>>>,
    #[ignore_malloc_size_of = "promises are hard"]
    decode_resolvers: DomRefCell<HashMap<String, DecodeResolver>>,
    /// <https://webaudio.github.io/web-audio-api/#dom-baseaudiocontext-samplerate>
    sample_rate: f32,
    /// <https://webaudio.github.io/web-audio-api/#dom-baseaudiocontext-state>
    /// Although servo-media already keeps track of the control thread state,
    /// we keep a state flag here as well. This is so that we can synchronously
    /// throw when trying to do things on the context when the context has just
    /// been "closed()".
    state: Cell<AudioContextState>,
    channel_count: u32,
}

impl BaseAudioContext {
    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    pub(crate) fn new_inherited(
        options: BaseAudioContextOptions,
        pipeline_id: PipelineId,
    ) -> Fallible<BaseAudioContext> {
        let (sample_rate, channel_count) = match options {
            BaseAudioContextOptions::AudioContext(ref opt) => (opt.sample_rate, 2),
            BaseAudioContextOptions::OfflineAudioContext(ref opt) => {
                (opt.sample_rate, opt.channels)
            },
        };

        let client_context_id =
            ClientContextId::build(pipeline_id.namespace_id.0, pipeline_id.index.0.get());
        let audio_context_impl = ServoMedia::get()
            .create_audio_context(&client_context_id, options.convert())
            .map_err(|_| Error::NotSupported)?;

        Ok(BaseAudioContext {
            eventtarget: EventTarget::new_inherited(),
            audio_context_impl,
            destination: Default::default(),
            listener: Default::default(),
            in_flight_resume_promises_queue: Default::default(),
            pending_resume_promises: Default::default(),
            decode_resolvers: Default::default(),
            sample_rate,
            state: Cell::new(AudioContextState::Suspended),
            channel_count: channel_count.into(),
        })
    }

    /// Tells whether this is an OfflineAudioContext or not.
    pub(crate) fn is_offline(&self) -> bool {
        false
    }

    pub(crate) fn audio_context_impl(&self) -> Arc<Mutex<AudioContext>> {
        self.audio_context_impl.clone()
    }

    pub(crate) fn destination_node(&self) -> NodeId {
        self.audio_context_impl.lock().unwrap().dest_node()
    }

    pub(crate) fn listener(&self) -> NodeId {
        self.audio_context_impl.lock().unwrap().listener()
    }

    // https://webaudio.github.io/web-audio-api/#allowed-to-start
    pub(crate) fn is_allowed_to_start(&self) -> bool {
        self.state.get() == AudioContextState::Suspended
    }

    fn push_pending_resume_promise(&self, promise: &Rc<Promise>) {
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
    fn take_pending_resume_promises(&self, result: ErrorResult) {
        let pending_resume_promises =
            std::mem::take(&mut *self.pending_resume_promises.borrow_mut());
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
    /// `#[cfg_attr(crown, allow(crown::unrooted_must_root))]` on even more functions, potentially
    /// hiding actual safety bugs.
    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
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
    pub(crate) fn control_thread_state(&self) -> ProcessingState {
        self.audio_context_impl.lock().unwrap().state()
    }

    /// Set audio context state
    pub(crate) fn set_state_attribute(&self, state: AudioContextState) {
        self.state.set(state);
    }

    pub(crate) fn resume(&self) {
        let this = Trusted::new(self);
        // Set the rendering thread state to 'running' and start
        // rendering the audio graph.
        match self.audio_context_impl.lock().unwrap().resume() {
            Ok(()) => {
                self.take_pending_resume_promises(Ok(()));
                self.global().task_manager().dom_manipulation_task_source().queue(
                    task!(resume_success: move || {
                        let this = this.root();
                        this.fulfill_in_flight_resume_promises(|| {
                            if this.state.get() != AudioContextState::Running {
                                this.state.set(AudioContextState::Running);
                                this.global().task_manager().dom_manipulation_task_source().queue_simple_event(
                                    this.upcast(),
                                    atom!("statechange"),
                                    );
                            }
                        });
                    })
                );
            },
            Err(()) => {
                self.take_pending_resume_promises(Err(Error::Type(
                    "Something went wrong".to_owned(),
                )));
                self.global()
                    .task_manager()
                    .dom_manipulation_task_source()
                    .queue(task!(resume_error: move || {
                        this.root().fulfill_in_flight_resume_promises(|| {})
                    }));
            },
        }
    }

    pub(crate) fn channel_count(&self) -> u32 {
        self.channel_count
    }
}

impl BaseAudioContextMethods<crate::DomTypeHolder> for BaseAudioContext {
    /// <https://webaudio.github.io/web-audio-api/#dom-baseaudiocontext-samplerate>
    fn SampleRate(&self) -> Finite<f32> {
        Finite::wrap(self.sample_rate)
    }

    /// <https://webaudio.github.io/web-audio-api/#dom-baseaudiocontext-currenttime>
    fn CurrentTime(&self) -> Finite<f64> {
        let current_time = self.audio_context_impl.lock().unwrap().current_time();
        Finite::wrap(current_time)
    }

    /// <https://webaudio.github.io/web-audio-api/#dom-baseaudiocontext-state>
    fn State(&self) -> AudioContextState {
        self.state.get()
    }

    /// <https://webaudio.github.io/web-audio-api/#dom-baseaudiocontext-resume>
    fn Resume(&self, comp: InRealm, can_gc: CanGc) -> Rc<Promise> {
        // Step 1.
        let promise = Promise::new_in_current_realm(comp, can_gc);

        // Step 2.
        if self.audio_context_impl.lock().unwrap().state() == ProcessingState::Closed {
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

    /// <https://webaudio.github.io/web-audio-api/#dom-baseaudiocontext-destination>
    fn Destination(&self) -> DomRoot<AudioDestinationNode> {
        let global = self.global();
        self.destination.or_init(|| {
            let mut options = AudioNodeOptions::empty();
            options.channelCount = Some(self.channel_count);
            options.channelCountMode = Some(ChannelCountMode::Explicit);
            options.channelInterpretation = Some(ChannelInterpretation::Speakers);
            AudioDestinationNode::new(&global, self, &options, CanGc::note())
        })
    }

    /// <https://webaudio.github.io/web-audio-api/#dom-baseaudiocontext-listener>
    fn Listener(&self) -> DomRoot<AudioListener> {
        let global = self.global();
        let window = global.as_window();
        self.listener
            .or_init(|| AudioListener::new(window, self, CanGc::note()))
    }

    // https://webaudio.github.io/web-audio-api/#dom-baseaudiocontext-onstatechange
    event_handler!(statechange, GetOnstatechange, SetOnstatechange);

    /// <https://webaudio.github.io/web-audio-api/#dom-baseaudiocontext-createoscillator>
    fn CreateOscillator(&self, can_gc: CanGc) -> Fallible<DomRoot<OscillatorNode>> {
        OscillatorNode::new(
            self.global().as_window(),
            self,
            &OscillatorOptions::empty(),
            can_gc,
        )
    }

    /// <https://webaudio.github.io/web-audio-api/#dom-baseaudiocontext-creategain>
    fn CreateGain(&self, can_gc: CanGc) -> Fallible<DomRoot<GainNode>> {
        GainNode::new(
            self.global().as_window(),
            self,
            &GainOptions::empty(),
            can_gc,
        )
    }

    /// <https://webaudio.github.io/web-audio-api/#dom-baseaudiocontext-createpanner>
    fn CreatePanner(&self, can_gc: CanGc) -> Fallible<DomRoot<PannerNode>> {
        PannerNode::new(
            self.global().as_window(),
            self,
            &PannerOptions::empty(),
            can_gc,
        )
    }

    /// <https://webaudio.github.io/web-audio-api/#dom-baseaudiocontext-createanalyser>
    fn CreateAnalyser(&self, can_gc: CanGc) -> Fallible<DomRoot<AnalyserNode>> {
        AnalyserNode::new(
            self.global().as_window(),
            self,
            &AnalyserOptions::empty(),
            can_gc,
        )
    }

    /// <https://webaudio.github.io/web-audio-api/#dom-baseaudiocontext-createbiquadfilter>
    fn CreateBiquadFilter(&self, can_gc: CanGc) -> Fallible<DomRoot<BiquadFilterNode>> {
        BiquadFilterNode::new(
            self.global().as_window(),
            self,
            &BiquadFilterOptions::empty(),
            can_gc,
        )
    }

    /// <https://webaudio.github.io/web-audio-api/#dom-baseaudiocontext-createstereopanner>
    fn CreateStereoPanner(&self, can_gc: CanGc) -> Fallible<DomRoot<StereoPannerNode>> {
        StereoPannerNode::new(
            self.global().as_window(),
            self,
            &StereoPannerOptions::empty(),
            can_gc,
        )
    }

    /// <https://webaudio.github.io/web-audio-api/#dom-baseaudiocontext-createconstantsource>
    fn CreateConstantSource(&self, can_gc: CanGc) -> Fallible<DomRoot<ConstantSourceNode>> {
        ConstantSourceNode::new(
            self.global().as_window(),
            self,
            &ConstantSourceOptions::empty(),
            can_gc,
        )
    }

    /// <https://webaudio.github.io/web-audio-api/#dom-baseaudiocontext-createchannelmerger>
    fn CreateChannelMerger(
        &self,
        count: u32,
        can_gc: CanGc,
    ) -> Fallible<DomRoot<ChannelMergerNode>> {
        let mut opts = ChannelMergerOptions::empty();
        opts.numberOfInputs = count;
        ChannelMergerNode::new(self.global().as_window(), self, &opts, can_gc)
    }

    /// <https://webaudio.github.io/web-audio-api/#dom-baseaudiocontext-createchannelsplitter>
    fn CreateChannelSplitter(
        &self,
        count: u32,
        can_gc: CanGc,
    ) -> Fallible<DomRoot<ChannelSplitterNode>> {
        let mut opts = ChannelSplitterOptions::empty();
        opts.numberOfOutputs = count;
        ChannelSplitterNode::new(self.global().as_window(), self, &opts, can_gc)
    }

    /// <https://webaudio.github.io/web-audio-api/#dom-baseaudiocontext-createbuffer>
    fn CreateBuffer(
        &self,
        number_of_channels: u32,
        length: u32,
        sample_rate: Finite<f32>,
        can_gc: CanGc,
    ) -> Fallible<DomRoot<AudioBuffer>> {
        if number_of_channels == 0 ||
            number_of_channels > MAX_CHANNEL_COUNT ||
            length == 0 ||
            *sample_rate <= 0.
        {
            return Err(Error::NotSupported);
        }
        Ok(AudioBuffer::new(
            self.global().as_window(),
            number_of_channels,
            length,
            *sample_rate,
            None,
            can_gc,
        ))
    }

    // https://webaudio.github.io/web-audio-api/#dom-baseaudiocontext-createbuffersource
    fn CreateBufferSource(&self, can_gc: CanGc) -> Fallible<DomRoot<AudioBufferSourceNode>> {
        AudioBufferSourceNode::new(
            self.global().as_window(),
            self,
            &AudioBufferSourceOptions::empty(),
            can_gc,
        )
    }

    // https://webaudio.github.io/web-audio-api/#dom-baseaudiocontext-decodeaudiodata
    fn DecodeAudioData(
        &self,
        audio_data: CustomAutoRooterGuard<ArrayBuffer>,
        decode_success_callback: Option<Rc<DecodeSuccessCallback>>,
        decode_error_callback: Option<Rc<DecodeErrorCallback>>,
        comp: InRealm,
        can_gc: CanGc,
    ) -> Rc<Promise> {
        // Step 1.
        let promise = Promise::new_in_current_realm(comp, can_gc);

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
            let decoded_audio__ = decoded_audio.clone();
            // servo-media returns an audio channel position along
            // with the AudioDecoderCallback progress callback, which
            // may not be the same as the index of the decoded_audio
            // Vec.
            let channels = Arc::new(Mutex::new(HashMap::new()));
            let this = Trusted::new(self);
            let this_ = this.clone();
            let task_source = self
                .global()
                .task_manager()
                .dom_manipulation_task_source()
                .to_sendable();
            let task_source_clone = task_source.clone();
            let callbacks = AudioDecoderCallbacks::new()
                .ready(move |channel_count| {
                    decoded_audio
                        .lock()
                        .unwrap()
                        .resize(channel_count as usize, Vec::new());
                })
                .progress(move |buffer, channel_pos_mask| {
                    let mut decoded_audio = decoded_audio_.lock().unwrap();
                    let mut channels = channels.lock().unwrap();
                    let channel = match channels.entry(channel_pos_mask) {
                        Entry::Occupied(entry) => *entry.get(),
                        Entry::Vacant(entry) => {
                            let x = (channel_pos_mask as f32).log2() as usize;
                            *entry.insert(x)
                        },
                    };
                    decoded_audio[channel].extend_from_slice((*buffer).as_ref());
                })
                .eos(move || {
                    task_source.queue(task!(audio_decode_eos: move || {
                        let this = this.root();
                        let decoded_audio = decoded_audio__.lock().unwrap();
                        let length = if decoded_audio.len() >= 1 {
                            decoded_audio[0].len()
                        } else {
                            0
                        };
                        let buffer = AudioBuffer::new(
                            this.global().as_window(),
                            decoded_audio.len() as u32 /* number of channels */,
                            length as u32,
                            this.sample_rate,
                            Some(decoded_audio.as_slice()),
                            CanGc::note());
                        let mut resolvers = this.decode_resolvers.borrow_mut();
                        assert!(resolvers.contains_key(&uuid_));
                        let resolver = resolvers.remove(&uuid_).unwrap();
                        if let Some(callback) = resolver.success_callback {
                            let _ = callback.Call__(&buffer, ExceptionHandling::Report);
                        }
                        resolver.promise.resolve_native(&buffer);
                    }));
                })
                .error(move |error| {
                    task_source_clone.queue(task!(audio_decode_eos: move || {
                        let this = this_.root();
                        let mut resolvers = this.decode_resolvers.borrow_mut();
                        assert!(resolvers.contains_key(&uuid));
                        let resolver = resolvers.remove(&uuid).unwrap();
                        if let Some(callback) = resolver.error_callback {
                            let _ = callback.Call__(
                                &DOMException::new(&this.global(), DOMErrorName::DataCloneError, CanGc::note()),
                                ExceptionHandling::Report);
                        }
                        let error = format!("Audio decode error {:?}", error);
                        resolver.promise.reject_error(Error::Type(error));
                    }));
                })
                .build();
            self.audio_context_impl
                .lock()
                .unwrap()
                .decode_audio_data(audio_data, callbacks);
        } else {
            // Step 3.
            promise.reject_error(Error::DataClone);
            return promise;
        }

        // Step 4.
        promise
    }

    /// <https://webaudio.github.io/web-audio-api/#dom-baseaudiocontext-createiirfilter>
    fn CreateIIRFilter(
        &self,
        feedforward: Vec<Finite<f64>>,
        feedback: Vec<Finite<f64>>,
        can_gc: CanGc,
    ) -> Fallible<DomRoot<IIRFilterNode>> {
        let opts = IIRFilterOptions {
            parent: AudioNodeOptions::empty(),
            feedback,
            feedforward,
        };
        IIRFilterNode::new(self.global().as_window(), self, &opts, can_gc)
    }
}

impl Convert<AudioContextOptions> for BaseAudioContextOptions {
    fn convert(self) -> AudioContextOptions {
        match self {
            BaseAudioContextOptions::AudioContext(options) => {
                AudioContextOptions::RealTimeAudioContext(options)
            },
            BaseAudioContextOptions::OfflineAudioContext(options) => {
                AudioContextOptions::OfflineAudioContext(options)
            },
        }
    }
}

impl Convert<AudioContextState> for ProcessingState {
    fn convert(self) -> AudioContextState {
        match self {
            ProcessingState::Suspended => AudioContextState::Suspended,
            ProcessingState::Running => AudioContextState::Running,
            ProcessingState::Closed => AudioContextState::Closed,
        }
    }
}
