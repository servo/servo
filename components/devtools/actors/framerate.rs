/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use rustc_serialize::json;
use std::mem;
use std::net::TcpStream;
use time::precise_time_ns;

use actor::{Actor, ActorRegistry};

pub struct FramerateActor {
    name: String,
    start_time: Option<u64>,
    is_recording: bool,
    ticks: Vec<u64>,
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
    pub fn create(registry: &ActorRegistry) -> String {
        let actor_name = registry.new_name("framerate");
        let mut actor = FramerateActor {
            name: actor_name.clone(),
            start_time: None,
            is_recording: false,
            ticks: Vec::new(),
        };

        actor.start_recording();
        registry.register_later(box actor);
        actor_name
    }

    // callback on request animation frame
    #[allow(dead_code)]
    pub fn on_refresh_driver_tick(&mut self) {
        if !self.is_recording {
            return;
        }
        // TODO: Need implement requesting animation frame
        // http://hg.mozilla.org/mozilla-central/file/0a46652bd992/dom/base/nsGlobalWindow.cpp#l5314

        let start_time = self.start_time.as_ref().unwrap();
        self.ticks.push(*start_time - precise_time_ns());
    }

    pub fn take_pending_ticks(&mut self) -> Vec<u64> {
        mem::replace(&mut self.ticks, Vec::new())
    }

    fn start_recording(&mut self) {
        self.is_recording = true;
        self.start_time = Some(precise_time_ns());

        // TODO(#5681): Need implement requesting animation frame
        // http://hg.mozilla.org/mozilla-central/file/0a46652bd992/dom/base/nsGlobalWindow.cpp#l5314
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
