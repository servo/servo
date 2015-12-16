/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Tracking of pending loads in a document.
//! https://html.spec.whatwg.org/multipage/#the-end

use msg::constellation_msg::PipelineId;
use net_traits::AsyncResponseTarget;
use net_traits::{PendingAsyncLoad, ResourceTask};
use std::sync::Arc;
use url::Url;

#[derive(JSTraceable, PartialEq, Clone, Debug, HeapSizeOf)]
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

#[derive(JSTraceable, HeapSizeOf)]
pub struct DocumentLoader {
    /// We use an `Arc<ResourceTask>` here in order to avoid file descriptor exhaustion when there
    /// are lots of iframes.
    #[ignore_heap_size_of = "channels are hard"]
    pub resource_task: Arc<ResourceTask>,
    pipeline: Option<PipelineId>,
    blocking_loads: Vec<LoadType>,
    events_inhibited: bool,
}

impl DocumentLoader {
    pub fn new(existing: &DocumentLoader) -> DocumentLoader {
        DocumentLoader::new_with_task(existing.resource_task.clone(), None, None)
    }

    /// We use an `Arc<ResourceTask>` here in order to avoid file descriptor exhaustion when there
    /// are lots of iframes.
    pub fn new_with_task(resource_task: Arc<ResourceTask>,
                         pipeline: Option<PipelineId>,
                         initial_load: Option<Url>)
                         -> DocumentLoader {
        let initial_loads = initial_load.into_iter().map(LoadType::PageSource).collect();

        DocumentLoader {
            resource_task: resource_task,
            pipeline: pipeline,
            blocking_loads: initial_loads,
            events_inhibited: false,
        }
    }

    /// Create a new pending network request, which can be initiated at some point in
    /// the future.
    pub fn prepare_async_load(&mut self, load: LoadType) -> PendingAsyncLoad {
        let url = load.url().clone();
        self.blocking_loads.push(load);
        PendingAsyncLoad::new((*self.resource_task).clone(), url, self.pipeline)
    }

    /// Create and initiate a new network request.
    pub fn load_async(&mut self, load: LoadType, listener: AsyncResponseTarget) {
        let pending = self.prepare_async_load(load);
        pending.load_async(listener)
    }


    /// Mark an in-progress network request complete.
    pub fn finish_load(&mut self, load: LoadType) {
        let idx = self.blocking_loads.iter().position(|unfinished| *unfinished == load);
        self.blocking_loads.remove(idx.expect(&format!("unknown completed load {:?}", load)));
    }

    pub fn is_blocked(&self) -> bool {
        // TODO: Ensure that we report blocked if parsing is still ongoing.
        !self.blocking_loads.is_empty()
    }

    pub fn inhibit_events(&mut self) {
        self.events_inhibited = true;
    }
    pub fn events_inhibited(&self) -> bool {
        self.events_inhibited
    }
}
