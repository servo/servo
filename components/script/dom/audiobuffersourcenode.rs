/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::audiobuffer::AudioBuffer;
use dom::audioparam::AudioParam;
use dom::audioscheduledsourcenode::AudioScheduledSourceNode;
use dom::baseaudiocontext::BaseAudioContext;
use dom::bindings::codegen::Bindings::AudioBufferSourceNodeBinding;
use dom::bindings::codegen::Bindings::AudioBufferSourceNodeBinding::AudioBufferSourceNodeMethods;
use dom::bindings::codegen::Bindings::AudioBufferSourceNodeBinding::AudioBufferSourceOptions;
use dom::bindings::codegen::Bindings::AudioNodeBinding::{ChannelCountMode, ChannelInterpretation};
use dom::bindings::codegen::Bindings::AudioNodeBinding::AudioNodeOptions;
use dom::bindings::codegen::Bindings::AudioParamBinding::AutomationRate;
use dom::bindings::codegen::Bindings::AudioScheduledSourceNodeBinding::AudioScheduledSourceNodeMethods;
use dom::bindings::error::{Error, Fallible};
use dom::bindings::inheritance::Castable;
use dom::bindings::num::Finite;
use dom::bindings::reflector::reflect_dom_object;
use dom::bindings::root::{Dom, DomRoot, MutNullableDom};
use dom::window::Window;
use dom_struct::dom_struct;
use servo_media::audio::buffer_source_node::AudioBufferSourceNodeMessage;
use servo_media::audio::buffer_source_node::AudioBufferSourceNodeOptions;
use servo_media::audio::node::{AudioNodeMessage, AudioNodeInit};
use servo_media::audio::param::ParamType;
use std::cell::Cell;
use std::f32;

#[dom_struct]
pub struct AudioBufferSourceNode {
    source_node: AudioScheduledSourceNode,
    buffer: MutNullableDom<AudioBuffer>,
    playback_rate: Dom<AudioParam>,
    detune: Dom<AudioParam>,
    loop_enabled: Cell<bool>,
    loop_start: Cell<f64>,
    loop_end: Cell<f64>,
}

impl AudioBufferSourceNode {
    #[allow(unrooted_must_root)]
    fn new_inherited(
        window: &Window,
        context: &BaseAudioContext,
        options: &AudioBufferSourceOptions,
    ) -> AudioBufferSourceNode {
        let mut node_options = AudioNodeOptions::empty();
        node_options.channelCount = Some(2);
        node_options.channelCountMode = Some(ChannelCountMode::Max);
        node_options.channelInterpretation = Some(ChannelInterpretation::Speakers);
        let source_node = AudioScheduledSourceNode::new_inherited(
            AudioNodeInit::AudioBufferSourceNode(options.into()),
            context,
            &node_options,
            0, /* inputs */
            1, /* outputs */
        );
        let node_id = source_node.node().node_id();
        let playback_rate = AudioParam::new(
            &window,
            context,
            node_id,
            ParamType::PlaybackRate,
            AutomationRate::K_rate,
            *options.playbackRate,
            f32::MIN,
            f32::MAX,
        );
        let detune = AudioParam::new(
            &window,
            context,
            node_id,
            ParamType::Detune,
            AutomationRate::K_rate,
            *options.detune,
            f32::MIN,
            f32::MAX,
        );
        AudioBufferSourceNode {
            source_node,
            buffer: Default::default(),
            playback_rate: Dom::from_ref(&playback_rate),
            detune: Dom::from_ref(&detune),
            loop_enabled: Cell::new(options.loop_),
            loop_start: Cell::new(*options.loopStart),
            loop_end: Cell::new(*options.loopEnd),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(
        window: &Window,
        context: &BaseAudioContext,
        options: &AudioBufferSourceOptions,
    ) -> DomRoot<AudioBufferSourceNode> {
        let node = AudioBufferSourceNode::new_inherited(window, context, options);
        reflect_dom_object(Box::new(node), window, AudioBufferSourceNodeBinding::Wrap)
    }

    pub fn Constructor(
        window: &Window,
        context: &BaseAudioContext,
        options: &AudioBufferSourceOptions,
    ) -> Fallible<DomRoot<AudioBufferSourceNode>> {
        Ok(AudioBufferSourceNode::new(window, context, options))
    }
}

impl AudioBufferSourceNodeMethods for AudioBufferSourceNode {
    // https://webaudio.github.io/web-audio-api/#dom-audiobuffersourcenode-buffer
    fn GetBuffer(&self) -> Fallible<Option<DomRoot<AudioBuffer>>> {
        Ok(self.buffer.get())
    }

    // https://webaudio.github.io/web-audio-api/#dom-audiobuffersourcenode-buffer
    fn SetBuffer(&self, new_buffer: Option<&AudioBuffer>) -> Fallible<()> {
        if new_buffer.is_some() && self.buffer.get().is_some() {
            return Err(Error::InvalidState);
        }

        self.buffer.set(new_buffer);

        if self.source_node.started() {
            if let Some(buffer) = self.buffer.get() {
                let buffer = buffer.acquire_contents();
                self.source_node
                    .node()
                    .message(AudioNodeMessage::AudioBufferSourceNode(
                        AudioBufferSourceNodeMessage::SetBuffer(buffer),
                    ));
            }
        }

        Ok(())
    }

    // https://webaudio.github.io/web-audio-api/#dom-audiobuffersourcenode-playbackrate
    fn PlaybackRate(&self) -> DomRoot<AudioParam> {
        DomRoot::from_ref(&self.playback_rate)
    }

    // https://webaudio.github.io/web-audio-api/#dom-audiobuffersourcenode-detune
    fn Detune(&self) -> DomRoot<AudioParam> {
        DomRoot::from_ref(&self.detune)
    }

    // https://webaudio.github.io/web-audio-api/#dom-audiobuffersourcenode-loop
    fn Loop(&self) -> bool {
        self.loop_enabled.get()
    }

    // https://webaudio.github.io/web-audio-api/#dom-audiobuffersourcenode-loop
    fn SetLoop(&self, should_loop: bool) {
        self.loop_enabled.set(should_loop);
    }

    // https://webaudio.github.io/web-audio-api/#dom-audiobuffersourcenode-loopstart
    fn LoopStart(&self) -> Finite<f64> {
        Finite::wrap(self.loop_start.get())
    }

    // https://webaudio.github.io/web-audio-api/#dom-audiobuffersourcenode-loopstart
    fn SetLoopStart(&self, loop_start: Finite<f64>) {
        self.loop_start.set(*loop_start);
    }

    // https://webaudio.github.io/web-audio-api/#dom-audiobuffersourcenode-loopend
    fn LoopEnd(&self) -> Finite<f64> {
        Finite::wrap(self.loop_end.get())
    }

    // https://webaudio.github.io/web-audio-api/#dom-audiobuffersourcenode-loopend
    fn SetLoopEnd(&self, loop_end: Finite<f64>) {
        self.loop_end.set(*loop_end)
    }

    // https://webaudio.github.io/web-audio-api/#dom-audiobuffersourcenode-start
    fn Start(
        &self,
        when: Finite<f64>,
        offset: Option<Finite<f64>>,
        duration: Option<Finite<f64>>,
    ) -> Fallible<()> {
        if *when < 0. {
            return Err(Error::Range("'when' must be a positive value".to_owned()));
        }

        if let Some(offset) = offset {
            if *offset < 0. {
                return Err(Error::Range("'offset' must be a positive value".to_owned()));
            }
        }

        if let Some(duration) = duration {
            if *duration < 0. {
                return Err(Error::Range(
                    "'duration' must be a positive value".to_owned(),
                ));
            }
        }

        if let Some(buffer) = self.buffer.get() {
            let buffer = buffer.acquire_contents();
            self.source_node
                .node()
                .message(AudioNodeMessage::AudioBufferSourceNode(
                    AudioBufferSourceNodeMessage::SetBuffer(buffer),
                ));
        }
        self.source_node
            .upcast::<AudioScheduledSourceNode>()
            .Start(when)
    }
}

impl<'a> From<&'a AudioBufferSourceOptions> for AudioBufferSourceNodeOptions {
    fn from(options: &'a AudioBufferSourceOptions) -> Self {
        Self {
            buffer: None,
            detune: *options.detune,
            loop_enabled: options.loop_,
            loop_end: Some(*options.loopEnd),
            loop_start: Some(*options.loopStart),
            playback_rate: *options.playbackRate,
        }
    }
}
