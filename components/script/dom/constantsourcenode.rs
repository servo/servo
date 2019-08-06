/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::audionode::{AudioNode, UnwrappedAudioNodeOptions};
use crate::dom::audioparam::AudioParam;
use crate::dom::audioscheduledsourcenode::AudioScheduledSourceNode;
use crate::dom::baseaudiocontext::BaseAudioContext;
use crate::dom::bindings::codegen::Bindings::ConstantSourceNodeBinding;
use crate::dom::bindings::codegen::Bindings::ConstantSourceNodeBinding::ConstantSourceNodeMethods;
use crate::dom::bindings::codegen::Bindings::ConstantSourceNodeBinding::{
    self, ConstantSourceOptions,
};
use crate::dom::bindings::codegen::Bindings::AudioParamBinding::AutomationRate;
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::window::Window;
use servo_media::audio::constant_source_node::ConstantSourceNodeOptions as ServoMediaConstantSourceOptions;
use servo_media::audio::node::AudioNodeInit;
use servo_media::audio::param::ParamType;

#[dom_struct]
pub struct ConstantSourceNode {
    source_node: AudioScheduledSourceNode,
    offset: Dom<AudioParam>,
}

impl ConstantSourceNode {
    fn new_inherited(
        window: &Window,
        context: &BaseAudioContext,
        options: &ConstantSourceOptions,
    ) -> Fallible<ConstantSourceNode> {
        let node_options = Default::default();
        let source_node = AudioScheduledSourceNode::new_inherited(
            AudioNodeInit::ConstantSourceNode(options.into()),
            context,
            node_options, /* 2, MAX, Speakers */
            0, /* inputs */
            1, /* outputs */
        )?;
        let node_id = source_node.node().node_id();
        let offset = AudioParam::new(
            window,
            context,
            node_id,
            ParamType::Offset,
            AutomationRate::A_rate,
            *options.offset,
            f32::MIN,
            f32::MAX,
        );

        Ok(ConstantSourceNode {
            source_node,
            offset: Dom::from_ref(&offset),
        })
    }

    #[allow(unrooted_must_root)]
    pub fn new(
        window: &Window,
        context: &BaseAudioContext,
        options: &ConstantSourceOptions,
    ) -> Fallible<DomRoot<ConstantSourceNode>> {
        let node = ConstantSourceNode::new_inherited(window, context, options)?;
        Ok(reflect_dom_object(
            Box::new(node),
            window,
            ConstantSourceNodeBinding::Wap,
        ))
    }

    pub fn Constructor(
        window: &Window,
        context: &BaseAudioContext,
        options: &ConstantSourceOptions,
    ) -> Fallible<DomRoot<ConstantSourceNode>> {
        ConstantSourceNode::new(window, context, options)
    }
}

impl ConstantSourceNodeMethods for ConstantSourceNode {
    fn Offset(&self) -> DomRoot<AudioParam> {
       DomRoot::from_ref(&self.offset) 
    }
}

impl<'a> From<&'a ConstantSourceOptions> for ServoMediaConstantSourceOptions {
    fn from(options: &'a ConstantSourceOptions) -> Self {
        Self {
            offset: *options.offset,
        }
    }
}
