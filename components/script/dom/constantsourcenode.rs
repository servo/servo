/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::f32;

use dom_struct::dom_struct;
use js::rust::HandleObject;
use servo_media::audio::constant_source_node::ConstantSourceNodeOptions as ServoMediaConstantSourceOptions;
use servo_media::audio::node::{AudioNodeInit, AudioNodeType};
use servo_media::audio::param::ParamType;

use crate::dom::audioparam::AudioParam;
use crate::dom::audioscheduledsourcenode::AudioScheduledSourceNode;
use crate::dom::baseaudiocontext::BaseAudioContext;
use crate::dom::bindings::codegen::Bindings::AudioParamBinding::AutomationRate;
use crate::dom::bindings::codegen::Bindings::ConstantSourceNodeBinding::{
    ConstantSourceNodeMethods, ConstantSourceOptions,
};
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::reflector::reflect_dom_object_with_proto;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::window::Window;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct ConstantSourceNode {
    source_node: AudioScheduledSourceNode,
    offset: Dom<AudioParam>,
}

impl ConstantSourceNode {
    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
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
            0,            /* inputs */
            1,            /* outputs */
        )?;
        let node_id = source_node.node().node_id();
        let offset = AudioParam::new(
            window,
            context,
            node_id,
            AudioNodeType::ConstantSourceNode,
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

    pub(crate) fn new(
        window: &Window,
        context: &BaseAudioContext,
        options: &ConstantSourceOptions,
        can_gc: CanGc,
    ) -> Fallible<DomRoot<ConstantSourceNode>> {
        Self::new_with_proto(window, None, context, options, can_gc)
    }

    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    fn new_with_proto(
        window: &Window,
        proto: Option<HandleObject>,
        context: &BaseAudioContext,
        options: &ConstantSourceOptions,
        can_gc: CanGc,
    ) -> Fallible<DomRoot<ConstantSourceNode>> {
        let node = ConstantSourceNode::new_inherited(window, context, options)?;
        Ok(reflect_dom_object_with_proto(
            Box::new(node),
            window,
            proto,
            can_gc,
        ))
    }
}

impl ConstantSourceNodeMethods<crate::DomTypeHolder> for ConstantSourceNode {
    // https://webaudio.github.io/web-audio-api/#dom-constantsourcenode-constantsourcenode
    fn Constructor(
        window: &Window,
        proto: Option<HandleObject>,
        can_gc: CanGc,
        context: &BaseAudioContext,
        options: &ConstantSourceOptions,
    ) -> Fallible<DomRoot<ConstantSourceNode>> {
        ConstantSourceNode::new_with_proto(window, proto, context, options, can_gc)
    }

    // https://webaudio.github.io/web-audio-api/#dom-constantsourcenode-offset
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
