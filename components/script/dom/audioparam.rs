/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::sync::mpsc;

use dom_struct::dom_struct;
use servo_media::audio::graph::NodeId;
use servo_media::audio::node::{AudioNodeMessage, AudioNodeType};
use servo_media::audio::param::{ParamRate, ParamType, RampKind, UserAutomationEvent};

use crate::conversions::Convert;
use crate::dom::baseaudiocontext::BaseAudioContext;
use crate::dom::bindings::codegen::Bindings::AudioParamBinding::{
    AudioParamMethods, AutomationRate,
};
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::num::Finite;
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::window::Window;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct AudioParam {
    reflector_: Reflector,
    context: Dom<BaseAudioContext>,
    #[ignore_malloc_size_of = "servo_media"]
    #[no_trace]
    node: NodeId,
    #[ignore_malloc_size_of = "servo_media"]
    #[no_trace]
    node_type: AudioNodeType,
    #[ignore_malloc_size_of = "servo_media"]
    #[no_trace]
    param: ParamType,
    automation_rate: Cell<AutomationRate>,
    default_value: f32,
    min_value: f32,
    max_value: f32,
}

impl AudioParam {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new_inherited(
        context: &BaseAudioContext,
        node: NodeId,
        node_type: AudioNodeType,
        param: ParamType,
        automation_rate: AutomationRate,
        default_value: f32,
        min_value: f32,
        max_value: f32,
    ) -> AudioParam {
        AudioParam {
            reflector_: Reflector::new(),
            context: Dom::from_ref(context),
            node,
            node_type,
            param,
            automation_rate: Cell::new(automation_rate),
            default_value,
            min_value,
            max_value,
        }
    }

    #[allow(clippy::too_many_arguments)]
    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    pub(crate) fn new(
        window: &Window,
        context: &BaseAudioContext,
        node: NodeId,
        node_type: AudioNodeType,
        param: ParamType,
        automation_rate: AutomationRate,
        default_value: f32,
        min_value: f32,
        max_value: f32,
        can_gc: CanGc,
    ) -> DomRoot<AudioParam> {
        let audio_param = AudioParam::new_inherited(
            context,
            node,
            node_type,
            param,
            automation_rate,
            default_value,
            min_value,
            max_value,
        );
        reflect_dom_object(Box::new(audio_param), window, can_gc)
    }

    fn message_node(&self, message: AudioNodeMessage) {
        self.context
            .audio_context_impl()
            .lock()
            .unwrap()
            .message_node(self.node, message);
    }

    pub(crate) fn context(&self) -> &BaseAudioContext {
        &self.context
    }

    pub(crate) fn node_id(&self) -> NodeId {
        self.node
    }

    pub(crate) fn param_type(&self) -> ParamType {
        self.param
    }
}

impl AudioParamMethods<crate::DomTypeHolder> for AudioParam {
    // https://webaudio.github.io/web-audio-api/#dom-audioparam-automationrate
    fn AutomationRate(&self) -> AutomationRate {
        self.automation_rate.get()
    }

    // https://webaudio.github.io/web-audio-api/#dom-audioparam-automationrate
    fn SetAutomationRate(&self, automation_rate: AutomationRate) -> Fallible<()> {
        // > AudioBufferSourceNode
        // > The AudioParams playbackRate and detune MUST be "k-rate". An InvalidStateError must be
        // > thrown if the rate is changed to "a-rate".
        if automation_rate == AutomationRate::A_rate &&
            self.node_type == AudioNodeType::AudioBufferSourceNode &&
            (self.param == ParamType::Detune || self.param == ParamType::PlaybackRate)
        {
            return Err(Error::InvalidState);
        }

        self.automation_rate.set(automation_rate);
        self.message_node(AudioNodeMessage::SetParamRate(
            self.param,
            automation_rate.convert(),
        ));

        Ok(())
    }

    // https://webaudio.github.io/web-audio-api/#dom-audioparam-value
    fn Value(&self) -> Finite<f32> {
        let (tx, rx) = mpsc::channel();
        self.message_node(AudioNodeMessage::GetParamValue(self.param, tx));
        Finite::wrap(rx.recv().unwrap())
    }

    // https://webaudio.github.io/web-audio-api/#dom-audioparam-value
    fn SetValue(&self, value: Finite<f32>) {
        self.message_node(AudioNodeMessage::SetParam(
            self.param,
            UserAutomationEvent::SetValue(*value),
        ));
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
    fn SetValueAtTime(
        &self,
        value: Finite<f32>,
        start_time: Finite<f64>,
    ) -> Fallible<DomRoot<AudioParam>> {
        if *start_time < 0. {
            return Err(Error::Range(format!(
                "start time {} should not be negative",
                *start_time
            )));
        }
        self.message_node(AudioNodeMessage::SetParam(
            self.param,
            UserAutomationEvent::SetValueAtTime(*value, *start_time),
        ));
        Ok(DomRoot::from_ref(self))
    }

    // https://webaudio.github.io/web-audio-api/#dom-audioparam-linearramptovalueattime
    fn LinearRampToValueAtTime(
        &self,
        value: Finite<f32>,
        end_time: Finite<f64>,
    ) -> Fallible<DomRoot<AudioParam>> {
        if *end_time < 0. {
            return Err(Error::Range(format!(
                "end time {} should not be negative",
                *end_time
            )));
        }
        self.message_node(AudioNodeMessage::SetParam(
            self.param,
            UserAutomationEvent::RampToValueAtTime(RampKind::Linear, *value, *end_time),
        ));
        Ok(DomRoot::from_ref(self))
    }

    // https://webaudio.github.io/web-audio-api/#dom-audioparam-exponentialramptovalueattime
    fn ExponentialRampToValueAtTime(
        &self,
        value: Finite<f32>,
        end_time: Finite<f64>,
    ) -> Fallible<DomRoot<AudioParam>> {
        if *end_time < 0. {
            return Err(Error::Range(format!(
                "end time {} should not be negative",
                *end_time
            )));
        }
        if *value == 0. {
            return Err(Error::Range(format!(
                "target value {} should not be 0",
                *value
            )));
        }
        self.message_node(AudioNodeMessage::SetParam(
            self.param,
            UserAutomationEvent::RampToValueAtTime(RampKind::Exponential, *value, *end_time),
        ));
        Ok(DomRoot::from_ref(self))
    }

    // https://webaudio.github.io/web-audio-api/#dom-audioparam-settargetattime
    fn SetTargetAtTime(
        &self,
        target: Finite<f32>,
        start_time: Finite<f64>,
        time_constant: Finite<f32>,
    ) -> Fallible<DomRoot<AudioParam>> {
        if *start_time < 0. {
            return Err(Error::Range(format!(
                "start time {} should not be negative",
                *start_time
            )));
        }
        if *time_constant < 0. {
            return Err(Error::Range(format!(
                "time constant {} should not be negative",
                *time_constant
            )));
        }
        self.message_node(AudioNodeMessage::SetParam(
            self.param,
            UserAutomationEvent::SetTargetAtTime(*target, *start_time, (*time_constant).into()),
        ));
        Ok(DomRoot::from_ref(self))
    }

    // https://webaudio.github.io/web-audio-api/#dom-audioparam-setvaluecurveattime
    fn SetValueCurveAtTime(
        &self,
        values: Vec<Finite<f32>>,
        start_time: Finite<f64>,
        end_time: Finite<f64>,
    ) -> Fallible<DomRoot<AudioParam>> {
        if *start_time < 0. {
            return Err(Error::Range(format!(
                "start time {} should not be negative",
                *start_time
            )));
        }
        if values.len() < 2. as usize {
            return Err(Error::InvalidState);
        }

        if *end_time < 0. {
            return Err(Error::Range(format!(
                "end time {} should not be negative",
                *end_time
            )));
        }
        self.message_node(AudioNodeMessage::SetParam(
            self.param,
            UserAutomationEvent::SetValueCurveAtTime(
                values.into_iter().map(|v| *v).collect(),
                *start_time,
                *end_time,
            ),
        ));
        Ok(DomRoot::from_ref(self))
    }

    // https://webaudio.github.io/web-audio-api/#dom-audioparam-cancelscheduledvalues
    fn CancelScheduledValues(&self, cancel_time: Finite<f64>) -> Fallible<DomRoot<AudioParam>> {
        if *cancel_time < 0. {
            return Err(Error::Range(format!(
                "cancel time {} should not be negative",
                *cancel_time
            )));
        }
        self.message_node(AudioNodeMessage::SetParam(
            self.param,
            UserAutomationEvent::CancelScheduledValues(*cancel_time),
        ));
        Ok(DomRoot::from_ref(self))
    }

    // https://webaudio.github.io/web-audio-api/#dom-audioparam-cancelandholdattime
    fn CancelAndHoldAtTime(&self, cancel_time: Finite<f64>) -> Fallible<DomRoot<AudioParam>> {
        if *cancel_time < 0. {
            return Err(Error::Range(format!(
                "cancel time {} should not be negative",
                *cancel_time
            )));
        }
        self.message_node(AudioNodeMessage::SetParam(
            self.param,
            UserAutomationEvent::CancelAndHoldAtTime(*cancel_time),
        ));
        Ok(DomRoot::from_ref(self))
    }
}

// https://webaudio.github.io/web-audio-api/#enumdef-automationrate
impl Convert<ParamRate> for AutomationRate {
    fn convert(self) -> ParamRate {
        match self {
            AutomationRate::A_rate => ParamRate::ARate,
            AutomationRate::K_rate => ParamRate::KRate,
        }
    }
}
