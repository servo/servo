extern crate servo_media;
extern crate servo_media_dummy;

use servo_media::ServoMedia;
use servo_media_dummy::DummyBackend;

fn main() {
    ServoMedia::init::<DummyBackend>();
    ServoMedia::get();
}
