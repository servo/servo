/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::sync::Arc;

use dom_struct::dom_struct;
use itertools::Itertools;
use js::context::JSContext;
use js::gc::CustomAutoRooterGuard;
use js::rust::HandleObject;
use js::typedarray::Float32Array;
use script_bindings::reflector::reflect_dom_object_with_proto;
use servo_media::audio::audio_node::AudioNodeInit;
use servo_media::audio::graph::NodeId;
use servo_media::audio::iir_filter_node::{IIRFilterNode as IIRFilter, IIRFilterNodeOptions};

use crate::conversions::Convert;
use crate::dom::audio::audionode::{AudioNode, AudioNodeOptionsHelper};
use crate::dom::audio::baseaudiocontext::BaseAudioContext;
use crate::dom::audionode::UnwrappedAudioNodeOptions;
use crate::dom::bindings::codegen::Bindings::AudioNodeBinding::{
    ChannelCountMode, ChannelInterpretation,
};
use crate::dom::bindings::codegen::Bindings::IIRFilterNodeBinding::{
    IIRFilterNodeMethods, IIRFilterOptions,
};
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::num::Finite;
use crate::dom::bindings::root::DomRoot;
use crate::dom::window::Window;

#[dom_struct]
pub(crate) struct IIRFilterNode {
    node: AudioNode,
    feedforward: Vec<Finite<f64>>,
    feedback: Vec<Finite<f64>>,
}

impl IIRFilterNode {
    fn new_inherited(
        node_id: Option<NodeId>,
        context: &BaseAudioContext,
        node_options: UnwrappedAudioNodeOptions,
        feedforward: Vec<Finite<f64>>,
        feedback: Vec<Finite<f64>>,
    ) -> Self {
        IIRFilterNode {
            node: AudioNode::new_inherited(
                node_id,
                context,
                node_options,
                1, // inputs
                1, // outputs
            ),
            feedforward,
            feedback,
        }
    }
}

impl IIRFilterNodeMethods<crate::DomTypeHolder> for IIRFilterNode {
    /// <https://webaudio.github.io/web-audio-api/#dom-iirfilternode-iirfilternode>
    fn Constructor(
        cx: &mut JSContext,
        window: &Window,
        proto: Option<HandleObject>,
        context: &BaseAudioContext,
        options: &IIRFilterOptions,
    ) -> Fallible<DomRoot<IIRFilterNode>> {
        if !(1..=20).contains(&options.feedforward.len()) ||
            !(1..=20).contains(&options.feedback.len())
        {
            return Err(Error::NotSupported(None));
        }
        if options.feedforward.iter().all(|v| **v == 0.0) || *options.feedback[0] == 0.0 {
            return Err(Error::InvalidState(None));
        }
        let node_options =
            options
                .parent
                .unwrap_or(2, ChannelCountMode::Max, ChannelInterpretation::Speakers);

        if AudioNode::channel_count_check(node_options.count) {
            return Err(Error::NotSupported(Some(String::from(
                "Channel count not supported",
            ))));
        }

        let feedforward = (*options.feedforward).to_vec();
        let feedback = (*options.feedback).to_vec();
        let init_options = options.clone().convert();
        let audio_node_init = AudioNodeInit::IIRFilterNode(init_options);
        let node_id =
            AudioNode::generate_audio_node_id(cx, &node_options, audio_node_init, context);

        Ok(reflect_dom_object_with_proto(
            cx,
            Box::new(IIRFilterNode::new_inherited(
                node_id,
                context,
                node_options,
                feedforward,
                feedback,
            )),
            window,
            proto,
        ))
    }

    /// <https://webaudio.github.io/web-audio-api/#dom-iirfilternode-getfrequencyresponse>
    fn GetFrequencyResponse(
        &self,
        frequency_hz: CustomAutoRooterGuard<Float32Array>,
        mut mag_response: CustomAutoRooterGuard<Float32Array>,
        mut phase_response: CustomAutoRooterGuard<Float32Array>,
    ) -> Result<(), Error> {
        let frequency_hz_vec = frequency_hz.to_vec().unwrap_or(vec![]);
        let mut mag_response_vec = mag_response.to_vec().unwrap_or(vec![]);
        let mut phase_response_vec = phase_response.to_vec().unwrap_or(vec![]);

        let len = frequency_hz.len();
        if len != mag_response.len() || len != phase_response.len() {
            return Err(Error::InvalidAccess(None));
        }
        let feedforward: Vec<f64> = (self.feedforward.iter().map(|v| **v).collect_vec()).to_vec();
        let feedback: Vec<f64> = (self.feedback.iter().map(|v| **v).collect_vec()).to_vec();
        IIRFilter::get_frequency_response(
            &feedforward,
            &feedback,
            &frequency_hz_vec,
            &mut mag_response_vec,
            &mut phase_response_vec,
        );

        mag_response.update(&mag_response_vec);
        phase_response.update(&phase_response_vec);

        Ok(())
    }
}

impl Convert<IIRFilterNodeOptions> for IIRFilterOptions {
    fn convert(self) -> IIRFilterNodeOptions {
        let feedforward: Vec<f64> = (*self.feedforward.iter().map(|v| **v).collect_vec()).to_vec();
        let feedback: Vec<f64> = (*self.feedback.iter().map(|v| **v).collect_vec()).to_vec();
        IIRFilterNodeOptions {
            feedforward: Arc::new(feedforward),
            feedback: Arc::new(feedback),
        }
    }
}
