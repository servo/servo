/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The service worker manager persists the descriptor of any registered service workers.
//! It also stores an active workers map, which holds descriptors of running service workers.
//! If an active service worker timeouts, then it removes the descriptor entry from its
//! active_workers map

use devtools_traits::{DevtoolsPageInfo, ScriptToDevtoolsControlMsg};
use dom::serviceworkerglobalscope::ServiceWorkerGlobalScope;
use dom::serviceworkerregistration::longest_prefix_match;
use ipc_channel::ipc::{self, IpcSender, IpcReceiver};
use script_traits::{ServiceWorkerMsg, ScopeThings, SWManagerMsg};
use std::collections::HashMap;
use std::sync::mpsc::channel;
use url::Url;
use util::thread::spawn_named;

pub struct ServiceWorkerManager {
    // map of registered service worker descriptors
    registered_workers: HashMap<Url, ScopeThings>,
    // map of active service worker descriptors
    active_workers: HashMap<Url, ScopeThings>,
    // own sender to send messages here
    own_sender: IpcSender<ServiceWorkerMsg>,
    // receiver to receive messages from constellation
    own_port: IpcReceiver<ServiceWorkerMsg>,
}

impl ServiceWorkerManager {
    fn new(own_sender: IpcSender<ServiceWorkerMsg>,
           from_constellation_receiver: IpcReceiver<ServiceWorkerMsg>) -> ServiceWorkerManager {
        ServiceWorkerManager {
            registered_workers: HashMap::new(),
            active_workers: HashMap::new(),
            own_sender: own_sender,
            own_port: from_constellation_receiver
        }
    }

    pub fn spawn_manager(from_swmanager_sender: IpcSender<SWManagerMsg>) {
        let (own_sender, from_constellation_receiver) = ipc::channel().unwrap();
        from_swmanager_sender.send(SWManagerMsg::OwnSender(own_sender.clone())).unwrap();
        spawn_named("ServiceWorkerManager".to_owned(), move || {
            ServiceWorkerManager::new(own_sender, from_constellation_receiver).start();
        });
    }

    pub fn prepare_activation(&mut self, load_url: &Url) {
        let mut scope_url = None;
        for scope in self.registered_workers.keys() {
            if longest_prefix_match(&scope, load_url) {
                scope_url = Some(scope.clone());
                break;
            }
        }

        if let Some(ref scope_url) = scope_url {
            if self.active_workers.contains_key(&scope_url) {
                // do not run the same worker if already active.
                warn!("Service worker for {:?} already active", scope_url);
                return;
            }
            let scope_things = self.registered_workers.get(&scope_url);
            if let Some(scope_things) = scope_things {
                let (sender, receiver) = channel();
                let (devtools_sender, devtools_receiver) = ipc::channel().unwrap();
                if let Some(ref chan) = scope_things.devtools_chan {
                    let title = format!("ServiceWorker for {}", scope_things.script_url);
                    let page_info = DevtoolsPageInfo {
                        title: title,
                        url: scope_things.script_url.clone(),
                    };
                    let _ = chan.send(ScriptToDevtoolsControlMsg::NewGlobal((scope_things.pipeline_id,
                                                                             Some(scope_things.worker_id)),
                                                                             devtools_sender.clone(),
                                                                             page_info));
                };
                ServiceWorkerGlobalScope::run_serviceworker_scope(scope_things.clone(),
                                                                              sender,
                                                                              receiver,
                                                                              devtools_receiver,
                                                                              self.own_sender.clone(),
                                                                              scope_url.clone());
                // store the worker in active_workers map
                self.active_workers.insert(scope_url.clone(), scope_things.clone());
            } else {
                warn!("Unable to activate service worker");
            }
        }
    }

    fn start(&mut self) {
        while let Ok(msg) = self.own_port.recv() {
            match msg {
                ServiceWorkerMsg::RegisterServiceWorker(scope_things, scope) => {
                    if self.registered_workers.contains_key(&scope) {
                        warn!("ScopeThings for {:?} already stored in SW-Manager", scope);
                    } else {
                        self.registered_workers.insert(scope, scope_things);
                    }
                }
                ServiceWorkerMsg::Timeout(scope) => {
                    if self.active_workers.contains_key(&scope) {
                        let _ = self.active_workers.remove(&scope);
                    } else {
                        warn!("ScopeThings for {:?} is not active", scope);
                    }
                }
                ServiceWorkerMsg::ActivateWorker(mediator) => {
                    self.prepare_activation(&mediator.load_url);
                    // TODO XXXcreativcoder this net_sender will need to be send to the appropriate service worker
                    // so that it may do the sending of custom responses.
                    // For now we just send a None from here itself
                    let _ = mediator.response_chan.send(None);

                }
                ServiceWorkerMsg::Exit => break
            }
        }
    }
}
