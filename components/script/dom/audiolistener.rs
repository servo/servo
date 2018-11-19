/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::audioparam::AudioParam;
use crate::dom::baseaudiocontext::BaseAudioContext;
use crate::dom::bindings::codegen::Bindings::AudioListenerBinding::{self, AudioListenerMethods};
use crate::dom::bindings::codegen::Bindings::AudioParamBinding::AutomationRate;
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::window::Window;
use dom_struct::dom_struct;
use servo_media::audio::param::{ParamDir, ParamType};
use std::f32;

#[dom_struct]
pub struct AudioListener {
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
            ParamType::Position(ParamDir::X),
            AutomationRate::A_rate,
            0.,       // default value
            f32::MIN, // min value
            f32::MAX, // max value
        );
        let position_y = AudioParam::new(
            window,
            context,
            node,
            ParamType::Position(ParamDir::Y),
            AutomationRate::A_rate,
            0.,       // default value
            f32::MIN, // min value
            f32::MAX, // max value
        );
        let position_z = AudioParam::new(
            window,
            context,
            node,
            ParamType::Position(ParamDir::Z),
            AutomationRate::A_rate,
            0.,       // default value
            f32::MIN, // min value
            f32::MAX, // max value
        );
        let forward_x = AudioParam::new(
            window,
            context,
            node,
            ParamType::Forward(ParamDir::X),
            AutomationRate::A_rate,
            0.,       // default value
            f32::MIN, // min value
            f32::MAX, // max value
        );
        let forward_y = AudioParam::new(
            window,
            context,
            node,
            ParamType::Forward(ParamDir::Y),
            AutomationRate::A_rate,
            0.,       // default value
            f32::MIN, // min value
            f32::MAX, // max value
        );
        let forward_z = AudioParam::new(
            window,
            context,
            node,
            ParamType::Forward(ParamDir::Z),
            AutomationRate::A_rate,
            -1.,      // default value
            f32::MIN, // min value
            f32::MAX, // max value
        );
        let up_x = AudioParam::new(
            window,
            context,
            node,
            ParamType::Up(ParamDir::X),
            AutomationRate::A_rate,
            0.,       // default value
            f32::MIN, // min value
            f32::MAX, // max value
        );
        let up_y = AudioParam::new(
            window,
            context,
            node,
            ParamType::Up(ParamDir::Y),
            AutomationRate::A_rate,
            1.,       // default value
            f32::MIN, // min value
            f32::MAX, // max value
        );
        let up_z = AudioParam::new(
            window,
            context,
            node,
            ParamType::Up(ParamDir::Z),
            AutomationRate::A_rate,
            0.,       // default value
            f32::MIN, // min value
            f32::MAX, // max value
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

    #[allow(unrooted_must_root)]
    pub fn new(window: &Window, context: &BaseAudioContext) -> DomRoot<AudioListener> {
        let node = AudioListener::new_inherited(window, context);
        reflect_dom_object(Box::new(node), window, AudioListenerBinding::Wrap)
    }
}

impl AudioListenerMethods for AudioListener {
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
}
