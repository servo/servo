use crate::block::{Chunk, Tick};
use crate::node::{AudioNodeEngine, AudioNodeType, BlockInfo, ChannelInfo};
use crate::param::{Param, ParamType};

#[derive(Copy, Clone, Debug)]
pub struct GainNodeOptions {
    pub gain: f32,
}

impl Default for GainNodeOptions {
    fn default() -> Self {
        GainNodeOptions { gain: 1. }
    }
}

#[derive(AudioNodeCommon)]
pub(crate) struct GainNode {
    channel_info: ChannelInfo,
    gain: Param,
}

impl GainNode {
    pub fn new(options: GainNodeOptions, channel_info: ChannelInfo) -> Self {
        Self {
            channel_info,
            gain: Param::new(options.gain),
        }
    }

    pub fn update_parameters(&mut self, info: &BlockInfo, tick: Tick) -> bool {
        self.gain.update(info, tick)
    }
}

impl AudioNodeEngine for GainNode {
    fn node_type(&self) -> AudioNodeType {
        AudioNodeType::GainNode
    }

    fn process(&mut self, mut inputs: Chunk, info: &BlockInfo) -> Chunk {
        debug_assert!(inputs.len() == 1);

        if inputs.blocks[0].is_silence() {
            return inputs;
        }

        {
            let mut iter = inputs.blocks[0].iter();
            let mut gain = self.gain.value();

            while let Some(mut frame) = iter.next() {
                if self.update_parameters(info, frame.tick()) {
                    gain = self.gain.value();
                }
                frame.mutate_with(|sample, _| *sample = *sample * gain);
            }
        }
        inputs
    }

    fn get_param(&mut self, id: ParamType) -> &mut Param {
        match id {
            ParamType::Gain => &mut self.gain,
            _ => panic!("Unknown param {:?} for GainNode", id),
        }
    }
}
