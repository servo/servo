/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::baseaudiocontext::BaseAudioContext;
use dom::bindings::codegen::Bindings::AudioContextBinding;
use dom::bindings::codegen::Bindings::AudioContextBinding::AudioContextMethods;
use dom::bindings::codegen::Bindings::AudioContextBinding::{AudioContextOptions, AudioTimestamp};
use dom::bindings::error::Fallible;
use dom::bindings::inheritance::Castable;
use dom::bindings::num::Finite;
use dom::bindings::reflector::{DomObject, reflect_dom_object};
use dom::bindings::root::DomRoot;
use dom::globalscope::GlobalScope;
use dom::promise::Promise;
use dom::window::Window;
use dom_struct::dom_struct;
use std::rc::Rc;

#[dom_struct]
pub struct AudioContext {
    context: BaseAudioContext,
    base_latency: f64,
    output_latency: f64,
}

impl AudioContext {
    fn new_inherited(global: &GlobalScope, sample_rate: f32) -> AudioContext {
        AudioContext {
            context: BaseAudioContext::new_inherited(global, 2 /* channel_count */, sample_rate),
            base_latency: 0., // TODO
            output_latency: 0., // TODO
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(global: &GlobalScope,
               options: &AudioContextOptions) -> DomRoot<AudioContext> {
        let context = AudioContext::new_inherited(
            global,
            *options.sampleRate.unwrap_or(Finite::wrap(0.)),
            ); // TODO
        reflect_dom_object(Box::new(context), global, AudioContextBinding::Wrap)
    }

    pub fn Constructor(window: &Window,
                       options: &AudioContextOptions) -> Fallible<DomRoot<AudioContext>> {
        let global = window.upcast::<GlobalScope>();
        Ok(AudioContext::new(global, options))
    }
}

impl AudioContextMethods for AudioContext {
    fn BaseLatency(&self) -> Finite<f64> {
        Finite::wrap(self.base_latency)
    }

    fn OutputLatency(&self) -> Finite<f64> {
        Finite::wrap(self.output_latency)
    }

    fn GetOutputTimestamp(&self) -> AudioTimestamp {
        // TODO
        AudioTimestamp {
            contextTime: Some(Finite::wrap(0.)),
            performanceTime: Some(Finite::wrap(0.)),
        }
    }

    #[allow(unrooted_must_root)]
    fn Suspend(&self) -> Rc<Promise> {
        // TODO
        Promise::new(&self.global())
    }

    #[allow(unrooted_must_root)]
    fn Close(&self) -> Rc<Promise> {
        // TODO
        Promise::new(&self.global())
    }
}
