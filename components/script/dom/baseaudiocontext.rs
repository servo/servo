/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::audiodestinationnode::AudioDestinationNode;
use dom::bindings::codegen::Bindings::AudioNodeBinding::AudioNodeOptions;
use dom::bindings::codegen::Bindings::AudioNodeBinding::{ChannelCountMode, ChannelInterpretation};
use dom::bindings::codegen::Bindings::BaseAudioContextBinding::BaseAudioContextMethods;
use dom::bindings::codegen::Bindings::BaseAudioContextBinding::AudioContextState;
use dom::bindings::codegen::Bindings::OscillatorNodeBinding::OscillatorOptions;
use dom::bindings::num::Finite;
use dom::bindings::reflector::{DomObject, Reflector};
use dom::bindings::root::DomRoot;
use dom::globalscope::GlobalScope;
use dom::promise::Promise;
use dom::oscillatornode::OscillatorNode;
use dom_struct::dom_struct;
use std::rc::Rc;

#[dom_struct]
pub struct BaseAudioContext {
    reflector_: Reflector,
    destination: Option<DomRoot<AudioDestinationNode>>,
    sample_rate: f32,
    current_time: f64,
    state: AudioContextState,
}

impl BaseAudioContext {
    #[allow(unrooted_must_root)]
    #[allow(unsafe_code)]
    pub fn new_inherited(
        global: &GlobalScope,
        channel_count: u32,
        sample_rate: f32,
    ) -> BaseAudioContext {
        let mut context = BaseAudioContext {
            reflector_: Reflector::new(),
            destination: None,
            current_time: 0.,
            sample_rate,
            state: AudioContextState::Suspended,
        };

        let mut options = unsafe { AudioNodeOptions::empty(global.get_cx()) };
        options.channelCount = Some(channel_count);
        options.channelCountMode = Some(ChannelCountMode::Explicit);
        options.channelInterpretation = Some(ChannelInterpretation::Speakers);

        context.destination = Some(AudioDestinationNode::new(global, &context, &options));

        context
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
