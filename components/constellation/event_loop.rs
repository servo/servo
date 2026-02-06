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
use base::generic_channel::{self, GenericReceiver, GenericSender, SendError};
use base::id::ScriptEventLoopId;
use constellation_traits::ServiceWorkerManagerFactory;
use embedder_traits::ScriptToEmbedderChan;
use ipc_channel::IpcError;
use layout_api::ScriptThreadFactory;
use log::error;
use media::WindowGLContext;
use script_traits::{InitialScriptState, ScriptThreadMessage};
use serde::{Deserialize, Serialize};
use servo_config::opts::{self, Opts};
use servo_config::prefs::{self, Preferences};

use crate::sandboxing::spawn_multiprocess;
use crate::{Constellation, UnprivilegedContent};

/// <https://html.spec.whatwg.org/multipage/#event-loop>
pub struct EventLoop {
    script_chan: GenericSender<ScriptThreadMessage>,
    id: ScriptEventLoopId,
    /// When running in another process, this is an `IpcSender` to the BackgroundHangMonitor
    /// on the other side of the process boundary. When running in the same process, the
    /// BackgroundHangMonitor is shared among all [`EventLoop`]s so this will be `None`.
    background_hang_monitor_sender: Option<GenericSender<BackgroundHangMonitorControlMsg>>,
    dont_send_or_sync: PhantomData<Rc<()>>,
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
        self.send_message_to_background_hang_monitor(&BackgroundHangMonitorControlMsg::Exit);

        if let Err(error) = self.script_chan.send(ScriptThreadMessage::ExitScriptThread) {
            error!("Did not successfully request EventLoop exit: {error}");
        }
    }
}

impl EventLoop {
    pub(crate) fn spawn<STF: ScriptThreadFactory, SWF: ServiceWorkerManagerFactory>(
        constellation: &mut Constellation<STF, SWF>,
        is_private: bool,
    ) -> Result<Rc<Self>, IpcError> {
        let (script_chan, script_port) =
            base::generic_channel::channel().expect("Pipeline script chan");

        let embedder_chan = constellation.embedder_proxy.sender.clone();
        let eventloop_waker = constellation.embedder_proxy.event_loop_waker.clone();
        let script_to_embedder_sender = ScriptToEmbedderChan::new(embedder_chan, eventloop_waker);

        let resource_threads = if is_private {
            constellation.private_resource_threads.clone()
        } else {
            constellation.public_resource_threads.clone()
        };
        let storage_threads = if is_private {
            constellation.private_storage_threads.clone()
        } else {
            constellation.public_storage_threads.clone()
        };

        let event_loop_id = ScriptEventLoopId::new();
        let initial_script_state = InitialScriptState {
            id: event_loop_id,
            script_to_constellation_sender: constellation.script_sender.clone(),
            script_to_embedder_sender,
            namespace_request_sender: constellation.namespace_ipc_sender.clone(),
            devtools_server_sender: constellation.script_to_devtools_callback(),
            #[cfg(feature = "bluetooth")]
            bluetooth_sender: constellation.bluetooth_ipc_sender.clone(),
            system_font_service: constellation.system_font_service.to_sender(),
            resource_threads,
            storage_threads,
            time_profiler_sender: constellation.time_profiler_chan.clone(),
            memory_profiler_sender: constellation.mem_profiler_chan.clone(),
            constellation_to_script_sender: script_chan,
            constellation_to_script_receiver: script_port,
            pipeline_namespace_id: constellation.next_pipeline_namespace_id(),
            cross_process_paint_api: constellation.paint_proxy.cross_process_paint_api.clone(),
            webgl_chan: constellation
                .webgl_threads
                .as_ref()
                .map(|threads| threads.pipeline()),
            webxr_registry: constellation.webxr_registry.clone(),
            player_context: WindowGLContext::get(),
            privileged_urls: constellation.privileged_urls.clone(),
            user_contents_for_manager_id: constellation.user_contents_for_manager_id.clone(),
        };

        let event_loop = if opts::get().multiprocess {
            Self::spawn_in_process(constellation, initial_script_state)?
        } else {
            Self::spawn_in_thread(constellation, initial_script_state)
        };

        let event_loop = Rc::new(event_loop);
        constellation.add_event_loop(&event_loop);
        Ok(event_loop)
    }

    fn spawn_in_thread<STF: ScriptThreadFactory, SWF: ServiceWorkerManagerFactory>(
        constellation: &mut Constellation<STF, SWF>,
        initial_script_state: InitialScriptState,
    ) -> Self {
        let script_chan = initial_script_state.constellation_to_script_sender.clone();
        let id = initial_script_state.id;
        let background_hang_monitor_register = constellation
            .background_monitor_register
            .clone()
            .expect("Couldn't start content, no background monitor has been initiated");
        let join_handle = STF::create(
            initial_script_state,
            constellation.layout_factory.clone(),
            constellation.image_cache_factory.clone(),
            background_hang_monitor_register,
        );
        constellation.add_event_loop_join_handle(join_handle);

        Self {
            script_chan,
            id,
            background_hang_monitor_sender: None,
            dont_send_or_sync: PhantomData,
        }
    }

    fn spawn_in_process<STF: ScriptThreadFactory, SWF: ServiceWorkerManagerFactory>(
        constellation: &mut Constellation<STF, SWF>,
        initial_script_state: InitialScriptState,
    ) -> Result<Self, IpcError> {
        let script_chan = initial_script_state.constellation_to_script_sender.clone();
        let id = initial_script_state.id;

        let (background_hand_monitor_sender, backgrond_hand_monitor_receiver) =
            generic_channel::channel().expect("Sampler chan");
        let (lifeline_sender, lifeline_receiver) =
            generic_channel::channel().expect("Failed to create lifeline channel");

        let process = spawn_multiprocess(UnprivilegedContent::ScriptEventLoop(
            NewScriptEventLoopProcessInfo {
                initial_script_state,
                constellation_to_bhm_receiver: backgrond_hand_monitor_receiver,
                bhm_to_constellation_sender: constellation.background_hang_monitor_sender.clone(),
                lifeline_sender,
                opts: (*opts::get()).clone(),
                prefs: Box::new(prefs::get().clone()),
                broken_image_icon_data: constellation.broken_image_icon_data.clone(),
            },
        ))?;

        let crossbeam_receiver = lifeline_receiver.route_preserving_errors();
        constellation
            .process_manager
            .add(crossbeam_receiver, process);

        Ok(Self {
            script_chan,
            id,
            background_hang_monitor_sender: Some(background_hand_monitor_sender),
            dont_send_or_sync: PhantomData,
        })
    }

    pub(crate) fn id(&self) -> ScriptEventLoopId {
        self.id
    }

    /// Send a message to the event loop.
    pub fn send(&self, msg: ScriptThreadMessage) -> Result<(), SendError> {
        self.script_chan.send(msg)
    }

    /// If this is [`EventLoop`] is in another process, send a message to its `BackgroundHangMonitor`,
    /// otherwise do nothing.
    pub(crate) fn send_message_to_background_hang_monitor(
        &self,
        message: &BackgroundHangMonitorControlMsg,
    ) {
        if let Some(background_hang_monitor_sender) = &self.background_hang_monitor_sender {
            if let Err(error) = background_hang_monitor_sender.send(message.clone()) {
                error!("Could not send message ({message:?}) to BHM: {error}");
            }
        }
    }
}

/// All of the information necessary to create a new script [`EventLoop`] in a new process.
#[derive(Deserialize, Serialize)]
pub struct NewScriptEventLoopProcessInfo {
    pub initial_script_state: InitialScriptState,
    pub constellation_to_bhm_receiver: GenericReceiver<BackgroundHangMonitorControlMsg>,
    pub bhm_to_constellation_sender: GenericSender<HangMonitorAlert>,
    pub lifeline_sender: GenericSender<()>,
    pub opts: Opts,
    pub prefs: Box<Preferences>,
    /// The broken image icon data that is used to create an image to show in place of broken images.
    pub broken_image_icon_data: Vec<u8>,
}
