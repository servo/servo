/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::VecDeque;
use std::sync::Arc;

use f32;
use log::error;
use malloc_size_of_derive::MallocSizeOf;
use parking_lot::RwLock;

use crate::audio_node::{
    AudioNodeEngine, AudioNodeType, BlockInfo, ChannelInfo, ChannelInterpretation,
};
use crate::block::{Block, Chunk};
use crate::param::{Param, ParamType};

mod delay_reader;
mod delay_writer;
pub(crate) use delay_reader::DelayReader;
pub(crate) use delay_writer::DelayWriter;

// Share with internal nodes. Use Arc because AudioNodeEngine requires Send
type DelayBuffer = Arc<RwLock<VecDeque<Block>>>;
type CachedUpmixedBlock = Arc<RwLock<Option<UpmixedBlock>>>;

#[derive(Copy, Clone, Debug, MallocSizeOf)]
pub struct DelayNodeOptions {
    pub max_delay_time: f64,
    pub delay_time: f64,
}

impl Default for DelayNodeOptions {
    fn default() -> Self {
        DelayNodeOptions {
            max_delay_time: 1.,
            delay_time: 0.,
        }
    }
}

#[derive(AudioNodeCommon)]
pub(crate) struct DelayNode {
    channel_info: ChannelInfo,
    delay_writer: Option<Box<DelayWriter>>,
    delay_reader: Option<Box<DelayReader>>,
}

/// UpmixedBlock is a Block that has been upmixed to the output channel count of the DelayReader
#[derive(Debug)]
struct UpmixedBlock {
    // The index of the upmixed block in the delay line
    index: usize,
    block: Block,
}

impl UpmixedBlock {
    fn new(
        index: usize,
        channel_count: u8,
        channel_interpretation: ChannelInterpretation,
        block: &Block,
    ) -> Self {
        let mut block = block.clone();
        block.mix(channel_count, channel_interpretation);
        UpmixedBlock { index, block }
    }

    fn index(&self) -> usize {
        self.index
    }

    fn block(&self) -> &Block {
        &self.block
    }

    fn increment_index(&mut self) {
        self.index += 1;
    }
}

impl DelayNode {
    pub fn new(options: DelayNodeOptions, channel_info: ChannelInfo) -> Self {
        let delay_line = Arc::new(RwLock::new(VecDeque::with_capacity(0)));
        let upmixed_block = Arc::new(RwLock::new(None));
        DelayNode {
            channel_info,
            delay_writer: Some(Box::new(DelayWriter::new(
                delay_line.clone(),
                upmixed_block.clone(),
                channel_info,
                options.max_delay_time,
            ))),
            delay_reader: Some(Box::new(DelayReader::new(
                delay_line,
                upmixed_block,
                Param::new(options.delay_time as f32),
                channel_info,
            ))),
        }
    }
}

impl AudioNodeEngine for DelayNode {
    fn node_type(&self) -> AudioNodeType {
        AudioNodeType::DelayNode
    }

    fn process(&mut self, inputs: Chunk, info: &BlockInfo) -> Chunk {
        let Some(delay_writer) = &mut self.delay_writer else {
            error!("No DelayWriter initialized!");
            return Chunk::explicit_silence();
        };
        delay_writer.process(inputs, info);

        // Read from the internal buffer
        let Some(delay_reader) = &mut self.delay_reader else {
            error!("No DelayReader initialized!");
            return Chunk::explicit_silence();
        };
        delay_reader.process(Chunk::default(), info)
    }

    fn get_param(&mut self, id: ParamType) -> &mut Param {
        // DelayReader should not be None when `get_param` is called.
        // The only time DelayNode gives up ownership of its DelayReader is within a render quantum
        // processing loop, when it gives ownership to the graph. In this case it has removed itself
        // from the graph so it will not be processed, and therefore never call `get_param` within the
        // processing loop.
        self.delay_reader
            .as_mut()
            .expect("Tried to get delay_time Param without an owned DelayReader.")
            .get_param(id)
    }
}
