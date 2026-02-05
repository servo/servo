extern crate servo_media;
extern crate servo_media_auto;

use std::sync::Arc;
use std::{thread, time};

use servo_media::audio::biquad_filter_node::{
    BiquadFilterNodeMessage, BiquadFilterNodeOptions, FilterType,
};
use servo_media::audio::node::{AudioNodeInit, AudioNodeMessage, AudioScheduledSourceNodeMessage};
use servo_media::audio::oscillator_node::OscillatorNodeOptions;
use servo_media::audio::param::{ParamType, RampKind, UserAutomationEvent};
use servo_media::{ClientContextId, ServoMedia};

fn run_example(servo_media: Arc<ServoMedia>) {
    let context = servo_media
        .create_audio_context(&ClientContextId::build(1, 1), Default::default())
        .unwrap();
    let context = context.lock().unwrap();
    let dest = context.dest_node();
    let mut options = OscillatorNodeOptions::default();
    options.freq = 100.;
    let osc1 = context
        .create_node(
            AudioNodeInit::OscillatorNode(options.clone()),
            Default::default(),
        )
        .expect("Failed to create oscillator node");
    options.freq = 800.;
    let osc2 = context
        .create_node(
            AudioNodeInit::OscillatorNode(options.clone()),
            Default::default(),
        )
        .expect("Failed to create oscillator node");
    let mut options = BiquadFilterNodeOptions::default();
    options.frequency = 50.;
    options.filter = FilterType::LowPass;
    let biquad = context
        .create_node(AudioNodeInit::BiquadFilterNode(options), Default::default())
        .expect("Failed to create biquad filter node");
    context.connect_ports(osc1.output(0), biquad.input(0));
    context.connect_ports(osc2.output(0), biquad.input(0));
    context.connect_ports(biquad.output(0), dest.input(0));
    let _ = context.resume();
    context.message_node(
        osc1,
        AudioNodeMessage::AudioScheduledSourceNode(AudioScheduledSourceNodeMessage::Start(0.)),
    );
    context.message_node(
        osc2,
        AudioNodeMessage::AudioScheduledSourceNode(AudioScheduledSourceNodeMessage::Start(0.)),
    );
    context.message_node(
        biquad,
        AudioNodeMessage::SetParam(
            ParamType::Frequency,
            UserAutomationEvent::RampToValueAtTime(RampKind::Linear, 1000., 2.),
        ),
    );

    thread::sleep(time::Duration::from_millis(2200));
    context.message_node(
        biquad,
        AudioNodeMessage::BiquadFilterNode(BiquadFilterNodeMessage::SetFilterType(
            FilterType::BandPass,
        )),
    );

    thread::sleep(time::Duration::from_millis(1000));
}

fn main() {
    ServoMedia::init::<servo_media_auto::Backend>();
    let servo_media = ServoMedia::get();
    run_example(servo_media);
}
