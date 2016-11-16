/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The service worker manager persists the descriptor of any registered service workers.
//! It also stores an active workers map, which holds descriptors of running service workers.
//! If an active service worker timeouts, then it removes the descriptor entry from its
//! active_workers map

use devtools_traits::{DevtoolsPageInfo, ScriptToDevtoolsControlMsg};
use dom::abstractworker::WorkerScriptMsg;
use dom::bindings::structuredclone::StructuredCloneData;
use dom::serviceworkerglobalscope::{ServiceWorkerGlobalScope, ServiceWorkerScriptMsg};
use dom::serviceworkerregistration::longest_prefix_match;
use ipc_channel::ipc::{self, IpcSender};
use ipc_channel::router::ROUTER;
use net_traits::{CustomResponseMediator, CoreResourceMsg};
use script_traits::{ServiceWorkerMsg, ScopeThings, SWManagerMsg, SWManagerSenders, DOMMessage};
use servo_url::ServoUrl;
use std::collections::HashMap;
use std::sync::mpsc::{channel, Sender, Receiver, RecvError};
use util::prefs::PREFS;
use util::thread::spawn_named;

enum Message {
    FromResource(CustomResponseMediator),
    FromConstellation(ServiceWorkerMsg)
}

pub struct ServiceWorkerManager {
    // map of registered service worker descriptors
    registered_workers: HashMap<ServoUrl, ScopeThings>,
    // map of active service worker descriptors
    active_workers: HashMap<ServoUrl, Sender<ServiceWorkerScriptMsg>>,
    // own sender to send messages here
    own_sender: IpcSender<ServiceWorkerMsg>,
    // receiver to receive messages from constellation
    own_port: Receiver<ServiceWorkerMsg>,
    // to receive resource messages
    resource_receiver: Receiver<CustomResponseMediator>
}

impl ServiceWorkerManager {
    fn new(own_sender: IpcSender<ServiceWorkerMsg>,
           from_constellation_receiver: Receiver<ServiceWorkerMsg>,
           resource_port: Receiver<CustomResponseMediator>) -> ServiceWorkerManager {
        ServiceWorkerManager {
            registered_workers: HashMap::new(),
            active_workers: HashMap::new(),
            own_sender: own_sender,
            own_port: from_constellation_receiver,
            resource_receiver: resource_port
        }
    }

    pub fn spawn_manager(sw_senders: SWManagerSenders) {
        let (own_sender, from_constellation_receiver) = ipc::channel().unwrap();
        let (resource_chan, resource_port) = ipc::channel().unwrap();
        let from_constellation = ROUTER.route_ipc_receiver_to_new_mpsc_receiver(from_constellation_receiver);
        let resource_port = ROUTER.route_ipc_receiver_to_new_mpsc_receiver(resource_port);
        let _ = sw_senders.resource_sender.send(CoreResourceMsg::NetworkMediator(resource_chan));
        let _ = sw_senders.swmanager_sender.send(SWManagerMsg::OwnSender(own_sender.clone()));
        spawn_named("ServiceWorkerManager".to_owned(), move || {
            ServiceWorkerManager::new(own_sender,
                                      from_constellation,
                                      resource_port).handle_message();
        });
    }

    pub fn get_matching_scope(&self, load_url: &ServoUrl) -> Option<ServoUrl> {
        for scope in self.registered_workers.keys() {
            if longest_prefix_match(&scope, load_url) {
                return Some(scope.clone());
            }
        }
        None
    }

    pub fn wakeup_serviceworker(&mut self, scope_url: ServoUrl) -> Option<Sender<ServiceWorkerScriptMsg>> {
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
                let _ = chan.send(ScriptToDevtoolsControlMsg::NewGlobal((scope_things.init.pipeline_id,
                                                                         Some(scope_things.worker_id)),
                                                                         devtools_sender,
                                                                         page_info));
            };
            ServiceWorkerGlobalScope::run_serviceworker_scope(scope_things.clone(),
                                                              sender.clone(),
                                                              receiver,
                                                              devtools_receiver,
                                                              self.own_sender.clone(),
                                                              scope_url.clone());
            // We store the activated worker
            self.active_workers.insert(scope_url, sender.clone());
            return Some(sender);
        } else {
            warn!("Unable to activate service worker");
            None
        }
    }

    fn handle_message(&mut self) {
        while let Ok(message) = self.receive_message() {
            let should_continue = match message {
                Message::FromConstellation(msg) => {
                    self.handle_message_from_constellation(msg)
                },
                Message::FromResource(msg) => {
                    self.handle_message_from_resource(msg)
                }
            };
            if !should_continue {
                break;
            }
        }
    }

    fn forward_message(&self, msg: DOMMessage, sender: &Sender<ServiceWorkerScriptMsg>) {
        let DOMMessage(data) = msg;
        let data = StructuredCloneData::Vector(data);
        let _ = sender.send(ServiceWorkerScriptMsg::CommonWorker(WorkerScriptMsg::DOMMessage(data)));
    }

    fn handle_message_from_constellation(&mut self, msg: ServiceWorkerMsg) -> bool {
        match msg {
            ServiceWorkerMsg::RegisterServiceWorker(scope_things, scope) => {
                if self.registered_workers.contains_key(&scope) {
                    warn!("ScopeThings for {:?} already stored in SW-Manager", scope);
                } else {
                    self.registered_workers.insert(scope, scope_things);
                }
                true
            }
            ServiceWorkerMsg::Timeout(scope) => {
                if self.active_workers.contains_key(&scope) {
                    let _ = self.active_workers.remove(&scope);
                } else {
                    warn!("ServiceWorker for {:?} is not active", scope);
                }
                true
            },
            ServiceWorkerMsg::ForwardDOMMessage(msg, scope_url) => {
                if self.active_workers.contains_key(&scope_url) {
                    if let Some(ref sender) = self.active_workers.get(&scope_url) {
                        self.forward_message(msg, &sender);
                    }
                } else {
                    if let Some(ref sender) = self.wakeup_serviceworker(scope_url) {
                        self.forward_message(msg, &sender);
                    }
                }
                true
            }
            ServiceWorkerMsg::Exit => false
        }
    }

    fn handle_message_from_resource(&mut self, mediator: CustomResponseMediator) -> bool {
        if serviceworker_enabled() {
            if let Some(scope) = self.get_matching_scope(&mediator.load_url) {
                if self.active_workers.contains_key(&scope) {
                    if let Some(sender) = self.active_workers.get(&scope) {
                        let _ = sender.send(ServiceWorkerScriptMsg::Response(mediator));
                    }
                } else {
                    if let Some(sender) = self.wakeup_serviceworker(scope) {
                        let _ = sender.send(ServiceWorkerScriptMsg::Response(mediator));
                    }
                }
            } else {
                let  _ = mediator.response_chan.send(None);
            }
        } else {
            let _ = mediator.response_chan.send(None);
        }
        true
    }

    #[allow(unsafe_code)]
    fn receive_message(&mut self) -> Result<Message, RecvError> {
        let msg_from_constellation = &self.own_port;
        let msg_from_resource = &self.resource_receiver;
        select! {
            msg = msg_from_constellation.recv() => msg.map(Message::FromConstellation),
            msg = msg_from_resource.recv() => msg.map(Message::FromResource)
        }
    }
}

pub fn serviceworker_enabled() -> bool {
    PREFS.get("dom.serviceworker.enabled").as_boolean().unwrap_or(false)
}
