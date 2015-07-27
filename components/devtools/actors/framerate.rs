/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use ipc_channel::ipc::{self, IpcSender};
use ipc_channel::router::ROUTER;
use rustc_serialize::json;
use std::mem;
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::Sender;
use time::precise_time_ns;

use msg::constellation_msg::PipelineId;
use actor::{Actor, ActorRegistry};
use actors::timeline::HighResolutionStamp;
use devtools_traits::{ChromeToDevtoolsControlMsg, DevtoolsControlMsg, DevtoolScriptControlMsg};

pub struct FramerateActor {
    name: String,
    pipeline: PipelineId,
    script_sender: IpcSender<DevtoolScriptControlMsg>,
    devtools_sender: Sender<DevtoolsControlMsg>,
    start_time: Option<u64>,
    is_recording: Arc<Mutex<bool>>,
    ticks: Arc<Mutex<Vec<HighResolutionStamp>>>,
}

impl Actor for FramerateActor {
    fn name(&self) -> String {
        self.name.clone()
    }


    fn handle_message(&self,
                      _registry: &ActorRegistry,
                      _msg_type: &str,
                      _msg: &json::Object,
                      _stream: &mut TcpStream) -> Result<bool, ()> {
        Ok(false)
    }
}

impl FramerateActor {
    /// return name of actor
    pub fn create(registry: &ActorRegistry,
                  pipeline_id: PipelineId,
                  script_sender: IpcSender<DevtoolScriptControlMsg>,
                  devtools_sender: Sender<DevtoolsControlMsg>) -> String {
        let actor_name = registry.new_name("framerate");
        let mut actor = FramerateActor {
            name: actor_name.clone(),
            pipeline: pipeline_id,
            script_sender: script_sender,
            devtools_sender: devtools_sender,
            start_time: None,
            is_recording: Arc::new(Mutex::new(false)),
            ticks: Arc::new(Mutex::new(Vec::new())),
        };

        actor.start_recording();
        registry.register_later(box actor);
        actor_name
    }

    pub fn add_tick(&self, tick: f64) {
        let mut lock = self.ticks.lock();
        let mut ticks = lock.as_mut().unwrap();
        ticks.push(HighResolutionStamp::wrap(tick));
    }

    pub fn take_pending_ticks(&self) -> Vec<HighResolutionStamp> {
        let mut lock = self.ticks.lock();
        let mut ticks = lock.as_mut().unwrap();
        mem::replace(ticks, Vec::new())
    }

    fn start_recording(&mut self) {
        let mut lock = self.is_recording.lock();
        if **lock.as_ref().unwrap() {
            return;
        }

        self.start_time = Some(precise_time_ns());
        let is_recording = lock.as_mut();
        **is_recording.unwrap() = true;

        fn get_closure(is_recording: Arc<Mutex<bool>>,
                       name: String,
                       pipeline: PipelineId,
                       script_sender: IpcSender<DevtoolScriptControlMsg>,
                       devtools_sender: Sender<DevtoolsControlMsg>)
                          -> Box<Fn(f64, ) + Send> {

            let closure = move |now: f64| {
                let msg = DevtoolsControlMsg::FromChrome(ChromeToDevtoolsControlMsg::FramerateTick(
                        name.clone(), now));
                devtools_sender.send(msg).unwrap();

                if !*is_recording.lock().unwrap() {
                    return;
                }

                let closure = get_closure(is_recording.clone(),
                                          name.clone(),
                                          pipeline.clone(),
                                          script_sender.clone(),
                                          devtools_sender.clone());
                let (request_animation_frame_sender, request_animation_frame_receiver) =
                    ipc::channel().unwrap();
                ROUTER.add_route(request_animation_frame_receiver.to_opaque(), box move |message| {
                    let value: f64 = message.to().unwrap();
                    closure(value);
                });
                let msg = DevtoolScriptControlMsg::RequestAnimationFrame(
                    pipeline,
                    request_animation_frame_sender);
                script_sender.send(msg).unwrap();
            };
            Box::new(closure)
        };

        let closure = get_closure(self.is_recording.clone(),
                                  self.name(),
                                  self.pipeline.clone(),
                                  self.script_sender.clone(),
                                  self.devtools_sender.clone());
        let (request_animation_frame_sender, request_animation_frame_receiver) =
            ipc::channel().unwrap();
        ROUTER.add_route(request_animation_frame_receiver.to_opaque(), box move |message| {
            let value: f64 = message.to().unwrap();
            closure(value);
        });
        let msg = DevtoolScriptControlMsg::RequestAnimationFrame(self.pipeline,
                                                                 request_animation_frame_sender);
        self.script_sender.send(msg).unwrap();
    }

    fn stop_recording(&mut self) {
        let mut lock = self.is_recording.lock();
        if !**lock.as_ref().unwrap() {
            return;
        }

        let is_recording = lock.as_mut();
        **is_recording.unwrap() = false;
        self.start_time = None;
    }

}

impl Drop for FramerateActor {
    fn drop(&mut self) {
        self.stop_recording();
    }
}
