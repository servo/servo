/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![deny(unsafe_code)]

use bincode;
use crossbeam_channel::{unbounded, Sender};
use ipc_channel::ipc::{self, IpcSender};
use ipc_channel::router::ROUTER;
use msg::constellation_msg::IpcCallbackId;
use profile_traits::ipc as ProfiledIpc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize)]
pub struct IpcHandle {
    pub callback_id: IpcCallbackId,
    pub sender: IpcSender<IpcCallbackMsg>,
}

impl IpcHandle {
    pub fn send<T: Serialize>(&self, msg: T) -> Result<(), bincode::Error> {
        // TODO:: Use a IpcBytesSender/Receiver to avoid double serialization?
        // Requires passing T to the callback, instead of a Vec<u8>.
        //
        // Note: attempt at creating a Vec smaller than the one allocated inside send.
        // Basically (4096 - 64(for the Id)) - 64(for the IpcCallbackMsg::Callback).
        // The 3968 left is then for the msg, and hopefully we don't need to re-allocate(twice?).
        let mut bytes = Vec::with_capacity(3968);
        bincode::serialize_into(&mut bytes, &msg)?;
        self.sender
            .send(IpcCallbackMsg::Callback(self.callback_id.clone(), bytes))
    }

    /// Drop the associated callback.
    ///
    /// This cannot be done inside Drop, since the handle will drop on each process hop.
    /// Therefore, it is the responsability of the user of the handle to call drop_callback,
    /// when it will not be used anymore.
    pub fn drop_callback(&mut self) {
        let _ = self
            .sender
            .send(IpcCallbackMsg::DropCallback(self.callback_id.clone()));
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub enum IpcCallbackMsg {
    AddCallback,
    Callback(IpcCallbackId, Vec<u8>),
    DropCallback(IpcCallbackId),
}

pub type IpcCallback = Box<dyn FnMut(Vec<u8>) + Send>;

pub struct SharedIpcRouter {
    ipc_sender: IpcSender<IpcCallbackMsg>,
    sender: Sender<(IpcCallbackId, IpcCallback)>,
}

impl SharedIpcRouter {
    pub fn new(profiler: Option<profile_traits::time::ProfilerChan>) -> Self {
        let (callback_sender, callback_receiver) = match profiler {
            Some(profiler) => {
                let (sender, receiver) =
                    ProfiledIpc::channel(profiler).expect("SharedIpcRouter profiled chan");
                (sender, receiver.to_opaque())
            },
            None => {
                let (sender, receiver) = ipc::channel().expect("SharedIpcRouter ipc chan");
                (sender, receiver.to_opaque())
            },
        };
        let (sender, receiver) = unbounded();
        let ipc_script_router = SharedIpcRouter {
            sender: sender,
            ipc_sender: callback_sender,
        };
        let mut callbacks: HashMap<IpcCallbackId, IpcCallback> = HashMap::new();
        ROUTER.add_route(
            callback_receiver,
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
