/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

extern crate servo_media;
extern crate servo_media_auto;

use std::f32::consts::PI;
use std::sync::Arc;
use std::{thread, time};

use servo_media::audio::audio_node::{
    AudioNodeInit, AudioNodeMessage, AudioScheduledSourceNodeMessage,
};
use servo_media::audio::oscillator_node::OscillatorNodeOptions;
use servo_media::audio::oscillator_node::OscillatorType::{Sawtooth, Square, Triangle};
use servo_media::audio::periodic_wave::{PeriodicWave, PeriodicWaveOptions};
use servo_media::{ClientContextId, ServoMedia};

fn run_waveform(
    servo_media: &Arc<ServoMedia>,
    options: OscillatorNodeOptions,
    context_id: &ClientContextId,
) {
    let context = servo_media
        .clone()
        .create_audio_context(context_id, Default::default())
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
    thread::sleep(time::Duration::from_millis(100));

    let _ = context.resume();
    context.message_node(
        osc5,
        AudioNodeMessage::AudioScheduledSourceNode(AudioScheduledSourceNodeMessage::Start(0.)),
    );

    thread::sleep(time::Duration::from_millis(3000));
    let _ = context.close();
    thread::sleep(time::Duration::from_millis(100));
}

fn run_example(servo_media: Arc<ServoMedia>) {
    let mut options = OscillatorNodeOptions::default();
    run_waveform(&servo_media, options.clone(), &ClientContextId::build(1, 1));

    options.oscillator_type = Square;
    run_waveform(&servo_media, options.clone(), &ClientContextId::build(1, 2));

    options.oscillator_type = Sawtooth;
    run_waveform(&servo_media, options.clone(), &ClientContextId::build(1, 3));

    options.oscillator_type = Triangle;
    run_waveform(&servo_media, options.clone(), &ClientContextId::build(1, 4));
}

fn main() {
    ServoMedia::init::<servo_media_auto::Backend>();
    let servo_media = ServoMedia::get();
    run_example(servo_media);
}
