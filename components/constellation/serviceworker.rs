/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use constellation_traits::{SWManagerSenders, ServiceWorkerManagerFactory};
use ipc_channel::Error;
use ipc_channel::ipc::IpcSender;
use serde::{Deserialize, Serialize};
use servo_config::opts::{self, Opts};
use servo_config::prefs;
use servo_config::prefs::Preferences;
use servo_url::ImmutableOrigin;

use crate::process_manager::Process;
use crate::sandboxing::{UnprivilegedContent, spawn_multiprocess};

/// Conceptually, this is glue to start an agent-cluster for a service worker agent.
/// <https://html.spec.whatwg.org/multipage/#obtain-a-service-worker-agent>
#[derive(Deserialize, Serialize)]
pub struct ServiceWorkerUnprivilegedContent {
    opts: Opts,
    prefs: Box<Preferences>,
    senders: SWManagerSenders,
    origin: ImmutableOrigin,
    lifeline_sender: Option<IpcSender<()>>,
}

impl ServiceWorkerUnprivilegedContent {
    pub fn new(
        senders: SWManagerSenders,
        origin: ImmutableOrigin,
        lifeline_sender: Option<IpcSender<()>>,
    ) -> ServiceWorkerUnprivilegedContent {
        ServiceWorkerUnprivilegedContent {
            opts: (*opts::get()).clone(),
            prefs: Box::new(prefs::get().clone()),
            senders,
            origin,
            lifeline_sender,
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
    pub fn spawn_multiprocess(self) -> Result<Process, Error> {
        spawn_multiprocess(UnprivilegedContent::ServiceWorker(self))
    }

    pub fn opts(&self) -> Opts {
        self.opts.clone()
    }

    pub fn prefs(&self) -> &Preferences {
        &self.prefs
    }
}
