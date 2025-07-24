/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crossbeam_channel::{self, unbounded};
use ipc_channel::ipc::IpcSender;
use net::clientstorage::thread_factory::ClientStorageThreadFactory;
use net_traits::clientstorage::actors_child::ClientStorageTestChild;
use net_traits::clientstorage::proxy::ClientStorageProxy;
use net_traits::clientstorage::thread_msg::ClientStorageThreadMsg;

use super::utils::{TestClientStorageProxySender, TestOnceFlag};

#[test]
fn test_sync_ping() {
    let thread: IpcSender<ClientStorageThreadMsg> = ClientStorageThreadFactory::new(None);

    let msg_sender = TestClientStorageProxySender::new_boxed();

    let proxy = ClientStorageProxy::new(thread, msg_sender);

    let child = ClientStorageTestChild::new();

    proxy.send_test_constructor(&child);

    child.send_sync_ping();

    child.send_delete();

    proxy.send_exit();

    // Workaround for https://github.com/servo/servo/issues/32912
    #[cfg(windows)]
    std::thread::sleep(std::time::Duration::from_millis(1000));
}

#[test]
fn test_ping() {
    let thread: IpcSender<ClientStorageThreadMsg> = ClientStorageThreadFactory::new(None);

    let (sender, receiver) = unbounded();

    let msg_sender = TestClientStorageProxySender::from_sender_boxed(sender);

    let proxy = ClientStorageProxy::new(thread, msg_sender);

    let child = ClientStorageTestChild::new();

    proxy.send_test_constructor(&child);

    let flag = TestOnceFlag::new();

    child.set_pong_callback(flag.as_callback());

    child.send_ping();

    loop {
        let msg = receiver.recv().unwrap();

        proxy.recv_proxy_message(msg);

        if flag.is_set() {
            break;
        }
    }

    child.send_delete();

    proxy.send_exit();

    // Workaround for https://github.com/servo/servo/issues/32912
    #[cfg(windows)]
    std::thread::sleep(std::time::Duration::from_millis(1000));
}
