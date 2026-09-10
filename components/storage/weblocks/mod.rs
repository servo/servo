use std::thread;

use servo_base::generic_channel::{self, GenericSender};
use storage_traits::weblocks::WebLocksThreadMsg;

pub trait WebLocksThreadFactory {
    fn new() -> Self;
}

impl WebLocksThreadFactory for GenericSender<WebLocksThreadMsg> {
    fn new() -> Self {
        let (chan, port) = generic_channel::channel().unwrap();
        thread::Builder::new()
            .name("WebLocksManager".to_owned())
            .spawn(move || {
                // TODO: Manager
                drop(port);
            })
            .expect("thread spawning failed");
        chan
    }
}
