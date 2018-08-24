extern crate servo_channel;

use servo_channel::{channel, ChannelError};

#[test]
fn send_after_receiver_dropped() {
    let (sender, receiver) = channel();
    drop(receiver);
    assert_eq!(sender.send(1), Err(ChannelError::ChannelClosedError));
}