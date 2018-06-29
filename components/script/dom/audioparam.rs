/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::AudioParamBinding;
use dom::bindings::codegen::Bindings::AudioParamBinding::{AudioParamMethods, AutomationRate};
use dom::bindings::num::Finite;
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::bindings::root::DomRoot;
use dom::bindings::trace::JSTraceable;
use dom::window::Window;
use dom_struct::dom_struct;
use malloc_size_of::MallocSizeOf;
use servo_media::audio::param::RampKind;
use std::cell::Cell;

pub trait AudioParamImpl: JSTraceable + MallocSizeOf {
    fn set_value(&self, value: f32);
    fn set_value_at_time(&self, value: f32, start_time: f64);
    fn ramp_to_value_at_time(&self, ramp_kind: RampKind, value: f32, end_time: f64);
    fn set_target_at_time(&self, value: f32, start_time: f64, time_constant: f32);
    fn cancel_scheduled_values(&self, cancel_time: f64);
    fn cancel_and_hold_at_time(&self, cancel_time: f64);
}

#[dom_struct]
pub struct AudioParam {
    reflector_: Reflector,
    param_impl: Box<AudioParamImpl>,
    automation_rate: Cell<AutomationRate>,
    default_value: f32,
    min_value: f32,
    max_value: f32,
}

impl AudioParam {
    pub fn new_inherited(param_impl: Box<AudioParamImpl>,
                         automation_rate: AutomationRate,
                         default_value: f32,
                         min_value: f32,
                         max_value: f32) -> AudioParam {
        AudioParam {
            reflector_: Reflector::new(),
            param_impl,
            automation_rate: Cell::new(automation_rate),
            default_value,
            min_value,
            max_value,
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(window: &Window,
               param_impl: Box<AudioParamImpl>,
               automation_rate: AutomationRate,
               default_value: f32,
               min_value: f32,
               max_value: f32) -> DomRoot<AudioParam> {
        let audio_param = AudioParam::new_inherited(param_impl, automation_rate,
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
        self.param_impl.set_value(*value);
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
        self.param_impl.set_value_at_time(*value, *start_time);
        DomRoot::from_ref(self)
    }

    fn LinearRampToValueAtTime(&self, value: Finite<f32>, end_time: Finite<f64>)
        -> DomRoot<AudioParam>
    {
        self.param_impl.ramp_to_value_at_time(RampKind::Linear, *value, *end_time);
        DomRoot::from_ref(self)
    }

    fn ExponentialRampToValueAtTime(&self, value: Finite<f32>, end_time: Finite<f64>)
        -> DomRoot<AudioParam>
    {
        self.param_impl.ramp_to_value_at_time(RampKind::Exponential, *value, *end_time);
        DomRoot::from_ref(self)
    }

    fn SetTargetAtTime(&self, target: Finite<f32>, start_time: Finite<f64>, time_constant: Finite<f32>)
        -> DomRoot<AudioParam>
    {
        self.param_impl.set_target_at_time(*target, *start_time, *time_constant);
        DomRoot::from_ref(self)
    }

    fn CancelScheduledValues(&self, cancel_time: Finite<f64>) -> DomRoot<AudioParam> {
        self.param_impl.cancel_scheduled_values(*cancel_time);
        DomRoot::from_ref(self)
    }

    fn CancelAndHoldAtTime(&self, cancel_time: Finite<f64>) -> DomRoot<AudioParam> {
        self.param_impl.cancel_and_hold_at_time(*cancel_time);
        DomRoot::from_ref(self)
    }
}
