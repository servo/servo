/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::mem;

use base::generic_channel::GenericSender;
use base::id::PipelineId;
use devtools_traits::DevtoolScriptControlMsg;

use crate::actor::{Actor, ActorRegistry};
use crate::actors::timeline::HighResolutionStamp;

pub struct FramerateActor {
    name: String,
    pipeline_id: PipelineId,
    script_sender: GenericSender<DevtoolScriptControlMsg>,
    is_recording: bool,
    ticks: Vec<HighResolutionStamp>,
}

impl Actor for FramerateActor {
    fn name(&self) -> String {
        self.name.clone()
    }
}

impl FramerateActor {
    /// Return name of actor
    pub fn create(
        registry: &ActorRegistry,
        pipeline_id: PipelineId,
        script_sender: GenericSender<DevtoolScriptControlMsg>,
    ) -> String {
        let actor_name = registry.new_name("framerate");
        let mut actor = FramerateActor {
            name: actor_name.clone(),
            pipeline_id,
            script_sender,
            is_recording: false,
            ticks: Vec::new(),
        };

        actor.start_recording();
        registry.register_later(actor);
        actor_name
    }

    pub fn add_tick(&mut self, tick: f64) {
        self.ticks.push(HighResolutionStamp::wrap(tick));

        if self.is_recording {
            let msg = DevtoolScriptControlMsg::RequestAnimationFrame(self.pipeline_id, self.name());
            self.script_sender.send(msg).unwrap();
        }
    }

    pub fn take_pending_ticks(&mut self) -> Vec<HighResolutionStamp> {
        mem::take(&mut self.ticks)
    }

    fn start_recording(&mut self) {
        if self.is_recording {
            return;
        }

        self.is_recording = true;

        let msg = DevtoolScriptControlMsg::RequestAnimationFrame(self.pipeline_id, self.name());
        self.script_sender.send(msg).unwrap();
    }

    fn stop_recording(&mut self) {
        if !self.is_recording {
            return;
        }
        self.is_recording = false;
    }
}

impl Drop for FramerateActor {
    fn drop(&mut self) {
        self.stop_recording();
    }
}
