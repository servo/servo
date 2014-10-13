/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Tracking of pending loads in a document.
//! https://html.spec.whatwg.org/multipage/#the-end

use script_task::{ScriptMsg, ScriptChan};
use msg::constellation_msg::{PipelineId};
use net_traits::{LoadResponse, Metadata, load_whole_resource, ResourceTask};
use net_traits::{ControlMsg, LoadData, LoadConsumer};
use url::Url;

use std::sync::mpsc::{Receiver, channel};

#[jstraceable]
#[derive(PartialEq, Clone)]
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
            LoadType::Image(ref url) |
            LoadType::Script(ref url) |
            LoadType::Subframe(ref url) |
            LoadType::Stylesheet(ref url) |
            LoadType::PageSource(ref url) => url,
        }
    }
}

#[jstraceable]
pub struct DocumentLoader {
    pub resource_task: ResourceTask,
    notifier_data: Option<NotifierData>,
    blocking_loads: Vec<LoadType>,
}

#[jstraceable]
pub struct NotifierData {
    pub script_chan: Box<ScriptChan + Send>,
    pub pipeline: PipelineId,
}

impl DocumentLoader {
    pub fn new(existing: &DocumentLoader) -> DocumentLoader {
        DocumentLoader::new_with_task(existing.resource_task.clone(), None, None)
    }

    pub fn new_with_task(resource_task: ResourceTask,
                         data: Option<NotifierData>,
                         initial_load: Option<Url>,)
                         -> DocumentLoader {
        let mut initial_loads = vec!();
        if let Some(load) = initial_load {
            initial_loads.push(LoadType::PageSource(load));
        }

        DocumentLoader {
            resource_task: resource_task,
            notifier_data: data,
            blocking_loads: initial_loads,
        }
    }

    pub fn load_async(&mut self, load: LoadType) -> Receiver<LoadResponse> {
        let (tx, rx) = channel();
        self.blocking_loads.push(load.clone());
        let pipeline = self.notifier_data.as_ref().map(|data| data.pipeline);
        let load_data = LoadData::new(load.url().clone(), pipeline);
        self.resource_task.send(ControlMsg::Load(load_data, LoadConsumer::Channel(tx))).unwrap();
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

        if let Some(NotifierData { ref script_chan, pipeline }) = self.notifier_data {
            if !self.is_blocked() {
                script_chan.send(ScriptMsg::DocumentLoadsComplete(pipeline)).unwrap();
            }
        }
    }

    pub fn is_blocked(&self) -> bool {
        !self.blocking_loads.is_empty()
    }

    pub fn inhibit_events(&mut self) {
        self.notifier_data = None;
    }
}
