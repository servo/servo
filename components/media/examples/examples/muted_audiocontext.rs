/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

extern crate servo_media;
extern crate servo_media_auto;

use std::sync::Arc;
use std::{thread, time};

use servo_media::audio::node::{AudioNodeInit, AudioNodeMessage, AudioScheduledSourceNodeMessage};
use servo_media::audio::oscillator_node::OscillatorNodeOptions;
use servo_media::audio::oscillator_node::OscillatorType::Sawtooth;
use servo_media::{ClientContextId, ServoMedia};

fn run_example(servo_media: Arc<ServoMedia>) {
    let context_id1 = &ClientContextId::build(1, 1);
    let context1 = servo_media.create_audio_context(&context_id1, Default::default());
    {
        let context1 = context1.unwrap();
        let context = context1.lock().unwrap();
        let dest = context.dest_node();
        let options = OscillatorNodeOptions::default();
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
    }

    let context_id2 = &ClientContextId::build(1, 3);
    let context2 = servo_media.create_audio_context(&context_id2, Default::default());
    {
        let mut options = OscillatorNodeOptions::default();
        options.oscillator_type = Sawtooth;
        let context2 = context2.unwrap();
        let context = context2.lock().unwrap();
        let dest = context.dest_node();
        let osc3 = context
            .create_node(
                AudioNodeInit::OscillatorNode(options.clone()),
                Default::default(),
            )
            .expect("Failed to create oscillator node");
        context.connect_ports(osc3.output(0), dest.input(0));

        let _ = context.resume();
        context.message_node(
            osc3,
            AudioNodeMessage::AudioScheduledSourceNode(AudioScheduledSourceNodeMessage::Start(0.)),
        );
    }

    println!("servo_media raw s1");
    servo_media.mute(&context_id2, true);
    thread::sleep(time::Duration::from_millis(2000));

    println!("servo_media raw s2");
    servo_media.mute(&context_id1, true);
    servo_media.mute(&context_id2, false);
    thread::sleep(time::Duration::from_millis(2000));

    println!("servo_media s1+s2");
    servo_media.mute(&context_id1, false);
    thread::sleep(time::Duration::from_millis(2000));

    println!("servo_media muting s1");
    servo_media.mute(&context_id1, true);
    thread::sleep(time::Duration::from_millis(2000));

    println!("servo_media muting s2");
    servo_media.mute(&context_id2, true);
    thread::sleep(time::Duration::from_millis(2000));

    println!("servo_media unmuting s2");
    servo_media.mute(&context_id2, false);
    thread::sleep(time::Duration::from_millis(2000));

    println!("servo_media unmuting s1");
    servo_media.mute(&context_id1, false);
    thread::sleep(time::Duration::from_millis(2000));
}

fn main() {
    ServoMedia::init::<servo_media_auto::Backend>();
    let servo_media = ServoMedia::get();
    run_example(servo_media);
}
