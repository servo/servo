extern crate servo_media;
extern crate servo_media_auto;

use std::sync::Arc;
use std::{thread, time};

use servo_media::ServoMedia;

fn run_example(servo_media: Arc<ServoMedia>) {
    if let Some(stream) = servo_media.create_audioinput_stream(Default::default()) {
        let mut output = servo_media.create_stream_output();
        output.add_stream(&stream);
        thread::sleep(time::Duration::from_millis(6000));
    } else {
        print!("No audio input elements available");
    }
}

fn main() {
    ServoMedia::init::<servo_media_auto::Backend>();
    let servo_media = ServoMedia::get();
    run_example(servo_media);
}
