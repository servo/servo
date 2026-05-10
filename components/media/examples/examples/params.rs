/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

extern crate servo_media;
extern crate servo_media_auto;

use std::sync::Arc;
use std::{thread, time};

use servo_media::audio::gain_node::GainNodeOptions;
use servo_media::audio::node::{AudioNodeInit, AudioNodeMessage, AudioScheduledSourceNodeMessage};
use servo_media::audio::param::{ParamType, RampKind, UserAutomationEvent};
use servo_media::{ClientContextId, ServoMedia};

fn run_example(servo_media: Arc<ServoMedia>) {
    let context = servo_media
        .create_audio_context(&ClientContextId::build(1, 1), Default::default())
        .unwrap();
    let context = context.lock().unwrap();
    let dest = context.dest_node();
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
    context.connect_ports(osc.output(0), gain.input(0));
    context.connect_ports(gain.output(0), dest.input(0));
    let _ = context.resume();
    context.message_node(
        osc,
        AudioNodeMessage::AudioScheduledSourceNode(AudioScheduledSourceNodeMessage::Start(0.)),
    );
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
    // 0.75s - 1.5s: Exponentially ramp gain to 1
    context.message_node(
        gain,
        AudioNodeMessage::SetParam(
            ParamType::Gain,
            UserAutomationEvent::RampToValueAtTime(RampKind::Exponential, 1., 1.5),
        ),
    );
    // 0.75s - 1.75s: Linearly ramp frequency to 880Hz
    context.message_node(
        osc,
        AudioNodeMessage::SetParam(
            ParamType::Frequency,
            UserAutomationEvent::RampToValueAtTime(RampKind::Linear, 880., 1.75),
        ),
    );
    // 1.75s - 2.5s: Exponentially ramp frequency to 110Hz
    context.message_node(
        osc,
        AudioNodeMessage::SetParam(
            ParamType::Frequency,
            UserAutomationEvent::RampToValueAtTime(RampKind::Exponential, 110., 2.5),
        ),
    );

    // 2.75s: Exponentially approach 110Hz
    context.message_node(
        osc,
        AudioNodeMessage::SetParam(
            ParamType::Frequency,
            UserAutomationEvent::SetTargetAtTime(1100., 2.75, 1.1),
        ),
    );
    // 3.3s: But actually stop at 3.3Hz and hold
    context.message_node(
        osc,
        AudioNodeMessage::SetParam(
            ParamType::Frequency,
            UserAutomationEvent::CancelAndHoldAtTime(3.3),
        ),
    );
    thread::sleep(time::Duration::from_millis(5000));
}

fn main() {
    ServoMedia::init::<servo_media_auto::Backend>();
    let servo_media = ServoMedia::get();
    run_example(servo_media);
}
