/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

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
