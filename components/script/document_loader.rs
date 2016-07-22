/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Tracking of pending loads in a document.
//! https://html.spec.whatwg.org/multipage/#the-end

use dom::bindings::js::JS;
use dom::document::Document;
use msg::constellation_msg::PipelineId;
use net_traits::{PendingAsyncLoad, AsyncResponseTarget, LoadContext};
use net_traits::{ResourceThreads, IpcSend};
use std::thread;
use url::Url;

#[derive(JSTraceable, PartialEq, Clone, Debug, HeapSizeOf)]
pub enum LoadType {
    Image(Url),
    Script(Url),
    Subframe(Url),
    Stylesheet(Url),
    PageSource(Url),
    Media(Url),
}

impl LoadType {
    fn url(&self) -> &Url {
        match *self {
            LoadType::Image(ref url) |
            LoadType::Script(ref url) |
            LoadType::Subframe(ref url) |
            LoadType::Stylesheet(ref url) |
            LoadType::Media(ref url) |
            LoadType::PageSource(ref url) => url,
        }
    }

    fn to_load_context(&self) -> LoadContext {
        match *self {
            LoadType::Image(_) => LoadContext::Image,
            LoadType::Script(_) => LoadContext::Script,
            LoadType::Subframe(_) | LoadType::PageSource(_) => LoadContext::Browsing,
            LoadType::Stylesheet(_) => LoadContext::Style,
            LoadType::Media(_) => LoadContext::AudioVideo,
        }
    }
}

/// Canary value ensuring that manually added blocking loads (ie. ones that weren't
/// created via DocumentLoader::prepare_async_load) are always removed by the time
/// that the owner is destroyed.
#[derive(JSTraceable, HeapSizeOf)]
#[must_root]
pub struct LoadBlocker {
    /// The document whose load event is blocked by this object existing.
    doc: JS<Document>,
    /// The load that is blocking the document's load event.
    load: Option<LoadType>,
}

impl LoadBlocker {
    /// Mark the document's load event as blocked on this new load.
    pub fn new(doc: &Document, load: LoadType) -> LoadBlocker {
        doc.add_blocking_load(load.clone());
        LoadBlocker {
            doc: JS::from_ref(doc),
            load: Some(load),
        }
    }

    /// Remove this load from the associated document's list of blocking loads.
    pub fn terminate(blocker: &mut Option<LoadBlocker>) {
        if let Some(this) = blocker.as_mut() {
            this.doc.finish_load(this.load.take().unwrap());
        }
        *blocker = None;
    }

    /// Return the url associated with this load.
    pub fn url(&self) -> Option<&Url> {
        self.load.as_ref().map(LoadType::url)
    }
}

impl Drop for LoadBlocker {
    fn drop(&mut self) {
        if !thread::panicking() {
            debug_assert!(self.load.is_none());
        }
    }
}

#[derive(JSTraceable, HeapSizeOf)]
pub struct DocumentLoader {
    resource_threads: ResourceThreads,
    pipeline: Option<PipelineId>,
    blocking_loads: Vec<LoadType>,
    events_inhibited: bool,
}

impl DocumentLoader {
    pub fn new(existing: &DocumentLoader) -> DocumentLoader {
        DocumentLoader::new_with_threads(existing.resource_threads.clone(), None, None)
    }

    pub fn new_with_threads(resource_threads: ResourceThreads,
                            pipeline: Option<PipelineId>,
                            initial_load: Option<Url>) -> DocumentLoader {
        let initial_loads = initial_load.into_iter().map(LoadType::PageSource).collect();

        DocumentLoader {
            resource_threads: resource_threads,
            pipeline: pipeline,
            blocking_loads: initial_loads,
            events_inhibited: false,
        }
    }

    /// Add a load to the list of blocking loads.
    pub fn add_blocking_load(&mut self, load: LoadType) {
        self.blocking_loads.push(load);
    }

    /// Create a new pending network request, which can be initiated at some point in
    /// the future.
    pub fn prepare_async_load(&mut self,
                              load: LoadType,
                              referrer: &Document) -> PendingAsyncLoad {
        let context = load.to_load_context();
        let url = load.url().clone();
        self.add_blocking_load(load);
        PendingAsyncLoad::new(context,
                              self.resource_threads.sender(),
                              url,
                              self.pipeline,
                              referrer.get_referrer_policy(),
                              Some(referrer.url().clone()))
    }

    /// Create and initiate a new network request.
    pub fn load_async(&mut self,
                      load: LoadType,
                      listener: AsyncResponseTarget,
                      referrer: &Document) {
        let pending = self.prepare_async_load(load, referrer);
        pending.load_async(listener)
    }

    /// Mark an in-progress network request complete.
    pub fn finish_load(&mut self, load: &LoadType) {
        let idx = self.blocking_loads.iter().position(|unfinished| *unfinished == *load);
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

    pub fn resource_threads(&self) -> &ResourceThreads {
        &self.resource_threads
    }
}
