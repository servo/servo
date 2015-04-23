/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Tracking of pending loads in a document.
//! https://html.spec.whatwg.org/multipage/syntax.html#the-end

use script_task::{ScriptMsg, ScriptChan};
use msg::constellation_msg::{PipelineId};
use net_traits::{LoadResponse, Metadata, load_whole_resource, ResourceTask, PendingAsyncLoad};
use url::Url;

use std::sync::mpsc::Receiver;

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

    /// Create a new pending network request, which can be initiated at some point in
    /// the future.
    pub fn prep_async_load(&mut self, load: LoadType) -> PendingAsyncLoad {
        self.blocking_loads.push(load.clone());
        PendingAsyncLoad::new(self.resource_task.clone(), load.url().clone())
    }

    /// Create and initiate a new network request.
    pub fn load_async(&mut self, load: LoadType) -> Receiver<LoadResponse> {
        let pending = self.prep_async_load(load);
        pending.load()
    }

    /// Create, initiate, and await the response for a new network request.
    pub fn load_sync(&mut self, load: LoadType) -> Result<(Metadata, Vec<u8>), String> {
        self.blocking_loads.push(load.clone());
        let result = load_whole_resource(&self.resource_task, load.url().clone());
        self.finish_load(load);
        result
    }

    /// Mark an in-progress network request complete.
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
