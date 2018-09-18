/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::audionode::AudioNode;
use dom::audioparam::AudioParam;
use dom::baseaudiocontext::BaseAudioContext;
use dom::bindings::codegen::Bindings::AudioNodeBinding::{ChannelCountMode, ChannelInterpretation};
use dom::bindings::codegen::Bindings::AudioParamBinding::{AudioParamMethods, AutomationRate};
use dom::bindings::codegen::Bindings::PannerNodeBinding::{self, PannerNodeMethods, PannerOptions};
use dom::bindings::codegen::Bindings::PannerNodeBinding::{DistanceModelType, PanningModelType};
use dom::bindings::error::{Error, Fallible};
use dom::bindings::inheritance::Castable;
use dom::bindings::num::Finite;
use dom::bindings::reflector::reflect_dom_object;
use dom::bindings::root::{Dom, DomRoot};
use dom::window::Window;
use dom_struct::dom_struct;
use servo_media::audio::node::{AudioNodeInit, AudioNodeMessage};
use servo_media::audio::panner_node::{DistanceModel, PannerNodeOptions, PanningModel};
use servo_media::audio::panner_node::PannerNodeMessage;
use servo_media::audio::param::{ParamDir, ParamType};
use std::cell::Cell;
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
    #[ignore_malloc_size_of = "servo_media"]
    panning_model: Cell<PanningModel>,
    #[ignore_malloc_size_of = "servo_media"]
    distance_model: Cell<DistanceModel>,
    ref_distance: Cell<f64>,
    max_distance: Cell<f64>,
    rolloff_factor: Cell<f64>,
    cone_inner_angle: Cell<f64>,
    cone_outer_angle: Cell<f64>,
    cone_outer_gain: Cell<f64>,
}

impl PannerNode {
    #[allow(unrooted_must_root)]
    pub fn new_inherited(
        window: &Window,
        context: &BaseAudioContext,
        options: &PannerOptions,
    ) -> Fallible<PannerNode> {
        let node_options = options.parent.unwrap_or(
            2,
            ChannelCountMode::Clamped_max,
            ChannelInterpretation::Speakers,
        );
        if node_options.mode == ChannelCountMode::Max {
            return Err(Error::NotSupported);
        }
        if node_options.count > 2 || node_options.count == 0 {
            return Err(Error::NotSupported);
        }
        if *options.maxDistance <= 0. {
            return Err(Error::Range("maxDistance should be positive".into()));
        }
        if *options.refDistance < 0. {
            return Err(Error::Range("refDistance should be non-negative".into()));
        }
        if *options.rolloffFactor < 0. {
            return Err(Error::Range("rolloffFactor should be non-negative".into()));
        }
        if *options.coneOuterGain < 0. || *options.coneOuterGain > 1. {
            return Err(Error::InvalidState);
        }
        let options = options.into();
        let node = AudioNode::new_inherited(
            AudioNodeInit::PannerNode(options),
            context,
            node_options,
            1, // inputs
            1, // outputs
        )?;
        let id = node.node_id();
        let position_x = AudioParam::new(
            window,
            context,
            id,
            ParamType::Position(ParamDir::X),
            AutomationRate::A_rate,
            options.position_x, // default value
            f32::MIN,           // min value
            f32::MAX,           // max value
        );
        let position_y = AudioParam::new(
            window,
            context,
            id,
            ParamType::Position(ParamDir::Y),
            AutomationRate::A_rate,
            options.position_y, // default value
            f32::MIN,           // min value
            f32::MAX,           // max value
        );
        let position_z = AudioParam::new(
            window,
            context,
            id,
            ParamType::Position(ParamDir::Z),
            AutomationRate::A_rate,
            options.position_z, // default value
            f32::MIN,           // min value
            f32::MAX,           // max value
        );
        let orientation_x = AudioParam::new(
            window,
            context,
            id,
            ParamType::Orientation(ParamDir::X),
            AutomationRate::A_rate,
            options.orientation_x, // default value
            f32::MIN,              // min value
            f32::MAX,              // max value
        );
        let orientation_y = AudioParam::new(
            window,
            context,
            id,
            ParamType::Orientation(ParamDir::Y),
            AutomationRate::A_rate,
            options.orientation_y, // default value
            f32::MIN,              // min value
            f32::MAX,              // max value
        );
        let orientation_z = AudioParam::new(
            window,
            context,
            id,
            ParamType::Orientation(ParamDir::Z),
            AutomationRate::A_rate,
            options.orientation_z, // default value
            f32::MIN,              // min value
            f32::MAX,              // max value
        );
        Ok(PannerNode {
            node,
            position_x: Dom::from_ref(&position_x),
            position_y: Dom::from_ref(&position_y),
            position_z: Dom::from_ref(&position_z),
            orientation_x: Dom::from_ref(&orientation_x),
            orientation_y: Dom::from_ref(&orientation_y),
            orientation_z: Dom::from_ref(&orientation_z),
            panning_model: Cell::new(options.panning_model),
            distance_model: Cell::new(options.distance_model),
            ref_distance: Cell::new(options.ref_distance),
            max_distance: Cell::new(options.max_distance),
            rolloff_factor: Cell::new(options.rolloff_factor),
            cone_inner_angle: Cell::new(options.cone_inner_angle),
            cone_outer_angle: Cell::new(options.cone_outer_angle),
            cone_outer_gain: Cell::new(options.cone_outer_gain),
        })
    }

    #[allow(unrooted_must_root)]
    pub fn new(
        window: &Window,
        context: &BaseAudioContext,
        options: &PannerOptions,
    ) -> Fallible<DomRoot<PannerNode>> {
        let node = PannerNode::new_inherited(window, context, options)?;
        Ok(reflect_dom_object(
            Box::new(node),
            window,
            PannerNodeBinding::Wrap,
        ))
    }

    pub fn Constructor(
        window: &Window,
        context: &BaseAudioContext,
        options: &PannerOptions,
    ) -> Fallible<DomRoot<PannerNode>> {
        PannerNode::new(window, context, options)
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

    // https://webaudio.github.io/web-audio-api/#dom-pannernode-distancemodel
    fn DistanceModel(&self) -> DistanceModelType {
        match self.distance_model.get() {
            DistanceModel::Linear => DistanceModelType::Linear,
            DistanceModel::Inverse => DistanceModelType::Inverse,
            DistanceModel::Exponential => DistanceModelType::Exponential,
        }
    }
    // https://webaudio.github.io/web-audio-api/#dom-pannernode-distancemodel
    fn SetDistanceModel(&self, model: DistanceModelType) {
        self.distance_model.set(model.into());
        let msg = PannerNodeMessage::SetDistanceModel(self.distance_model.get());
        self.upcast::<AudioNode>()
            .message(AudioNodeMessage::PannerNode(msg));
    }
    // https://webaudio.github.io/web-audio-api/#dom-pannernode-panningmodel
    fn PanningModel(&self) -> PanningModelType {
        match self.panning_model.get() {
            PanningModel::EqualPower => PanningModelType::Equalpower,
            PanningModel::HRTF => PanningModelType::HRTF,
        }
    }
    // https://webaudio.github.io/web-audio-api/#dom-pannernode-panningmodel
    fn SetPanningModel(&self, model: PanningModelType) {
        self.panning_model.set(model.into());
        let msg = PannerNodeMessage::SetPanningModel(self.panning_model.get());
        self.upcast::<AudioNode>()
            .message(AudioNodeMessage::PannerNode(msg));
    }
    // https://webaudio.github.io/web-audio-api/#dom-pannernode-refdistance
    fn RefDistance(&self) -> Finite<f64> {
        Finite::wrap(self.ref_distance.get())
    }
    // https://webaudio.github.io/web-audio-api/#dom-pannernode-refdistance
    fn SetRefDistance(&self, val: Finite<f64>) -> Fallible<()> {
        if *val < 0. {
            return Err(Error::Range("value should be non-negative".into()));
        }
        self.ref_distance.set(*val);
        let msg = PannerNodeMessage::SetRefDistance(self.ref_distance.get());
        self.upcast::<AudioNode>()
            .message(AudioNodeMessage::PannerNode(msg));
        Ok(())
    }
    // https://webaudio.github.io/web-audio-api/#dom-pannernode-maxdistance
    fn MaxDistance(&self) -> Finite<f64> {
        Finite::wrap(self.max_distance.get())
    }
    // https://webaudio.github.io/web-audio-api/#dom-pannernode-maxdistance
    fn SetMaxDistance(&self, val: Finite<f64>) -> Fallible<()> {
        if *val <= 0. {
            return Err(Error::Range("value should be positive".into()));
        }
        self.max_distance.set(*val);
        let msg = PannerNodeMessage::SetMaxDistance(self.max_distance.get());
        self.upcast::<AudioNode>()
            .message(AudioNodeMessage::PannerNode(msg));
        Ok(())
    }
    // https://webaudio.github.io/web-audio-api/#dom-pannernode-rollofffactor
    fn RolloffFactor(&self) -> Finite<f64> {
        Finite::wrap(self.rolloff_factor.get())
    }
    // https://webaudio.github.io/web-audio-api/#dom-pannernode-rollofffactor
    fn SetRolloffFactor(&self, val: Finite<f64>) -> Fallible<()> {
        if *val < 0. {
            return Err(Error::Range("value should be non-negative".into()));
        }
        self.rolloff_factor.set(*val);
        let msg = PannerNodeMessage::SetRolloff(self.rolloff_factor.get());
        self.upcast::<AudioNode>()
            .message(AudioNodeMessage::PannerNode(msg));
        Ok(())
    }
    // https://webaudio.github.io/web-audio-api/#dom-pannernode-coneinnerangle
    fn ConeInnerAngle(&self) -> Finite<f64> {
        Finite::wrap(self.cone_inner_angle.get())
    }
    // https://webaudio.github.io/web-audio-api/#dom-pannernode-coneinnerangle
    fn SetConeInnerAngle(&self, val: Finite<f64>) {
        self.cone_inner_angle.set(*val);
        let msg = PannerNodeMessage::SetConeInner(self.cone_inner_angle.get());
        self.upcast::<AudioNode>()
            .message(AudioNodeMessage::PannerNode(msg));
    }
    // https://webaudio.github.io/web-audio-api/#dom-pannernode-coneouterangle
    fn ConeOuterAngle(&self) -> Finite<f64> {
        Finite::wrap(self.cone_outer_angle.get())
    }
    // https://webaudio.github.io/web-audio-api/#dom-pannernode-coneouterangle
    fn SetConeOuterAngle(&self, val: Finite<f64>) {
        self.cone_outer_angle.set(*val);
        let msg = PannerNodeMessage::SetConeOuter(self.cone_outer_angle.get());
        self.upcast::<AudioNode>()
            .message(AudioNodeMessage::PannerNode(msg));
    }
    // https://webaudio.github.io/web-audio-api/#dom-pannernode-coneoutergain
    fn ConeOuterGain(&self) -> Finite<f64> {
        Finite::wrap(self.cone_outer_gain.get())
    }
    // https://webaudio.github.io/web-audio-api/#dom-pannernode-coneoutergain
    fn SetConeOuterGain(&self, val: Finite<f64>) -> Fallible<()> {
        if *val < 0. || *val > 1. {
            return Err(Error::InvalidState);
        }
        self.cone_outer_gain.set(*val);
        let msg = PannerNodeMessage::SetConeGain(self.cone_outer_gain.get());
        self.upcast::<AudioNode>()
            .message(AudioNodeMessage::PannerNode(msg));
        Ok(())
    }

    // https://webaudio.github.io/web-audio-api/#dom-pannernode-setposition
    fn SetPosition(&self, x: Finite<f32>, y: Finite<f32>, z: Finite<f32>) {
        self.position_x.SetValue(x);
        self.position_y.SetValue(y);
        self.position_z.SetValue(z);
    }

    // https://webaudio.github.io/web-audio-api/#dom-pannernode-setorientation
    fn SetOrientation(&self, x: Finite<f32>, y: Finite<f32>, z: Finite<f32>) {
        self.orientation_x.SetValue(x);
        self.orientation_y.SetValue(y);
        self.orientation_z.SetValue(z);
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
