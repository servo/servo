/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use ipc_channel::ipc::{self, IpcSender};
use net::filemanager_thread::FileManagerThreadFactory;
use net_traits::filemanager_thread::{FileManagerThreadMsg, FileManagerThreadError};

#[test]
fn test_filemanager() {
    let chan: IpcSender<FileManagerThreadMsg> = FileManagerThreadFactory::new();

    let (tx, rx) = ipc::channel().unwrap();
    let _ = chan.send(FileManagerThreadMsg::SelectFile(tx));

    match rx.recv().unwrap() {
        Err(FileManagerThreadError::InvalidSelection) => {},
        _ => assert!(false, "Should be an invalid selection before dialog is implemented"),
    }

    let _ = chan.send(FileManagerThreadMsg::Exit);
}
