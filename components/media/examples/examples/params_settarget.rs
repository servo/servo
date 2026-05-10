/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

extern crate servo_media;
extern crate servo_media_auto;

use std::sync::Arc;
use std::{thread, time};

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
    context.connect_ports(osc.output(0), dest.input(0));
    let _ = context.resume();
    context.message_node(
        osc,
        AudioNodeMessage::AudioScheduledSourceNode(AudioScheduledSourceNodeMessage::Start(0.)),
    );
    // 0.1s: Set frequency to 110Hz
    context.message_node(
        osc,
        AudioNodeMessage::SetParam(
            ParamType::Frequency,
            UserAutomationEvent::SetValueAtTime(110., 0.1),
        ),
    );
    // 0.3s: Start increasing frequency to 440Hz exponentially with a time constant of 1
    context.message_node(
        osc,
        AudioNodeMessage::SetParam(
            ParamType::Frequency,
            UserAutomationEvent::SetTargetAtTime(440., 0.3, 1.),
        ),
    );
    // 1.5s: Start increasing frequency to 1760Hz exponentially
    // this event effectively doesn't happen, but instead sets a starting point
    // for the next ramp event
    context.message_node(
        osc,
        AudioNodeMessage::SetParam(
            ParamType::Frequency,
            UserAutomationEvent::SetTargetAtTime(1760., 1.5, 0.1),
        ),
    );
    // 1.5s - 3s Linearly ramp down from the previous event (1.5s) to 110Hz
    context.message_node(
        osc,
        AudioNodeMessage::SetParam(
            ParamType::Frequency,
            UserAutomationEvent::RampToValueAtTime(RampKind::Linear, 110., 3.0),
        ),
    );
    thread::sleep(time::Duration::from_millis(5000));
}

fn main() {
    ServoMedia::init::<servo_media_auto::Backend>();
    let servo_media = ServoMedia::get();
    run_example(servo_media);
}
