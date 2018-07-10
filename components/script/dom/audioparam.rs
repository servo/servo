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
use servo_media::audio::param::{ParamType, RampKind, UserAutomationEvent};
use std::cell::Cell;

#[dom_struct]
pub struct AudioParam {
    reflector_: Reflector,
    context: Dom<BaseAudioContext>,
    #[ignore_malloc_size_of = "servo_media"]
    node: NodeId,
    #[ignore_malloc_size_of = "servo_media"]
    param: ParamType,
    automation_rate: Cell<AutomationRate>,
    default_value: f32,
    min_value: f32,
    max_value: f32,
}

impl AudioParam {
    pub fn new_inherited(context: &BaseAudioContext,
                         node: NodeId,
                         param: ParamType,
                         automation_rate: AutomationRate,
                         default_value: f32,
                         min_value: f32,
                         max_value: f32) -> AudioParam {
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
    pub fn new(window: &Window,
               context: &BaseAudioContext,
               node: NodeId,
               param: ParamType,
               automation_rate: AutomationRate,
               default_value: f32,
               min_value: f32,
               max_value: f32) -> DomRoot<AudioParam> {
        let audio_param = AudioParam::new_inherited(context, node, param, automation_rate,
                                                    default_value, min_value, max_value);
        reflect_dom_object(Box::new(audio_param), window, AudioParamBinding::Wrap)
    }
}

impl AudioParamMethods for AudioParam {
    fn AutomationRate(&self) -> AutomationRate {
        self.automation_rate.get()
    }

    fn SetAutomationRate(&self, automation_rate: AutomationRate) {
        self.automation_rate.set(automation_rate);
        // XXX set servo-media param automation rate
    }

    fn Value(&self) -> Finite<f32> {
        // XXX
        Finite::wrap(0.)
    }

    fn SetValue(&self, value: Finite<f32>) {
        self.context.audio_context_impl()
            .message_node(self.node,
                          AudioNodeMessage::SetParam(self.param,
                            UserAutomationEvent::SetValue(
                                *value
                            )
                          )
                         );
    }

    fn DefaultValue(&self) -> Finite<f32> {
        Finite::wrap(self.default_value)
    }

    fn MinValue(&self) -> Finite<f32> {
        Finite::wrap(self.min_value)
    }

    fn MaxValue(&self) -> Finite<f32> {
        Finite::wrap(self.max_value)
    }

    fn SetValueAtTime(&self, value: Finite<f32>, start_time: Finite<f64>)
        -> DomRoot<AudioParam>
    {
        self.context.audio_context_impl()
            .message_node(self.node,
                          AudioNodeMessage::SetParam(self.param,
                            UserAutomationEvent::SetValueAtTime(
                                *value, *start_time
                            )
                          )
                         );
        DomRoot::from_ref(self)
    }

    fn LinearRampToValueAtTime(&self, value: Finite<f32>, end_time: Finite<f64>)
        -> DomRoot<AudioParam>
    {
        self.context.audio_context_impl()
            .message_node(self.node,
                          AudioNodeMessage::SetParam(self.param,
                            UserAutomationEvent::RampToValueAtTime(
                                RampKind::Linear, *value, *end_time
                            )
                          )
                         );
        DomRoot::from_ref(self)
    }

    fn ExponentialRampToValueAtTime(&self, value: Finite<f32>, end_time: Finite<f64>)
        -> DomRoot<AudioParam>
    {
        self.context.audio_context_impl()
            .message_node(self.node,
                          AudioNodeMessage::SetParam(self.param,
                            UserAutomationEvent::RampToValueAtTime(
                                RampKind::Exponential, *value, *end_time
                            )
                          )
                         );
        DomRoot::from_ref(self)
    }

    fn SetTargetAtTime(&self, target: Finite<f32>, start_time: Finite<f64>, time_constant: Finite<f32>)
        -> DomRoot<AudioParam>
    {
        self.context.audio_context_impl()
            .message_node(self.node,
                          AudioNodeMessage::SetParam(self.param,
                            UserAutomationEvent::SetTargetAtTime(
                                *target, *start_time, (*time_constant).into()
                            )
                          )
                         );
        DomRoot::from_ref(self)
    }

    fn CancelScheduledValues(&self, cancel_time: Finite<f64>) -> DomRoot<AudioParam> {
        self.context.audio_context_impl()
            .message_node(self.node,
                          AudioNodeMessage::SetParam(self.param,
                            UserAutomationEvent::CancelScheduledValues(
                                *cancel_time
                            )
                          )
                         );
        DomRoot::from_ref(self)
    }

    fn CancelAndHoldAtTime(&self, cancel_time: Finite<f64>) -> DomRoot<AudioParam> {
        self.context.audio_context_impl()
            .message_node(self.node,
                          AudioNodeMessage::SetParam(self.param,
                            UserAutomationEvent::CancelAndHoldAtTime(
                                *cancel_time
                            )
                          )
                         );
        DomRoot::from_ref(self)
    }
}
