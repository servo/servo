/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! The listener that encapsulates all state for an in-progress document request.
//! Any redirects that are encountered are followed. Whenever a non-redirect
//! response is received, it is forwarded to the appropriate script thread.

use bincode;
use crossbeam_channel::Sender;
use http::header::LOCATION;
use ipc_channel::ipc;
use msg::constellation_msg::PipelineId;
use net::http_loader::{set_default_accept, set_default_accept_language};
use net_traits::request::{Destination, Referrer, RequestBuilder};
use net_traits::response::ResponseInit;
use net_traits::{CoreResourceMsg, FetchChannels, FetchMetadata, FetchResponseMsg};
use net_traits::{IpcSend, NetworkError, ResourceThreads};
use shared_ipc_router::SharedIpcRouter;

pub struct NetworkListener {
    res_init: Option<ResponseInit>,
    request_builder: RequestBuilder,
    pipeline_id: PipelineId,
    resource_threads: ResourceThreads,
    sender: Sender<(PipelineId, FetchResponseMsg)>,
    should_send: bool,
}

impl NetworkListener {
    pub fn new(
        request_builder: RequestBuilder,
        pipeline_id: PipelineId,
        resource_threads: ResourceThreads,
        sender: Sender<(PipelineId, FetchResponseMsg)>,
    ) -> NetworkListener {
        NetworkListener {
            res_init: None,
            request_builder,
            pipeline_id,
            resource_threads,
            sender,
            should_send: false,
        }
    }

    pub fn initiate_fetch(&self, cancel_chan: Option<ipc::IpcSharedMemory>) {
        // Note: this doesn't actually create a new router each time,
        // although it does clone the handle to the router.
        // It would be better to store it somewhere, and that is made harder
        // by initiate_fetch being called itself from inside a the message handler.
        let ipc_router = SharedIpcRouter::new();
        if ipc_router.requires_setup_for_resource_manager() {
            let _ = self
                .resource_threads
                .sender()
                .send(CoreResourceMsg::NewDispatcher(
                    ipc_router.router_id.clone(),
                    ipc_router.ipc_sender.clone(),
                ));
        }

        let mut listener = NetworkListener {
            res_init: self.res_init.clone(),
            request_builder: self.request_builder.clone(),
            resource_threads: self.resource_threads.clone(),
            sender: self.sender.clone(),
            pipeline_id: self.pipeline_id.clone(),
            should_send: false,
        };

        if self.res_init.is_none() {
            set_default_accept(Destination::Document, &mut listener.request_builder.headers);
            set_default_accept_language(&mut listener.request_builder.headers);
        }

        let request_builder = listener.request_builder.clone();

        let handle = ipc_router.add_callback(Box::new(move |data: Vec<u8>| {
            let response: Result<FetchResponseMsg, bincode::Error> = bincode::deserialize(&data[..]);
            match response {
                Ok(FetchResponseMsg::ProcessResponse(res)) => listener.check_redirect(res),
                Ok(msg_) => listener.send(msg_),
                Err(e) => warn!("Error while receiving network listener message: {}", e),
            };
        }));

        let msg = match self.res_init {
            Some(ref res_init_) => CoreResourceMsg::FetchRedirect(
                self.request_builder.clone(),
                res_init_.clone(),
                handle,
                None,
            ),
            None => {
                CoreResourceMsg::Fetch(
                    request_builder,
                    FetchChannels::ResponseHandle(handle, cancel_chan),
                )
            },
        };

        if let Err(e) = self.resource_threads.sender().send(msg) {
            warn!("Resource thread unavailable ({})", e);
        }
    }

    fn check_redirect(&mut self, message: Result<(FetchMetadata), NetworkError>) {
        match message {
            Ok(res_metadata) => {
                let metadata = match res_metadata {
                    FetchMetadata::Filtered { ref unsafe_, .. } => unsafe_,
                    FetchMetadata::Unfiltered(ref m) => m,
                };

                match metadata.headers {
                    Some(ref headers) if headers.contains_key(LOCATION) => {
                        if self.request_builder.url_list.is_empty() {
                            self.request_builder
                                .url_list
                                .push(self.request_builder.url.clone());
                        }
                        self.request_builder
                            .url_list
                            .push(metadata.final_url.clone());

                        self.request_builder.referrer = metadata
                            .referrer
                            .clone()
                            .map(|referrer_url| Referrer::ReferrerUrl(referrer_url));
                        self.request_builder.referrer_policy = metadata.referrer_policy;

                        self.res_init = Some(ResponseInit {
                            url: metadata.final_url.clone(),
                            location_url: metadata.location_url.clone(),
                            headers: headers.clone().into_inner(),
                            referrer: metadata.referrer.clone(),
                            status_code: metadata
                                .status
                                .as_ref()
                                .map(|&(code, _)| code)
                                .unwrap_or(200),
                        });

                        // XXXManishearth we don't have the cancel_chan anymore and
                        // can't use it here.
                        //
                        // Ideally the Fetch code would handle manual redirects on its own
                        self.initiate_fetch(None);
                    },
                    _ => {
                        // Response should be processed by script thread.
                        self.should_send = true;
                        self.send(FetchResponseMsg::ProcessResponse(Ok(res_metadata)));
                    },
                };
            },
            Err(e) => {
                self.should_send = true;
                self.send(FetchResponseMsg::ProcessResponse(Err(e)))
            },
        };
    }

    fn send(&mut self, msg: FetchResponseMsg) {
        if self.should_send {
            if let Err(e) = self.sender.send((self.pipeline_id, msg)) {
                warn!(
                    "Failed to forward network message to pipeline {}: {:?}",
                    self.pipeline_id, e
                );
            }
        }
    }
}
