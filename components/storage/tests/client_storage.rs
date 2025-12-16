/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use base::generic_channel::GenericSender;
use storage::ClientStorageThreadFactory;
use storage_traits::client_storage::{ClientStorageProxy, ClientStorageThreadMessage};

#[test]
fn test_exit() {
    let thread: GenericSender<ClientStorageThreadMessage> = ClientStorageThreadFactory::new(None);

    let proxy = ClientStorageProxy::new(thread);

    proxy.send_exit();

    // Workaround for https://github.com/servo/servo/issues/32912
    #[cfg(windows)]
    std::thread::sleep(std::time::Duration::from_millis(1000));
}
