/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use ipc_channel::ipc::IpcSender;
use rustc_serialize::json;
use std::mem;
use std::net::TcpStream;
use time::precise_time_ns;

use actor::{Actor, ActorRegistry, ActorMessageStatus};
use actors::timeline::HighResolutionStamp;
use devtools_traits::DevtoolScriptControlMsg;
use msg::constellation_msg::PipelineId;

pub struct FramerateActor {
    name: String,
    pipeline: PipelineId,
    script_sender: IpcSender<DevtoolScriptControlMsg>,
    start_time: Option<u64>,
    is_recording: bool,
    ticks: Vec<HighResolutionStamp>,
}

impl Actor for FramerateActor {
    fn name(&self) -> String {
        self.name.clone()
    }


    fn handle_message(&self,
                      _registry: &ActorRegistry,
                      _msg_type: &str,
                      _msg: &json::Object,
                      _stream: &mut TcpStream) -> Result<ActorMessageStatus, ()> {
        Ok(ActorMessageStatus::Ignored)
    }
}

impl FramerateActor {
    /// return name of actor
    pub fn create(registry: &ActorRegistry,
                  pipeline_id: PipelineId,
                  script_sender: IpcSender<DevtoolScriptControlMsg>) -> String {
        let actor_name = registry.new_name("framerate");
        let mut actor = FramerateActor {
            name: actor_name.clone(),
            pipeline: pipeline_id,
            script_sender: script_sender,
            start_time: None,
            is_recording: false,
            ticks: Vec::new(),
        };

        actor.start_recording();
        registry.register_later(box actor);
        actor_name
    }

    pub fn add_tick(&mut self, tick: f64) {
        self.ticks.push(HighResolutionStamp::wrap(tick));

        if self.is_recording {
            let msg = DevtoolScriptControlMsg::RequestAnimationFrame(self.pipeline,
                                                                     self.name());
            self.script_sender.send(msg).unwrap();
        }
    }

    pub fn take_pending_ticks(&mut self) -> Vec<HighResolutionStamp> {
        mem::replace(&mut self.ticks, Vec::new())
    }

    fn start_recording(&mut self) {
        if self.is_recording {
            return;
        }

        self.start_time = Some(precise_time_ns());
        self.is_recording = true;

        let msg = DevtoolScriptControlMsg::RequestAnimationFrame(self.pipeline,
                                                                 self.name());
        self.script_sender.send(msg).unwrap();
    }

    fn stop_recording(&mut self) {
        if !self.is_recording {
            return;
        }
        self.is_recording = false;
        self.start_time = None;
    }

}

impl Drop for FramerateActor {
    fn drop(&mut self) {
        self.stop_recording();
    }
}
