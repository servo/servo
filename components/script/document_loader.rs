/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Tracking of pending loads in a document.
//!
//! <https://html.spec.whatwg.org/multipage/#the-end>

use net_traits::request::RequestBuilder;
use net_traits::{BoxedFetchCallback, ResourceThreads, fetch_async};
use servo_url::ServoUrl;

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::root::Dom;
use crate::dom::document::Document;
use crate::fetch::FetchCanceller;
use crate::script_runtime::CanGc;

#[derive(Clone, Debug, JSTraceable, MallocSizeOf, PartialEq)]
pub(crate) enum LoadType {
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
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
pub(crate) struct LoadBlocker {
    /// The document whose load event is blocked by this object existing.
    doc: Dom<Document>,
    /// The load that is blocking the document's load event.
    load: Option<LoadType>,
}

impl LoadBlocker {
    /// Mark the document's load event as blocked on this new load.
    pub(crate) fn new(doc: &Document, load: LoadType) -> LoadBlocker {
        doc.loader_mut().add_blocking_load(load.clone());
        LoadBlocker {
            doc: Dom::from_ref(doc),
            load: Some(load),
        }
    }

    /// Remove this load from the associated document's list of blocking loads.
    pub(crate) fn terminate(blocker: &DomRefCell<Option<LoadBlocker>>, can_gc: CanGc) {
        let Some(load) = blocker
            .borrow_mut()
            .as_mut()
            .and_then(|blocker| blocker.load.take())
        else {
            return;
        };

        if let Some(blocker) = blocker.borrow().as_ref() {
            blocker.doc.finish_load(load, can_gc);
        }

        *blocker.borrow_mut() = None;
    }
}

impl Drop for LoadBlocker {
    fn drop(&mut self) {
        if let Some(load) = self.load.take() {
            self.doc.finish_load(load, CanGc::note());
        }
    }
}

#[derive(JSTraceable, MallocSizeOf)]
pub(crate) struct DocumentLoader {
    #[no_trace]
    resource_threads: ResourceThreads,
    blocking_loads: Vec<LoadType>,
    events_inhibited: bool,
    cancellers: Vec<FetchCanceller>,
}

impl DocumentLoader {
    pub(crate) fn new(existing: &DocumentLoader) -> DocumentLoader {
        DocumentLoader::new_with_threads(existing.resource_threads.clone(), None)
    }

    pub(crate) fn new_with_threads(
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

    pub(crate) fn cancel_all_loads(&mut self) -> bool {
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

    /// Initiate a new fetch given a response callback.
    pub(crate) fn fetch_async_with_callback(
        &mut self,
        load: LoadType,
        request: RequestBuilder,
        callback: BoxedFetchCallback,
    ) {
        self.add_blocking_load(load);
        self.fetch_async_background(request, callback);
    }

    /// Initiate a new fetch that does not block the document load event.
    pub(crate) fn fetch_async_background(
        &mut self,
        request: RequestBuilder,
        callback: BoxedFetchCallback,
    ) {
        self.cancellers.push(FetchCanceller::new(request.id));
        fetch_async(&self.resource_threads.core_thread, request, None, callback);
    }

    /// Mark an in-progress network request complete.
    pub(crate) fn finish_load(&mut self, load: &LoadType) {
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

    pub(crate) fn is_blocked(&self) -> bool {
        // TODO: Ensure that we report blocked if parsing is still ongoing.
        !self.blocking_loads.is_empty()
    }

    pub(crate) fn is_only_blocked_by_iframes(&self) -> bool {
        self.blocking_loads
            .iter()
            .all(|load| matches!(*load, LoadType::Subframe(_)))
    }

    pub(crate) fn inhibit_events(&mut self) {
        self.events_inhibited = true;
    }

    pub(crate) fn events_inhibited(&self) -> bool {
        self.events_inhibited
    }

    pub(crate) fn resource_threads(&self) -> &ResourceThreads {
        &self.resource_threads
    }
}
