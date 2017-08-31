/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use canvas_traits::webgl;
use euclid::Size2D;
use ipc_channel::ipc;
use ipc_channel::ipc::{IpcReceiver, IpcSender};
use msg::constellation_msg::PipelineId;
use rust_webvr::VRServiceManager;
use script_traits::ConstellationMsg;
use servo_config::prefs::PREFS;
use std::{thread, time};
use std::collections::{HashMap, HashSet};
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use webvr_traits::{WebVRMsg, WebVRResult};
use webvr_traits::webvr::*;

/// WebVRThread owns native VRDisplays, handles their life cycle inside Servo and
/// acts a doorman for untrusted VR requests from DOM Objects. These are the key components
///    * WebVRThread::spawn() creates a long living thread that waits for VR Commands from DOM objects
///      and handles them in its trusted thread. The back and forth comunication with DOM is implemented
///      using IPC-channels. This thread creates the VRServiceManager instance, which handles the life cycle
///      of all VR Vendor SDKs and owns all the native VRDisplays. These displays are guaranteed to live while
///      the spawned thread is alive. The WebVRThread is unique and it's closed using the Exit message when the
///      whole browser is going to be closed.
///    * A Event Polling thread is created in order to implement WebVR Events (connected, disconnected,..).
///      This thread wakes up the WebVRThread from time to time by sending a PollEvents message. This thread
///      is only created when there is at least one live JavaScript context using the WebVR APIs and shuts down it when
///      the tab is closed. A single instance of the thread is used to handle multiple JavaScript contexts.
///      Constellation channel is used to notify events to the Script Thread.
///    * When the WeVR APIs are used in a tab, it's pipeline_id is registered using the RegisterContext message. When
///      the tab is closed, UnregisterContext message is sent. This way the WebVR thread has a list of the pipeline
///      ids using the WebVR APIs. These ids are used to implement privacy guidelines defined in the WebVR Spec.
///    * When a JavaScript thread gains access to present to a headset, WebVRThread is not used as a intermediary in
///      the VRDisplay.requestAnimationFrame loop in order to minimize latency. A direct communication with WebRender
///      is used instead. See WebVRCompositorHandler and the WebVRCommands for more details.
pub struct WebVRThread {
    receiver: IpcReceiver<WebVRMsg>,
    sender: IpcSender<WebVRMsg>,
    service: VRServiceManager,
    contexts: HashSet<PipelineId>,
    constellation_chan: Sender<ConstellationMsg>,
    vr_compositor_chan: WebVRCompositorSender,
    polling_events: bool,
    presenting: HashMap<u32, PipelineId>
}

impl WebVRThread {
    fn new(receiver: IpcReceiver<WebVRMsg>,
           sender: IpcSender<WebVRMsg>,
           constellation_chan: Sender<ConstellationMsg>,
           vr_compositor_chan: WebVRCompositorSender)
           -> WebVRThread {
        let mut service = VRServiceManager::new();
        service.register_defaults();
        WebVRThread {
            receiver: receiver,
            sender: sender,
            service: service,
            contexts: HashSet::new(),
            constellation_chan: constellation_chan,
            vr_compositor_chan: vr_compositor_chan,
            polling_events: false,
            presenting: HashMap::new()
        }
    }

    pub fn spawn(vr_compositor_chan: WebVRCompositorSender)
                 -> (IpcSender<WebVRMsg>, Sender<Sender<ConstellationMsg>>) {
        let (sender, receiver) = ipc::channel().unwrap();
        let (constellation_sender, constellation_receiver) = mpsc::channel();
        let sender_clone = sender.clone();
        thread::Builder::new().name("WebVRThread".into()).spawn(move || {
            let constellation_chan = constellation_receiver.recv().unwrap();
            WebVRThread::new(receiver, sender_clone, constellation_chan, vr_compositor_chan).start();
        }).expect("Thread spawning failed");

        (sender, constellation_sender)
    }

    fn start(&mut self) {
        while let Ok(msg) = self.receiver.recv() {
            match msg {
                WebVRMsg::RegisterContext(context) => {
                    self.handle_register_context(context);
                    self.schedule_poll_events();
                },
                WebVRMsg::UnregisterContext(context) => {
                    self.handle_unregister_context(context);
                },
                WebVRMsg::PollEvents(sender) => {
                    self.poll_events(sender);
                },
                WebVRMsg::GetDisplays(sender) => {
                    self.handle_get_displays(sender);
                    self.schedule_poll_events();
                },
                WebVRMsg::GetFrameData(pipeline_id, display_id, near, far, sender) => {
                    self.handle_framedata(pipeline_id, display_id, near, far, sender);
                },
                WebVRMsg::ResetPose(pipeline_id, display_id, sender) => {
                    self.handle_reset_pose(pipeline_id, display_id, sender);
                },
                WebVRMsg::RequestPresent(pipeline_id, display_id, sender) => {
                    self.handle_request_present(pipeline_id, display_id, sender);
                },
                WebVRMsg::ExitPresent(pipeline_id, display_id, sender) => {
                    self.handle_exit_present(pipeline_id, display_id, sender);
                },
                WebVRMsg::CreateCompositor(display_id) => {
                    self.handle_create_compositor(display_id);
                },
                WebVRMsg::GetGamepads(synced_ids, sender) => {
                    self.handle_get_gamepads(synced_ids, sender);
                }
                WebVRMsg::Exit => {
                    break
                },
            }
        }
    }

    fn handle_register_context(&mut self, ctx: PipelineId) {
        self.contexts.insert(ctx);
    }

    fn handle_unregister_context(&mut self, ctx: PipelineId) {
        self.contexts.remove(&ctx);
    }

    fn handle_get_displays(&mut self, sender: IpcSender<WebVRResult<Vec<VRDisplayData>>>) {
        let displays = self.service.get_displays();
        let mut result = Vec::new();
        for display in displays {
            result.push(display.borrow().data());
        }
        sender.send(Ok(result)).unwrap();
    }

    fn handle_framedata(&mut self,
                        pipeline: PipelineId,
                        display_id: u32,
                        near: f64,
                        far: f64,
                        sender: IpcSender<WebVRResult<VRFrameData>>) {
      match self.access_check(pipeline, display_id) {
            Ok(display) => {
                sender.send(Ok(display.borrow().inmediate_frame_data(near, far))).unwrap()
            },
            Err(msg) => sender.send(Err(msg.into())).unwrap()
        }
    }

    fn handle_reset_pose(&mut self,
                         pipeline: PipelineId,
                         display_id: u32,
                         sender: IpcSender<WebVRResult<VRDisplayData>>) {
        match self.access_check(pipeline, display_id) {
            Ok(display) => {
                display.borrow_mut().reset_pose();
                sender.send(Ok(display.borrow().data())).unwrap();
            },
            Err(msg) => {
                sender.send(Err(msg.into())).unwrap()
            }
        }
    }

    // This method implements the privacy and security guidelines defined in the WebVR spec.
    // For example a secondary tab is not allowed to read VRDisplay data or stop a VR presentation
    // while the user is having a VR experience in the current tab.
    // These security rules also avoid multithreading race conditions between WebVRThread and
    // Webrender thread. See WebVRCompositorHandler implementation notes for more details about this.
    fn access_check(&self, pipeline: PipelineId, display_id: u32) -> Result<&VRDisplayPtr, &'static str> {
        if *self.presenting.get(&display_id).unwrap_or(&pipeline) != pipeline {
            return Err("No access granted to this Display because it's presenting on other JavaScript Tab");
        }
        self.service.get_display(display_id).ok_or("Device not found")
    }

    fn handle_request_present(&mut self,
                              pipeline: PipelineId,
                              display_id: u32,
                              sender: IpcSender<WebVRResult<()>>) {
        match self.access_check(pipeline, display_id).map(|d| d.clone()) {
            Ok(display) => {
                self.presenting.insert(display_id, pipeline);
                let data = display.borrow().data();
                sender.send(Ok(())).unwrap();
                self.notify_event(VRDisplayEvent::PresentChange(data, true).into());
            },
            Err(msg) => {
                sender.send(Err(msg.into())).unwrap();
            }
        }
    }

    fn handle_exit_present(&mut self,
                         pipeline: PipelineId,
                         display_id: u32,
                         sender: Option<IpcSender<WebVRResult<()>>>) {
        match self.access_check(pipeline, display_id).map(|d| d.clone()) {
            Ok(display) => {
                self.presenting.remove(&display_id);
                if let Some(sender) = sender {
                    sender.send(Ok(())).unwrap();
                }
                let data = display.borrow().data();
                self.notify_event(VRDisplayEvent::PresentChange(data, false).into());
            },
            Err(msg) => {
                if let Some(sender) = sender {
                    sender.send(Err(msg.into())).unwrap();
                }
            }
        }
    }

    fn handle_create_compositor(&mut self, display_id: u32) {
        let compositor = self.service.get_display(display_id).map(|d| WebVRCompositor(d.as_ptr()));
        self.vr_compositor_chan.send(compositor).unwrap();
    }

    fn handle_get_gamepads(&mut self,
                           synced_ids: Vec<u32>,
                           sender: IpcSender<WebVRResult<Vec<(Option<VRGamepadData>, VRGamepadState)>>>) {
        let gamepads = self.service.get_gamepads();
        let data = gamepads.iter().map(|g| {
            let g = g.borrow();
            // Optimization, don't fetch and send gamepad static data when the gamepad is already synced.
            let data = if synced_ids.iter().any(|v| *v == g.id()) {
                None
            } else {
                Some(g.data())
            };
            (data, g.state())
        }).collect();
        sender.send(Ok(data)).unwrap();
    }

    fn poll_events(&mut self, sender: IpcSender<bool>) {
        loop {
            let events = self.service.poll_events();
            if events.is_empty() {
                break;
            }
            self.notify_events(events)
        }

        // Stop polling events if the callers are not using VR
        self.polling_events = self.contexts.len() > 0;
        sender.send(self.polling_events).unwrap();
    }

    fn notify_events(&self, events: Vec<VREvent>) {
        let pipeline_ids: Vec<PipelineId> = self.contexts.iter().map(|c| *c).collect();
        self.constellation_chan.send(ConstellationMsg::WebVREvents(pipeline_ids.clone(), events)).unwrap();
    }

    #[inline]
    fn notify_event(&self, event: VREvent) {
        self.notify_events(vec![event]);
    }

    fn schedule_poll_events(&mut self) {
        if !self.service.is_initialized() || self.polling_events {
            return;
        }
        self.polling_events = true;
        let webvr_thread = self.sender.clone();
        let (sender, receiver) = ipc::channel().unwrap();

        // Defines the polling interval time in ms for VR Events such as VRDisplay connected, disconnected, etc.
        let polling_interval: u64 = PREFS.get("dom.webvr.event_polling_interval").as_u64().unwrap_or(500);

        thread::Builder::new().name("WebVRPollEvents".into()).spawn(move || {
            loop {
                if webvr_thread.send(WebVRMsg::PollEvents(sender.clone())).is_err() {
                    // WebVR Thread closed
                    break;
                }
                if !receiver.recv().unwrap_or(false) {
                    // WebVR Thread asked to unschedule this thread
                    break;
                }
                thread::sleep(time::Duration::from_millis(polling_interval));
            }
        }).expect("Thread spawning failed");
    }
}

/// Notes about WebVRCompositorHandler implementation:
/// Raw pointers are used instead of Arc<Mutex> as a heavy optimization for latency reasons.
/// This also avoids "JS DDoS" attacks: like a secondary JavaScript tab degrading performance
/// by flooding the WebVRThread with messages while the main JavaScript tab is presenting to the headset.
/// Multithreading won't be a problem because:
///    * Thanks to the security rules implemented in the WebVRThread, when a VRDisplay is in a presenting loop
///      no other JSContext is granted access to the VRDisplay. So really there aren’t multithreading race conditions.
///    * VRDisplay implementations are designed to allow calling compositor functions
///      in another thread by using the Send + Sync traits.
/// VRDisplays pointers are guaranteed to be valid memory:
///    * VRDisplays are owned by the VRServiceManager which lives in the WebVRThread.
///    * WebVRCompositorHandler is stopped automatically when a JS tab is closed or the whole browser is closed.
///    * WebVRThread and its VRDisplays are destroyed after all tabs are dropped and the browser is about to exit.
///      WebVRThread is closed using the Exit message.

pub struct WebVRCompositor(*mut VRDisplay);
pub struct WebVRCompositorHandler {
    compositors: HashMap<webgl::WebVRDeviceId, WebVRCompositor>,
    webvr_thread_receiver: Receiver<Option<WebVRCompositor>>,
    webvr_thread_sender: Option<IpcSender<WebVRMsg>>
}

#[allow(unsafe_code)]
unsafe impl Send for WebVRCompositor {}

pub type WebVRCompositorSender = Sender<Option<WebVRCompositor>>;

impl WebVRCompositorHandler {
    pub fn new() -> (Box<WebVRCompositorHandler>, WebVRCompositorSender) {
        let (sender, receiver) = mpsc::channel();
        let instance = Box::new(WebVRCompositorHandler {
            compositors: HashMap::new(),
            webvr_thread_receiver: receiver,
            webvr_thread_sender: None
        });

        (instance, sender)
    }
}

impl webgl::WebVRRenderHandler for WebVRCompositorHandler {
    #[allow(unsafe_code)]
    fn handle(&mut self, cmd: webgl::WebVRCommand, texture: Option<(u32, Size2D<i32>)>) {
        match cmd {
            webgl::WebVRCommand::Create(compositor_id) => {
                self.create_compositor(compositor_id);
            }
            webgl::WebVRCommand::SyncPoses(compositor_id, near, far, sender) => {
                if let Some(compositor) = self.compositors.get(&compositor_id) {
                    let pose = unsafe {
                        (*compositor.0).sync_poses();
                        (*compositor.0).synced_frame_data(near, far).to_bytes()
                    };
                    let _ = sender.send(Ok(pose));
                } else {
                    let _ = sender.send(Err(()));
                }
            }
            webgl::WebVRCommand::SubmitFrame(compositor_id, left_bounds, right_bounds) => {
                if let Some(compositor) = self.compositors.get(&compositor_id) {
                    if let Some((texture_id, size)) = texture {
                        let layer = VRLayer {
                            texture_id: texture_id,
                            left_bounds: left_bounds,
                            right_bounds: right_bounds,
                            texture_size: Some((size.width as u32, size.height as u32))
                        };
                        unsafe {
                            (*compositor.0).render_layer(&layer);
                            (*compositor.0).submit_frame();
                        }
                    }
                }
            }
            webgl::WebVRCommand::Release(compositor_id) => {
                self.compositors.remove(&compositor_id);
            }
        }
    }
}

impl WebVRCompositorHandler {
    #[allow(unsafe_code)]
    fn create_compositor(&mut self, display_id: webgl::WebVRDeviceId) {
        let sender = match self.webvr_thread_sender {
            Some(ref s) => s,
            None => return,
        };

        sender.send(WebVRMsg::CreateCompositor(display_id as u32)).unwrap();
        let display = self.webvr_thread_receiver.recv().unwrap();

        match display {
            Some(display) => {
                self.compositors.insert(display_id, display);
            },
            None => {
                error!("VRDisplay not found when creating a new VRCompositor");
            }
        };
    }

    // This is done on only a per-platform basis on initialization.
    pub fn set_webvr_thread_sender(&mut self, sender: IpcSender<WebVRMsg>) {
        self.webvr_thread_sender = Some(sender);
    }
}
