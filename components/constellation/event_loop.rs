/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use ipc_channel::ipc::IpcSender;
use script_traits::ConstellationControlMsg;
use std::io::Error as IOError;
use std::marker::PhantomData;
use std::rc::Rc;

/// https://html.spec.whatwg.org/multipage/#event-loop
pub struct EventLoop {
    script_chan: IpcSender<ConstellationControlMsg>,
    dont_send_or_sync: PhantomData<Rc<()>>,
}

impl Drop for EventLoop {
    fn drop(&mut self) {
        let _ = self.script_chan.send(ConstellationControlMsg::ExitScriptThread);
    }
}

impl EventLoop {
    pub fn send(&self, msg: ConstellationControlMsg) -> Result<(), IOError> {
        self.script_chan.send(msg)
    }
    pub fn new(script_chan: IpcSender<ConstellationControlMsg>) -> Rc<EventLoop> {
        Rc::new(EventLoop {
            script_chan: script_chan,
            dont_send_or_sync: PhantomData,
        })
    }
    pub fn sender(&self) -> IpcSender<ConstellationControlMsg> {
        self.script_chan.clone()
    }
}

