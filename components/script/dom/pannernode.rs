/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::audionode::AudioNode;
use dom::audioparam::AudioParam;
use dom::baseaudiocontext::BaseAudioContext;
use dom::bindings::codegen::Bindings::AudioNodeBinding::{ChannelCountMode, ChannelInterpretation};
use dom::bindings::codegen::Bindings::AudioNodeBinding::AudioNodeOptions;
use dom::bindings::codegen::Bindings::AudioParamBinding::AutomationRate;
use dom::bindings::codegen::Bindings::PannerNodeBinding::{self, PannerNodeMethods, PannerOptions};
use dom::bindings::codegen::Bindings::PannerNodeBinding::{DistanceModelType, PanningModelType};
use dom::bindings::error::Fallible;
use dom::bindings::reflector::reflect_dom_object;
use dom::bindings::root::{Dom, DomRoot};
use dom::window::Window;
use dom_struct::dom_struct;
use servo_media::audio::panner_node::{DistanceModel, PannerNodeOptions, PanningModel};
use servo_media::audio::node::AudioNodeInit;
use servo_media::audio::param::{ParamDir, ParamType};
use std::f32;

#[dom_struct]
pub struct PannerNode {
    node: AudioNode,
    position_x: Dom<AudioParam>,
    position_y: Dom<AudioParam>,
    position_z: Dom<AudioParam>,
    orientation_x: Dom<AudioParam>,
    orientation_y: Dom<AudioParam>,
    orientation_z: Dom<AudioParam>,
}

impl PannerNode {
    #[allow(unrooted_must_root)]
    pub fn new_inherited(
        window: &Window,
        context: &BaseAudioContext,
        options: &PannerOptions,
    ) -> PannerNode {
        let mut node_options = AudioNodeOptions::empty();
        node_options.channelCount = Some(2);
        node_options.channelCountMode = Some(ChannelCountMode::Clamped_max);
        node_options.channelInterpretation = Some(ChannelInterpretation::Speakers);
        let options = options.into();
        let node = AudioNode::new_inherited(
            AudioNodeInit::PannerNode(options),
            context,
            &node_options,
            1, // inputs
            1, // outputs
        );
        let id = node.node_id();
        let position_x = AudioParam::new(
            window,
            context,
            id,
            ParamType::Position(ParamDir::X),
            AutomationRate::A_rate,
            options.position_x,       // default value
            f32::MIN, // min value
            f32::MAX, // max value
        );
        let position_y = AudioParam::new(
            window,
            context,
            id,
            ParamType::Position(ParamDir::Y),
            AutomationRate::A_rate,
            options.position_y,       // default value
            f32::MIN, // min value
            f32::MAX, // max value
        );
        let position_z = AudioParam::new(
            window,
            context,
            id,
            ParamType::Position(ParamDir::Z),
            AutomationRate::A_rate,
            options.position_z,       // default value
            f32::MIN, // min value
            f32::MAX, // max value
        );
        let orientation_x = AudioParam::new(
            window,
            context,
            id,
            ParamType::Orientation(ParamDir::X),
            AutomationRate::A_rate,
            options.orientation_x,       // default value
            f32::MIN, // min value
            f32::MAX, // max value
        );
        let orientation_y = AudioParam::new(
            window,
            context,
            id,
            ParamType::Orientation(ParamDir::Y),
            AutomationRate::A_rate,
            options.orientation_y,       // default value
            f32::MIN, // min value
            f32::MAX, // max value
        );
        let orientation_z = AudioParam::new(
            window,
            context,
            id,
            ParamType::Orientation(ParamDir::Z),
            AutomationRate::A_rate,
            options.orientation_z,       // default value
            f32::MIN, // min value
            f32::MAX, // max value
        );
        PannerNode {
            node,
            position_x: Dom::from_ref(&position_x),
            position_y: Dom::from_ref(&position_y),
            position_z: Dom::from_ref(&position_z),
            orientation_x: Dom::from_ref(&orientation_x),
            orientation_y: Dom::from_ref(&orientation_y),
            orientation_z: Dom::from_ref(&orientation_z),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(
        window: &Window,
        context: &BaseAudioContext,
        options: &PannerOptions,
    ) -> DomRoot<PannerNode> {
        let node = PannerNode::new_inherited(window, context, options);
        reflect_dom_object(Box::new(node), window, PannerNodeBinding::Wrap)
    }

    pub fn Constructor(
        window: &Window,
        context: &BaseAudioContext,
        options: &PannerOptions,
    ) -> Fallible<DomRoot<PannerNode>> {
        Ok(PannerNode::new(window, context, options))
    }
}

impl PannerNodeMethods for PannerNode {
    // https://webaudio.github.io/web-audio-api/#dom-pannernode-positionx
    fn PositionX(&self) -> DomRoot<AudioParam> {
        DomRoot::from_ref(&self.position_x)
    }
    // https://webaudio.github.io/web-audio-api/#dom-pannernode-positiony
    fn PositionY(&self) -> DomRoot<AudioParam> {
        DomRoot::from_ref(&self.position_y)
    }
    // https://webaudio.github.io/web-audio-api/#dom-pannernode-positionz
    fn PositionZ(&self) -> DomRoot<AudioParam> {
        DomRoot::from_ref(&self.position_z)
    }

    // https://webaudio.github.io/web-audio-api/#dom-pannernode-orientationx
    fn OrientationX(&self) -> DomRoot<AudioParam> {
        DomRoot::from_ref(&self.orientation_x)
    }
    // https://webaudio.github.io/web-audio-api/#dom-pannernode-orientationy
    fn OrientationY(&self) -> DomRoot<AudioParam> {
        DomRoot::from_ref(&self.orientation_y)
    }
    // https://webaudio.github.io/web-audio-api/#dom-pannernode-orientationz
    fn OrientationZ(&self) -> DomRoot<AudioParam> {
        DomRoot::from_ref(&self.orientation_z)
    }
}

impl<'a> From<&'a PannerOptions> for PannerNodeOptions {
    fn from(options: &'a PannerOptions) -> Self {
        Self {
            panning_model: options.panningModel.into(),
            distance_model: options.distanceModel.into(),
            position_x: *options.positionX,
            position_y: *options.positionY,
            position_z: *options.positionZ,
            orientation_x: *options.orientationX,
            orientation_y: *options.orientationY,
            orientation_z: *options.orientationZ,
            ref_distance: *options.refDistance,
            max_distance: *options.maxDistance,
            rolloff_factor: *options.rolloffFactor,
            cone_inner_angle: *options.coneInnerAngle,
            cone_outer_angle: *options.coneOuterAngle,
            cone_outer_gain: *options.coneOuterGain,
        }
    }
}

impl From<DistanceModelType> for DistanceModel {
    fn from(model: DistanceModelType) -> Self {
        match model {
            DistanceModelType::Linear => DistanceModel::Linear,
            DistanceModelType::Inverse => DistanceModel::Inverse,
            DistanceModelType::Exponential => DistanceModel::Exponential,
        }
    }
}

impl From<PanningModelType> for PanningModel {
    fn from(model: PanningModelType) -> Self {
        match model {
            PanningModelType::Equalpower => PanningModel::EqualPower,
            PanningModelType::HRTF => PanningModel::HRTF,
        }
    }
}
