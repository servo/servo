/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! This module contains the `EventLoop` type, which is the constellation's
//! view of a script thread. When an `EventLoop` is dropped, an `ExitScriptThread`
//! message is sent to the script thread, asking it to shut down.

use std::hash::Hash;
use std::marker::PhantomData;
use std::rc::Rc;

use background_hang_monitor_api::{BackgroundHangMonitorControlMsg, HangMonitorAlert};
use base::generic_channel::{GenericReceiver, GenericSender};
use base::id::ScriptEventLoopId;
use ipc_channel::Error;
use ipc_channel::ipc::IpcSender;
use script_traits::{InitialScriptState, NewPipelineInfo, ScriptThreadMessage};
use serde::{Deserialize, Serialize};
use servo_config::opts::Opts;
use servo_config::prefs::Preferences;

/// <https://html.spec.whatwg.org/multipage/#event-loop>
pub struct EventLoop {
    script_chan: GenericSender<ScriptThreadMessage>,
    dont_send_or_sync: PhantomData<Rc<()>>,
    id: ScriptEventLoopId,
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
    pub fn new(script_chan: GenericSender<ScriptThreadMessage>) -> Rc<EventLoop> {
        Rc::new(EventLoop {
            script_chan,
            dont_send_or_sync: PhantomData,
            id: ScriptEventLoopId::new(),
        })
    }

    pub(crate) fn id(&self) -> ScriptEventLoopId {
        self.id
    }

    /// Send a message to the event loop.
    pub fn send(&self, msg: ScriptThreadMessage) -> Result<(), Error> {
        self.script_chan
            .send(msg)
            .map_err(|_err| Box::new(ipc_channel::ErrorKind::Custom("SendError".into())))
    }
}

/// All of the information necessary to create a new script [`EventLoop`] in a new process.
#[derive(Deserialize, Serialize)]
pub struct NewScriptEventLoopProcessInfo {
    pub new_pipeline_info: NewPipelineInfo,
    pub initial_script_state: InitialScriptState,
    pub constellation_to_bhm_receiver: GenericReceiver<BackgroundHangMonitorControlMsg>,
    pub bhm_to_constellation_sender: GenericSender<HangMonitorAlert>,
    pub lifeline_sender: IpcSender<()>,
    pub opts: Opts,
    pub prefs: Box<Preferences>,
    /// The broken image icon data that is used to create an image to show in place of broken images.
    pub broken_image_icon_data: Vec<u8>,
}
