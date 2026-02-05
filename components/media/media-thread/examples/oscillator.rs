extern crate servo_media;
extern crate servo_media_auto;

use std::sync::Arc;
use std::{thread, time};

use servo_media::audio::node::{AudioNodeInit, AudioNodeMessage, AudioScheduledSourceNodeMessage};
use servo_media::audio::oscillator_node::OscillatorNodeOptions;
use servo_media::audio::oscillator_node::OscillatorType::{Custom, Sawtooth, Square, Triangle};
use servo_media::{ClientContextId, ServoMedia};

fn run_example(servo_media: Arc<ServoMedia>) {
    let context = servo_media
        .create_audio_context(&ClientContextId::build(1, 1), Default::default())
        .unwrap();
    let context = context.lock().unwrap();
    let dest = context.dest_node();
    let mut options = OscillatorNodeOptions::default();
    let osc1 = context
        .create_node(
            AudioNodeInit::OscillatorNode(options.clone()),
            Default::default(),
        )
        .expect("Failed to create oscillator node");
    context.connect_ports(osc1.output(0), dest.input(0));
    let _ = context.resume();
    context.message_node(
        osc1,
        AudioNodeMessage::AudioScheduledSourceNode(AudioScheduledSourceNodeMessage::Start(0.)),
    );

    thread::sleep(time::Duration::from_millis(3000));
    let _ = context.close();
    thread::sleep(time::Duration::from_millis(3000));

    options.oscillator_type = Square;
    let context = servo_media
        .create_audio_context(&ClientContextId::build(1, 2), Default::default())
        .unwrap();
    let context = context.lock().unwrap();
    let dest = context.dest_node();
    let osc2 = context
        .create_node(
            AudioNodeInit::OscillatorNode(options.clone()),
            Default::default(),
        )
        .expect("Failed to create oscillator node");
    context.connect_ports(osc2.output(0), dest.input(0));
    let _ = context.resume();
    context.message_node(
        osc2,
        AudioNodeMessage::AudioScheduledSourceNode(AudioScheduledSourceNodeMessage::Start(0.)),
    );

    thread::sleep(time::Duration::from_millis(3000));
    let _ = context.close();
    thread::sleep(time::Duration::from_millis(1000));

    options.oscillator_type = Sawtooth;
    let context = servo_media
        .create_audio_context(&ClientContextId::build(1, 3), Default::default())
        .unwrap();
    let context = context.lock().unwrap();
    let dest = context.dest_node();
    let osc3 = context
        .create_node(
            AudioNodeInit::OscillatorNode(options.clone()),
            Default::default(),
        )
        .expect("Failed to create oscillator node");
    context.connect_ports(osc3.output(0), dest.input(0));
    thread::sleep(time::Duration::from_millis(3000));

    let _ = context.resume();
    context.message_node(
        osc3,
        AudioNodeMessage::AudioScheduledSourceNode(AudioScheduledSourceNodeMessage::Start(0.)),
    );
    thread::sleep(time::Duration::from_millis(3000));
    let _ = context.close();
    thread::sleep(time::Duration::from_millis(1000));

    options.oscillator_type = Triangle;
    let context = servo_media
        .create_audio_context(&ClientContextId::build(1, 4), Default::default())
        .unwrap();
    let context = context.lock().unwrap();
    let dest = context.dest_node();
    let osc4 = context
        .create_node(
            AudioNodeInit::OscillatorNode(options.clone()),
            Default::default(),
        )
        .expect("Failed to create oscillator node");
    context.connect_ports(osc4.output(0), dest.input(0));
    thread::sleep(time::Duration::from_millis(3000));

    let _ = context.resume();
    context.message_node(
        osc4,
        AudioNodeMessage::AudioScheduledSourceNode(AudioScheduledSourceNodeMessage::Start(0.)),
    );
    thread::sleep(time::Duration::from_millis(3000));
    let _ = context.close();

    thread::sleep(time::Duration::from_millis(3000));

    options.oscillator_type = Custom;

    let context = servo_media
        .create_audio_context(&ClientContextId::build(1, 5), Default::default())
        .unwrap();
    let context = context.lock().unwrap();
    let dest = context.dest_node();
    let osc5 = context
        .create_node(
            AudioNodeInit::OscillatorNode(options.clone()),
            Default::default(),
        )
        .expect("Failed to create oscillator node");
    context.connect_ports(osc5.output(0), dest.input(0));
    thread::sleep(time::Duration::from_millis(3000));

    let _ = context.resume();
    context.message_node(
        osc4,
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
