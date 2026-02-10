/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::block::Chunk;
use crate::node::{AudioNodeEngine, AudioNodeType, BlockInfo, ChannelCountMode, ChannelInfo};

#[derive(AudioNodeCommon)]
pub(crate) struct DestinationNode {
    channel_info: ChannelInfo,
    chunk: Option<Chunk>,
}

impl DestinationNode {
    pub fn new(channel_count: u8) -> Self {
        DestinationNode {
            channel_info: ChannelInfo {
                mode: ChannelCountMode::Explicit,
                count: channel_count,
                ..Default::default()
            },
            chunk: None,
        }
    }
}

impl AudioNodeEngine for DestinationNode {
    fn node_type(&self) -> AudioNodeType {
        AudioNodeType::DestinationNode
    }

    fn process(&mut self, inputs: Chunk, _: &BlockInfo) -> Chunk {
        self.chunk = Some(inputs);
        Chunk::default()
    }

    fn destination_data(&mut self) -> Option<Chunk> {
        self.chunk.take()
    }

    fn output_count(&self) -> u32 {
        0
    }
}
