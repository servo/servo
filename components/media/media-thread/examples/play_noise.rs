extern crate rand;
extern crate servo_media;
extern crate servo_media_auto;

use std::sync::Arc;
use std::{thread, time};

use servo_media::audio::buffer_source_node::{AudioBuffer, AudioBufferSourceNodeMessage};
use servo_media::audio::node::{
    AudioNodeInit, AudioNodeMessage, AudioScheduledSourceNodeMessage, OnEndedCallback,
};
use servo_media::{ClientContextId, ServoMedia};

fn run_example(servo_media: Arc<ServoMedia>) {
    let context = servo_media
        .create_audio_context(&ClientContextId::build(1, 1), Default::default())
        .unwrap();
    let context = context.lock().unwrap();
    let buffer_source = context
        .create_node(
            AudioNodeInit::AudioBufferSourceNode(Default::default()),
            Default::default(),
        )
        .expect("Failed to create buffer source node");
    let dest = context.dest_node();
    context.connect_ports(buffer_source.output(0), dest.input(0));
    let mut buffers = vec![Vec::with_capacity(4096), Vec::with_capacity(4096)];
    for _ in 0..4096 {
        buffers[0].push(rand::random::<f32>());
        buffers[1].push(rand::random::<f32>());
    }
    context.message_node(
        buffer_source,
        AudioNodeMessage::AudioScheduledSourceNode(AudioScheduledSourceNodeMessage::Start(0.)),
    );
    context.message_node(
        buffer_source,
        AudioNodeMessage::AudioBufferSourceNode(AudioBufferSourceNodeMessage::SetBuffer(Some(
            AudioBuffer::from_buffers(buffers, 44100.),
        ))),
    );
    let callback = OnEndedCallback::new(|| {
        println!("Playback ended");
    });
    context.message_node(
        buffer_source,
        AudioNodeMessage::AudioScheduledSourceNode(
            AudioScheduledSourceNodeMessage::RegisterOnEndedCallback(callback),
        ),
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
