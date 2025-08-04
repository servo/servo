/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::rc::Rc;

use crossbeam_channel::{self, unbounded};
use ipc_channel::ipc::IpcSender;
use net::clientstorage::thread_factory::ClientStorageThreadFactory;
use net_traits::clientstorage::actors_child::{
    ClientStorageTestChild, ClientStorageTestCursorChild,
};
use net_traits::clientstorage::proxy::ClientStorageProxy;
use net_traits::clientstorage::thread_msg::ClientStorageThreadMsg;

use super::utils::{TestClientStorageProxySender, TestCounter, TestOnceFlag};

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

#[test]
fn test_cursor() {
    let thread: IpcSender<ClientStorageThreadMsg> = ClientStorageThreadFactory::new(None);

    let (sender, receiver) = unbounded();

    let msg_sender = TestClientStorageProxySender::from_sender_boxed(sender);

    let proxy = ClientStorageProxy::new(thread, msg_sender);

    let child = ClientStorageTestChild::new();

    proxy.send_test_constructor(&child);

    let cursor_child = ClientStorageTestCursorChild::new();

    child.send_test_cursor_constructor(&cursor_child);

    let counter = TestCounter::new(3);

    cursor_child.set_response_callback({
        let counter = Rc::clone(&counter);
        let cursor_child = Rc::clone(&cursor_child);
        move |_number| {
            counter.increment();
            if !counter.done() {
                cursor_child.send_continue();
            }
        }
    });

    cursor_child.send_continue();

    loop {
        let msg = receiver.recv().unwrap();

        proxy.recv_proxy_message(msg);

        if counter.done() {
            break;
        }
    }

    cursor_child.send_delete();

    child.send_delete();

    proxy.send_exit();

    // Workaround for https://github.com/servo/servo/issues/32912
    #[cfg(windows)]
    std::thread::sleep(std::time::Duration::from_millis(1000));
}
