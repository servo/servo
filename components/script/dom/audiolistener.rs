/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::f32;

use dom_struct::dom_struct;
use servo_media::audio::node::AudioNodeType;
use servo_media::audio::param::{ParamDir, ParamType};

use crate::dom::audioparam::AudioParam;
use crate::dom::baseaudiocontext::BaseAudioContext;
use crate::dom::bindings::codegen::Bindings::AudioListenerBinding::AudioListenerMethods;
use crate::dom::bindings::codegen::Bindings::AudioParamBinding::{
    AudioParamMethods, AutomationRate,
};
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::num::Finite;
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::window::Window;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct AudioListener {
    reflector_: Reflector,
    position_x: Dom<AudioParam>,
    position_y: Dom<AudioParam>,
    position_z: Dom<AudioParam>,
    forward_x: Dom<AudioParam>,
    forward_y: Dom<AudioParam>,
    forward_z: Dom<AudioParam>,
    up_x: Dom<AudioParam>,
    up_y: Dom<AudioParam>,
    up_z: Dom<AudioParam>,
}

impl AudioListener {
    fn new_inherited(window: &Window, context: &BaseAudioContext) -> AudioListener {
        let node = context.listener();

        let position_x = AudioParam::new(
            window,
            context,
            node,
            AudioNodeType::AudioListenerNode,
            ParamType::Position(ParamDir::X),
            AutomationRate::A_rate,
            0.,       // default value
            f32::MIN, // min value
            f32::MAX, // max value
            CanGc::note(),
        );
        let position_y = AudioParam::new(
            window,
            context,
            node,
            AudioNodeType::AudioListenerNode,
            ParamType::Position(ParamDir::Y),
            AutomationRate::A_rate,
            0.,       // default value
            f32::MIN, // min value
            f32::MAX, // max value
            CanGc::note(),
        );
        let position_z = AudioParam::new(
            window,
            context,
            node,
            AudioNodeType::AudioListenerNode,
            ParamType::Position(ParamDir::Z),
            AutomationRate::A_rate,
            0.,       // default value
            f32::MIN, // min value
            f32::MAX, // max value
            CanGc::note(),
        );
        let forward_x = AudioParam::new(
            window,
            context,
            node,
            AudioNodeType::AudioListenerNode,
            ParamType::Forward(ParamDir::X),
            AutomationRate::A_rate,
            0.,       // default value
            f32::MIN, // min value
            f32::MAX, // max value
            CanGc::note(),
        );
        let forward_y = AudioParam::new(
            window,
            context,
            node,
            AudioNodeType::AudioListenerNode,
            ParamType::Forward(ParamDir::Y),
            AutomationRate::A_rate,
            0.,       // default value
            f32::MIN, // min value
            f32::MAX, // max value
            CanGc::note(),
        );
        let forward_z = AudioParam::new(
            window,
            context,
            node,
            AudioNodeType::AudioListenerNode,
            ParamType::Forward(ParamDir::Z),
            AutomationRate::A_rate,
            -1.,      // default value
            f32::MIN, // min value
            f32::MAX, // max value
            CanGc::note(),
        );
        let up_x = AudioParam::new(
            window,
            context,
            node,
            AudioNodeType::AudioListenerNode,
            ParamType::Up(ParamDir::X),
            AutomationRate::A_rate,
            0.,       // default value
            f32::MIN, // min value
            f32::MAX, // max value
            CanGc::note(),
        );
        let up_y = AudioParam::new(
            window,
            context,
            node,
            AudioNodeType::AudioListenerNode,
            ParamType::Up(ParamDir::Y),
            AutomationRate::A_rate,
            1.,       // default value
            f32::MIN, // min value
            f32::MAX, // max value
            CanGc::note(),
        );
        let up_z = AudioParam::new(
            window,
            context,
            node,
            AudioNodeType::AudioListenerNode,
            ParamType::Up(ParamDir::Z),
            AutomationRate::A_rate,
            0.,       // default value
            f32::MIN, // min value
            f32::MAX, // max value
            CanGc::note(),
        );

        AudioListener {
            reflector_: Reflector::new(),
            position_x: Dom::from_ref(&position_x),
            position_y: Dom::from_ref(&position_y),
            position_z: Dom::from_ref(&position_z),
            forward_x: Dom::from_ref(&forward_x),
            forward_y: Dom::from_ref(&forward_y),
            forward_z: Dom::from_ref(&forward_z),
            up_x: Dom::from_ref(&up_x),
            up_y: Dom::from_ref(&up_y),
            up_z: Dom::from_ref(&up_z),
        }
    }

    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    pub(crate) fn new(
        window: &Window,
        context: &BaseAudioContext,
        can_gc: CanGc,
    ) -> DomRoot<AudioListener> {
        let node = AudioListener::new_inherited(window, context);
        reflect_dom_object(Box::new(node), window, can_gc)
    }
}

#[allow(non_snake_case)]
impl AudioListenerMethods<crate::DomTypeHolder> for AudioListener {
    // https://webaudio.github.io/web-audio-api/#dom-audiolistener-positionx
    fn PositionX(&self) -> DomRoot<AudioParam> {
        DomRoot::from_ref(&self.position_x)
    }
    // https://webaudio.github.io/web-audio-api/#dom-audiolistener-positiony
    fn PositionY(&self) -> DomRoot<AudioParam> {
        DomRoot::from_ref(&self.position_y)
    }
    // https://webaudio.github.io/web-audio-api/#dom-audiolistener-positionz
    fn PositionZ(&self) -> DomRoot<AudioParam> {
        DomRoot::from_ref(&self.position_z)
    }

    // https://webaudio.github.io/web-audio-api/#dom-audiolistener-forwardx
    fn ForwardX(&self) -> DomRoot<AudioParam> {
        DomRoot::from_ref(&self.forward_x)
    }
    // https://webaudio.github.io/web-audio-api/#dom-audiolistener-forwardy
    fn ForwardY(&self) -> DomRoot<AudioParam> {
        DomRoot::from_ref(&self.forward_y)
    }
    // https://webaudio.github.io/web-audio-api/#dom-audiolistener-forwardz
    fn ForwardZ(&self) -> DomRoot<AudioParam> {
        DomRoot::from_ref(&self.forward_z)
    }

    // https://webaudio.github.io/web-audio-api/#dom-audiolistener-upx
    fn UpX(&self) -> DomRoot<AudioParam> {
        DomRoot::from_ref(&self.up_x)
    }
    // https://webaudio.github.io/web-audio-api/#dom-audiolistener-upy
    fn UpY(&self) -> DomRoot<AudioParam> {
        DomRoot::from_ref(&self.up_y)
    }
    // https://webaudio.github.io/web-audio-api/#dom-audiolistener-upz
    fn UpZ(&self) -> DomRoot<AudioParam> {
        DomRoot::from_ref(&self.up_z)
    }

    // https://webaudio.github.io/web-audio-api/#dom-audiolistener-setorientation
    fn SetOrientation(
        &self,
        x: Finite<f32>,
        y: Finite<f32>,
        z: Finite<f32>,
        xUp: Finite<f32>,
        yUp: Finite<f32>,
        zUp: Finite<f32>,
    ) -> Fallible<DomRoot<AudioListener>> {
        self.forward_x.SetValue(x);
        self.forward_y.SetValue(y);
        self.forward_z.SetValue(z);
        self.up_x.SetValue(xUp);
        self.up_y.SetValue(yUp);
        self.up_z.SetValue(zUp);
        Ok(DomRoot::from_ref(self))
    }

    // https://webaudio.github.io/web-audio-api/#dom-audiolistener-setposition
    fn SetPosition(
        &self,
        x: Finite<f32>,
        y: Finite<f32>,
        z: Finite<f32>,
    ) -> Fallible<DomRoot<AudioListener>> {
        self.position_x.SetValue(x);
        self.position_y.SetValue(y);
        self.position_z.SetValue(z);
        Ok(DomRoot::from_ref(self))
    }
}
