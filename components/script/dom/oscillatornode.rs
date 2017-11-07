/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v.2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::audioscheduledsourcenode::AudioScheduledSourceNode;
use dom::baseaudiocontext::BaseAudioContext;
use dom::bindings::codegen::Bindings::AudioNodeBinding::AudioNodeOptions;
use dom::bindings::codegen::Bindings::AudioNodeBinding::{ChannelCountMode, ChannelInterpretation};
use dom::bindings::codegen::Bindings::OscillatorNodeBinding;
use dom::bindings::codegen::Bindings::OscillatorNodeBinding::OscillatorOptions;
use dom::bindings::error::Fallible;
use dom::bindings::reflector::reflect_dom_object;
use dom::bindings::root::DomRoot;
use dom::window::Window;
use dom_struct::dom_struct;

#[dom_struct]
pub struct OscillatorNode {
    node: AudioScheduledSourceNode,
    //    oscillator_type: OscillatorType,
    //    frequency: AudioParam,
    //    detune: AudioParam,
}

impl OscillatorNode {
    #[allow(unrooted_must_root)]
    #[allow(unsafe_code)]
    pub fn new_inherited(
        window: &Window,
        context: &BaseAudioContext,
    ) -> OscillatorNode {
        let mut options = unsafe { AudioNodeOptions::empty(window.get_cx()) };
        options.channelCount = Some(2);
        options.channelCountMode = Some(ChannelCountMode::Max);
        options.channelInterpretation = Some(ChannelInterpretation::Speakers);
        OscillatorNode {
            node: AudioScheduledSourceNode::new_inherited(
                context,
                &options,
                0, /* inputs */
                1, /* outputs */
            ),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(
        window: &Window,
        context: &BaseAudioContext,
        _options: &OscillatorOptions,
    ) -> DomRoot<OscillatorNode> {
        let node = OscillatorNode::new_inherited(window, context);
        reflect_dom_object(Box::new(node), window, OscillatorNodeBinding::Wrap)
    }

    pub fn Constructor(
        window: &Window,
        context: &BaseAudioContext,
        options: &OscillatorOptions,
    ) -> Fallible<DomRoot<OscillatorNode>> {
        Ok(OscillatorNode::new(window, context, options))
    }
}

/*impl OscillatorNodeMethods for OscillatorNode {
    fn SetPeriodicWave(&self, periodic_wave: PeriodicWave) {
        // XXX
    }

    fn Type(&self) -> OscillatorType {
        self.oscillator_type
    }

    fn Frequency(&self) -> DomRoot<AudioParam> {
        DomRoot::from_ref(&self.frequency)
    }

    fn Detune(&self) -> DomRoot<AudioParam> {
        DomRoot::from_ref(&self.detune)
    }
}*/
