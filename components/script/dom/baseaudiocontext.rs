/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::audiodestinationnode::AudioDestinationNode;
use dom::bindings::cell::DomRefCell;
use dom::bindings::codegen::Bindings::AudioNodeBinding::AudioNodeOptions;
use dom::bindings::codegen::Bindings::AudioNodeBinding::{ChannelCountMode, ChannelInterpretation};
use dom::bindings::codegen::Bindings::BaseAudioContextBinding::BaseAudioContextMethods;
use dom::bindings::codegen::Bindings::BaseAudioContextBinding::AudioContextState;
use dom::bindings::codegen::Bindings::OscillatorNodeBinding::OscillatorOptions;
use dom::bindings::inheritance::Castable;
use dom::bindings::num::Finite;
use dom::bindings::refcounted::Trusted;
use dom::bindings::reflector::{DomObject, Reflector};
use dom::bindings::root::DomRoot;
use dom::globalscope::GlobalScope;
use dom::oscillatornode::OscillatorNode;
use dom::promise::Promise;
use dom::window::Window;
use dom_struct::dom_struct;
use servo_media::ServoMedia;
use servo_media::audio::graph::AudioGraph;
use servo_media::audio::graph::{OfflineAudioGraphOptions, RealTimeAudioGraphOptions};
use servo_media::audio::graph_impl::NodeId;
use servo_media::audio::node::AudioNodeType;
use std::cell::Cell;
use std::rc::Rc;
use task_source::TaskSource;

pub enum BaseAudioContextOptions {
    AudioContext(RealTimeAudioGraphOptions),
    OfflineAudioContext(OfflineAudioGraphOptions),
}

#[dom_struct]
pub struct BaseAudioContext {
    reflector_: Reflector,
    #[ignore_malloc_size_of = "servo_media"]
    audio_graph: AudioGraph,
    /// https://webaudio.github.io/web-audio-api/#dom-baseaudiocontext-destination
    destination: Option<DomRoot<AudioDestinationNode>>,
    /// https://webaudio.github.io/web-audio-api/#dom-baseaudiocontext-samplerate
    sample_rate: f32,
    /// https://webaudio.github.io/web-audio-api/#dom-baseaudiocontext-currenttime
    current_time: f64,
    /// https://webaudio.github.io/web-audio-api/#dom-baseaudiocontext-state
    state: Cell<AudioContextState>,
    /// https://webaudio.github.io/web-audio-api/#pendingresumepromises
    #[ignore_malloc_size_of = "promises are hard"]
    pending_resume_promises: DomRefCell<Vec<Rc<Promise>>>,
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
            reflector_: Reflector::new(),
            audio_graph: ServoMedia::get().unwrap().create_audio_graph(Some(options.into())),
            destination: None,
            current_time: 0.,
            sample_rate,
            state: Cell::new(AudioContextState::Suspended),
            pending_resume_promises: Default::default(),
        };

        let mut options = unsafe { AudioNodeOptions::empty(global.get_cx()) };
        options.channelCount = Some(2);
        options.channelCountMode = Some(ChannelCountMode::Explicit);
        options.channelInterpretation = Some(ChannelInterpretation::Speakers);

        context.destination = Some(AudioDestinationNode::new(global, &context, &options));

        context
    }

    pub fn create_node_engine(&self, node_type: AudioNodeType) -> NodeId {
        self.audio_graph.create_node(node_type)
    }

    // https://webaudio.github.io/web-audio-api/#allowed-to-start
    pub fn is_allowed_to_start(&self) -> bool {
        self.state.get() == AudioContextState::Suspended
    }

    pub fn resume(&self) {
        let window = DomRoot::downcast::<Window>(self.global()).unwrap();
        let task_source = window.dom_manipulation_task_source();

        // Set the state attribute to `running`.
        let this = Trusted::new(self);
        task_source.queue(task!(set_state: move || {
            let this = this.root();
            this.state.set(AudioContextState::Running);
        }), window.upcast()).unwrap();

        // Queue a task to fire a simple event named `statechange` at the AudioContext.
        task_source.queue_simple_event(
            self.upcast(),
            atom!("statechange"),
            &window,
            );
    }
}

impl BaseAudioContextMethods for BaseAudioContext {
    // https://webaudio.github.io/web-audio-api/#dom-baseaudiocontext-samplerate
    fn SampleRate(&self) -> Finite<f32> {
        Finite::wrap(self.sample_rate)
    }

    // https://webaudio.github.io/web-audio-api/#dom-baseaudiocontext-currenttime
    fn CurrentTime(&self) -> Finite<f64> {
        Finite::wrap(self.current_time)
    }

    // https://webaudio.github.io/web-audio-api/#dom-baseaudiocontext-state
    fn State(&self) -> AudioContextState {
        self.state.get()
    }

    // https://webaudio.github.io/web-audio-api/#dom-baseaudiocontext-resume
    #[allow(unrooted_must_root)]
    fn Resume(&self) -> Rc<Promise> {
        // TODO
        Promise::new(&self.global())
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
}
