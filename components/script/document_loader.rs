/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Tracking of pending loads in a document.
//! https://html.spec.whatwg.org/multipage/syntax.html#the-end

use script_task::{ScriptMsg, ScriptChan};
use servo_msg::constellation_msg::{PipelineId};
use servo_net::resource_task::{LoadResponse, Metadata, load_whole_resource, ResourceTask};
use servo_net::resource_task::{ControlMsg, LoadData};
use url::Url;

#[jstraceable]
#[deriving(PartialEq, Clone)]
pub enum LoadType {
    Image(Url),
    Script(Url),
    Subframe(Url),
    Stylesheet(Url),
    PageSource(Url),
}

impl LoadType {
    fn url(&self) -> &Url {
        match *self {
            LoadType::Image(ref url) | LoadType::Script(ref url) | LoadType::Subframe(ref url) |
            LoadType::Stylesheet(ref url) | LoadType::PageSource(ref url) => url,
        }
    }
}

#[jstraceable]
pub struct DocumentLoader {
    pub resource_task: ResourceTask,
    script_chan: Box<ScriptChan + Send>,
    blocking_loads: Vec<LoadType>,
    pipeline: PipelineId,
    notify: bool,
}

impl DocumentLoader {
    pub fn new(existing: &DocumentLoader) -> DocumentLoader {
        DocumentLoader::new_with_task(existing.resource_task.clone(),
                                      existing.script_chan.clone(),
                                      existing.pipeline)
    }

    pub fn new_with_task(resource_task: ResourceTask, script_chan: Box<ScriptChan + Send>,
                         pipeline: PipelineId) -> DocumentLoader {
        DocumentLoader {
            resource_task: resource_task,
            script_chan: script_chan,
            pipeline: pipeline,
            blocking_loads: vec!(),
            notify: true,
        }
    }

    pub fn load_async(&mut self, load: LoadType) -> Receiver<LoadResponse> {
        self.load_async_with(load, |_| {})
    }

    pub fn load_async_with(&mut self, load: LoadType, cb: |load_data: &mut LoadData|) -> Receiver<LoadResponse> {
        let (tx, rx) = channel();
        self.blocking_loads.push(load.clone());
        let mut load_data = LoadData::new(load.url().clone(), tx);
        cb(&mut load_data);
        self.resource_task.send(ControlMsg::Load(load_data));
        rx
    }

    pub fn load_sync(&mut self, load: LoadType) -> Result<(Metadata, Vec<u8>), String> {
        self.blocking_loads.push(load.clone());
        let result = load_whole_resource(&self.resource_task, load.url().clone());
        self.finish_load(load);
        result
    }

    pub fn finish_load(&mut self, load: LoadType) {
        let idx = self.blocking_loads.iter().position(|unfinished| *unfinished == load);
        self.blocking_loads.remove(idx.expect("unknown completed load"));

        if !self.is_blocked() && self.notify {
            self.script_chan.send(ScriptMsg::DocumentLoadsComplete(self.pipeline));
        }
    }

    pub fn is_blocked(&self) -> bool {
        !self.blocking_loads.is_empty()
    }

    pub fn inhibit_events(&mut self) {
        self.notify = false;
    }
}
