/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::sync::mpsc::{Receiver, Sender};

use malloc_size_of_derive::MallocSizeOf;
use servo_media_streams::{MediaSocket, MediaStreamId};

use crate::analyser_node::AnalyserNode;
use crate::biquad_filter_node::BiquadFilterNode;
use crate::block::{Chunk, FRAMES_PER_BLOCK, Tick};
use crate::buffer_source_node::AudioBufferSourceNode;
use crate::channel_node::{ChannelMergerNode, ChannelSplitterNode};
use crate::constant_source_node::ConstantSourceNode;
use crate::context::{AudioContextOptions, ProcessingState, StateChangeResult};
use crate::gain_node::GainNode;
use crate::graph::{AudioGraph, InputPort, NodeId, OutputPort, PortId};
use crate::iir_filter_node::IIRFilterNode;
use crate::media_element_source_node::MediaElementSourceNode;
use crate::media_stream_destination_node::MediaStreamDestinationNode;
use crate::media_stream_source_node::MediaStreamSourceNode;
use crate::node::{AudioNodeEngine, AudioNodeInit, AudioNodeMessage, BlockInfo, ChannelInfo};
use crate::offline_sink::OfflineAudioSink;
use crate::oscillator_node::OscillatorNode;
use crate::panner_node::PannerNode;
use crate::sink::{AudioSink, AudioSinkError};
use crate::stereo_panner::StereoPannerNode;
use crate::wave_shaper_node::WaveShaperNode;
use crate::{AudioBackend, AudioStreamReader};

pub type SinkEosCallback = Box<dyn Fn(Box<dyn AsRef<[f32]>>) + Send + Sync + 'static>;

#[derive(MallocSizeOf)]
pub enum AudioRenderThreadMsg {
    CreateNode(AudioNodeInit, Sender<Option<NodeId>>, ChannelInfo),
    ConnectPorts(PortId<OutputPort>, PortId<InputPort>),
    MessageNode(NodeId, AudioNodeMessage),
    Resume(Sender<StateChangeResult>),
    Suspend(Sender<StateChangeResult>),
    Close(Sender<StateChangeResult>),
    SinkNeedData,
    GetCurrentTime(Sender<f64>),

    DisconnectAllFrom(NodeId),
    DisconnectOutput(PortId<OutputPort>),
    DisconnectBetween(NodeId, NodeId),
    DisconnectTo(NodeId, PortId<InputPort>),
    DisconnectOutputBetween(PortId<OutputPort>, NodeId),
    DisconnectOutputBetweenTo(PortId<OutputPort>, PortId<InputPort>),

    SetSinkEosCallback(#[ignore_malloc_size_of = "Fn"] SinkEosCallback),

    SetMute(bool),
}

pub enum Sink {
    RealTime(Box<dyn AudioSink>),
    Offline(OfflineAudioSink),
}

impl AudioSink for Sink {
    fn init(
        &self,
        sample_rate: f32,
        sender: Sender<AudioRenderThreadMsg>,
    ) -> Result<(), AudioSinkError> {
        match *self {
            Sink::RealTime(ref sink) => sink.init(sample_rate, sender),
            Sink::Offline(ref sink) => {
                sink.init(sample_rate, sender).unwrap();
                Ok(())
            },
        }
    }

    fn init_stream(&self, _: u8, _: f32, _: Box<dyn MediaSocket>) -> Result<(), AudioSinkError> {
        unreachable!("Sink should never be used for MediaStreamDestinationNode")
    }

    fn play(&self) -> Result<(), AudioSinkError> {
        match *self {
            Sink::RealTime(ref sink) => sink.play(),
            Sink::Offline(ref sink) => {
                sink.play().unwrap();
                Ok(())
            },
        }
    }

    fn stop(&self) -> Result<(), AudioSinkError> {
        match *self {
            Sink::RealTime(ref sink) => sink.stop(),
            Sink::Offline(ref sink) => {
                sink.stop().unwrap();
                Ok(())
            },
        }
    }

    fn has_enough_data(&self) -> bool {
        match *self {
            Sink::RealTime(ref sink) => sink.has_enough_data(),
            Sink::Offline(ref sink) => sink.has_enough_data(),
        }
    }

    fn push_data(&self, chunk: Chunk) -> Result<(), AudioSinkError> {
        match *self {
            Sink::RealTime(ref sink) => sink.push_data(chunk),
            Sink::Offline(ref sink) => {
                sink.push_data(chunk).unwrap();
                Ok(())
            },
        }
    }

    fn set_eos_callback(
        &self,
        callback: Box<dyn Fn(Box<dyn AsRef<[f32]>>) + Send + Sync + 'static>,
    ) {
        match *self {
            Sink::RealTime(ref sink) => sink.set_eos_callback(callback),
            Sink::Offline(ref sink) => sink.set_eos_callback(callback),
        }
    }
}

pub type ReaderFactoryCallback =
    dyn Fn(MediaStreamId, f32) -> Result<Box<dyn AudioStreamReader + Send>, AudioSinkError>;

pub struct AudioRenderThread {
    pub graph: AudioGraph,
    pub sink: Sink,
    pub sink_factory: Box<dyn Fn() -> Result<Box<dyn AudioSink + 'static>, AudioSinkError>>,
    pub reader_factory: Box<ReaderFactoryCallback>,
    pub state: ProcessingState,
    pub sample_rate: f32,
    pub current_time: f64,
    pub current_frame: Tick,
    pub muted: bool,
}

impl AudioRenderThread {
    /// Initializes the AudioRenderThread object
    ///
    /// You must call .event_loop() on this to run it!
    fn prepare_thread<B: AudioBackend>(
        sender: Sender<AudioRenderThreadMsg>,
        sample_rate: f32,
        graph: AudioGraph,
        options: AudioContextOptions,
    ) -> Result<Self, AudioSinkError> {
        let sink_factory = Box::new(|| B::make_sink().map(|s| Box::new(s) as Box<dyn AudioSink>));
        let reader_factory = Box::new(|id, sample_rate| B::make_streamreader(id, sample_rate));
        let sink = match options {
            AudioContextOptions::RealTimeAudioContext(_) => Sink::RealTime(sink_factory()?),
            AudioContextOptions::OfflineAudioContext(options) => Sink::Offline(
                OfflineAudioSink::new(options.channels as usize, options.length),
            ),
        };

        sink.init(sample_rate, sender)?;

        Ok(Self {
            graph,
            sink,
            sink_factory,
            reader_factory,
            state: ProcessingState::Suspended,
            sample_rate,
            current_time: 0.,
            current_frame: Tick(0),
            muted: false,
        })
    }

    /// Start the audio render thread
    ///
    /// In case something fails, it will instead start a thread with a dummy backend
    pub fn start<B: AudioBackend>(
        event_queue: Receiver<AudioRenderThreadMsg>,
        sender: Sender<AudioRenderThreadMsg>,
        sample_rate: f32,
        graph: AudioGraph,
        options: AudioContextOptions,
        init_sender: Sender<Result<(), AudioSinkError>>,
    ) {
        let mut thread =
            match Self::prepare_thread::<B>(sender.clone(), sample_rate, graph, options) {
                Ok(thread) => {
                    let _ = init_sender.send(Ok(()));
                    thread
                },
                Err(e) => {
                    let _ = init_sender.send(Err(e));
                    return;
                },
            };

        thread.event_loop(event_queue);
    }

    make_render_thread_state_change!(resume, Running, play);

    make_render_thread_state_change!(suspend, Suspended, stop);

    fn create_node(&mut self, node_type: AudioNodeInit, ch: ChannelInfo) -> Option<NodeId> {
        let mut needs_listener = false;
        let mut is_dest = false;
        let node: Box<dyn AudioNodeEngine> = match node_type {
            AudioNodeInit::AnalyserNode(sender) => Box::new(AnalyserNode::new(sender, ch)),
            AudioNodeInit::AudioBufferSourceNode(options) => {
                Box::new(AudioBufferSourceNode::new(options, ch))
            },
            AudioNodeInit::BiquadFilterNode(options) => {
                Box::new(BiquadFilterNode::new(options, ch, self.sample_rate))
            },
            AudioNodeInit::GainNode(options) => Box::new(GainNode::new(options, ch)),
            AudioNodeInit::StereoPannerNode(options) => {
                Box::new(StereoPannerNode::new(options, ch))
            },
            AudioNodeInit::PannerNode(options) => {
                needs_listener = true;
                Box::new(PannerNode::new(options, ch))
            },
            AudioNodeInit::MediaStreamSourceNode(id) => {
                let reader = (self.reader_factory)(id, self.sample_rate);
                Box::new(MediaStreamSourceNode::new(reader.ok()?, ch))
            },
            AudioNodeInit::OscillatorNode(options) => Box::new(OscillatorNode::new(options, ch)),
            AudioNodeInit::ChannelMergerNode(options) => {
                Box::new(ChannelMergerNode::new(options, ch))
            },
            AudioNodeInit::ConstantSourceNode(options) => {
                Box::new(ConstantSourceNode::new(options, ch))
            },
            AudioNodeInit::MediaStreamDestinationNode(socket) => {
                is_dest = true;
                Box::new(MediaStreamDestinationNode::new(
                    socket,
                    self.sample_rate,
                    (self.sink_factory)().unwrap(),
                    ch,
                ))
            },
            AudioNodeInit::ChannelSplitterNode => Box::new(ChannelSplitterNode::new(ch)),
            AudioNodeInit::WaveShaperNode(options) => Box::new(WaveShaperNode::new(options, ch)),
            AudioNodeInit::MediaElementSourceNode => Box::new(MediaElementSourceNode::new(ch)),
            AudioNodeInit::IIRFilterNode(options) => Box::new(IIRFilterNode::new(options, ch)),
            _ => unimplemented!(),
        };
        let id = self.graph.add_node(node);
        if needs_listener {
            let listener = self.graph.listener_id().output(0);
            self.graph.add_edge(listener, id.listener());
        }
        if is_dest {
            self.graph.add_extra_dest(id);
        }
        Some(id)
    }

    fn connect_ports(&mut self, output: PortId<OutputPort>, input: PortId<InputPort>) {
        self.graph.add_edge(output, input)
    }

    fn process(&mut self) -> Chunk {
        if self.muted {
            return Chunk::explicit_silence();
        }

        let info = BlockInfo {
            sample_rate: self.sample_rate,
            frame: self.current_frame,
            time: self.current_time,
        };
        self.graph.process(&info)
    }

    fn set_mute(&mut self, val: bool) {
        self.muted = val;
    }

    fn event_loop(&mut self, event_queue: Receiver<AudioRenderThreadMsg>) {
        let sample_rate = self.sample_rate;
        let handle_msg = move |context: &mut Self, msg: AudioRenderThreadMsg| -> bool {
            let mut break_loop = false;
            match msg {
                AudioRenderThreadMsg::CreateNode(node_type, tx, ch) => {
                    let _ = tx.send(context.create_node(node_type, ch));
                },
                AudioRenderThreadMsg::ConnectPorts(output, input) => {
                    context.connect_ports(output, input);
                },
                AudioRenderThreadMsg::Resume(tx) => {
                    let _ = tx.send(context.resume());
                },
                AudioRenderThreadMsg::Suspend(tx) => {
                    let _ = tx.send(context.suspend());
                },
                AudioRenderThreadMsg::Close(tx) => {
                    let _ = tx.send(context.suspend());
                    break_loop = true;
                },
                AudioRenderThreadMsg::GetCurrentTime(response) => {
                    response.send(context.current_time).unwrap()
                },
                AudioRenderThreadMsg::MessageNode(id, msg) => {
                    context.graph.node_mut(id).message(msg, sample_rate)
                },
                AudioRenderThreadMsg::SinkNeedData => {
                    // Do nothing. This will simply unblock the thread so we
                    // can restart the non-blocking event loop.
                },
                AudioRenderThreadMsg::DisconnectAllFrom(id) => {
                    context.graph.disconnect_all_from(id)
                },
                AudioRenderThreadMsg::DisconnectOutput(out) => context.graph.disconnect_output(out),
                AudioRenderThreadMsg::DisconnectBetween(from, to) => {
                    context.graph.disconnect_between(from, to)
                },
                AudioRenderThreadMsg::DisconnectTo(from, to) => {
                    context.graph.disconnect_to(from, to)
                },
                AudioRenderThreadMsg::DisconnectOutputBetween(from, to) => {
                    context.graph.disconnect_output_between(from, to)
                },
                AudioRenderThreadMsg::DisconnectOutputBetweenTo(from, to) => {
                    context.graph.disconnect_output_between_to(from, to)
                },
                AudioRenderThreadMsg::SetSinkEosCallback(callback) => {
                    context.sink.set_eos_callback(callback);
                },
                AudioRenderThreadMsg::SetMute(val) => {
                    context.set_mute(val);
                },
            };

            break_loop
        };

        loop {
            if self.sink.has_enough_data() || self.state == ProcessingState::Suspended {
                // If we are not processing audio or
                // if we have already pushed enough data into the audio sink
                // we wait for messages coming from the control thread or
                // the audio sink. The audio sink will notify whenever it
                // needs more data.
                if event_queue.recv().is_ok_and(|msg| handle_msg(self, msg)) {
                    break;
                }
            } else {
                // If we have not pushed enough data into the audio sink yet,
                // we process the control message queue
                if event_queue
                    .try_recv()
                    .is_ok_and(|msg| handle_msg(self, msg))
                {
                    break;
                }

                if self.state == ProcessingState::Suspended {
                    // Bail out if we just suspended processing.
                    continue;
                }

                // push into the audio sink the result of processing a
                // render quantum.
                let data = self.process();
                if self.sink.push_data(data).is_ok() {
                    // increment current frame by the render quantum size.
                    self.current_frame += FRAMES_PER_BLOCK;
                    self.current_time = self.current_frame / self.sample_rate as f64;
                } else {
                    eprintln!("Could not push data to audio sink");
                }
            }
        }
    }
}
