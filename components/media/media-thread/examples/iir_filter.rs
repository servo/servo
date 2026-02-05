extern crate servo_media;
extern crate servo_media_auto;

use std::sync::Arc;
use std::{thread, time};

use servo_media::audio::iir_filter_node::{IIRFilterNode, IIRFilterNodeOptions};
use servo_media::audio::node::{AudioNodeInit, AudioNodeMessage, AudioScheduledSourceNodeMessage};
use servo_media::audio::oscillator_node::OscillatorNodeOptions;
use servo_media::{ClientContextId, ServoMedia};

fn run_example(servo_media: Arc<ServoMedia>) {
    let id = ClientContextId::build(1, 1);
    let context = servo_media
        .create_audio_context(&id, Default::default())
        .unwrap();

    {
        let context = context.lock().unwrap();

        let dest = context.dest_node();
        let osc = context
            .create_node(
                AudioNodeInit::OscillatorNode(OscillatorNodeOptions::default()),
                Default::default(),
            )
            .expect("Failed to create oscillator node");

        let feedback = Arc::new(vec![7.0, 1.0, 1.0]);
        let feedforward = Arc::new(vec![1.0, 1.0, 1.0]);

        let iir = context
            .create_node(
                AudioNodeInit::IIRFilterNode(IIRFilterNodeOptions {
                    feedback: feedback.clone(),
                    feedforward: feedforward.clone(),
                }),
                Default::default(),
            )
            .expect("Failed to create IIR filter node");

        context.connect_ports(osc.output(0), dest.input(0));
        let _ = context.resume();
        context.message_node(
            osc,
            AudioNodeMessage::AudioScheduledSourceNode(AudioScheduledSourceNodeMessage::Start(0.)),
        );

        println!("raw oscillator");
        thread::sleep(time::Duration::from_millis(2000));

        println!("oscillator through iir filter");
        context.disconnect_output(osc.output(0));
        context.connect_ports(osc.output(0), iir.input(0));
        context.connect_ports(iir.output(0), dest.input(0));
        thread::sleep(time::Duration::from_millis(2000));

        println!("raw oscillator");
        context.disconnect_output(osc.output(0));
        context.disconnect_output(iir.output(0));
        context.connect_ports(osc.output(0), dest.input(0));
        thread::sleep(time::Duration::from_millis(2000));

        let freqs = vec![0.0, 0.3, 0.5];
        let mut mag = vec![0.0; 3];
        let mut phase = vec![0.0; 3];
        IIRFilterNode::get_frequency_response(
            &feedforward,
            &feedback,
            &freqs,
            &mut mag,
            &mut phase,
        );
        print!(
            "GetFrequencyResponse for freqs: {:?}\n mag: {:?}\n phase: {:?}",
            &freqs, &mag, &phase
        );
    }
}

fn main() {
    ServoMedia::init::<servo_media_auto::Backend>();
    let servo_media = ServoMedia::get();
    run_example(servo_media);
}
