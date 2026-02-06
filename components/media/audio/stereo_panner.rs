/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::f32::consts::PI;

use crate::block::{Chunk, FRAMES_PER_BLOCK, Tick};
use crate::node::{AudioNodeEngine, AudioNodeType, BlockInfo, ChannelInfo};
use crate::param::{Param, ParamType};

#[derive(Copy, Clone, Debug)]
pub struct StereoPannerOptions {
    pub pan: f32,
}

impl Default for StereoPannerOptions {
    fn default() -> Self {
        StereoPannerOptions { pan: 0. }
    }
}

#[derive(AudioNodeCommon)]
pub(crate) struct StereoPannerNode {
    channel_info: ChannelInfo,
    pan: Param,
}

impl StereoPannerNode {
    pub fn new(options: StereoPannerOptions, channel_info: ChannelInfo) -> Self {
        Self {
            channel_info,
            pan: Param::new(options.pan),
        }
    }

    pub fn update_parameters(&mut self, info: &BlockInfo, tick: Tick) -> bool {
        self.pan.update(info, tick)
    }
}

impl AudioNodeEngine for StereoPannerNode {
    fn node_type(&self) -> AudioNodeType {
        AudioNodeType::StereoPannerNode
    }

    fn process(&mut self, mut inputs: Chunk, info: &BlockInfo) -> Chunk {
        debug_assert!(inputs.len() == 1);

        {
            let block = &mut inputs.blocks[0];

            block.explicit_repeat();

            let mono = if block.chan_count() == 1 {
                block.resize_silence(2);
                true
            } else {
                debug_assert!(block.chan_count() == 2);
                false
            };

            let (l, r) = block.data_mut().split_at_mut(FRAMES_PER_BLOCK.0 as usize);
            let mut pan = self.pan.value();
            for frame in 0..FRAMES_PER_BLOCK.0 {
                let frame = Tick(frame);
                if self.update_parameters(info, frame) {
                    pan = self.pan.value();
                }

                // https://webaudio.github.io/web-audio-api/#stereopanner-algorithm

                // clamp pan to [-1, 1]
                pan = pan.clamp(-1., 1.);

                let x = if mono {
                    (pan + 1.) / 2.
                } else if pan <= 0. {
                    pan + 1.
                } else {
                    pan
                };
                let x = x * PI / 2.;

                let mut gain_l = x.cos();
                let mut gain_r = x.sin();
                // 9. * PI / 2 is often slightly negative, clamp
                if gain_l <= 0. {
                    gain_l = 0.
                }
                if gain_r <= 0. {
                    gain_r = 0.;
                }

                let index = frame.0 as usize;
                if mono {
                    let input = l[index];
                    l[index] = input * gain_l;
                    r[index] = input * gain_r;
                } else if pan <= 0. {
                    l[index] += r[index] * gain_l;
                    r[index] *= gain_r;
                } else {
                    r[index] += l[index] * gain_r;
                    l[index] *= gain_l;
                }
            }
        }

        inputs
    }

    fn input_count(&self) -> u32 {
        1
    }

    fn get_param(&mut self, id: ParamType) -> &mut Param {
        match id {
            ParamType::Pan => &mut self.pan,
            _ => panic!("Unknown param {:?} for PannerNode", id),
        }
    }
}
