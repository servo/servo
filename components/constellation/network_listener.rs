/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! The listener that encapsulates all state for an in-progress document request.
//! Any redirects that are encountered are followed. Whenever a non-redirect
//! response is received, it is forwarded to the appropriate script thread.

use base::id::PipelineId;
use crossbeam_channel::Sender;
use http::HeaderMap;
use ipc_channel::ipc;
use ipc_channel::router::ROUTER;
use log::warn;
use net::http_loader::{set_default_accept, set_default_accept_language};
use net_traits::request::{Destination, Referrer, RequestBuilder};
use net_traits::response::ResponseInit;
use net_traits::{
    CoreResourceMsg, FetchChannels, FetchMetadata, FetchResponseMsg, IpcSend, NetworkError,
    ResourceThreads,
};

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

    pub fn initiate_fetch(&self, cancel_chan: Option<ipc::IpcReceiver<()>>) {
        let (ipc_sender, ipc_receiver) = ipc::channel().expect("Failed to create IPC channel!");

        let mut listener = NetworkListener {
            res_init: self.res_init.clone(),
            request_builder: self.request_builder.clone(),
            resource_threads: self.resource_threads.clone(),
            sender: self.sender.clone(),
            pipeline_id: self.pipeline_id,
            should_send: false,
        };

        let msg = match self.res_init {
            Some(ref res_init_) => CoreResourceMsg::FetchRedirect(
                self.request_builder.clone(),
                res_init_.clone(),
                ipc_sender,
                None,
            ),
            None => {
                set_default_accept(Destination::Document, &mut listener.request_builder.headers);
                set_default_accept_language(&mut listener.request_builder.headers);

                CoreResourceMsg::Fetch(
                    listener.request_builder.clone(),
                    FetchChannels::ResponseMsg(ipc_sender, cancel_chan),
                )
            },
        };

        ROUTER.add_route(
            ipc_receiver.to_opaque(),
            Box::new(move |message| {
                let msg = message.to();
                match msg {
                    Ok(FetchResponseMsg::ProcessResponse(res)) => listener.check_redirect(res),
                    Ok(msg_) => listener.send(msg_),
                    Err(e) => warn!("Error while receiving network listener message: {}", e),
                };
            }),
        );

        if let Err(e) = self.resource_threads.sender().send(msg) {
            warn!("Resource thread unavailable ({})", e);
        }
    }

    fn check_redirect(&mut self, message: Result<FetchMetadata, NetworkError>) {
        match message {
            Ok(res_metadata) => {
                let metadata = match res_metadata {
                    FetchMetadata::Filtered { ref unsafe_, .. } => unsafe_,
                    FetchMetadata::Unfiltered(ref m) => m,
                };

                match metadata.location_url {
                    // https://html.spec.whatwg.org/multipage/#process-a-navigate-fetch
                    // Step 7-4.
                    Some(Ok(ref location_url))
                        if matches!(location_url.scheme(), "http" | "https") =>
                    {
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
                            .map(Referrer::ReferrerUrl)
                            .unwrap_or(Referrer::NoReferrer);
                        self.request_builder.referrer_policy = metadata.referrer_policy;

                        let headers = if let Some(ref headers) = metadata.headers {
                            headers.clone().into_inner()
                        } else {
                            HeaderMap::new()
                        };

                        self.res_init = Some(ResponseInit {
                            url: metadata.final_url.clone(),
                            location_url: metadata.location_url.clone(),
                            headers,
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
                    "Failed to forward network message to pipeline {:?}: {:?}",
                    self.pipeline_id, e
                );
            }
        }
    }
}
