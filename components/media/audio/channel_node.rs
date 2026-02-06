/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::block::{Block, Chunk, FRAMES_PER_BLOCK_USIZE};
use crate::node::{
    AudioNodeEngine, AudioNodeType, BlockInfo, ChannelCountMode, ChannelInfo, ChannelInterpretation,
};

#[derive(Copy, Clone, Debug)]
pub struct ChannelNodeOptions {
    pub channels: u8,
}

#[derive(AudioNodeCommon)]
pub(crate) struct ChannelMergerNode {
    channel_info: ChannelInfo,
    channels: u8,
}

impl ChannelMergerNode {
    pub fn new(params: ChannelNodeOptions, channel_info: ChannelInfo) -> Self {
        ChannelMergerNode {
            channel_info,
            channels: params.channels,
        }
    }
}

impl AudioNodeEngine for ChannelMergerNode {
    fn node_type(&self) -> AudioNodeType {
        AudioNodeType::ChannelMergerNode
    }

    fn process(&mut self, mut inputs: Chunk, _: &BlockInfo) -> Chunk {
        debug_assert!(inputs.len() == self.channels as usize);

        let mut block = Block::default();
        block.repeat(self.channels);
        block.explicit_repeat();

        for (i, channel) in block
            .data_mut()
            .chunks_mut(FRAMES_PER_BLOCK_USIZE)
            .enumerate()
        {
            channel.copy_from_slice(inputs.blocks[i].data_mut())
        }

        inputs.blocks.clear();
        inputs.blocks.push(block);
        inputs
    }

    fn input_count(&self) -> u32 {
        self.channels as u32
    }

    fn set_channel_count_mode(&mut self, _: ChannelCountMode) {
        panic!("channel merger nodes cannot have their mode changed");
    }

    fn set_channel_count(&mut self, _: u8) {
        panic!("channel merger nodes cannot have their channel count changed");
    }
}

#[derive(AudioNodeCommon)]
pub(crate) struct ChannelSplitterNode {
    channel_info: ChannelInfo,
}

impl ChannelSplitterNode {
    pub fn new(channel_info: ChannelInfo) -> Self {
        ChannelSplitterNode { channel_info }
    }
}

impl AudioNodeEngine for ChannelSplitterNode {
    fn node_type(&self) -> AudioNodeType {
        AudioNodeType::ChannelSplitterNode
    }

    fn process(&mut self, mut inputs: Chunk, _: &BlockInfo) -> Chunk {
        debug_assert!(inputs.len() == 1);

        let original = inputs.blocks.pop().unwrap();

        if original.is_silence() {
            inputs
                .blocks
                .resize(original.chan_count() as usize, Block::default())
        } else {
            for chan in 0..original.chan_count() {
                let mut block = Block::empty();
                block.push_chan(original.data_chan(chan));
                inputs.blocks.push(block);
            }
        }

        inputs
    }

    fn output_count(&self) -> u32 {
        self.channel_count() as u32
    }

    fn set_channel_count_mode(&mut self, _: ChannelCountMode) {
        panic!("channel splitter nodes cannot have their mode changed");
    }

    fn set_channel_interpretation(&mut self, _: ChannelInterpretation) {
        panic!("channel splitter nodes cannot have their channel interpretation changed");
    }

    fn set_channel_count(&mut self, _: u8) {
        panic!("channel splitter nodes cannot have their channel count changed");
    }
}
