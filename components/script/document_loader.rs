/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Tracking of pending loads in a document.
//! https://html.spec.whatwg.org/multipage/#the-end

use dom::bindings::js::JS;
use dom::document::Document;
use ipc_channel::ipc::IpcSender;
use net_traits::{CoreResourceMsg, FetchResponseMsg, ResourceThreads, IpcSend};
use net_traits::request::RequestInit;
use servo_url::ServoUrl;
use std::thread;

#[derive(JSTraceable, PartialEq, Clone, Debug, HeapSizeOf)]
pub enum LoadType {
    Image(ServoUrl),
    Script(ServoUrl),
    Subframe(ServoUrl),
    Stylesheet(ServoUrl),
    PageSource(ServoUrl),
    Media(ServoUrl),
}

impl LoadType {
    fn url(&self) -> &ServoUrl {
        match *self {
            LoadType::Image(ref url) |
            LoadType::Script(ref url) |
            LoadType::Subframe(ref url) |
            LoadType::Stylesheet(ref url) |
            LoadType::Media(ref url) |
            LoadType::PageSource(ref url) => url,
        }
    }
}

/// Canary value ensuring that manually added blocking loads (ie. ones that weren't
/// created via DocumentLoader::fetch_async) are always removed by the time
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
        doc.mut_loader().add_blocking_load(load.clone());
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
    pub fn url(&self) -> Option<&ServoUrl> {
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
    blocking_loads: Vec<LoadType>,
    events_inhibited: bool,
}

impl DocumentLoader {
    pub fn new(existing: &DocumentLoader) -> DocumentLoader {
        DocumentLoader::new_with_threads(existing.resource_threads.clone(), None)
    }

    pub fn new_with_threads(resource_threads: ResourceThreads,
                            initial_load: Option<ServoUrl>) -> DocumentLoader {
        debug!("Initial blocking load {:?}.", initial_load);
        let initial_loads = initial_load.into_iter().map(LoadType::PageSource).collect();

        DocumentLoader {
            resource_threads: resource_threads,
            blocking_loads: initial_loads,
            events_inhibited: false,
        }
    }

    /// Add a load to the list of blocking loads.
    fn add_blocking_load(&mut self, load: LoadType) {
        debug!("Adding blocking load {:?} ({}).", load, self.blocking_loads.len());
        self.blocking_loads.push(load);
    }

    /// Initiate a new fetch.
    pub fn fetch_async(&mut self,
                       load: LoadType,
                       request: RequestInit,
                       fetch_target: IpcSender<FetchResponseMsg>) {
        self.add_blocking_load(load);
        self.resource_threads.sender().send(CoreResourceMsg::Fetch(request, fetch_target)).unwrap();
    }

    /// Mark an in-progress network request complete.
    pub fn finish_load(&mut self, load: &LoadType) {
        debug!("Removing blocking load {:?} ({}).", load, self.blocking_loads.len());
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
