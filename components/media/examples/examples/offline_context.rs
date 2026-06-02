/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

extern crate servo_media;
extern crate servo_media_auto;

use std::sync::{Arc, Mutex, mpsc};
use std::{thread, time};

use servo_media::audio::block::FRAMES_PER_BLOCK_USIZE;
use servo_media::audio::buffer_source_node::{AudioBuffer, AudioBufferSourceNodeMessage};
use servo_media::audio::context::{AudioContextOptions, OfflineAudioContextOptions};
use servo_media::audio::node::{AudioNodeInit, AudioNodeMessage, AudioScheduledSourceNodeMessage};
use servo_media::{ClientContextId, ServoMedia};

fn run_example(servo_media: Arc<ServoMedia>) {
    // Create offline context to process 1024 blocks of a oscillator node produced
    // sine wave.
    let mut options = <OfflineAudioContextOptions>::default();
    options.channels = 2;
    options.length = 1024 * FRAMES_PER_BLOCK_USIZE;
    let sample_rate = options.sample_rate;
    let options = AudioContextOptions::OfflineAudioContext(options);
    let context = servo_media
        .create_audio_context(&ClientContextId::build(1, 1), options)
        .unwrap();
    let context = context.lock().unwrap();
    let processed_audio = Arc::new(Mutex::new(Vec::new()));
    let processed_audio_ = processed_audio.clone();
    let (sender, receiver) = mpsc::channel();
    let sender = Mutex::new(sender);
    context.set_eos_callback(Box::new(move |buffer| {
        processed_audio
            .lock()
            .unwrap()
            .extend_from_slice((*buffer).as_ref());
        sender.lock().unwrap().send(()).unwrap();
    }));
    let osc = context
        .create_node(
            AudioNodeInit::OscillatorNode(Default::default()),
            Default::default(),
        )
        .expect("Failed to create oscillator node");
    let dest = context.dest_node();
    context.connect_ports(osc.output(0), dest.input(0));
    context.message_node(
        osc,
        AudioNodeMessage::AudioScheduledSourceNode(AudioScheduledSourceNodeMessage::Start(0.)),
    );
    let _ = context.resume();
    // Block until we processed the data.
    receiver.recv().unwrap();
    // Close offline context.
    let _ = context.close();
    // Create audio context to play the processed audio.
    let context = servo_media
        .create_audio_context(&ClientContextId::build(1, 2), Default::default())
        .unwrap();
    let context = context.lock().unwrap();
    let buffer_source = context
        .create_node(
            AudioNodeInit::AudioBufferSourceNode(Default::default()),
            Default::default(),
        )
        .expect("Failed to create oscillator node");
    let dest = context.dest_node();
    context.connect_ports(buffer_source.output(0), dest.input(0));
    context.message_node(
        buffer_source,
        AudioNodeMessage::AudioScheduledSourceNode(AudioScheduledSourceNodeMessage::Start(0.)),
    );
    context.message_node(
        buffer_source,
        AudioNodeMessage::AudioBufferSourceNode(AudioBufferSourceNodeMessage::SetBuffer(Some(
            AudioBuffer::from_buffer(processed_audio_.lock().unwrap().to_vec(), sample_rate),
        ))),
    );
    let _ = context.resume();
    thread::sleep(time::Duration::from_millis(5000));
    let _ = context.close();
}

fn main() {
    ServoMedia::init::<servo_media_auto::Backend>();
    let servo_media = ServoMedia::get();
    run_example(servo_media);
}
