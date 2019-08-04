/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! The script runtime contains common traits and structs commonly used by the
//! script thread, the dom, and the worker threads.

use crossbeam_channel::{unbounded, Sender};
use ipc_channel::ipc::{self, IpcSender};
use ipc_channel::router::ROUTER;
use msg::constellation_msg::{IpcCallbackId, IpcCallbackMsg, IpcHandle};
use std::collections::HashMap;

pub type IpcCallback = Box<dyn FnMut(Vec<u8>) + Send>;

pub struct ScriptIpcRouter {
    ipc_sender: IpcSender<IpcCallbackMsg>,
    sender: Sender<(IpcCallbackId, IpcCallback)>,
}

impl ScriptIpcRouter {
    pub fn new() -> Self {
        let (callback_sender, callback_receiver) =
            ipc::channel().expect("ScriptIpcRouter ipc chan");
        let (sender, receiver) = unbounded();
        let ipc_script_router = ScriptIpcRouter {
            sender: sender,
            ipc_sender: callback_sender,
        };
        let mut callbacks: HashMap<IpcCallbackId, IpcCallback> = HashMap::new();
        ROUTER.add_route(
            callback_receiver.to_opaque(),
            Box::new(move |message| {
                match message.to().expect("ScriptIpcRouter handle incoming msg") {
                    IpcCallbackMsg::AddCallback => {
                        let (id, callback) =
                            receiver.recv().expect("Callback message to be received");
                        callbacks.insert(id, callback);
                    },
                    IpcCallbackMsg::DropCallback(id) => {
                        let _ = callbacks
                            .remove(&id)
                            .expect("Callbackt to be removed to exists");
                    },
                    IpcCallbackMsg::Callback(id, data) => {
                        let callback = callbacks
                            .get_mut(&id)
                            .expect("Callback to be called to exists");
                        callback(data);
                    },
                }
            }),
        );
        ipc_script_router
    }

    pub fn add_callback(&self, callback: IpcCallback) -> IpcHandle {
        let callback_id = IpcCallbackId::new();
        if let Ok(_) = self.sender.send((callback_id.clone(), callback)) {
            self.ipc_sender
                .send(IpcCallbackMsg::AddCallback)
                .expect("The script ipc router to be available");
            return IpcHandle {
                callback_id,
                sender: self.ipc_sender.clone(),
            };
        }
        unreachable!("Adding an ipc callback failed");
    }
}
