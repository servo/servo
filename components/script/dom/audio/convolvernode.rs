/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use std::cell::Cell;

use dom_struct::dom_struct;
use js::context::JSContext;
use js::rust::HandleObject;
use script_bindings::codegen::GenericBindings::AudioBufferBinding::AudioBufferMethods;
use script_bindings::codegen::GenericBindings::AudioNodeBinding::{AudioNodeMethods, ChannelCountMode, ChannelInterpretation};
use script_bindings::codegen::GenericBindings::BaseAudioContextBinding::BaseAudioContextMethods;
use script_bindings::reflector::reflect_dom_object_with_proto;
use script_bindings::{codegen::GenericBindings::ConvolverNodeBinding::ConvolverNodeMethods};
use servo_media::audio::audio_node::{AudioNodeInit, AudioNodeMessage};
use servo_media::audio::convolver_node::{ConvolverNodeMessage, ConvolverNodeOptions};

use crate::conversions::ConvertWithCx;
use crate::dom::bindings::codegen::Bindings::ConvolverNodeBinding::ConvolverOptions;
use crate::dom::types::{AudioBuffer, AudioNode};
use crate::dom::audio::audionode::AudioNodeOptionsHelper;
use crate::dom::audio::baseaudiocontext::BaseAudioContext;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::root::{DomRoot, MutNullableDom};
use crate::dom::window::Window;

#[dom_struct]
pub(crate) struct ConvolverNode {
    source_node: AudioNode,
    buffer: MutNullableDom<AudioBuffer>,
    normalize: Cell<bool>,
}

impl ConvolverNodeMethods<crate::DomTypeHolder> for ConvolverNode {
    /// <https://webaudio.github.io/web-audio-api/#dom-convolvernode-convolvernode>
    #[cfg_attr(crown, expect(crown::unrooted_must_root))]
    fn Constructor(
        cx: &mut JSContext,
        window: &Window,
        proto: Option<HandleObject>,
        context: &BaseAudioContext,
        options: &ConvolverOptions,
    ) -> Fallible<DomRoot<ConvolverNode>> {
        let node_options = options.parent.unwrap_or(2, ChannelCountMode::Clamped_max, ChannelInterpretation::Speakers);
        /// <https://webaudio.github.io/web-audio-api/#audionode-channelcount-constraints>
        if node_options.count > 2 {
            return Err(Error::NotSupported(Some(String::from("Channel count greater than 2 is not supported"))));
        }
        /// <https://webaudio.github.io/web-audio-api/#audionode-channelcountmode-constraints>
        if let ChannelCountMode::Max = node_options.mode {
            return Err(Error::NotSupported(Some(String::from("Channel count mode Max is not supported"))));
        }
        let convolver_options = AudioNodeInit::ConvolverNode(options.convert(cx));
        let source_node = AudioNode::new_inherited(cx, convolver_options, context, node_options, 1, 1)?;
        let node = ConvolverNode {
            source_node,
            buffer: Default::default(),
            // Set the attributes normalize to the inverse of the value of disableNormalization.
            normalize: Cell::new(!options.disableNormalization),
        };
        // If buffer exists, set the buffer attribute to its value.
        if let Some(Some(ref buffer)) = options.buffer {
            node.SetBuffer(cx, Some(buffer))?;
        }
        Ok(reflect_dom_object_with_proto(
            cx,
            Box::new(node),
            window,
            proto,
        ))
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
        // If the buffer number of channels is not 1, 2, 4, a NotSupportedError MUST be thrown.
        if !vec![1u32, 2, 4].contains(&new_buffer.NumberOfChannels()) {
            return Err(Error::NotSupported(Some(String::from("Buffer number of channels is not 1, 2 or 4"))));
        }
        // If the sample-rate of the buffer is not the same
        // as the sample-rate of its associated BaseAudioContext, a NotSupportedError MUST be thrown.
        if new_buffer.SampleRate() != self.source_node.Context().SampleRate() {
            return Err(Error::NotSupported(Some(String::from("Buffer sample rate not equal to sample rate of convolver node's associated base audio context"))));
        }
        self.buffer.set(Some(new_buffer));

        // Acquire the AudioBuffer content
        if let Some(buffer) = self.buffer.get()
        {
            let buffer = buffer.get_channels(cx);
            if buffer.is_some() {
                self.source_node
                    .message(AudioNodeMessage::ConvolverNode(
                        ConvolverNodeMessage::SetBuffer((*buffer).clone()),
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
        self.source_node
        .message(AudioNodeMessage::ConvolverNode(
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
                .and_then(|b| (*b.as_ref()?.get_channels(cx)).clone()),
            normalize: !self.disableNormalization,
        }
    }
}
