/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate servo_media_derive;

extern crate servo_media_player as player;

extern crate byte_slice_cast;
extern crate euclid;
extern crate log;
extern crate num_complex;
extern crate num_traits;
extern crate petgraph;
extern crate smallvec;
extern crate speexdsp_resampler;
#[macro_use]
pub mod macros;
extern crate servo_media_streams;
extern crate servo_media_traits;

pub mod analyser_node;
pub mod biquad_filter_node;
pub mod block;
pub mod buffer_source_node;
pub mod channel_node;
pub mod constant_source_node;
pub mod context;
pub mod decoder;
pub mod destination_node;
pub mod gain_node;
pub mod graph;
pub mod iir_filter_node;
pub mod listener;
pub mod media_element_source_node;
pub mod media_stream_destination_node;
pub mod media_stream_source_node;
pub mod node;
pub mod offline_sink;
pub mod oscillator_node;
pub mod panner_node;
pub mod param;
pub mod render_thread;
pub mod sink;
pub mod stereo_panner;
pub mod wave_shaper_node;

pub trait AudioBackend {
    type Sink: sink::AudioSink + 'static;
    fn make_decoder() -> Box<dyn decoder::AudioDecoder>;
    fn make_sink() -> Result<Self::Sink, sink::AudioSinkError>;
    fn make_streamreader(
        id: servo_media_streams::MediaStreamId,
        sample_rate: f32,
    ) -> Result<Box<dyn AudioStreamReader + Send>, sink::AudioSinkError>;
}

pub trait AudioStreamReader {
    fn pull(&self) -> block::Block;
    fn start(&self);
    fn stop(&self);
}
