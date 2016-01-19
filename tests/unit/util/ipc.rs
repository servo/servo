/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use ipc_channel::ipc;
use std::sync::mpsc::channel;
use util::ipc::Unserializable;

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
enum Dummy {
    Serializable(i32),
    Unserializable(Unserializable<i32>),
}

#[test]
fn test_serializable() {
    let (tx, rx) = ipc::channel().unwrap();
    tx.send(Dummy::Serializable(4)).unwrap();
    assert_eq!(rx.recv().unwrap(), Dummy::Serializable(4));
}

#[test]
fn test_unserializable_in_process() {
    let (tx, rx) = channel();
    tx.send(Dummy::Unserializable(Unserializable::new(4))).unwrap();
    assert_eq!(rx.recv().unwrap(), Dummy::Unserializable(Unserializable::new(4)));
}

#[test]
#[should_panic]
fn test_unserializable_over_ipc() {
    let (tx, _rx) = ipc::channel().unwrap();
    tx.send(Dummy::Unserializable(Unserializable::new(4))).unwrap();
}
