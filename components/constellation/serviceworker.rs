/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#[cfg(feature = "multiprocess")]
use ipc_channel::IpcError;
use serde::{Deserialize, Serialize};
use servo_base::generic_channel::GenericSender;
#[cfg(feature = "multiprocess")]
use servo_config::opts::{self, Opts};
#[cfg(feature = "multiprocess")]
use servo_config::{prefs, prefs::Preferences};
use servo_constellation_traits::{SWManagerSenders, ServiceWorkerManagerFactory};
use servo_url::ImmutableOrigin;

#[cfg(feature = "multiprocess")]
use crate::process_manager::Process;
#[cfg(feature = "multiprocess")]
use crate::sandboxing::UnprivilegedContent;
#[cfg(feature = "multiprocess")]
use crate::sandboxing::spawn_multiprocess;

/// Conceptually, this is glue to start an agent-cluster for a service worker agent.
/// <https://html.spec.whatwg.org/multipage/#obtain-a-service-worker-agent>
#[derive(Deserialize, Serialize)]
pub struct ServiceWorkerUnprivilegedContent {
    #[cfg(feature = "multiprocess")]
    pub opts: Opts,
    #[cfg(feature = "multiprocess")]
    pub prefs: Box<Preferences>,
    senders: SWManagerSenders,
    origin: ImmutableOrigin,
    lifeline_sender: Option<GenericSender<()>>,
}

impl ServiceWorkerUnprivilegedContent {
    pub fn new(
        senders: SWManagerSenders,
        origin: ImmutableOrigin,
        lifeline_sender: Option<GenericSender<()>>,
    ) -> ServiceWorkerUnprivilegedContent {
        ServiceWorkerUnprivilegedContent {
            #[cfg(feature = "multiprocess")]
            opts: (*opts::get()).clone(),
            #[cfg(feature = "multiprocess")]
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
    #[cfg(feature = "multiprocess")]
    pub fn spawn_multiprocess(self) -> Result<Process, IpcError> {
        spawn_multiprocess(UnprivilegedContent::ServiceWorker(self))
    }
}
