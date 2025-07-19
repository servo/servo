/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use ipc_channel::ipc::IpcSender;
use net::clientstorage::thread_factory::ClientStorageThreadFactory;
use net_traits::clientstorage::proxy::ClientStorageProxy;
use net_traits::clientstorage::thread_msg::ClientStorageThreadMsg;

use super::utils::TestClientStorageProxySender;

#[test]
fn test_exit() {
    let thread: IpcSender<ClientStorageThreadMsg> = ClientStorageThreadFactory::new(None);

    let msg_sender = TestClientStorageProxySender::new_boxed();

    let proxy = ClientStorageProxy::new(thread, msg_sender);

    proxy.send_exit();

    // Workaround for https://github.com/servo/servo/issues/32912
    #[cfg(windows)]
    std::thread::sleep(std::time::Duration::from_millis(1000));
}
