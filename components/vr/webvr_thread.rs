/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use ipc_channel::ipc;
use ipc_channel::ipc::{IpcReceiver, IpcSender};
use msg::constellation_msg::PipelineId;
use script_traits::{ConstellationMsg, WebVREventMsg};
use std::{thread, time};
use std::collections::{HashMap, HashSet};
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use vr_traits::{WebVRMsg, WebVRResult};
use vr_traits::webvr::*;
use webrender_traits;

// Defines the polling interval time in ms for VR Events such as VRDevice connected, disconnected, etc.
const EVENT_POLLING_INTERVAL: u64 = 500;

// WebVRThread owns native VRDevices, handles their life cycle inside Servo and
// acts a doorman for untrusted VR requests from DOM Objects.
// It waits for VR Commands from DOM objects and handles them in its trusted thread.
pub struct WebVRThread {
    receiver: IpcReceiver<WebVRMsg>,
    sender: IpcSender<WebVRMsg>,
    service: VRServiceManager,
    contexts: HashSet<PipelineId>,
    constellation_chan: Sender<ConstellationMsg>,
    vr_compositor_chan: WebVRCompositorSender,
    polling_events: bool,
    presenting: HashMap<u64, PipelineId>
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

    pub fn spawn(constellation_chan: Sender<ConstellationMsg>,
                 vr_compositor_chan: WebVRCompositorSender)
                 -> IpcSender<WebVRMsg> {
        let (sender, receiver) = ipc::channel().unwrap();
        let sender_clone = sender.clone();
        thread::Builder::new().name("WebVRThread".into()).spawn(move || {
            WebVRThread::new(receiver, sender_clone, constellation_chan, vr_compositor_chan).start();
        }).expect("Thread spawning failed");
        sender
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
                WebVRMsg::GetVRDisplays(sender) => {
                    self.handle_get_displays(sender);
                    self.schedule_poll_events();
                },
                WebVRMsg::GetFrameData(pipeline_id, device_id, near, far, sender) => {
                    self.handle_framedata(pipeline_id, device_id, near, far, sender);
                },
                WebVRMsg::ResetPose(pipeline_id, device_id, sender) => {
                    self.handle_reset_pose(pipeline_id, device_id, sender);
                },
                WebVRMsg::RequestPresent(pipeline_id, device_id, sender) => {
                    self.handle_request_present(pipeline_id, device_id, sender);
                },
                WebVRMsg::ExitPresent(pipeline_id, device_id, sender) => {
                    self.handle_exit_present(pipeline_id, device_id, sender);
                },
                WebVRMsg::CreateCompositor(device_id) => {
                    self.handle_create_compositor(device_id);
                },
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
        let devices = self.service.get_devices();
        let mut displays = Vec::new();
        for device in devices {
            displays.push(device.borrow().display_data());
        }
        sender.send(Ok(displays)).unwrap();
    }

    fn handle_framedata(&mut self,
                        pipeline: PipelineId,
                        device_id: u64,
                        near: f64,
                        far: f64,
                        sender: IpcSender<WebVRResult<VRFrameData>>) {
      match self.access_check(pipeline, device_id) {
            Ok(device) => {
                sender.send(Ok(device.borrow().inmediate_frame_data(near, far))).unwrap()
            },
            Err(msg) => sender.send(Err(msg.into())).unwrap()
        }
    }

    fn handle_reset_pose(&mut self,
                         pipeline: PipelineId,
                         device_id: u64,
                         sender: IpcSender<WebVRResult<VRDisplayData>>) {
        match self.access_check(pipeline, device_id) {
            Ok(device) => {
                device.borrow_mut().reset_pose();
                sender.send(Ok(device.borrow().display_data())).unwrap();
            },
            Err(msg) => {
                sender.send(Err(msg.into())).unwrap()
            }
        }
    }

    fn access_check(&self, pipeline: PipelineId, device_id: u64) -> Result<&VRDevicePtr, &'static str> {
        if *self.presenting.get(&device_id).unwrap_or(&pipeline) != pipeline {
            return Err("Device owned by another context");
        }
        self.service.get_device(device_id).ok_or("Device not found")
    }

    fn handle_request_present(&mut self,
                         pipeline: PipelineId,
                         device_id: u64,
                         sender: IpcSender<WebVRResult<()>>) {
        match self.access_check(pipeline, device_id).map(|d| d.clone()) {
            Ok(device) => {
                self.presenting.insert(device_id, pipeline);
                let data = device.borrow().display_data();
                sender.send(Ok(())).unwrap();
                self.notify_event(VRDisplayEvent::PresentChange(data, true));
            },
            Err(msg) => {
                sender.send(Err(msg.into())).unwrap();
            }
        }
    }

    fn handle_exit_present(&mut self,
                         pipeline: PipelineId,
                         device_id: u64,
                         sender: Option<IpcSender<WebVRResult<()>>>) {
        match self.access_check(pipeline, device_id).map(|d| d.clone()) {
            Ok(device) => {
                self.presenting.remove(&device_id);
                if let Some(sender) = sender {
                    sender.send(Ok(())).unwrap();
                }
                let data = device.borrow().display_data();
                self.notify_event(VRDisplayEvent::PresentChange(data, false));
            },
            Err(msg) => {
                if let Some(sender) = sender {
                    sender.send(Err(msg.into())).unwrap();
                }
            }
        }
    }

    fn handle_create_compositor(&mut self, device_id: u64) {
        let compositor = self.service.get_device(device_id).map(|d| WebVRCompositor(d.as_ptr()));
        self.vr_compositor_chan.send(compositor).unwrap();
    }

    fn poll_events(&mut self, sender: IpcSender<bool>) {
        let events = self.service.poll_events();
        if !events.is_empty() {
            let pipeline_ids: Vec<PipelineId> = self.contexts.iter().map(|c| *c).collect();
            for event in events {
                let event = WebVREventMsg::DisplayEvent(event);
                self.constellation_chan.send(ConstellationMsg::WebVREvent(pipeline_ids.clone(), event)).unwrap();
            }
        }

        // Stop polling events if the callers are not using VR
        self.polling_events = self.contexts.len() > 0;
        sender.send(self.polling_events).unwrap();
    }

    fn notify_event(&self, event: VRDisplayEvent) {
        let pipeline_ids: Vec<PipelineId> = self.contexts.iter().map(|c| *c).collect();
        let event = WebVREventMsg::DisplayEvent(event);
        self.constellation_chan.send(ConstellationMsg::WebVREvent(pipeline_ids.clone(), event)).unwrap();
    }

    fn schedule_poll_events(&mut self) {
        if !self.service.is_initialized() || self.polling_events {
            return;
        }
        self.polling_events = true;
        let webvr_thread = self.sender.clone();
        let (sender, receiver) = ipc::channel().unwrap();

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
                thread::sleep(time::Duration::from_millis(EVENT_POLLING_INTERVAL));
            }
        }).expect("Thread spawning failed");
    }
}

// Notes about WebVRCompositorHandler implementation:
// Raw pointers are used instead of Arc<Mutex> as a heavy optimization for latency reasons.
// This also avoids "JS DDoS" attacks: like a secondary JavaScript tab degrading performance by flooding the WebVRThread
// with messages while the main JavaScript tab is presenting to the headset.
// Multithreading won't be a problem because:
//    * Thanks to the security rules implemented in the WebVRThread, when a VRDisplay is in a presenting loop
//      no other JSContext is granted access to the VRDisplay. So really there aren’t multithreading race conditions.
//    * VRDevice implementations are designed to allow calling compositor functions
//      in another thread by using the Send + Sync traits.
// VRDevices pointers are guaranteed to be valid memory:
//    * VRDevices are owned by the VRDeviceManager which lives in the WebVRThread.
//    * WebVRCompositorHandler is stopped automatically when a JS tab is closed or the whole browser is closed.
//    * WebVRThread and it's VRDevices are destroyed after all tabs are dropped and the browser is about to exit.
//      WebVRThread is closed using the Exit message.

pub struct WebVRCompositor(*mut VRDevice);
pub struct WebVRCompositorHandler {
    compositors: HashMap<webrender_traits::VRCompositorId, WebVRCompositor>,
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

impl webrender_traits::VRCompositorHandler for WebVRCompositorHandler {
    #[allow(unsafe_code)]
    fn handle(&mut self, cmd: webrender_traits::VRCompositorCommand, texture_id: Option<u32>) {
        match cmd {
            webrender_traits::VRCompositorCommand::Create(compositor_id) => {
                self.create_compositor(compositor_id);
            }
            webrender_traits::VRCompositorCommand::SyncPoses(compositor_id, near, far, sender) => {
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
            webrender_traits::VRCompositorCommand::SubmitFrame(compositor_id, left_bounds, right_bounds) => {
                if let Some(compositor) = self.compositors.get(&compositor_id) {
                    if let Some(texture_id) = texture_id {
                        let layer = VRLayer {
                            texture_id: texture_id,
                            left_bounds: left_bounds,
                            right_bounds: right_bounds
                        };
                        unsafe {
                            (*compositor.0).submit_frame(&layer);
                        }
                    }
                }
            }
            webrender_traits::VRCompositorCommand::Release(compositor_id) => {
                self.compositors.remove(&compositor_id);
            }
        }
    }
}

impl WebVRCompositorHandler {
    #[allow(unsafe_code)]
    fn create_compositor(&mut self, device_id: webrender_traits::VRCompositorId) {
        let sender = match self.webvr_thread_sender {
            Some(ref s) => s,
            None => return,
        };

        sender.send(WebVRMsg::CreateCompositor(device_id)).unwrap();
        let device = self.webvr_thread_receiver.recv().unwrap();

        match device {
            Some(device) => {
                self.compositors.insert(device_id, device);
            },
            None => {
                error!("VRDevice not found when creating a new VRCompositor");
            }
        };
    }

    // This is done only a per-platform basis on initialization.
    pub fn set_webvr_thread_sender(&mut self, sender: IpcSender<WebVRMsg>) {
        self.webvr_thread_sender = Some(sender);
    }
}
