/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::baseaudiocontext::BaseAudioContext;
use dom::bindings::codegen::Bindings::AudioParamBinding;
use dom::bindings::codegen::Bindings::AudioParamBinding::{AudioParamMethods, AutomationRate};
use dom::bindings::num::Finite;
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::bindings::root::{Dom, DomRoot};
use dom::window::Window;
use dom_struct::dom_struct;
use servo_media::audio::graph::NodeId;
use servo_media::audio::node::AudioNodeMessage;
use servo_media::audio::param::{ParamRate, ParamType, RampKind, UserAutomationEvent};
use std::cell::Cell;
use std::sync::mpsc;
use typeholder::TypeHolderTrait;

#[dom_struct]
pub struct AudioParam<TH: TypeHolderTrait> {
    reflector_: Reflector<TH>,
    context: Dom<BaseAudioContext<TH>>,
    #[ignore_malloc_size_of = "servo_media"]
    node: NodeId,
    #[ignore_malloc_size_of = "servo_media"]
    param: ParamType,
    automation_rate: Cell<AutomationRate>,
    default_value: f32,
    min_value: f32,
    max_value: f32,
}

impl<TH: TypeHolderTrait> AudioParam<TH> {
    pub fn new_inherited(
        context: &BaseAudioContext<TH>,
        node: NodeId,
        param: ParamType,
        automation_rate: AutomationRate,
        default_value: f32,
        min_value: f32,
        max_value: f32,
    ) -> AudioParam<TH> {
        AudioParam {
            reflector_: Reflector::new(),
            context: Dom::from_ref(context),
            node,
            param,
            automation_rate: Cell::new(automation_rate),
            default_value,
            min_value,
            max_value,
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(
        window: &Window<TH>,
        context: &BaseAudioContext<TH>,
        node: NodeId,
        param: ParamType,
        automation_rate: AutomationRate,
        default_value: f32,
        min_value: f32,
        max_value: f32,
    ) -> DomRoot<AudioParam<TH>> {
        let audio_param = AudioParam::new_inherited(
            context,
            node,
            param,
            automation_rate,
            default_value,
            min_value,
            max_value,
        );
        reflect_dom_object(Box::new(audio_param), window, AudioParamBinding::Wrap)
    }

    fn message_node(&self, message: AudioNodeMessage) {
        self.context.audio_context_impl().message_node(self.node, message);
    }

    pub fn context(&self) -> &BaseAudioContext<TH> {
        &self.context
    }

    pub fn node_id(&self) -> NodeId {
        self.node
    }

    pub fn param_type(&self) -> ParamType {
        self.param
    }
}

impl<TH: TypeHolderTrait> AudioParamMethods<TH> for AudioParam<TH> {
    // https://webaudio.github.io/web-audio-api/#dom-audioparam-automationrate
    fn AutomationRate(&self) -> AutomationRate {
        self.automation_rate.get()
    }

    // https://webaudio.github.io/web-audio-api/#dom-audioparam-automationrate
    fn SetAutomationRate(&self, automation_rate: AutomationRate) {
        self.automation_rate.set(automation_rate);
        self.message_node(
            AudioNodeMessage::SetParamRate(self.param, automation_rate.into())
        );
    }

    // https://webaudio.github.io/web-audio-api/#dom-audioparam-value
    fn Value(&self) -> Finite<f32> {
        let (tx, rx) = mpsc::channel();
        self.message_node(
            AudioNodeMessage::GetParamValue(self.param, tx)
        );
        Finite::wrap(rx.recv().unwrap())
    }

    // https://webaudio.github.io/web-audio-api/#dom-audioparam-value
    fn SetValue(&self, value: Finite<f32>) {
        self.message_node(
            AudioNodeMessage::SetParam(self.param, UserAutomationEvent::SetValue(*value)),
        );
    }

    // https://webaudio.github.io/web-audio-api/#dom-audioparam-defaultvalue
    fn DefaultValue(&self) -> Finite<f32> {
        Finite::wrap(self.default_value)
    }

    // https://webaudio.github.io/web-audio-api/#dom-audioparam-minvalue
    fn MinValue(&self) -> Finite<f32> {
        Finite::wrap(self.min_value)
    }

    // https://webaudio.github.io/web-audio-api/#dom-audioparam-maxvalue
    fn MaxValue(&self) -> Finite<f32> {
        Finite::wrap(self.max_value)
    }

    // https://webaudio.github.io/web-audio-api/#dom-audioparam-setvalueattime
    fn SetValueAtTime(&self, value: Finite<f32>, start_time: Finite<f64>) -> DomRoot<AudioParam<TH>> {
        self.message_node(
            AudioNodeMessage::SetParam(
                self.param,
                UserAutomationEvent::SetValueAtTime(*value, *start_time),
            )
        );
        DomRoot::from_ref(self)
    }

    // https://webaudio.github.io/web-audio-api/#dom-audioparam-linearramptovalueattime
    fn LinearRampToValueAtTime(
        &self,
        value: Finite<f32>,
        end_time: Finite<f64>,
    ) -> DomRoot<AudioParam<TH>> {
        self.message_node(
            AudioNodeMessage::SetParam(
                self.param,
                UserAutomationEvent::RampToValueAtTime(RampKind::Linear, *value, *end_time),
            ),
        );
        DomRoot::from_ref(self)
    }

    // https://webaudio.github.io/web-audio-api/#dom-audioparam-exponentialramptovalueattime
    fn ExponentialRampToValueAtTime(
        &self,
        value: Finite<f32>,
        end_time: Finite<f64>,
    ) -> DomRoot<AudioParam<TH>> {
        self.message_node(
            AudioNodeMessage::SetParam(
                self.param,
                UserAutomationEvent::RampToValueAtTime(RampKind::Exponential, *value, *end_time),
            ),
        );
        DomRoot::from_ref(self)
    }

    // https://webaudio.github.io/web-audio-api/#dom-audioparam-settargetattime
    fn SetTargetAtTime(
        &self,
        target: Finite<f32>,
        start_time: Finite<f64>,
        time_constant: Finite<f32>,
    ) -> DomRoot<AudioParam<TH>> {
        self.message_node(
            AudioNodeMessage::SetParam(
                self.param,
                UserAutomationEvent::SetTargetAtTime(*target, *start_time, (*time_constant).into()),
            ),
        );
        DomRoot::from_ref(self)
    }

    // https://webaudio.github.io/web-audio-api/#dom-audioparam-cancelscheduledvalues
    fn CancelScheduledValues(&self, cancel_time: Finite<f64>) -> DomRoot<AudioParam<TH>> {
        self.message_node(
            AudioNodeMessage::SetParam(
                self.param,
                UserAutomationEvent::CancelScheduledValues(*cancel_time),
            ),
        );
        DomRoot::from_ref(self)
    }

    // https://webaudio.github.io/web-audio-api/#dom-audioparam-cancelandholdattime
    fn CancelAndHoldAtTime(&self, cancel_time: Finite<f64>) -> DomRoot<AudioParam<TH>> {
        self.message_node(
            AudioNodeMessage::SetParam(
                self.param,
                UserAutomationEvent::CancelAndHoldAtTime(*cancel_time),
            ),
        );
        DomRoot::from_ref(self)
    }
}

// https://webaudio.github.io/web-audio-api/#enumdef-automationrate
impl From<AutomationRate> for ParamRate {
    fn from(rate: AutomationRate) -> Self {
        match rate {
            AutomationRate::A_rate => ParamRate::ARate,
            AutomationRate::K_rate => ParamRate::KRate,
        }
    }
}
