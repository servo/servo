/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::f32;

use dom_struct::dom_struct;
use js::rust::HandleObject;
use servo_media::audio::node::{AudioNodeInit, AudioNodeMessage, AudioNodeType};
use servo_media::audio::panner_node::{
    DistanceModel, PannerNodeMessage, PannerNodeOptions, PanningModel,
};
use servo_media::audio::param::{ParamDir, ParamType};

use crate::conversions::Convert;
use crate::dom::audionode::AudioNode;
use crate::dom::audioparam::AudioParam;
use crate::dom::baseaudiocontext::BaseAudioContext;
use crate::dom::bindings::codegen::Bindings::AudioNodeBinding::{
    ChannelCountMode, ChannelInterpretation,
};
use crate::dom::bindings::codegen::Bindings::AudioParamBinding::{
    AudioParamMethods, AutomationRate,
};
use crate::dom::bindings::codegen::Bindings::PannerNodeBinding::{
    DistanceModelType, PannerNodeMethods, PannerOptions, PanningModelType,
};
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::num::Finite;
use crate::dom::bindings::reflector::reflect_dom_object_with_proto;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::window::Window;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct PannerNode {
    node: AudioNode,
    position_x: Dom<AudioParam>,
    position_y: Dom<AudioParam>,
    position_z: Dom<AudioParam>,
    orientation_x: Dom<AudioParam>,
    orientation_y: Dom<AudioParam>,
    orientation_z: Dom<AudioParam>,
    #[ignore_malloc_size_of = "servo_media"]
    #[no_trace]
    panning_model: Cell<PanningModel>,
    #[ignore_malloc_size_of = "servo_media"]
    #[no_trace]
    distance_model: Cell<DistanceModel>,
    ref_distance: Cell<f64>,
    max_distance: Cell<f64>,
    rolloff_factor: Cell<f64>,
    cone_inner_angle: Cell<f64>,
    cone_outer_angle: Cell<f64>,
    cone_outer_gain: Cell<f64>,
}

impl PannerNode {
    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    pub(crate) fn new_inherited(
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
        let options = options.convert();
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
            AudioNodeType::PannerNode,
            ParamType::Position(ParamDir::X),
            AutomationRate::A_rate,
            options.position_x, // default value
            f32::MIN,           // min value
            f32::MAX,           // max value
            CanGc::note(),
        );
        let position_y = AudioParam::new(
            window,
            context,
            id,
            AudioNodeType::PannerNode,
            ParamType::Position(ParamDir::Y),
            AutomationRate::A_rate,
            options.position_y, // default value
            f32::MIN,           // min value
            f32::MAX,           // max value
            CanGc::note(),
        );
        let position_z = AudioParam::new(
            window,
            context,
            id,
            AudioNodeType::PannerNode,
            ParamType::Position(ParamDir::Z),
            AutomationRate::A_rate,
            options.position_z, // default value
            f32::MIN,           // min value
            f32::MAX,           // max value
            CanGc::note(),
        );
        let orientation_x = AudioParam::new(
            window,
            context,
            id,
            AudioNodeType::PannerNode,
            ParamType::Orientation(ParamDir::X),
            AutomationRate::A_rate,
            options.orientation_x, // default value
            f32::MIN,              // min value
            f32::MAX,              // max value
            CanGc::note(),
        );
        let orientation_y = AudioParam::new(
            window,
            context,
            id,
            AudioNodeType::PannerNode,
            ParamType::Orientation(ParamDir::Y),
            AutomationRate::A_rate,
            options.orientation_y, // default value
            f32::MIN,              // min value
            f32::MAX,              // max value
            CanGc::note(),
        );
        let orientation_z = AudioParam::new(
            window,
            context,
            id,
            AudioNodeType::PannerNode,
            ParamType::Orientation(ParamDir::Z),
            AutomationRate::A_rate,
            options.orientation_z, // default value
            f32::MIN,              // min value
            f32::MAX,              // max value
            CanGc::note(),
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

    pub(crate) fn new(
        window: &Window,
        context: &BaseAudioContext,
        options: &PannerOptions,
        can_gc: CanGc,
    ) -> Fallible<DomRoot<PannerNode>> {
        Self::new_with_proto(window, None, context, options, can_gc)
    }

    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    fn new_with_proto(
        window: &Window,
        proto: Option<HandleObject>,
        context: &BaseAudioContext,
        options: &PannerOptions,
        can_gc: CanGc,
    ) -> Fallible<DomRoot<PannerNode>> {
        let node = PannerNode::new_inherited(window, context, options)?;
        Ok(reflect_dom_object_with_proto(
            Box::new(node),
            window,
            proto,
            can_gc,
        ))
    }
}

impl PannerNodeMethods<crate::DomTypeHolder> for PannerNode {
    // https://webaudio.github.io/web-audio-api/#dom-pannernode-pannernode
    fn Constructor(
        window: &Window,
        proto: Option<HandleObject>,
        can_gc: CanGc,
        context: &BaseAudioContext,
        options: &PannerOptions,
    ) -> Fallible<DomRoot<PannerNode>> {
        PannerNode::new_with_proto(window, proto, context, options, can_gc)
    }

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
        self.distance_model.set(model.convert());
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
        self.panning_model.set(model.convert());
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

impl Convert<PannerNodeOptions> for &PannerOptions {
    fn convert(self) -> PannerNodeOptions {
        PannerNodeOptions {
            panning_model: self.panningModel.convert(),
            distance_model: self.distanceModel.convert(),
            position_x: *self.positionX,
            position_y: *self.positionY,
            position_z: *self.positionZ,
            orientation_x: *self.orientationX,
            orientation_y: *self.orientationY,
            orientation_z: *self.orientationZ,
            ref_distance: *self.refDistance,
            max_distance: *self.maxDistance,
            rolloff_factor: *self.rolloffFactor,
            cone_inner_angle: *self.coneInnerAngle,
            cone_outer_angle: *self.coneOuterAngle,
            cone_outer_gain: *self.coneOuterGain,
        }
    }
}

impl Convert<DistanceModel> for DistanceModelType {
    fn convert(self) -> DistanceModel {
        match self {
            DistanceModelType::Linear => DistanceModel::Linear,
            DistanceModelType::Inverse => DistanceModel::Inverse,
            DistanceModelType::Exponential => DistanceModel::Exponential,
        }
    }
}

impl Convert<PanningModel> for PanningModelType {
    fn convert(self) -> PanningModel {
        match self {
            PanningModelType::Equalpower => PanningModel::EqualPower,
            PanningModelType::HRTF => PanningModel::HRTF,
        }
    }
}
