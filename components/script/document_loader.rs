/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Tracking of pending loads in a document.
//! https://html.spec.whatwg.org/multipage/#the-end

use dom::bindings::js::JS;
use dom::document::Document;
use msg::constellation_msg::{PipelineId};
use net_traits::AsyncResponseTarget;
use net_traits::{Metadata, PendingAsyncLoad, ResourceTask, load_whole_resource};
use script_task::MainThreadScriptMsg;
use std::sync::Arc;
use std::sync::mpsc::Sender;
use std::thread;
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

/// Helpful destructor bomb to ensure that manual blocking loads are always dealt with
/// by the time that the owner is destroyed.
#[derive(JSTraceable, HeapSizeOf)]
#[must_root]
pub struct LoadBlocker {
    /// The document whose load event is blocked by this object existing.
    doc: JS<Document>,
    /// The load that is blocking the document's load event.
    load: Option<LoadType>,
}

impl LoadBlocker {
    pub fn new(doc: &Document, load: LoadType) -> LoadBlocker {
        doc.add_blocking_load(load.clone());
        LoadBlocker {
            doc: JS::from_ref(doc),
            load: Some(load),
        }
    }

    pub fn terminate(&mut self) {
        assert!(self.load.is_some());
        self.doc.root().finish_load(self.load.take().unwrap());
    }

    pub fn url(&self) -> Option<&Url> {
        self.load.as_ref().map(LoadType::url)
    }
}

impl Drop for LoadBlocker {
    fn drop(&mut self) {
        if !thread::panicking() {
            assert!(self.load.is_none());
        }
    }
}

#[derive(JSTraceable, HeapSizeOf)]
pub struct DocumentLoader {
    /// We use an `Arc<ResourceTask>` here in order to avoid file descriptor exhaustion when there
    /// are lots of iframes.
    #[ignore_heap_size_of = "channels are hard"]
    pub resource_task: Arc<ResourceTask>,
    notifier_data: Option<NotifierData>,
    blocking_loads: Vec<LoadType>,
}

#[derive(JSTraceable, HeapSizeOf)]
pub struct NotifierData {
    #[ignore_heap_size_of = "trait objects are hard"]
    pub script_chan: Sender<MainThreadScriptMsg>,
    pub pipeline: PipelineId,
}

impl DocumentLoader {
    pub fn new(existing: &DocumentLoader) -> DocumentLoader {
        DocumentLoader::new_with_task(existing.resource_task.clone(), None, None)
    }

    /// We use an `Arc<ResourceTask>` here in order to avoid file descriptor exhaustion when there
    /// are lots of iframes.
    pub fn new_with_task(resource_task: Arc<ResourceTask>,
                         data: Option<NotifierData>,
                         initial_load: Option<Url>)
                         -> DocumentLoader {
        let initial_loads = initial_load.into_iter().map(LoadType::PageSource).collect();

        DocumentLoader {
            resource_task: resource_task,
            notifier_data: data,
            blocking_loads: initial_loads,
        }
    }

    pub fn add_blocking_load(&mut self, load: LoadType) {
        self.blocking_loads.push(load);
    }

    /// Create a new pending network request, which can be initiated at some point in
    /// the future.
    pub fn prepare_async_load(&mut self, load: LoadType) -> PendingAsyncLoad {
        let url = load.url().clone();
        self.add_blocking_load(load);
        let pipeline = self.notifier_data.as_ref().map(|data| data.pipeline);
        PendingAsyncLoad::new((*self.resource_task).clone(), url, pipeline)
    }

    /// Create and initiate a new network request.
    pub fn load_async(&mut self, load: LoadType, listener: AsyncResponseTarget) {
        let pending = self.prepare_async_load(load);
        pending.load_async(listener)
    }

    /// Create, initiate, and await the response for a new network request.
    pub fn load_sync(&mut self, load: LoadType) -> Result<(Metadata, Vec<u8>), String> {
        self.add_blocking_load(load.clone());
        let result = load_whole_resource(&self.resource_task, load.url().clone());
        self.finish_load(load);
        result
    }

    /// Mark an in-progress network request complete.
    pub fn finish_load(&mut self, load: LoadType) {
        let idx = self.blocking_loads.iter().position(|unfinished| *unfinished == load);
        self.blocking_loads.remove(idx.expect(&format!("unknown completed load {:?}", load)));

        if let Some(NotifierData { ref script_chan, pipeline }) = self.notifier_data {
            if !self.is_blocked() {
                script_chan.send(MainThreadScriptMsg::DocumentLoadsComplete(pipeline)).unwrap();
            }
        }
    }

    pub fn is_blocked(&self) -> bool {
        //TODO: Ensure that we report blocked if parsing is still ongoing.
        !self.blocking_loads.is_empty()
    }

    pub fn inhibit_events(&mut self) {
        self.notifier_data = None;
    }
}
