/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::f32;

use dom_struct::dom_struct;
use js::rust::HandleObject;
use servo_media::audio::buffer_source_node::{
    AudioBufferSourceNodeMessage, AudioBufferSourceNodeOptions,
};
use servo_media::audio::node::{AudioNodeInit, AudioNodeMessage, AudioNodeType};
use servo_media::audio::param::ParamType;

use crate::dom::audiobuffer::AudioBuffer;
use crate::dom::audioparam::AudioParam;
use crate::dom::audioscheduledsourcenode::AudioScheduledSourceNode;
use crate::dom::baseaudiocontext::BaseAudioContext;
use crate::dom::bindings::codegen::Bindings::AudioBufferSourceNodeBinding::{
    AudioBufferSourceNodeMethods, AudioBufferSourceOptions,
};
use crate::dom::bindings::codegen::Bindings::AudioParamBinding::AutomationRate;
use crate::dom::bindings::codegen::Bindings::AudioScheduledSourceNodeBinding::AudioScheduledSourceNodeMethods;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::num::Finite;
use crate::dom::bindings::reflector::reflect_dom_object_with_proto;
use crate::dom::bindings::root::{Dom, DomRoot, MutNullableDom};
use crate::dom::window::Window;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct AudioBufferSourceNode {
    source_node: AudioScheduledSourceNode,
    buffer: MutNullableDom<AudioBuffer>,
    buffer_set: Cell<bool>,
    playback_rate: Dom<AudioParam>,
    detune: Dom<AudioParam>,
    loop_enabled: Cell<bool>,
    loop_start: Cell<f64>,
    loop_end: Cell<f64>,
}

impl AudioBufferSourceNode {
    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    fn new_inherited(
        window: &Window,
        context: &BaseAudioContext,
        options: &AudioBufferSourceOptions,
    ) -> Fallible<AudioBufferSourceNode> {
        let node_options = Default::default();
        let source_node = AudioScheduledSourceNode::new_inherited(
            AudioNodeInit::AudioBufferSourceNode(options.into()),
            context,
            node_options,
            0, /* inputs */
            1, /* outputs */
        )?;
        let node_id = source_node.node().node_id();
        let playback_rate = AudioParam::new(
            window,
            context,
            node_id,
            AudioNodeType::AudioBufferSourceNode,
            ParamType::PlaybackRate,
            AutomationRate::K_rate,
            *options.playbackRate,
            f32::MIN,
            f32::MAX,
        );
        let detune = AudioParam::new(
            window,
            context,
            node_id,
            AudioNodeType::AudioBufferSourceNode,
            ParamType::Detune,
            AutomationRate::K_rate,
            *options.detune,
            f32::MIN,
            f32::MAX,
        );
        let node = AudioBufferSourceNode {
            source_node,
            buffer: Default::default(),
            buffer_set: Cell::new(false),
            playback_rate: Dom::from_ref(&playback_rate),
            detune: Dom::from_ref(&detune),
            loop_enabled: Cell::new(options.loop_),
            loop_start: Cell::new(*options.loopStart),
            loop_end: Cell::new(*options.loopEnd),
        };
        if let Some(Some(ref buffer)) = options.buffer {
            node.SetBuffer(Some(buffer))?;
        }
        Ok(node)
    }

    pub(crate) fn new(
        window: &Window,
        context: &BaseAudioContext,
        options: &AudioBufferSourceOptions,
        can_gc: CanGc,
    ) -> Fallible<DomRoot<AudioBufferSourceNode>> {
        Self::new_with_proto(window, None, context, options, can_gc)
    }

    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    fn new_with_proto(
        window: &Window,
        proto: Option<HandleObject>,
        context: &BaseAudioContext,
        options: &AudioBufferSourceOptions,
        can_gc: CanGc,
    ) -> Fallible<DomRoot<AudioBufferSourceNode>> {
        let node = AudioBufferSourceNode::new_inherited(window, context, options)?;
        Ok(reflect_dom_object_with_proto(
            Box::new(node),
            window,
            proto,
            can_gc,
        ))
    }
}

impl AudioBufferSourceNodeMethods<crate::DomTypeHolder> for AudioBufferSourceNode {
    // https://webaudio.github.io/web-audio-api/#dom-audiobuffersourcenode-audiobuffersourcenode
    fn Constructor(
        window: &Window,
        proto: Option<HandleObject>,
        can_gc: CanGc,
        context: &BaseAudioContext,
        options: &AudioBufferSourceOptions,
    ) -> Fallible<DomRoot<AudioBufferSourceNode>> {
        AudioBufferSourceNode::new_with_proto(window, proto, context, options, can_gc)
    }

    // https://webaudio.github.io/web-audio-api/#dom-audiobuffersourcenode-buffer
    fn GetBuffer(&self) -> Fallible<Option<DomRoot<AudioBuffer>>> {
        Ok(self.buffer.get())
    }

    // https://webaudio.github.io/web-audio-api/#dom-audiobuffersourcenode-buffer
    fn SetBuffer(&self, new_buffer: Option<&AudioBuffer>) -> Fallible<()> {
        if new_buffer.is_some() {
            if self.buffer_set.get() {
                // Step 2.
                return Err(Error::InvalidState);
            }
            // Step 3.
            self.buffer_set.set(true);
        }

        // Step 4.
        self.buffer.set(new_buffer);

        // Step 5.
        if self.source_node.has_start() {
            if let Some(buffer) = self.buffer.get() {
                let buffer = buffer.get_channels();
                if buffer.is_some() {
                    self.source_node
                        .node()
                        .message(AudioNodeMessage::AudioBufferSourceNode(
                            AudioBufferSourceNodeMessage::SetBuffer((*buffer).clone()),
                        ));
                }
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
        let msg = AudioNodeMessage::AudioBufferSourceNode(
            AudioBufferSourceNodeMessage::SetLoopEnabled(should_loop),
        );
        self.source_node.node().message(msg);
    }

    // https://webaudio.github.io/web-audio-api/#dom-audiobuffersourcenode-loopstart
    fn LoopStart(&self) -> Finite<f64> {
        Finite::wrap(self.loop_start.get())
    }

    // https://webaudio.github.io/web-audio-api/#dom-audiobuffersourcenode-loopstart
    fn SetLoopStart(&self, loop_start: Finite<f64>) {
        self.loop_start.set(*loop_start);
        let msg = AudioNodeMessage::AudioBufferSourceNode(
            AudioBufferSourceNodeMessage::SetLoopStart(*loop_start),
        );
        self.source_node.node().message(msg);
    }

    // https://webaudio.github.io/web-audio-api/#dom-audiobuffersourcenode-loopend
    fn LoopEnd(&self) -> Finite<f64> {
        Finite::wrap(self.loop_end.get())
    }

    // https://webaudio.github.io/web-audio-api/#dom-audiobuffersourcenode-loopend
    fn SetLoopEnd(&self, loop_end: Finite<f64>) {
        self.loop_end.set(*loop_end);
        let msg = AudioNodeMessage::AudioBufferSourceNode(
            AudioBufferSourceNodeMessage::SetLoopEnd(*loop_end),
        );
        self.source_node.node().message(msg);
    }

    // https://webaudio.github.io/web-audio-api/#dom-audiobuffersourcenode-start
    fn Start(
        &self,
        when: Finite<f64>,
        offset: Option<Finite<f64>>,
        duration: Option<Finite<f64>>,
    ) -> Fallible<()> {
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
            let buffer = buffer.get_channels();
            if buffer.is_some() {
                self.source_node
                    .node()
                    .message(AudioNodeMessage::AudioBufferSourceNode(
                        AudioBufferSourceNodeMessage::SetBuffer((*buffer).clone()),
                    ));
            }
        }

        self.source_node
            .node()
            .message(AudioNodeMessage::AudioBufferSourceNode(
                AudioBufferSourceNodeMessage::SetStartParams(
                    *when,
                    offset.map(|f| *f),
                    duration.map(|f| *f),
                ),
            ));

        self.source_node
            .upcast::<AudioScheduledSourceNode>()
            .Start(when)
    }
}

impl<'a> From<&'a AudioBufferSourceOptions> for AudioBufferSourceNodeOptions {
    fn from(options: &'a AudioBufferSourceOptions) -> Self {
        Self {
            buffer: options
                .buffer
                .as_ref()
                .and_then(|b| (*b.as_ref()?.get_channels()).clone()),
            detune: *options.detune,
            loop_enabled: options.loop_,
            loop_end: Some(*options.loopEnd),
            loop_start: Some(*options.loopStart),
            playback_rate: *options.playbackRate,
        }
    }
}
