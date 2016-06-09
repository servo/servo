/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The service worker manager persists the descriptor of any active service workers.
//! If a service worker timeouts, then it removes the descriptor entry from its
//! active_descriptors map

use dom::serviceworkerglobalscope::ServiceWorkerGlobalScope;
use dom::serviceworkerregistration::longest_prefix_match;
use ipc_channel::ipc::{self, IpcSender, IpcReceiver};
use script_traits::{ServiceWorkerMsg, ScriptMsg, ScopeThings};
use std::collections::HashMap;
use std::sync::mpsc::channel;
use url::Url;
use util::thread::spawn_named;

struct ServiceWorkerManager {
    // stores a map of scope url with ScopeThings
    active_descriptors: HashMap<Url, ScopeThings>,
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
                ServiceWorkerMsg::FromConstellation(net_sender, load_url) => {
                    debug!("Service Worker Manager detected a navigation");
                    let mut scope_url = None;
                        for scope in self.active_descriptors.keys() {
                            if longest_prefix_match(&scope, &load_url) {
                                scope_url = Some(scope.clone());
                            }
                        }
                    self.own_sender.send(ServiceWorkerMsg::ActivateWorker(net_sender, scope_url)).unwrap();
                }
                ServiceWorkerMsg::ActivateWorker(net_sender, scope_url) => {
                    if scope_url.is_some() {
                        let scope_url = scope_url.unwrap();
                        let scope_things = self.active_descriptors.remove(&scope_url);
                        if scope_things.is_some() {
                            let (sender, receiver) = channel();
                                ServiceWorkerGlobalScope::run_serviceworker_scope(scope_things.unwrap(),
                                                                                  sender,
                                                                                  receiver,
                                                                                  self.own_sender.clone(),
                                                                                  scope_url);
                            } else { debug!("Unable to activate service worker"); }
                    } else { debug!("No registrations for this url"); }

                    // TODO this net_sender will need to be send to the appropriate service worker
                    // so that it may send do the sending of custom responses.
                    // For now we just send a None from here itself
                    net_sender.send(None).unwrap();
                }
                ServiceWorkerMsg::StoreScope(scope_things, scope) => {
                    if self.active_descriptors.contains_key(&scope) {
                        debug!("ScopeThings for {:?} already stored in SW-Manager", scope);
                    } else {
                        self.active_descriptors.insert(scope.clone(), scope_things);
                    }
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
