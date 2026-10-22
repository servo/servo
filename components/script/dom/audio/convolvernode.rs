/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use std::cell::Cell;

use dom_struct::dom_struct;
use js::context::JSContext;
use js::rust::HandleObject;
use script_bindings::codegen::GenericBindings::AudioBufferBinding::AudioBufferMethods;
use script_bindings::codegen::GenericBindings::AudioNodeBinding::{
    AudioNodeMethods, ChannelCountMode, ChannelInterpretation,
};
use script_bindings::codegen::GenericBindings::BaseAudioContextBinding::BaseAudioContextMethods;
use script_bindings::codegen::GenericBindings::ConvolverNodeBinding::ConvolverNodeMethods;
use script_bindings::reflector::reflect_dom_object_with_proto;
use servo_media::audio::audio_node::{AudioNodeInit, AudioNodeMessage};
use servo_media::audio::convolver_node::{ConvolverNodeMessage, ConvolverNodeOptions};
use servo_media::audio::graph::NodeId;

use crate::conversions::ConvertWithCx;
use crate::dom::audio::audionode::AudioNodeOptionsHelper;
use crate::dom::audio::baseaudiocontext::BaseAudioContext;
use crate::dom::audionode::UnwrappedAudioNodeOptions;
use crate::dom::bindings::codegen::Bindings::ConvolverNodeBinding::ConvolverOptions;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::root::{DomRoot, MutNullableDom};
use crate::dom::types::{AudioBuffer, AudioNode};
use crate::dom::window::Window;

#[dom_struct]
pub(crate) struct ConvolverNode {
    source_node: AudioNode,
    /// <https://webaudio.github.io/web-audio-api/#dom-convolvernode-buffer>
    buffer: MutNullableDom<AudioBuffer>,
    /// <https://webaudio.github.io/web-audio-api/#dom-convolvernode-normalize>
    normalize: Cell<bool>,
}

impl ConvolverNode {
    fn new_inherited(
        node_id: Option<NodeId>,
        context: &BaseAudioContext,
        node_options: UnwrappedAudioNodeOptions,
        normalize: bool,
    ) -> Self {
        ConvolverNode {
            source_node: AudioNode::new_inherited(node_id, context, node_options, 1, 1),
            buffer: Default::default(),
            normalize: Cell::new(normalize),
        }
    }
}

impl ConvolverNodeMethods<crate::DomTypeHolder> for ConvolverNode {
    /// <https://webaudio.github.io/web-audio-api/#dom-convolvernode-convolvernode>
    fn Constructor(
        cx: &mut JSContext,
        window: &Window,
        proto: Option<HandleObject>,
        context: &BaseAudioContext,
        options: &ConvolverOptions,
    ) -> Fallible<DomRoot<ConvolverNode>> {
        let node_options = options.parent.unwrap_or(
            2,
            ChannelCountMode::Clamped_max,
            ChannelInterpretation::Speakers,
        );
        if AudioNode::channel_count_check(node_options.count) {
            return Err(Error::NotSupported(Some(String::from(
                "Channel count not supported",
            ))));
        }
        // <https://webaudio.github.io/web-audio-api/#audionode-channelcount-constraints>
        // > The channel count cannot be greater than two, and a NotSupportedError exception
        // > MUST be thrown for any attempt to change it to a value greater than two.
        if node_options.count > 2 {
            return Err(Error::NotSupported(Some(String::from(
                "Channel count greater than 2 is not supported",
            ))));
        }
        // <https://webaudio.github.io/web-audio-api/#audionode-channelcountmode-constraints>
        // > The channel count mode cannot be set to "max", and a NotSupportedError exception
        // > MUST be thrown for any attempt to set it to "max".
        if let ChannelCountMode::Max = node_options.mode {
            return Err(Error::NotSupported(Some(String::from(
                "Channel count mode Max is not supported",
            ))));
        }
        let convolver_options = AudioNodeInit::ConvolverNode(options.convert(cx));
        let node_id =
            AudioNode::generate_audio_node_id(cx, &node_options, convolver_options, context);

        // Step 1. Set the attributes normalize to the inverse of the value of disableNormalization.
        let node = reflect_dom_object_with_proto(
            cx,
            Box::new(ConvolverNode::new_inherited(
                node_id,
                context,
                node_options,
                !options.disableNormalization,
            )),
            window,
            proto,
        );
        // Step 2. If buffer exists, set the buffer attribute to its value.
        if let Some(Some(ref buffer)) = options.buffer {
            node.SetBuffer(cx, Some(buffer))?;
        }
        Ok(node)
    }

    /// <https://webaudio.github.io/web-audio-api/#dom-convolvernode-buffer>
    fn GetBuffer(&self) -> Fallible<Option<DomRoot<AudioBuffer>>> {
        Ok(self.buffer.get())
    }

    /// <https://webaudio.github.io/web-audio-api/#dom-convolvernode-buffer>
    fn SetBuffer(&self, cx: &mut JSContext, new_buffer: Option<&AudioBuffer>) -> Fallible<()> {
        let Some(new_buffer) = new_buffer else {
            debug!("Setting buffer to null");
            self.buffer.set(new_buffer);
            return Ok(());
        };
        // Step 1. If the buffer number of channels is not 1, 2, 4,
        // or if the sample-rate of the buffer is not the same as the sample-rate of its associated BaseAudioContext,
        // a NotSupportedError MUST be thrown.
        if ![1u32, 2, 4].contains(&new_buffer.NumberOfChannels()) {
            return Err(Error::NotSupported(Some(String::from(
                "Buffer number of channels is not 1, 2 or 4",
            ))));
        }
        if new_buffer.SampleRate() != self.source_node.Context().SampleRate() {
            return Err(Error::NotSupported(Some(String::from(
                "Buffer sample rate not equal to sample rate of convolver node's associated base audio context",
            ))));
        }
        self.buffer.set(Some(new_buffer));

        // Step 2. Acquire the AudioBuffer content.
        if let Some(buffer) = self.buffer.get() {
            let buffer = buffer.get_channels(cx);
            if buffer.is_some() {
                self.source_node.message(AudioNodeMessage::ConvolverNode(
                    ConvolverNodeMessage::SetBuffer(buffer),
                ));
            }
        }

        Ok(())
    }

    /// <https://webaudio.github.io/web-audio-api/#dom-convolvernode-normalize>
    fn Normalize(&self) -> bool {
        self.normalize.get()
    }

    /// <https://webaudio.github.io/web-audio-api/#dom-convolvernode-normalize>
    fn SetNormalize(&self, value: bool) {
        self.normalize.set(value);
        self.source_node.message(AudioNodeMessage::ConvolverNode(
            ConvolverNodeMessage::SetNormalize(value),
        ));
    }
}

impl ConvertWithCx<ConvolverNodeOptions> for ConvolverOptions {
    fn convert(&self, cx: &mut JSContext) -> ConvolverNodeOptions {
        ConvolverNodeOptions {
            buffer: self
                .buffer
                .as_ref()
                .and_then(|b| b.as_ref()?.get_channels(cx).map(|buffer| (*buffer).clone())),
            normalize: !self.disableNormalization,
        }
    }
}
