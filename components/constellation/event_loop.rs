/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! This module contains the `EventLoop` type, which is the constellation's
//! view of a script thread. When an `EventLoop` is dropped, an `ExitScriptThread`
//! message is sent to the script thread, asking it to shut down.

use std::hash::Hash;
use std::marker::PhantomData;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};

use ipc_channel::Error;
use ipc_channel::ipc::IpcSender;
use script_traits::ScriptThreadMessage;

static CURRENT_EVENT_LOOP_ID: AtomicUsize = AtomicUsize::new(0);

/// <https://html.spec.whatwg.org/multipage/#event-loop>
pub struct EventLoop {
    script_chan: IpcSender<ScriptThreadMessage>,
    dont_send_or_sync: PhantomData<Rc<()>>,
    id: usize,
}

impl PartialEq for EventLoop {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for EventLoop {}

impl Hash for EventLoop {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl Drop for EventLoop {
    fn drop(&mut self) {
        let _ = self.script_chan.send(ScriptThreadMessage::ExitScriptThread);
    }
}

impl EventLoop {
    /// Create a new event loop from the channel to its script thread.
    pub fn new(script_chan: IpcSender<ScriptThreadMessage>) -> Rc<EventLoop> {
        let id = CURRENT_EVENT_LOOP_ID.fetch_add(1, Ordering::Relaxed);
        Rc::new(EventLoop {
            script_chan,
            dont_send_or_sync: PhantomData,
            id,
        })
    }

    /// Send a message to the event loop.
    pub fn send(&self, msg: ScriptThreadMessage) -> Result<(), Error> {
        self.script_chan.send(msg)
    }
}
