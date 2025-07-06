/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use ipc_channel::ipc::{self, IpcSender};
use net::clientstorage::thread_factory::ClientStorageThreadFactory;
use net_traits::clientstorage::thread_msg::ClientStorageThreadMsg;

#[test]
fn test_exit() {
    let thread: IpcSender<ClientStorageThreadMsg> = ClientStorageThreadFactory::new(None);

    let (sender, receiver) = ipc::channel().unwrap();
    thread.send(ClientStorageThreadMsg::Exit(sender)).unwrap();
    receiver.recv().unwrap();

    // Workaround for https://github.com/servo/servo/issues/32912
    #[cfg(windows)]
    std::thread::sleep(std::time::Duration::from_millis(1000));
}
