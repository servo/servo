extern crate servo_media;
extern crate servo_media_auto;

use std::sync::Arc;
use std::{thread, time};

use servo_media::audio::node::{AudioNodeInit, AudioNodeMessage, AudioScheduledSourceNodeMessage};
use servo_media::audio::oscillator_node::OscillatorNodeOptions;
use servo_media::streams::MediaStreamType;
use servo_media::{ClientContextId, ServoMedia};

fn run_example(servo_media: Arc<ServoMedia>) {
    let context = servo_media
        .create_audio_context(&ClientContextId::build(1, 1), Default::default())
        .unwrap();
    let context = context.lock().unwrap();
    let options = OscillatorNodeOptions::default();
    let osc1 = context
        .create_node(
            AudioNodeInit::OscillatorNode(options.clone()),
            Default::default(),
        )
        .expect("Failed to create oscillator node");

    let (socket, id) = servo_media.create_stream_and_socket(MediaStreamType::Audio);
    let dest = context
        .create_node(
            AudioNodeInit::MediaStreamDestinationNode(socket),
            Default::default(),
        )
        .expect("Failed to create stream destination node");
    context.connect_ports(osc1.output(0), dest.input(0));

    let mut output = servo_media.create_stream_output();
    output.add_stream(&id);
    let _ = context.resume();
    context.message_node(
        osc1,
        AudioNodeMessage::AudioScheduledSourceNode(AudioScheduledSourceNodeMessage::Start(0.)),
    );

    thread::sleep(time::Duration::from_millis(3000));
    let _ = context.close();
    thread::sleep(time::Duration::from_millis(1000));
}

fn main() {
    ServoMedia::init::<servo_media_auto::Backend>();
    let servo_media = ServoMedia::get();
    run_example(servo_media);
}
