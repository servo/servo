/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The listener that encapsulates all state for an in-progress document request.
//! Any redirects that are encountered are followed. Whenever a non-redirect
//! response is received, it is forwarded to the appropriate script thread.

use hyper::header::Location;
use ipc_channel::ipc;
use ipc_channel::router::ROUTER;
use msg::constellation_msg::PipelineId;
use net::http_loader::{set_default_accept, set_default_accept_language};
use net_traits::{CoreResourceMsg, FetchMetadata, FetchResponseMsg};
use net_traits::{IpcSend, NetworkError, ResourceThreads};
use net_traits::request::{Destination, RequestInit, Type};
use net_traits::response::ResponseInit;
use std::sync::mpsc::Sender;

pub struct NetworkListener {
    res_init: Option<ResponseInit>,
    req_init: RequestInit,
    pipeline_id: PipelineId,
    resource_threads: ResourceThreads,
    sender: Sender<(PipelineId, FetchResponseMsg)>,
    should_send: bool,
}

impl NetworkListener {
    pub fn new(req_init: RequestInit,
               pipeline_id: PipelineId,
               resource_threads: ResourceThreads,
               sender: Sender<(PipelineId, FetchResponseMsg)>) -> NetworkListener {
        NetworkListener {
            res_init: None,
            req_init,
            pipeline_id,
            resource_threads,
            sender,
            should_send: false
        }
    }

    pub fn initiate_fetch(&self) {
        let (ipc_sender, ipc_receiver) = ipc::channel().expect("Failed to create IPC channel!");

        let mut listener = NetworkListener {
            res_init: self.res_init.clone(),
            req_init: self.req_init.clone(),
            resource_threads: self.resource_threads.clone(),
            sender: self.sender.clone(),
            pipeline_id: self.pipeline_id.clone(),
            should_send: false,
        };

        let msg = match self.res_init {
            Some(ref res_init_) => CoreResourceMsg::FetchRedirect(
                                   self.req_init.clone(),
                                   res_init_.clone(),
                                   ipc_sender),
            None => {
                set_default_accept(Type::None, Destination::Document, &mut listener.req_init.headers);
                set_default_accept_language(&mut listener.req_init.headers);

                CoreResourceMsg::Fetch(
                listener.req_init.clone(),
                ipc_sender)
            }
        };

        ROUTER.add_route(ipc_receiver.to_opaque(), box move |message| {
            let msg = message.to();
            match msg {
                Ok(FetchResponseMsg::ProcessResponse(res)) => listener.check_redirect(res),
                Ok(msg_) => listener.send(msg_),
                Err(e) => warn!("Error while receiving network listener message: {}", e),
            };
        });

        if let Err(e) = self.resource_threads.sender().send(msg) {
            warn!("Resource thread unavailable ({})", e);
        }
    }

    fn check_redirect(&mut self,
                      message: Result<(FetchMetadata), NetworkError>) {
        match message {
            Ok(res_metadata) => {
                let metadata = match res_metadata {
                    FetchMetadata::Filtered { ref unsafe_, .. } => unsafe_,
                    FetchMetadata::Unfiltered(ref m) => m,
                };

                match metadata.headers {
                    Some(ref headers) if headers.has::<Location>() => {
                        if self.req_init.url_list.is_empty() {
                            self.req_init.url_list.push(self.req_init.url.clone());
                        }
                        self.req_init.url_list.push(metadata.final_url.clone());

                        self.req_init.referrer_url = metadata.referrer.clone();
                        self.req_init.referrer_policy = metadata.referrer_policy;

                        self.res_init = Some(ResponseInit {
                            url: metadata.final_url.clone(),
                            headers: headers.clone().into_inner(),
                            referrer: metadata.referrer.clone(),
                        });

                        self.initiate_fetch();
                    },
                    _ => {
                        // Response should be processed by script thread.
                        self.should_send = true;
                        self.send(FetchResponseMsg::ProcessResponse(Ok(res_metadata.clone())));
                    }
                };
            },
            Err(e) => {
                self.should_send = true;
                self.send(FetchResponseMsg::ProcessResponse(Err(e)))
            }
        };
    }

    fn send(&mut self, msg: FetchResponseMsg) {
        if self.should_send {
            if let Err(e) = self.sender.send((self.pipeline_id, msg)) {
                warn!("Failed to forward network message to pipeline {}: {:?}", self.pipeline_id, e);
            }
        }
    }
}
