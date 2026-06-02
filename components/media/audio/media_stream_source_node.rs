/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::AudioStreamReader;
use crate::block::Chunk;
use crate::node::{AudioNodeEngine, AudioNodeType, BlockInfo, ChannelInfo};
use crate::param::{Param, ParamType};

#[derive(AudioNodeCommon)]
pub(crate) struct MediaStreamSourceNode {
    channel_info: ChannelInfo,
    reader: Box<dyn AudioStreamReader + Send>,
    playing: bool,
}

impl MediaStreamSourceNode {
    pub fn new(reader: Box<dyn AudioStreamReader + Send>, channel_info: ChannelInfo) -> Self {
        Self {
            channel_info,
            reader,
            playing: false,
        }
    }
}

impl AudioNodeEngine for MediaStreamSourceNode {
    fn node_type(&self) -> AudioNodeType {
        AudioNodeType::MediaStreamSourceNode
    }

    fn process(&mut self, mut inputs: Chunk, _: &BlockInfo) -> Chunk {
        debug_assert!(inputs.is_empty());

        if !self.playing {
            self.playing = true;
            self.reader.start();
        }

        let block = self.reader.pull();
        inputs.blocks.push(block);

        inputs
    }

    fn input_count(&self) -> u32 {
        0
    }

    fn get_param(&mut self, _: ParamType) -> &mut Param {
        panic!("No params on MediaStreamSourceNode");
    }
}
