/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::any::Any;

use log::error;
use num_traits::Zero;

use crate::audio_node::{AudioNodeEngine, AudioNodeType, BlockInfo, ChannelInfo};
use crate::block::{Chunk, FRAMES_PER_BLOCK_USIZE};
use crate::delay_node::{AccessLock, CachedUpmixedBlock, DelayBuffer};

/// <https://webaudio.github.io/web-audio-api/#delaywriter>
/// > ...an object that has the same interface as an AudioNode,
/// > and that writes the input audio into the internal buffer of the DelayNode.
/// > It has the same input connections as the DelayNode it was created from.
#[derive(AudioNodeCommon)]
pub(crate) struct DelayWriter {
    channel_info: ChannelInfo,
    accessed_first: AccessLock,
    // Ring buffer where we push to the front.
    // Easier mental model since entries in the back are the oldest.
    delay_line: DelayBuffer,
    // Block that has been upmixed based on the channel count of the output.
    upmixed_block: CachedUpmixedBlock,
    // Maximum delay time in seconds. Passed from the DelayNode
    max_delay_time: f64,
}

impl DelayWriter {
    pub(super) fn new(
        accessed_first: AccessLock,
        buffer: DelayBuffer,
        upmixed_block: CachedUpmixedBlock,
        channel_info: ChannelInfo,
        max_delay_time: f64,
    ) -> Self {
        Self {
            channel_info,
            accessed_first,
            delay_line: buffer,
            upmixed_block,
            max_delay_time,
        }
    }

    /// Updates the capacity of the delay line if it is currently zero.
    fn update_delay_line_capacity(&self, capacity: usize) {
        let mut delay_line = self.delay_line.write();
        if delay_line.capacity().is_zero() {
            delay_line.reserve(capacity);
        }
    }

    fn update_accessed_first(&mut self) {
        let mut accessed_first = self.accessed_first.lock();
        *accessed_first = !(*accessed_first);
    }

    /// Writes the input block to the delay line
    fn write(&self, mut input: Chunk) {
        {
            // First remove the oldest values if we are at capacity (max_delay_time)
            let Some(block) = input.blocks.pop() else {
                error!(
                    "Attempted to write a chunk with no data to the delay node internal buffer. This should not have occurred due to size check prior to write"
                );
                return;
            };
            let mut delay_line = self.delay_line.write();
            let last_index = delay_line.capacity() - 1;
            delay_line.truncate(last_index);
            // Push the input block to the front of the delay line.
            delay_line.push_front(block);
        }
        // If the writer acquired the access lock first, it shifts the index of the existing upmixed block
        let access_lock = self.accessed_first.lock();
        if *access_lock && let Some(upmixed_block) = self.upmixed_block.write().as_mut() {
            upmixed_block.increment_index();
        }
    }
}

impl AudioNodeEngine for DelayWriter {
    fn node_type(&self) -> AudioNodeType {
        AudioNodeType::DelayWriter
    }

    fn process(&mut self, inputs: Chunk, info: &BlockInfo) -> Chunk {
        // Update the accessed_first lock
        self.update_accessed_first();
        debug_assert!(inputs.len() == 1);

        let max_delay_frame = self.max_delay_time * info.sample_rate as f64;
        let max_delay_block = (max_delay_frame / FRAMES_PER_BLOCK_USIZE as f64).ceil() as usize;

        // Update the delay line capacity if necessary, and write the input blocks to the internal buffer
        self.update_delay_line_capacity(max_delay_block + 1);

        // Write to the internal buffer
        self.write(inputs);
        Chunk::default()
    }

    fn output_count(&self) -> u32 {
        0
    }

    fn into_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }
}
