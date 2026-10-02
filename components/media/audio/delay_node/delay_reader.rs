/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use num_traits::Zero;

use crate::audio_node::{AudioNodeEngine, AudioNodeType, BlockInfo, ChannelInfo};
use crate::block::{Block, Chunk, FRAMES_PER_BLOCK_USIZE, Tick};
use crate::delay_node::{CachedUpmixedBlock, DelayBuffer, UpmixedBlock};
use crate::param::{Param, ParamType};

/// <https://webaudio.github.io/web-audio-api/#delayreader>
/// > ...an object that has the same interface as an AudioNode,
/// > and that can read the audio data from the internal buffer of the DelayNode.
/// > It is connected to the same AudioNodes as the DelayNode it was created from.
#[derive(AudioNodeCommon)]
pub(crate) struct DelayReader {
    channel_info: ChannelInfo,
    // Tracks the delay time in terms of number of frames, relative to each frame in the input block.
    // When reading from the buffer, we look for the stored block with the relevant frames
    delay_frames: [f32; FRAMES_PER_BLOCK_USIZE],
    // Ring buffer where we push to the front.
    // Easier mental model since entries in the back are the oldest.
    delay_line: DelayBuffer,
    // Block that has been upmixed based on the channel count of the output.
    upmixed_block: CachedUpmixedBlock,
    // delay_time param passed on from the delay node. Delay time in seconds.
    delay_time: Param,
}

fn find_block_with_index(delay_frame_index: usize) -> usize {
    delay_frame_index / FRAMES_PER_BLOCK_USIZE
}

impl DelayReader {
    pub(super) fn new(
        buffer: DelayBuffer,
        upmixed_block: CachedUpmixedBlock,
        delay_time: Param,
        channel_info: ChannelInfo,
    ) -> Self {
        DelayReader {
            channel_info,
            delay_frames: [0.; FRAMES_PER_BLOCK_USIZE],
            delay_line: buffer,
            upmixed_block,
            delay_time,
        }
    }

    /// <https://webaudio.github.io/web-audio-api/#dom-delaynode-delaytime>
    fn update_parameters(&mut self, info: &BlockInfo, tick: Tick) -> bool {
        let updated = self.delay_time.update(info, tick);
        self.update_delay_frames(tick.0 as usize, self.delay_time.value() * info.sample_rate);
        updated
    }

    /// 1.18.4
    /// > When producing an output buffer, a DelayReader MUST yield exactly the audio that was written to the
    /// > corresponding DelayWriter delayTime seconds ago.
    ///
    /// We are processing tick t, where t ranges from 0 to (FRAMES_PER_BLOCK - 1).
    /// The DelayWriter is writing a frame into the delay line every tick.
    /// Let delay_frame = delay_time * sample_rate, at tick t.
    /// Now let's process tick t + 1. delay_frame must increase by 1 to account for the new tick.
    /// There are FRAMES_PER_BLOCK - 1 - t ticks to process after tick t.
    /// So after processing all ticks (FRAMES_PER_BLOCK - 1),
    /// delay_frames[t] = delay_time * sample_rate + FRAMES_PER_BLOCK - 1 - t
    fn update_delay_frames(&mut self, tick: usize, value: f32) {
        self.delay_frames[tick] = value + FRAMES_PER_BLOCK_USIZE as f32 - 1. - tick as f32;
    }

    /// Calculates the output channel count
    /// <https://webaudio.github.io/web-audio-api/#tail-time>
    /// 4.3
    /// > When an AudioNode has a non-zero tail-time,
    /// > and an output channel count that depends on the input channels count,
    /// > the AudioNode’s tail-time must be taken into account when the input channel count changes.
    /// >
    /// > When there is a decrease in input channel count,
    /// > the change in output channel count MUST happen when the input that was received
    /// > with greater channel count no longer affects the output.
    /// >
    /// > When there is an increase in input channel count, the behavior depends on the AudioNode type:
    /// > * For a DelayNode or a DynamicsCompressorNode, the number of output channels MUST increase
    /// >   when the input that was received with greater channel count begins to affect the output.
    ///
    /// Therefore, we know that output channel count will be the highest channel count of the
    /// blocks read.
    fn calc_output_channel_count(&self) -> u8 {
        // If the delay line is empty, there's obviously nothing that was delayed, so there will be no output.
        if self.delay_line.read().is_empty() {
            return 0;
        }
        let (min_delay_frame, max_delay_frame) = self
            .delay_frames
            .iter()
            .fold((f32::MAX, f32::MIN), |frames, delay_frame| {
                (frames.0.min(*delay_frame), frames.1.max(*delay_frame))
            });
        // With the range of delay frames we can check which blocks we will be reading from.
        let earlier_block = find_block_with_index(max_delay_frame.ceil() as usize);
        let later_block = find_block_with_index(min_delay_frame.floor() as usize);
        // Now search through the potential blocks for their channel counts
        // By construction earlier blocks are in higher indices of the delay line
        let mut channel_count = 0;
        let delay_line = self.delay_line.read();
        for block in later_block..=(later_block.max(earlier_block.min(delay_line.len() - 1))) {
            channel_count = channel_count.max(
                delay_line
                    .get(block)
                    .map(|block| {
                        // Silent blocks don't affect the output.
                        if !block.is_silence() {
                            block.chan_count()
                        } else {
                            0
                        }
                    })
                    .unwrap_or_default(),
            );
        }
        channel_count
    }

    fn upmix_block(&self, index: usize, channel_count: u8, block: &Block) -> UpmixedBlock {
        UpmixedBlock::new(
            index,
            channel_count,
            self.channel_info.interpretation,
            block,
        )
    }

    /// Read frames from the delay line at the values indexed around the specified delays.
    pub(super) fn read(&mut self) -> Chunk {
        let channel_count = self.calc_output_channel_count();
        // If channel count is 0, then no data is outputted.
        // In this case we just return an output block with a single channel of 0s.
        if channel_count.is_zero() {
            return Chunk::explicit_silence();
        }
        let mut has_active_value = false;
        // Initialize the output block
        let mut output_block = Block::for_channels_explicit(channel_count);
        {
            let delay_line = self.delay_line.read();
            for (tick, delay_frame) in self.delay_frames.into_iter().enumerate() {
                let lower_frame_index = delay_frame.floor() as usize;
                let higher_frame_index = delay_frame.ceil() as usize;
                let lower_block_index = find_block_with_index(lower_frame_index);
                let higher_block_index = find_block_with_index(higher_frame_index);
                let mut linear_interpolation_factor = delay_frame.fract();
                for (frame_index, block_index) in [
                    (lower_frame_index, lower_block_index),
                    (higher_frame_index, higher_block_index),
                ]
                .into_iter()
                {
                    if !linear_interpolation_factor.is_zero() {
                        let Some(block) = delay_line.get(block_index) else {
                            continue;
                        };
                        // Update the upmixed block if necessary.
                        {
                            let mut maybe_upmixed_block = self.upmixed_block.write();
                            if let Some(upmixed_block) = maybe_upmixed_block.as_ref() {
                                if upmixed_block.index() != block_index {
                                    *maybe_upmixed_block =
                                        Some(self.upmix_block(block_index, channel_count, block));
                                }
                            } else {
                                *maybe_upmixed_block =
                                    Some(self.upmix_block(block_index, channel_count, block));
                            }
                        }
                        for channel in 0..channel_count as usize {
                            // Get the position of the target frame within the block
                            let position_for_block = frame_index % FRAMES_PER_BLOCK_USIZE;
                            // Remember that block buffer data goes from oldest to newest
                            let upmixed_value = self
                                .upmixed_block
                                .read()
                                .as_ref()
                                .map(|upmixed_block| {
                                    upmixed_block.block().data_chan_frame(
                                        FRAMES_PER_BLOCK_USIZE - 1 - position_for_block,
                                        channel as u8,
                                    )
                                })
                                .unwrap_or_default();
                            // Flag if we are actively processing
                            if upmixed_value.abs() >= f32::MIN && !has_active_value {
                                has_active_value = true;
                            }
                            let output_channel = output_block.data_chan_mut(channel as u8);
                            output_channel[tick] += linear_interpolation_factor * upmixed_value;
                        }
                    }
                    linear_interpolation_factor = 1. - linear_interpolation_factor;
                }
            }
        }
        // 1.5.3
        // > AudioNodes that are not actively processing output a single channel of silence.
        Chunk::new(output_block)
    }
}

impl AudioNodeEngine for DelayReader {
    fn node_type(&self) -> AudioNodeType {
        AudioNodeType::DelayReader
    }

    fn process(&mut self, _inputs: Chunk, info: &BlockInfo) -> Chunk {
        // Reset the delay frames array
        self.delay_frames = [0.; FRAMES_PER_BLOCK_USIZE];

        // Update delay_frames
        for i in 0..FRAMES_PER_BLOCK_USIZE {
            self.update_parameters(info, Tick(i as u64));
        }

        // Read from the internal buffer
        self.read()
    }

    fn get_param(&mut self, id: ParamType) -> &mut Param {
        match id {
            ParamType::DelayTime => &mut self.delay_time,
            _ => panic!("Unknown param {:?} for DelayNode", id),
        }
    }
}
