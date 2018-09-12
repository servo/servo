/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#[macro_use]
extern crate servo_channel;

use servo_channel::{channel, ChannelError};

#[test]
fn send_after_receiver_dropped() {
    let (sender, receiver) = channel();
    drop(receiver);
    assert_eq!(sender.send(1), Err(ChannelError::ChannelClosedError));
    let sent = select! {
        send(sender.select(), 1) => true,
        default => false
    };
    assert_eq!(sent, false);
}

#[test]
fn send_with_receiver_connected() {
    let (sender, _receiver) = channel();
    assert_eq!(sender.send(1), Ok(()));
    let sent = select! {
        send(sender.select(), 1) => true,
        default => false
    };
    assert_eq!(sent, true);
}
