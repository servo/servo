/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::RefCell;
use std::rc::Rc;

use base::generic_channel::GenericCallback;
use embedder_traits::{EmbedderMsg, EmbedderProxy};
use net_traits::ResourceThreads;
use rustc_hash::FxHashMap;

struct Callback {
    callback: Box<dyn FnOnce()>,
}

pub(crate) struct CallbackProxy {
    embedder_proxy: EmbedderProxy,
    id: usize,
}

impl CallbackProxy {
    pub fn new(embedder_proxy: EmbedderProxy, id: usize) -> Self {
        Self { embedder_proxy, id }
    }

    pub fn send(&self) {
        self.embedder_proxy
            .send(EmbedderMsg::RunSiteDataManagerCallback(self.id));
    }
}

struct AllCompleted {
    count: usize,
    callback: Option<Box<dyn FnOnce()>>,
}

impl AllCompleted {
    fn new(count: usize, callback: Box<dyn FnOnce()>) -> Self {
        Self {
            count,
            callback: Some(callback),
        }
    }

    fn complete(&mut self) {
        self.count -= 1;
        if self.count == 0 {
            if let Some(cb) = self.callback.take() {
                cb();
            }
        }
    }
}

pub struct SiteDataManager {
    embedder_proxy: EmbedderProxy,
    public_resource_threads: ResourceThreads,
    private_resource_threads: ResourceThreads,
    callbacks: FxHashMap<usize, Callback>,
    next_id: usize,
}

impl SiteDataManager {
    pub(crate) fn new(
        embedder_proxy: EmbedderProxy,
        public_resource_threads: ResourceThreads,
        private_resource_threads: ResourceThreads,
    ) -> Self {
        Self {
            embedder_proxy,
            public_resource_threads,
            private_resource_threads,
            callbacks: Default::default(),
            next_id: 1,
        }
    }

    pub fn clear_cache(&mut self, callback: Box<dyn FnOnce()>) {
        let mut proxies = self.create_joined_callback_proxies(2, callback);

        let public_done = proxies.pop().unwrap();
        let private_done = proxies.pop().unwrap();

        self.public_resource_threads.clear_cache(
            GenericCallback::new(move |_msg| {
                public_done.send();
            })
            .unwrap(),
        );

        self.private_resource_threads.clear_cache(
            GenericCallback::new(move |_msg| {
                private_done.send();
            })
            .unwrap(),
        );
    }

    pub(crate) fn create_callback_proxy(&mut self, callback: Box<dyn FnOnce()>) -> CallbackProxy {
        let id = self.next_id;
        self.next_id = id + 1;

        self.callbacks.insert(id, Callback { callback });

        CallbackProxy::new(self.embedder_proxy.clone(), id)
    }

    pub(crate) fn create_joined_callback_proxies(
        &mut self,
        count: usize,
        final_callback: Box<dyn FnOnce()>,
    ) -> Vec<CallbackProxy> {
        assert!(count > 0);

        let join = Rc::new(RefCell::new(AllCompleted::new(count, final_callback)));

        let mut proxies = Vec::with_capacity(count);

        for _ in 0..count {
            let join = Rc::clone(&join);
            let proxy = self.create_callback_proxy(Box::new(move || {
                join.borrow_mut().complete();
            }));
            proxies.push(proxy);
        }

        proxies
    }

    pub(crate) fn run_callback(&mut self, id: usize) {
        (self
            .callbacks
            .remove(&id)
            .expect("Received request to run unknown callback.")
            .callback)();
    }
}
