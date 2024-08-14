/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::sync::Arc;

use dom_struct::dom_struct;
use itertools::Itertools;
use js::gc::CustomAutoRooterGuard;
use js::rust::HandleObject;
use js::typedarray::Float32Array;
use servo_media::audio::iir_filter_node::{IIRFilterNode as IIRFilter, IIRFilterNodeOptions};
use servo_media::audio::node::AudioNodeInit;

use crate::dom::audionode::AudioNode;
use crate::dom::baseaudiocontext::BaseAudioContext;
use crate::dom::bindings::codegen::Bindings::AudioNodeBinding::{
    ChannelCountMode, ChannelInterpretation,
};
use crate::dom::bindings::codegen::Bindings::IIRFilterNodeBinding::{
    IIRFilterNodeMethods, IIRFilterOptions,
};
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::num::Finite;
use crate::dom::bindings::reflector::reflect_dom_object_with_proto;
use crate::dom::bindings::root::DomRoot;
use crate::dom::window::Window;

#[dom_struct]
pub struct IIRFilterNode {
    node: AudioNode,
    feedforward: Vec<Finite<f64>>,
    feedback: Vec<Finite<f64>>,
}

impl IIRFilterNode {
    #[allow(crown::unrooted_must_root)]
    pub fn new_inherited(
        _window: &Window,
        context: &BaseAudioContext,
        options: &IIRFilterOptions,
    ) -> Fallible<IIRFilterNode> {
        if !(1..=20).contains(&options.feedforward.len()) ||
            !(1..=20).contains(&options.feedback.len())
        {
            return Err(Error::NotSupported);
        }
        if options.feedforward.iter().all(|v| **v == 0.0) || *options.feedback[0] == 0.0 {
            return Err(Error::InvalidState);
        }
        let node_options =
            options
                .parent
                .unwrap_or(2, ChannelCountMode::Max, ChannelInterpretation::Speakers);
        let init_options = options.into();
        let node = AudioNode::new_inherited(
            AudioNodeInit::IIRFilterNode(init_options),
            context,
            node_options,
            1, // inputs
            1, // outputs
        )?;
        Ok(IIRFilterNode {
            node,
            feedforward: (*options.feedforward).to_vec(),
            feedback: (*options.feedback).to_vec(),
        })
    }

    pub fn new(
        window: &Window,
        context: &BaseAudioContext,
        options: &IIRFilterOptions,
    ) -> Fallible<DomRoot<IIRFilterNode>> {
        Self::new_with_proto(window, None, context, options)
    }

    #[allow(crown::unrooted_must_root)]
    fn new_with_proto(
        window: &Window,
        proto: Option<HandleObject>,
        context: &BaseAudioContext,
        options: &IIRFilterOptions,
    ) -> Fallible<DomRoot<IIRFilterNode>> {
        let node = IIRFilterNode::new_inherited(window, context, options)?;
        Ok(reflect_dom_object_with_proto(Box::new(node), window, proto))
    }

    #[allow(non_snake_case)]
    pub fn Constructor(
        window: &Window,
        proto: Option<HandleObject>,
        context: &BaseAudioContext,
        options: &IIRFilterOptions,
    ) -> Fallible<DomRoot<IIRFilterNode>> {
        IIRFilterNode::new_with_proto(window, proto, context, options)
    }
}

impl IIRFilterNodeMethods for IIRFilterNode {
    #[allow(unsafe_code)]
    /// <https://webaudio.github.io/web-audio-api/#dom-iirfilternode-getfrequencyresponse>
    fn GetFrequencyResponse(
        &self,
        frequency_hz: CustomAutoRooterGuard<Float32Array>,
        mut mag_response: CustomAutoRooterGuard<Float32Array>,
        mut phase_response: CustomAutoRooterGuard<Float32Array>,
    ) -> Result<(), Error> {
        let len = frequency_hz.len();
        if len != mag_response.len() || len != phase_response.len() {
            return Err(Error::InvalidAccess);
        }
        let feedforward: Vec<f64> = (self.feedforward.iter().map(|v| **v).collect_vec()).to_vec();
        let feedback: Vec<f64> = (self.feedback.iter().map(|v| **v).collect_vec()).to_vec();
        let frequency_hz_vec = frequency_hz.to_vec();
        let mut mag_response_vec = mag_response.to_vec();
        let mut phase_response_vec = phase_response.to_vec();
        IIRFilter::get_frequency_response(
            &feedforward,
            &feedback,
            &frequency_hz_vec,
            &mut mag_response_vec,
            &mut phase_response_vec,
        );

        unsafe {
            mag_response.update(&mag_response_vec);
            phase_response.update(&phase_response_vec);
        }

        Ok(())
    }
}

impl<'a> From<&'a IIRFilterOptions> for IIRFilterNodeOptions {
    fn from(options: &'a IIRFilterOptions) -> Self {
        let feedforward: Vec<f64> =
            (*options.feedforward.iter().map(|v| **v).collect_vec()).to_vec();
        let feedback: Vec<f64> = (*options.feedback.iter().map(|v| **v).collect_vec()).to_vec();
        Self {
            feedforward: Arc::new(feedforward),
            feedback: Arc::new(feedback),
        }
    }
}
