/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

extern crate servo_media;
extern crate servo_media_auto;

use std::sync::Arc;
use std::{thread, time};

use servo_media::audio::constant_source_node::ConstantSourceNodeOptions;
use servo_media::audio::gain_node::GainNodeOptions;
use servo_media::audio::node::{AudioNodeInit, AudioNodeMessage, AudioScheduledSourceNodeMessage};
use servo_media::audio::param::{ParamType, UserAutomationEvent};
use servo_media::{ClientContextId, ServoMedia};

fn run_example(servo_media: Arc<ServoMedia>) {
    let context = servo_media
        .create_audio_context(&ClientContextId::build(1, 1), Default::default())
        .unwrap();
    let context = context.lock().unwrap();
    let dest = context.dest_node();

    //Initializing the values vector for SetValueCurve function
    let values = vec![
        0., 0., 0., 0., 1., 1., 1., 1., 0., 0., 0., 0., 1., 1., 1., 1., 0., 0., 0., 0.,
    ];
    let start_time = 0.;
    let end_time = 5.;
    let n = values.len() as f32;
    let value_next = values[(n - 1.) as usize];

    let cs = context
        .create_node(
            AudioNodeInit::ConstantSourceNode(ConstantSourceNodeOptions::default()),
            Default::default(),
        )
        .expect("Failed to create ConstantSourceNode node");

    let mut gain_options = GainNodeOptions::default();
    gain_options.gain = 0.0;
    let gain = context
        .create_node(
            AudioNodeInit::GainNode(gain_options.clone()),
            Default::default(),
        )
        .expect("Failed to create gain node");

    let osc = context
        .create_node(
            AudioNodeInit::OscillatorNode(Default::default()),
            Default::default(),
        )
        .expect("Failed to create oscillator node");

    context.connect_ports(osc.output(0), gain.input(0));
    context.connect_ports(cs.output(0), gain.param(ParamType::Gain));
    context.connect_ports(gain.output(0), dest.input(0));

    let _ = context.resume();
    context.message_node(
        osc,
        AudioNodeMessage::AudioScheduledSourceNode(AudioScheduledSourceNodeMessage::Start(0.)),
    );

    context.message_node(
        gain,
        AudioNodeMessage::AudioScheduledSourceNode(AudioScheduledSourceNodeMessage::Start(0.)),
    );

    context.message_node(
        cs,
        AudioNodeMessage::AudioScheduledSourceNode(AudioScheduledSourceNodeMessage::Start(0.)),
    );

    context.message_node(
        cs,
        AudioNodeMessage::SetParam(
            ParamType::Offset,
            UserAutomationEvent::SetValueCurveAtTime(values, start_time, end_time),
        ),
    );

    context.message_node(
        osc,
        AudioNodeMessage::SetParam(
            ParamType::Frequency,
            UserAutomationEvent::SetValueAtTime(value_next, end_time),
        ),
    );

    thread::sleep(time::Duration::from_millis(7000));
    let _ = context.close();
}

fn main() {
    ServoMedia::init::<servo_media_auto::Backend>();
    let servo_media = ServoMedia::get();
    run_example(servo_media);
}
