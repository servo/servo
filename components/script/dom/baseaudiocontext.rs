/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::audiobuffer::AudioBuffer;
use dom::audiodestinationnode::AudioDestinationNode;
use dom::bindings::cell::DomRefCell;
use dom::bindings::codegen::Bindings::AudioNodeBinding::AudioNodeOptions;
use dom::bindings::codegen::Bindings::AudioNodeBinding::{ChannelCountMode, ChannelInterpretation};
use dom::bindings::codegen::Bindings::BaseAudioContextBinding::BaseAudioContextMethods;
use dom::bindings::codegen::Bindings::BaseAudioContextBinding::AudioContextState;
use dom::bindings::codegen::Bindings::GainNodeBinding::GainOptions;
use dom::bindings::codegen::Bindings::OscillatorNodeBinding::OscillatorOptions;
use dom::bindings::error::{Error, ErrorResult};
use dom::bindings::inheritance::Castable;
use dom::bindings::num::Finite;
use dom::bindings::refcounted::Trusted;
use dom::bindings::reflector::DomObject;
use dom::bindings::root::DomRoot;
use dom::eventtarget::EventTarget;
use dom::gainnode::GainNode;
use dom::globalscope::GlobalScope;
use dom::oscillatornode::OscillatorNode;
use dom::promise::Promise;
use dom::window::Window;
use dom_struct::dom_struct;
use servo_media::ServoMedia;
use servo_media::audio::context::{AudioContext, ProcessingState};
use servo_media::audio::context::{OfflineAudioContextOptions, RealTimeAudioContextOptions};
use servo_media::audio::graph::NodeId;
use std::cell::Cell;
use std::collections::VecDeque;
use std::mem;
use std::rc::Rc;
use task_source::TaskSource;

pub enum BaseAudioContextOptions {
    AudioContext(RealTimeAudioContextOptions),
    OfflineAudioContext(OfflineAudioContextOptions),
}

#[dom_struct]
pub struct BaseAudioContext {
    eventtarget: EventTarget,
    #[ignore_malloc_size_of = "servo_media"]
    audio_context_impl: Rc<AudioContext>,
    /// https://webaudio.github.io/web-audio-api/#dom-baseaudiocontext-destination
    destination: Option<DomRoot<AudioDestinationNode>>,
    /// Resume promises which are soon to be fulfilled by a queued task.
    #[ignore_malloc_size_of = "promises are hard"]
    in_flight_resume_promises_queue: DomRefCell<VecDeque<(Box<[Rc<Promise>]>, ErrorResult)>>,
    /// https://webaudio.github.io/web-audio-api/#pendingresumepromises
    #[ignore_malloc_size_of = "promises are hard"]
    pending_resume_promises: DomRefCell<Vec<Rc<Promise>>>,
    /// https://webaudio.github.io/web-audio-api/#dom-baseaudiocontext-samplerate
    sample_rate: f32,
    /// https://webaudio.github.io/web-audio-api/#dom-baseaudiocontext-state
    /// Although servo-media already keeps track of the control thread state,
    /// we keep a state flag here as well. This is so that we can synchronously
    /// throw when trying to do things on the context when the context has just
    /// been "closed()".
    state: Cell<AudioContextState>,
}

impl BaseAudioContext {
    #[allow(unrooted_must_root)]
    #[allow(unsafe_code)]
    pub fn new_inherited(
        global: &GlobalScope,
        options: BaseAudioContextOptions,
        ) -> BaseAudioContext {
        let options = match options {
            BaseAudioContextOptions::AudioContext(options) => options,
            BaseAudioContextOptions::OfflineAudioContext(_) => unimplemented!(),
        };

        let sample_rate = options.sample_rate;

        let mut context = BaseAudioContext {
            eventtarget: EventTarget::new_inherited(),
            audio_context_impl: Rc::new(ServoMedia::get().unwrap().create_audio_context(options.into())),
            destination: None,
            in_flight_resume_promises_queue: Default::default(),
            pending_resume_promises: Default::default(),
            sample_rate,
            state: Cell::new(AudioContextState::Suspended),
        };

        let mut options = unsafe { AudioNodeOptions::empty(global.get_cx()) };
        options.channelCount = Some(2);
        options.channelCountMode = Some(ChannelCountMode::Explicit);
        options.channelInterpretation = Some(ChannelInterpretation::Speakers);

        context.destination = Some(AudioDestinationNode::new(global, &context, &options));

        context
    }

    pub fn audio_context_impl(&self) -> Rc<AudioContext> {
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
    fn push_pending_resume_promise(&self, promise: &Rc<Promise>) {
        self.pending_resume_promises.borrow_mut().push(promise.clone());
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
        let pending_resume_promises = mem::replace(
            &mut *self.pending_resume_promises.borrow_mut(),
            vec![],
            );
        self.in_flight_resume_promises_queue.borrow_mut().push_back((
                pending_resume_promises.into(),
                result,
                ));
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
            let (promises, result) = self.in_flight_resume_promises_queue
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
        let window = DomRoot::downcast::<Window>(self.global()).unwrap();
        let task_source = window.dom_manipulation_task_source();
        let this = Trusted::new(self);
        // Set the rendering thread state to 'running' and start
        // rendering the audio graph.
        match self.audio_context_impl.resume() {
            Ok(()) => {
                self.take_pending_resume_promises(Ok(()));
                let _ = task_source.queue(task!(resume_success: move || {
                    let this = this.root();
                    this.fulfill_in_flight_resume_promises(|| {
                        if this.state.get() != AudioContextState::Running {
                            this.state.set(AudioContextState::Running);
                            let window = DomRoot::downcast::<Window>(this.global()).unwrap();
                            window.dom_manipulation_task_source().queue_simple_event(
                                this.upcast(),
                                atom!("statechange"),
                                &window
                                );
                        }
                    });
                }), window.upcast());
            },
            Err(()) => {
                self.take_pending_resume_promises(Err(Error::Type("Something went wrong".to_owned())));
                let _ = task_source.queue(task!(resume_error: move || {
                    this.root().fulfill_in_flight_resume_promises(|| {})
                }), window.upcast());
            }
        }
    }
}

impl BaseAudioContextMethods for BaseAudioContext {
    // https://webaudio.github.io/web-audio-api/#dom-baseaudiocontext-samplerate
    fn SampleRate(&self) -> Finite<f32> {
        Finite::wrap(self.sample_rate)
    }

    // https://webaudio.github.io/web-audio-api/#dom-baseaudiocontext-currenttime
    fn CurrentTime(&self) -> Finite<f64> {
        let current_time = self.audio_context_impl.current_time();
        Finite::wrap(current_time)
    }

    // https://webaudio.github.io/web-audio-api/#dom-baseaudiocontext-state
    fn State(&self) -> AudioContextState {
        self.state.get()
    }

    // https://webaudio.github.io/web-audio-api/#dom-baseaudiocontext-resume
    #[allow(unrooted_must_root)]
    fn Resume(&self) -> Rc<Promise> {
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

    // https://webaudio.github.io/web-audio-api/#dom-baseaudiocontext-destination
    fn Destination(&self) -> DomRoot<AudioDestinationNode> {
        DomRoot::from_ref(self.destination.as_ref().unwrap())
    }

    // https://webaudio.github.io/web-audio-api/#dom-baseaudiocontext-onstatechange
    event_handler!(statechange, GetOnstatechange, SetOnstatechange);

    #[allow(unsafe_code)]
    fn CreateOscillator(&self) -> DomRoot<OscillatorNode> {
        let global = self.global();
        let window = global.as_window();
        let options = unsafe { OscillatorOptions::empty(window.get_cx()) };
        OscillatorNode::new(&window, &self, &options)
    }

    #[allow(unsafe_code)]
    fn CreateGain(&self) -> DomRoot<GainNode> {
        let global = self.global();
        let window = global.as_window();
        let options = unsafe { GainOptions::empty(window.get_cx()) };
        GainNode::new(&window, &self, &options)
    }

    fn CreateBuffer(&self,
                    number_of_channels: u32,
                    length: u32,
                    sample_rate: Finite<f32>) -> DomRoot<AudioBuffer> {
        let global = self.global();
        AudioBuffer::new(&global.as_window(), number_of_channels, length, *sample_rate)
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
