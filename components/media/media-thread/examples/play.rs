extern crate servo_media;
extern crate servo_media_auto;

use std::sync::Arc;
use std::{thread, time};

use servo_media::audio::gain_node::GainNodeOptions;
use servo_media::audio::node::{
    AudioNodeInit, AudioNodeMessage, AudioScheduledSourceNodeMessage, OnEndedCallback,
};
use servo_media::audio::param::{ParamType, UserAutomationEvent};
use servo_media::{ClientContextId, ServoMedia};

fn run_example(servo_media: Arc<ServoMedia>) {
    let context = servo_media
        .create_audio_context(&ClientContextId::build(1, 1), Default::default())
        .unwrap();
    let context = context.lock().unwrap();
    let osc = context
        .create_node(
            AudioNodeInit::OscillatorNode(Default::default()),
            Default::default(),
        )
        .expect("Failed to create oscillator node");
    let mut options = GainNodeOptions::default();
    options.gain = 0.5;
    let gain = context
        .create_node(AudioNodeInit::GainNode(options), Default::default())
        .expect("Failed to create gain node");
    let dest = context.dest_node();
    context.connect_ports(osc.output(0), gain.input(0));
    context.connect_ports(gain.output(0), dest.input(0));
    context.message_node(
        osc,
        AudioNodeMessage::AudioScheduledSourceNode(AudioScheduledSourceNodeMessage::Start(0.)),
    );
    context.message_node(
        osc,
        AudioNodeMessage::AudioScheduledSourceNode(AudioScheduledSourceNodeMessage::Stop(3.)),
    );
    let callback = OnEndedCallback::new(|| {
        println!("Playback ended");
    });
    context.message_node(
        osc,
        AudioNodeMessage::AudioScheduledSourceNode(
            AudioScheduledSourceNodeMessage::RegisterOnEndedCallback(callback),
        ),
    );
    assert_eq!(context.current_time(), 0.);
    let _ = context.resume();
    // 0.5s: Set frequency to 110Hz
    context.message_node(
        osc,
        AudioNodeMessage::SetParam(
            ParamType::Frequency,
            UserAutomationEvent::SetValueAtTime(110., 0.5),
        ),
    );
    // 1s: Set frequency to 220Hz
    context.message_node(
        osc,
        AudioNodeMessage::SetParam(
            ParamType::Frequency,
            UserAutomationEvent::SetValueAtTime(220., 1.),
        ),
    );
    // 0.75s: Set gain to 0.25
    context.message_node(
        gain,
        AudioNodeMessage::SetParam(
            ParamType::Gain,
            UserAutomationEvent::SetValueAtTime(0.25, 0.75),
        ),
    );
    thread::sleep(time::Duration::from_millis(1200));
    // 1.2s: Suspend processing
    let _ = context.suspend();
    thread::sleep(time::Duration::from_millis(500));
    // 1.7s: Resume processing
    let _ = context.resume();
    let current_time = context.current_time();
    assert!(current_time > 0.);
    // Leave some time to enjoy the silence after stopping the
    // oscillator node.
    thread::sleep(time::Duration::from_millis(5000));
    // And check that we keep incrementing playback time.
    assert!(current_time < context.current_time());
    let _ = context.close();
}

fn main() {
    ServoMedia::init::<servo_media_auto::Backend>();
    let servo_media = ServoMedia::get();
    run_example(servo_media);
}
