use servo_media_streams::MediaSocket;

use crate::block::Chunk;
use crate::node::{AudioNodeEngine, AudioNodeType, BlockInfo, ChannelInfo};
use crate::sink::AudioSink;

#[derive(AudioNodeCommon)]
pub(crate) struct MediaStreamDestinationNode {
    channel_info: ChannelInfo,
    sink: Box<dyn AudioSink + 'static>,
}

impl MediaStreamDestinationNode {
    pub fn new(
        socket: Box<dyn MediaSocket>,
        sample_rate: f32,
        sink: Box<dyn AudioSink + 'static>,
        channel_info: ChannelInfo,
    ) -> Self {
        sink.init_stream(channel_info.count, sample_rate, socket)
            .expect("init_stream failed");
        sink.play().expect("Sink didn't start");
        MediaStreamDestinationNode { channel_info, sink }
    }
}

impl AudioNodeEngine for MediaStreamDestinationNode {
    fn node_type(&self) -> AudioNodeType {
        AudioNodeType::MediaStreamDestinationNode
    }

    fn process(&mut self, inputs: Chunk, _: &BlockInfo) -> Chunk {
        self.sink
            .push_data(inputs)
            .expect("Pushing to stream failed");
        Chunk::default()
    }

    fn output_count(&self) -> u32 {
        0
    }
}
