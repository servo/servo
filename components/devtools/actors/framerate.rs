/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use rustc_serialize::json;
use std::mem;
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{Sender, channel};
use time::precise_time_ns;

use msg::constellation_msg::PipelineId;
use actor::{Actor, ActorRegistry};
use actors::timeline::HighResolutionStamp;
use devtools_traits::DevtoolScriptControlMsg;
use util::task;

pub struct FramerateActor {
    name: String,
    pipeline: PipelineId,
    script_sender: Sender<DevtoolScriptControlMsg>,
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
                  script_sender: Sender<DevtoolScriptControlMsg>) -> String {
        let actor_name = registry.new_name("framerate");
        let mut actor = FramerateActor {
            name: actor_name.clone(),
            pipeline: pipeline_id,
            script_sender: script_sender,
            start_time: None,
            is_recording: Arc::new(Mutex::new(false)),
            ticks: Arc::new(Mutex::new(Vec::new())),
        };

        actor.start_recording();
        registry.register_later(box actor);
        actor_name
    }

    pub fn take_pending_ticks(&mut self) -> Vec<HighResolutionStamp> {
        let mut lock = self.ticks.lock();
        let mut ticks = lock.as_mut().unwrap();
        mem::replace(ticks, Vec::new())
    }

    fn start_recording(&mut self) {
        let is_recording = self.is_recording.clone();
        let script_sender = self.script_sender.clone();
        let ticks = self.ticks.clone();
        let pipeline = self.pipeline.clone();

        let mut lock = self.is_recording.lock();
        **lock.as_mut().unwrap() = true;
        self.start_time = Some(precise_time_ns());

        let (rx, tx) = channel::<HighResolutionStamp>();
        let rx_copy = rx.clone();

        task::spawn_named("Framerate worker".to_string(), move || {
            loop {
                if !*is_recording.lock().unwrap() {
                    break;
                }

                match tx.try_recv() {
                    Ok(stamp) => {
                        let mut lock = ticks.lock();
                        let rx = rx_copy.clone();
                        lock.as_mut().unwrap().push(stamp);

                        let closure = move |now: f64| {
                            rx.send(HighResolutionStamp::wrap(now)).unwrap();
                        };
                        script_sender.send(DevtoolScriptControlMsg::RequestAnimationFrame(pipeline, Box::new(closure))).unwrap();
                    },
                    Err(_) => (),
                }
            }
        });

        let closure = move |now: f64| {
            rx.send(HighResolutionStamp::wrap(now)).unwrap();
        };
        self.script_sender.send(DevtoolScriptControlMsg::RequestAnimationFrame(self.pipeline, Box::new(closure))).unwrap();
    }

    fn stop_recording(&mut self) {
        let mut lock = self.is_recording.lock();
        let mut is_recording = lock.as_mut().unwrap();
        if !**is_recording {
            return;
        }
        **is_recording = false;
        self.start_time = None;
    }

}

impl Drop for FramerateActor {
    fn drop(&mut self) {
        self.stop_recording();
    }
}
