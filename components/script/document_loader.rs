/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Tracking of pending loads in a document.
//!
//! <https://html.spec.whatwg.org/multipage/#the-end>

use ipc_channel::ipc::IpcSender;
use net_traits::request::RequestBuilder;
use net_traits::{CoreResourceMsg, FetchChannels, FetchResponseMsg, IpcSend, ResourceThreads};
use servo_url::ServoUrl;

use crate::dom::bindings::root::Dom;
use crate::dom::document::Document;
use crate::fetch::FetchCanceller;

#[derive(Clone, Debug, JSTraceable, MallocSizeOf, PartialEq)]
pub enum LoadType {
    Image(#[no_trace] ServoUrl),
    Script(#[no_trace] ServoUrl),
    Subframe(#[no_trace] ServoUrl),
    Stylesheet(#[no_trace] ServoUrl),
    PageSource(#[no_trace] ServoUrl),
    Media,
}

/// Canary value ensuring that manually added blocking loads (ie. ones that weren't
/// created via DocumentLoader::fetch_async) are always removed by the time
/// that the owner is destroyed.
#[derive(JSTraceable, MallocSizeOf)]
#[crown::unrooted_must_root_lint::must_root]
pub struct LoadBlocker {
    /// The document whose load event is blocked by this object existing.
    doc: Dom<Document>,
    /// The load that is blocking the document's load event.
    load: Option<LoadType>,
}

impl LoadBlocker {
    /// Mark the document's load event as blocked on this new load.
    pub fn new(doc: &Document, load: LoadType) -> LoadBlocker {
        doc.loader_mut().add_blocking_load(load.clone());
        LoadBlocker {
            doc: Dom::from_ref(doc),
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
}

impl Drop for LoadBlocker {
    fn drop(&mut self) {
        if let Some(load) = self.load.take() {
            self.doc.finish_load(load);
        }
    }
}

#[derive(JSTraceable, MallocSizeOf)]
pub struct DocumentLoader {
    #[no_trace]
    resource_threads: ResourceThreads,
    blocking_loads: Vec<LoadType>,
    events_inhibited: bool,
    cancellers: Vec<FetchCanceller>,
}

impl DocumentLoader {
    pub fn new(existing: &DocumentLoader) -> DocumentLoader {
        DocumentLoader::new_with_threads(existing.resource_threads.clone(), None)
    }

    pub fn new_with_threads(
        resource_threads: ResourceThreads,
        initial_load: Option<ServoUrl>,
    ) -> DocumentLoader {
        debug!("Initial blocking load {:?}.", initial_load);
        let initial_loads = initial_load.into_iter().map(LoadType::PageSource).collect();

        DocumentLoader {
            resource_threads,
            blocking_loads: initial_loads,
            events_inhibited: false,
            cancellers: Vec::new(),
        }
    }

    pub fn cancel_all_loads(&mut self) -> bool {
        let canceled_any = !self.cancellers.is_empty();
        // Associated fetches will be canceled when dropping the canceller.
        self.cancellers.clear();
        canceled_any
    }

    /// Add a load to the list of blocking loads.
    fn add_blocking_load(&mut self, load: LoadType) {
        debug!(
            "Adding blocking load {:?} ({}).",
            load,
            self.blocking_loads.len()
        );
        self.blocking_loads.push(load);
    }

    /// Initiate a new fetch.
    pub fn fetch_async(
        &mut self,
        load: LoadType,
        request: RequestBuilder,
        fetch_target: IpcSender<FetchResponseMsg>,
    ) {
        self.add_blocking_load(load);
        self.fetch_async_background(request, fetch_target);
    }

    /// Initiate a new fetch that does not block the document load event.
    pub fn fetch_async_background(
        &mut self,
        request: RequestBuilder,
        fetch_target: IpcSender<FetchResponseMsg>,
    ) {
        let mut canceller = FetchCanceller::new();
        let cancel_receiver = canceller.initialize();
        self.cancellers.push(canceller);
        self.resource_threads
            .sender()
            .send(CoreResourceMsg::Fetch(
                request,
                FetchChannels::ResponseMsg(fetch_target, Some(cancel_receiver)),
            ))
            .unwrap();
    }

    /// Mark an in-progress network request complete.
    pub fn finish_load(&mut self, load: &LoadType) {
        debug!(
            "Removing blocking load {:?} ({}).",
            load,
            self.blocking_loads.len()
        );
        let idx = self
            .blocking_loads
            .iter()
            .position(|unfinished| *unfinished == *load);
        match idx {
            Some(i) => {
                self.blocking_loads.remove(i);
            },
            None => warn!("unknown completed load {:?}", load),
        }
    }

    pub fn is_blocked(&self) -> bool {
        // TODO: Ensure that we report blocked if parsing is still ongoing.
        !self.blocking_loads.is_empty()
    }

    pub fn is_only_blocked_by_iframes(&self) -> bool {
        self.blocking_loads
            .iter()
            .all(|load| matches!(*load, LoadType::Subframe(_)))
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
