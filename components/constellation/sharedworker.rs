/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::sync::Arc;
use std::thread::JoinHandle;

#[cfg(feature = "multiprocess")]
use ipc_channel::IpcError;
use layout_api::ScriptThreadFactory;
use net_traits::image_cache::ImageCacheFactory;
use serde::{Deserialize, Serialize};
#[cfg(feature = "multiprocess")]
use servo_config::opts::{self, Opts};
#[cfg(feature = "multiprocess")]
use servo_config::{prefs, prefs::Preferences};
use servo_constellation_traits::SharedWorkerManagerSenders;
use servo_url::ImmutableOrigin;

#[cfg(feature = "multiprocess")]
use crate::process_manager::Process;
#[cfg(feature = "multiprocess")]
use crate::sandboxing::{UnprivilegedContent, spawn_multiprocess};

/// <https://html.spec.whatwg.org/multipage/#shared-worker-manager>
#[derive(Deserialize, Serialize)]
pub struct SharedWorkerUnprivilegedContent {
    #[cfg(feature = "multiprocess")]
    pub opts: Opts,
    #[cfg(feature = "multiprocess")]
    pub prefs: Box<Preferences>,
    senders: SharedWorkerManagerSenders,
    origin: ImmutableOrigin,
}

impl SharedWorkerUnprivilegedContent {
    pub fn new(senders: SharedWorkerManagerSenders, origin: ImmutableOrigin) -> Self {
        Self {
            #[cfg(feature = "multiprocess")]
            opts: (*opts::get()).clone(),
            #[cfg(feature = "multiprocess")]
            prefs: Box::new(prefs::get().clone()),
            senders,
            origin,
        }
    }

    pub fn start<STF: ScriptThreadFactory>(
        self,
        image_cache_factory: Arc<dyn ImageCacheFactory>,
    ) -> JoinHandle<()> {
        STF::create_shared_worker_manager(self.senders, self.origin, image_cache_factory)
    }

    #[cfg(feature = "multiprocess")]
    pub fn spawn_multiprocess(self) -> Result<Process, IpcError> {
        spawn_multiprocess(UnprivilegedContent::SharedWorker(self))
    }
}
