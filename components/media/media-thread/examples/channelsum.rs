extern crate servo_media;
extern crate servo_media_auto;

use std::sync::Arc;
use std::{thread, time};

use servo_media::audio::channel_node::ChannelNodeOptions;
use servo_media::audio::gain_node::GainNodeOptions;
use servo_media::audio::node::{AudioNodeInit, AudioNodeMessage, AudioScheduledSourceNodeMessage};
use servo_media::audio::oscillator_node::OscillatorNodeOptions;
use servo_media::{ClientContextId, ServoMedia};

fn run_example(servo_media: Arc<ServoMedia>) {
    let context = servo_media
        .create_audio_context(&ClientContextId::build(1, 1), Default::default())
        .unwrap();
    let context = context.lock().unwrap();
    let mut options = OscillatorNodeOptions::default();
    let osc = context
        .create_node(
            AudioNodeInit::OscillatorNode(options.clone()),
            Default::default(),
        )
        .expect("Failed to create oscillator node");
    options.freq = 213.;
    let osc2 = context
        .create_node(
            AudioNodeInit::OscillatorNode(options.clone()),
            Default::default(),
        )
        .expect("Failed to create oscillator node");
    options.freq = 100.;
    let osc3 = context
        .create_node(AudioNodeInit::OscillatorNode(options), Default::default())
        .expect("Failed to create oscillator node");
    let mut options = GainNodeOptions::default();
    options.gain = 0.7;
    let gain = context
        .create_node(AudioNodeInit::GainNode(options.clone()), Default::default())
        .expect("Failed to create gain node");

    let options = ChannelNodeOptions { channels: 2 };
    let merger = context
        .create_node(
            AudioNodeInit::ChannelMergerNode(options),
            Default::default(),
        )
        .expect("Failed to create channel merger node");

    let dest = context.dest_node();
    context.connect_ports(osc.output(0), merger.input(0));
    context.connect_ports(osc2.output(0), merger.input(1));
    context.connect_ports(merger.output(0), gain.input(0));
    context.connect_ports(osc3.output(0), gain.input(0));
    context.connect_ports(gain.output(0), dest.input(0));
    context.message_node(
        osc,
        AudioNodeMessage::AudioScheduledSourceNode(AudioScheduledSourceNodeMessage::Start(0.)),
    );
    context.message_node(
        osc2,
        AudioNodeMessage::AudioScheduledSourceNode(AudioScheduledSourceNodeMessage::Start(0.)),
    );
    context.message_node(
        osc3,
        AudioNodeMessage::AudioScheduledSourceNode(AudioScheduledSourceNodeMessage::Start(0.)),
    );
    let _ = context.resume();

    thread::sleep(time::Duration::from_millis(2000));
    context.message_node(dest, AudioNodeMessage::SetChannelCount(1));
    thread::sleep(time::Duration::from_millis(2000));
    let _ = context.close();
}

fn main() {
    ServoMedia::init::<servo_media_auto::Backend>();
    let servo_media = ServoMedia::get();
    run_example(servo_media);
}
