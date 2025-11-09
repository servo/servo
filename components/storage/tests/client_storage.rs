/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use base::generic_channel::{self, GenericSender};
use storage::ClientStorageThreadFactory;
use storage_traits::client_storage::ClientStorageThreadMsg;

#[test]
fn test_exit() {
    let thread: GenericSender<ClientStorageThreadMsg> = ClientStorageThreadFactory::new(None);

    let (sender, receiver) = generic_channel::channel().unwrap();
    thread.send(ClientStorageThreadMsg::Exit(sender)).unwrap();
    receiver.recv().unwrap();

    // Workaround for https://github.com/servo/servo/issues/32912
    #[cfg(windows)]
    std::thread::sleep(std::time::Duration::from_millis(1000));
}
