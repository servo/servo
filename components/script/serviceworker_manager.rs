/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The service worker manager persists the descriptor of any active service workers.
//! If a service worker timeouts, then it removes the descriptor entry from its
//! active_descriptors map

use dom::serviceworkerglobalscope::ServiceWorkerGlobalScope;
use ipc_channel::ipc::{self, IpcSender, IpcReceiver};
use script_traits::{ServiceWorkerMsg, ScriptMsg};
use std::collections::HashMap;
use std::sync::mpsc::channel;
use url::Url;
use util::thread::spawn_named;

struct ServiceWorkerManager {
    // stores a scope url and "is running" boolean
    // TODO; would store a 'uuid of a worker' instead of a redundant boolean; as per spec
    active_descriptors: HashMap<Url, bool>,
    // own sender to be used by active workers to signal timeout
    own_sender: IpcSender<ServiceWorkerMsg>,
    // receiver to receive ServiceWorkerMsg's from constellation
    from_constellation: IpcReceiver<ServiceWorkerMsg>,
    // to send messages to constellation
    constellation_sender: Option<IpcSender<ScriptMsg>>
}

impl ServiceWorkerManager {
    fn new(sender: IpcSender<ServiceWorkerMsg>, receiver: IpcReceiver<ServiceWorkerMsg>) -> ServiceWorkerManager {
        ServiceWorkerManager {
            active_descriptors: HashMap::new(),
            own_sender: sender,
            from_constellation: receiver,
            constellation_sender: None
        }
    }
    fn start(&mut self) {
        while let Ok(msg) = self.from_constellation.recv() {
            match msg {
                ServiceWorkerMsg::ScopeEntities(scope_things, url) => {
                    if self.active_descriptors.contains_key(&url) {
                        debug!("Service Worker at {:?} already active!", url);
                    } else {
                        self.active_descriptors.insert(url.clone(), true);
                        let (sender, receiver) = channel();
                        debug!("new service worker for {:?} spawned", url);
                        ServiceWorkerGlobalScope::run_serviceworker_scope(scope_things,
                                                                          sender,
                                                                          receiver,
                                                                          self.own_sender.clone(),
                                                                          url);
                    }
                }
                ServiceWorkerMsg::ConstellationSender(sender) => {
                    self.constellation_sender = Some(sender);
                }
                ServiceWorkerMsg::Timeout(url) => {
                    if self.active_descriptors.contains_key(&url) {
                        self.active_descriptors.remove(&url);
                    }

                    if let Some(ref sender) = self.constellation_sender {
                        sender.send(ScriptMsg::ServiceWorkerTimeout(url.clone())).unwrap();
                    } else {
                        debug!("service worker for {:?} timeout received, but sending event failed", url);
                    }
                }
                ServiceWorkerMsg::NetworkSender(net_sender) => {
                    debug!("Service Worker Manager detected a navigation");
                    // We send None as of now
                    net_sender.send(None).unwrap();
                }
                ServiceWorkerMsg::Exit => break
            }
        }
    }
 }

pub trait ServiceWorkerMessenger {
    fn new() -> Self;
}

impl ServiceWorkerMessenger for IpcSender<ServiceWorkerMsg> {
    fn new() -> IpcSender<ServiceWorkerMsg> {
        let (to_constellation, from_constellation) = ipc::channel().unwrap();
        let constellation_sender = to_constellation.clone();
        spawn_named("ServiceWorkerManager".to_owned(), move || {
            ServiceWorkerManager::new(constellation_sender, from_constellation).start();
        });
        to_constellation
    }
}
