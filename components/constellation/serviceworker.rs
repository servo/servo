/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;

use ipc_channel::Error;
use script_traits::{SWManagerSenders, ServiceWorkerManagerFactory};
use serde::{Deserialize, Serialize};
use servo_config::opts::{self, Opts};
use servo_config::prefs::{self, PrefValue};
use servo_url::ImmutableOrigin;

use crate::sandboxing::{spawn_multiprocess, UnprivilegedContent};

/// Conceptually, this is glue to start an agent-cluster for a service worker agent.
/// <https://html.spec.whatwg.org/multipage/#obtain-a-service-worker-agent>
#[derive(Deserialize, Serialize)]
pub struct ServiceWorkerUnprivilegedContent {
    opts: Opts,
    prefs: HashMap<String, PrefValue>,
    senders: SWManagerSenders,
    origin: ImmutableOrigin,
}

impl ServiceWorkerUnprivilegedContent {
    pub fn new(
        senders: SWManagerSenders,
        origin: ImmutableOrigin,
    ) -> ServiceWorkerUnprivilegedContent {
        ServiceWorkerUnprivilegedContent {
            opts: (*opts::get()).clone(),
            prefs: prefs::pref_map().iter().collect(),
            senders,
            origin,
        }
    }

    /// Start the agent-cluster.
    pub fn start<SWF>(self)
    where
        SWF: ServiceWorkerManagerFactory,
    {
        SWF::create(self.senders, self.origin);
    }

    /// Start the agent-cluster in it's own process.
    pub fn spawn_multiprocess(self) -> Result<(), Error> {
        spawn_multiprocess(UnprivilegedContent::ServiceWorker(self))
    }

    pub fn opts(&self) -> Opts {
        self.opts.clone()
    }

    pub fn prefs(&self) -> HashMap<String, PrefValue> {
        self.prefs.clone()
    }
}
