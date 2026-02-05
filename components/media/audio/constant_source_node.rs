use crate::block::{Chunk, Tick};
use crate::node::{
    AudioNodeEngine, AudioNodeType, AudioScheduledSourceNodeMessage, BlockInfo, ChannelInfo,
    OnEndedCallback, ShouldPlay,
};
use crate::param::{Param, ParamType};

#[derive(Copy, Clone, Debug)]
pub struct ConstantSourceNodeOptions {
    pub offset: f32,
}

impl Default for ConstantSourceNodeOptions {
    fn default() -> Self {
        ConstantSourceNodeOptions { offset: 1. }
    }
}

#[derive(AudioScheduledSourceNode, AudioNodeCommon)]
pub(crate) struct ConstantSourceNode {
    channel_info: ChannelInfo,
    offset: Param,
    start_at: Option<Tick>,
    stop_at: Option<Tick>,
    onended_callback: Option<OnEndedCallback>,
}

impl ConstantSourceNode {
    pub fn new(options: ConstantSourceNodeOptions, channel_info: ChannelInfo) -> Self {
        Self {
            channel_info,
            offset: Param::new(options.offset),
            start_at: None,
            stop_at: None,
            onended_callback: None,
        }
    }

    pub fn update_parameters(&mut self, info: &BlockInfo, tick: Tick) -> bool {
        self.offset.update(info, tick)
    }
}

impl AudioNodeEngine for ConstantSourceNode {
    fn node_type(&self) -> AudioNodeType {
        AudioNodeType::ConstantSourceNode
    }

    fn process(&mut self, mut inputs: Chunk, info: &BlockInfo) -> Chunk {
        debug_assert!(inputs.is_empty());

        inputs.blocks.push(Default::default());

        let (start_at, stop_at) = match self.should_play_at(info.frame) {
            ShouldPlay::No => {
                return inputs;
            },
            ShouldPlay::Between(start, end) => (start, end),
        };

        {
            inputs.blocks[0].explicit_silence();

            let mut iter = inputs.blocks[0].iter();
            let mut offset = self.offset.value();
            while let Some(mut frame) = iter.next() {
                let tick = frame.tick();
                if tick < start_at {
                    continue;
                } else if tick > stop_at {
                    break;
                }
                if self.update_parameters(info, frame.tick()) {
                    offset = self.offset.value();
                }
                frame.mutate_with(|sample, _| *sample = offset);
            }
        }
        inputs
    }
    fn input_count(&self) -> u32 {
        0
    }

    fn get_param(&mut self, id: ParamType) -> &mut Param {
        match id {
            ParamType::Offset => &mut self.offset,
            _ => panic!("Unknown param {:?} for the offset", id),
        }
    }
    make_message_handler!(AudioScheduledSourceNode: handle_source_node_message);
}
