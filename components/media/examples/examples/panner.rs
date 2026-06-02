/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

extern crate servo_media;
extern crate servo_media_auto;

use std::sync::Arc;
use std::{thread, time};

use servo_media::audio::node::{AudioNodeInit, AudioNodeMessage, AudioScheduledSourceNodeMessage};
use servo_media::audio::panner_node::PannerNodeOptions;
use servo_media::audio::param::{ParamDir, ParamType, RampKind, UserAutomationEvent};
use servo_media::{ClientContextId, ServoMedia};

fn run_example(servo_media: Arc<ServoMedia>) {
    let context = servo_media
        .create_audio_context(&ClientContextId::build(1, 1), Default::default())
        .unwrap();
    let context = context.lock().unwrap();
    let dest = context.dest_node();
    let listener = context.listener();
    let osc = context
        .create_node(
            AudioNodeInit::OscillatorNode(Default::default()),
            Default::default(),
        )
        .expect("Failed to create oscillator node");
    let mut options = PannerNodeOptions::default();
    options.cone_outer_angle = 0.;
    options.position_x = 100.;
    options.position_y = 0.;
    options.position_z = 100.;
    options.ref_distance = 100.;
    options.rolloff_factor = 0.01;
    let panner = context
        .create_node(AudioNodeInit::PannerNode(options), Default::default())
        .expect("Failed to create panner node");
    context.connect_ports(osc.output(0), panner.input(0));
    context.connect_ports(panner.output(0), dest.input(0));
    let _ = context.resume();
    context.message_node(
        osc,
        AudioNodeMessage::AudioScheduledSourceNode(AudioScheduledSourceNodeMessage::Start(0.)),
    );
    // trace a square around your head twice
    context.message_node(
        panner,
        AudioNodeMessage::SetParam(
            ParamType::Position(ParamDir::X),
            UserAutomationEvent::RampToValueAtTime(RampKind::Linear, -100., 0.2),
        ),
    );
    context.message_node(
        panner,
        AudioNodeMessage::SetParam(
            ParamType::Position(ParamDir::Z),
            UserAutomationEvent::RampToValueAtTime(RampKind::Linear, 100., 0.2),
        ),
    );
    context.message_node(
        panner,
        AudioNodeMessage::SetParam(
            ParamType::Position(ParamDir::X),
            UserAutomationEvent::RampToValueAtTime(RampKind::Linear, -100., 0.4),
        ),
    );
    context.message_node(
        panner,
        AudioNodeMessage::SetParam(
            ParamType::Position(ParamDir::Z),
            UserAutomationEvent::RampToValueAtTime(RampKind::Linear, -100., 0.4),
        ),
    );
    context.message_node(
        panner,
        AudioNodeMessage::SetParam(
            ParamType::Position(ParamDir::X),
            UserAutomationEvent::RampToValueAtTime(RampKind::Linear, 100., 0.6),
        ),
    );
    context.message_node(
        panner,
        AudioNodeMessage::SetParam(
            ParamType::Position(ParamDir::Z),
            UserAutomationEvent::RampToValueAtTime(RampKind::Linear, -100., 0.6),
        ),
    );
    context.message_node(
        panner,
        AudioNodeMessage::SetParam(
            ParamType::Position(ParamDir::X),
            UserAutomationEvent::RampToValueAtTime(RampKind::Linear, 100., 0.8),
        ),
    );
    context.message_node(
        panner,
        AudioNodeMessage::SetParam(
            ParamType::Position(ParamDir::Z),
            UserAutomationEvent::RampToValueAtTime(RampKind::Linear, 100., 0.8),
        ),
    );

    context.message_node(
        panner,
        AudioNodeMessage::SetParam(
            ParamType::Position(ParamDir::X),
            UserAutomationEvent::RampToValueAtTime(RampKind::Linear, -100., 1.0),
        ),
    );
    context.message_node(
        panner,
        AudioNodeMessage::SetParam(
            ParamType::Position(ParamDir::Z),
            UserAutomationEvent::RampToValueAtTime(RampKind::Linear, 100., 1.0),
        ),
    );
    context.message_node(
        panner,
        AudioNodeMessage::SetParam(
            ParamType::Position(ParamDir::X),
            UserAutomationEvent::RampToValueAtTime(RampKind::Linear, -100., 1.2),
        ),
    );
    context.message_node(
        panner,
        AudioNodeMessage::SetParam(
            ParamType::Position(ParamDir::Z),
            UserAutomationEvent::RampToValueAtTime(RampKind::Linear, -100., 1.2),
        ),
    );
    context.message_node(
        panner,
        AudioNodeMessage::SetParam(
            ParamType::Position(ParamDir::X),
            UserAutomationEvent::RampToValueAtTime(RampKind::Linear, 100., 1.4),
        ),
    );
    context.message_node(
        panner,
        AudioNodeMessage::SetParam(
            ParamType::Position(ParamDir::Z),
            UserAutomationEvent::RampToValueAtTime(RampKind::Linear, -100., 1.4),
        ),
    );
    context.message_node(
        panner,
        AudioNodeMessage::SetParam(
            ParamType::Position(ParamDir::X),
            UserAutomationEvent::RampToValueAtTime(RampKind::Linear, 100., 1.6),
        ),
    );
    context.message_node(
        panner,
        AudioNodeMessage::SetParam(
            ParamType::Position(ParamDir::Z),
            UserAutomationEvent::RampToValueAtTime(RampKind::Linear, 100., 1.6),
        ),
    );
    // now it runs away
    context.message_node(
        panner,
        AudioNodeMessage::SetParam(
            ParamType::Position(ParamDir::Z),
            UserAutomationEvent::RampToValueAtTime(RampKind::Linear, 10000., 3.),
        ),
    );
    context.message_node(
        listener,
        AudioNodeMessage::SetParam(
            ParamType::Position(ParamDir::Z),
            UserAutomationEvent::SetValueAtTime(0., 3.),
        ),
    );
    // chase it
    context.message_node(
        listener,
        AudioNodeMessage::SetParam(
            ParamType::Position(ParamDir::Z),
            UserAutomationEvent::RampToValueAtTime(RampKind::Linear, 10000., 4.),
        ),
    );
    thread::sleep(time::Duration::from_millis(4000));
}

fn main() {
    ServoMedia::init::<servo_media_auto::Backend>();
    let servo_media = ServoMedia::get();
    run_example(servo_media);
}
