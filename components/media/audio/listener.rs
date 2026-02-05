use crate::block::{Block, Chunk};
use crate::node::{AudioNodeEngine, AudioNodeType, BlockInfo, ChannelInfo};
use crate::param::{Param, ParamDir, ParamType};

/// AudioListeners are fake nodes; from the user's point of view they're
/// a non-node entity with zero inputs and outputs, but with AudioParams
/// that can be manipulated.
///
/// Internally, PannerNodes all have an implicit PortIndex::Listener connection
/// from a hidden output port on AudioListeners that contains all the position data.
///
/// This encodes the otherwise implicit dependency between AudioListeners and PannerNodes
/// so that if there is a cycle involving panner nodes and the audio params on the listener,
/// the cycle breaking algorithm can deal with it.
#[derive(AudioNodeCommon)]
pub(crate) struct AudioListenerNode {
    channel_info: ChannelInfo,
    position_x: Param,
    position_y: Param,
    position_z: Param,
    forward_x: Param,
    forward_y: Param,
    forward_z: Param,
    up_x: Param,
    up_y: Param,
    up_z: Param,
}

impl AudioListenerNode {
    pub fn new() -> Self {
        Self {
            channel_info: Default::default(),
            position_x: Param::new(0.),
            position_y: Param::new(0.),
            position_z: Param::new(0.),
            forward_x: Param::new(0.),
            forward_y: Param::new(0.),
            forward_z: Param::new(-1.),
            up_x: Param::new(0.),
            up_y: Param::new(1.),
            up_z: Param::new(0.),
        }
    }
}

impl AudioNodeEngine for AudioListenerNode {
    fn node_type(&self) -> AudioNodeType {
        AudioNodeType::AudioListenerNode
    }

    fn process(&mut self, mut inputs: Chunk, info: &BlockInfo) -> Chunk {
        debug_assert!(inputs.len() == 0);

        // XXXManishearth in the common case when all of these are constant,
        // it would be nice to instead send just the constant values down
        let mut block = Block::for_channels_explicit(9);
        self.position_x.flush_to_block(info, block.data_chan_mut(0));
        self.position_y.flush_to_block(info, block.data_chan_mut(1));
        self.position_z.flush_to_block(info, block.data_chan_mut(2));
        self.forward_x.flush_to_block(info, block.data_chan_mut(3));
        self.forward_y.flush_to_block(info, block.data_chan_mut(4));
        self.forward_z.flush_to_block(info, block.data_chan_mut(5));
        self.up_x.flush_to_block(info, block.data_chan_mut(6));
        self.up_y.flush_to_block(info, block.data_chan_mut(7));
        self.up_z.flush_to_block(info, block.data_chan_mut(8));

        inputs.blocks.push(block);
        inputs
    }

    fn input_count(&self) -> u32 {
        0
    }

    fn get_param(&mut self, id: ParamType) -> &mut Param {
        match id {
            ParamType::Position(ParamDir::X) => &mut self.position_x,
            ParamType::Position(ParamDir::Y) => &mut self.position_y,
            ParamType::Position(ParamDir::Z) => &mut self.position_z,
            ParamType::Forward(ParamDir::X) => &mut self.forward_x,
            ParamType::Forward(ParamDir::Y) => &mut self.forward_y,
            ParamType::Forward(ParamDir::Z) => &mut self.forward_z,
            ParamType::Up(ParamDir::X) => &mut self.up_x,
            ParamType::Up(ParamDir::Y) => &mut self.up_y,
            ParamType::Up(ParamDir::Z) => &mut self.up_z,
            _ => panic!("Unknown param {:?} for AudioListenerNode", id),
        }
    }
}
