/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::AudioParamBinding;
use dom::bindings::codegen::Bindings::AudioParamBinding::AudioParamMethods;
use dom::bindings::num::Finite;
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::bindings::root::DomRoot;
use dom::globalscope::GlobalScope;
use dom_struct::dom_struct;
use std::cell::Cell;

#[dom_struct]
pub struct AudioParam {
    reflector_: Reflector,
    value: Cell<f32>,
    default_value: f32,
    min_value: f32,
    max_value: f32,
}

impl AudioParam {
    pub fn new_inherited(default_value: f32,
                         min_value: f32,
                         max_value: f32) -> AudioParam {
        AudioParam {
            reflector_: Reflector::new(),
            value: Cell::new(default_value),
            default_value,
            min_value,
            max_value,
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(global: &GlobalScope,
               default_value: f32,
               min_value: f32,
               max_value: f32) -> DomRoot<AudioParam> {
        let audio_param = AudioParam::new_inherited(default_value, min_value, max_value);
        reflect_dom_object(Box::new(audio_param), global, AudioParamBinding::Wrap)
    }
}

impl AudioParamMethods for AudioParam {
    fn Value(&self) -> Finite<f32> {
        Finite::wrap(self.value.get())
    }

    fn SetValue(&self, value: Finite<f32>) {
        self.value.set(*value);
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
}
