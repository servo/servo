/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::baseaudiocontext::{BaseAudioContext, BaseAudioContextOptions};
use dom::bindings::codegen::Bindings::AudioContextBinding;
use dom::bindings::codegen::Bindings::AudioContextBinding::{AudioContextMethods, AudioContextOptions};
use dom::bindings::codegen::Bindings::AudioContextBinding::{AudioContextLatencyCategory, AudioTimestamp};
use dom::bindings::error::Fallible;
use dom::bindings::inheritance::Castable;
use dom::bindings::num::Finite;
use dom::bindings::reflector::{DomObject, reflect_dom_object};
use dom::bindings::root::DomRoot;
use dom::globalscope::GlobalScope;
use dom::promise::Promise;
use dom::window::Window;
use dom_struct::dom_struct;
use servo_media::audio::graph::{LatencyCategory, RealTimeAudioGraphOptions};
use std::rc::Rc;

#[dom_struct]
pub struct AudioContext {
    context: BaseAudioContext,
    latency_hint: AudioContextLatencyCategory,
    /// https://webaudio.github.io/web-audio-api/#dom-audiocontext-baselatency
    base_latency: f64,
    /// https://webaudio.github.io/web-audio-api/#dom-audiocontext-outputlatency
    output_latency: f64,
}

impl AudioContext {
    #[allow(unrooted_must_root)]
    // https://webaudio.github.io/web-audio-api/#AudioContext-constructors
    fn new_inherited(global: &GlobalScope, options: &AudioContextOptions) -> AudioContext {
        // Steps 1-3.
        let context = BaseAudioContext::new_inherited(global, BaseAudioContextOptions::AudioContext(options.into()));

        // Step 4.1.
        let latency_hint = options.latencyHint;

        // Step 4.2. The sample rate is set during the creation of the BaseAudioContext.
        // servo-media takes care of setting the default sample rate of the output device
        // and of resampling the audio output if needed.

        // Step 5.
        if context.is_allowed_to_start() {
            // Step 6.
            context.resume();
        }

        AudioContext {
            context,
            latency_hint,
            base_latency: 0., // TODO
            output_latency: 0., // TODO
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(global: &GlobalScope,
               options: &AudioContextOptions) -> DomRoot<AudioContext> {
        let context = AudioContext::new_inherited(
            global,
            options,
            ); // TODO
        reflect_dom_object(Box::new(context), global, AudioContextBinding::Wrap)
    }

    // https://webaudio.github.io/web-audio-api/#AudioContext-constructors
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

impl From<AudioContextLatencyCategory> for LatencyCategory {
    fn from(category: AudioContextLatencyCategory) -> Self {
        match category {
            AudioContextLatencyCategory::Balanced => LatencyCategory::Balanced,
            AudioContextLatencyCategory::Interactive => LatencyCategory::Interactive,
            AudioContextLatencyCategory::Playback => LatencyCategory::Playback,
        }
    }
}

impl<'a> From<&'a AudioContextOptions> for RealTimeAudioGraphOptions {
    fn from(options: &AudioContextOptions) -> Self {
        Self {
            sample_rate: *options.sampleRate.unwrap_or(Finite::wrap(48000.)),
            latency_hint: options.latencyHint.into(),
        }
    }
}
