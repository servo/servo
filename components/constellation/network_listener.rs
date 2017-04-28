/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use hyper::header::Location;
use ipc_channel::ipc;
use ipc_channel::router::ROUTER;
use msg::constellation_msg::PipelineId;
use net_traits::{CoreResourceMsg, FetchMetadata, FetchResponseMsg};
use net_traits::{IpcSend, NetworkError, ResourceThreads};
use net_traits::request::RequestInit;
use net_traits::response::ResponseInit;
use std::sync::mpsc::Sender;

pub struct NetworkListener {
    pub res_init: Option<ResponseInit>,
    pub req_init: RequestInit,
    pub pipeline_id: PipelineId,
    pub resource_threads: ResourceThreads,
    pub sender: Sender<(PipelineId, FetchResponseMsg)>,
    pub should_send: bool,
}

impl NetworkListener {
    pub fn initiate_fetch(&self) {
        let (ipc_sender, ipc_receiver) = ipc::channel().expect("Failed to create IPC channel!");

        let mut req_init = self.req_init.clone();
        if req_init.url_list.is_empty() {
            req_init.url_list.push(req_init.url.clone());
        }

        let mut listener = NetworkListener {
            res_init: self.res_init.clone(),
            req_init: req_init.clone(),
            resource_threads: self.resource_threads.clone(),
            sender: self.sender.clone(),
            pipeline_id: self.pipeline_id.clone(),
            should_send: false,
        };

        ROUTER.add_route(ipc_receiver.to_opaque(), box move |message| {
            let msg = message.to();
            match msg {
                Ok(FetchResponseMsg::ProcessResponse(res)) => listener.check_redirect(res),
                Ok(msg_) => listener.send(msg_),
                _ => {}
            };
        });

        let msg = match self.res_init {
            Some(ref res_init_) => CoreResourceMsg::FetchRedirect(
                                   req_init.clone(),
                                   res_init_.clone(),
                                   ipc_sender),
            None => CoreResourceMsg::Fetch(
                    req_init,
                    ipc_sender)
        };

        if let Err(e) = self.resource_threads.sender().send(msg) {
            warn!("Exit resource thread ({})", e);
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
                        if self.res_init.is_some() {
                            self.req_init.url_list.push(metadata.final_url.clone());
                        }
                        self.req_init.referrer_url = Some(self.req_init.url.clone());
                        // metadata.referrer.clone().map(|url| self.req_init.referrer_url = Some(url));

                        self.res_init = Some(ResponseInit {
                            url: metadata.final_url.clone(),
                            headers: headers.clone().into_inner(),
                            referrer: metadata.referrer.clone(),
                        });

                        self.initiate_fetch();
                    },
                    _ => {
                        // when response does not redirect
                        self.should_send = true;
                        self.send(FetchResponseMsg::ProcessResponse(Ok(res_metadata.clone())));
                    }
                };
            },
            Err(e) => self.send(FetchResponseMsg::ProcessResponse(Err(e))),
        };
    }

    fn send(&mut self, msg: FetchResponseMsg) {
        if self.should_send {
            if let Err(e) = self.sender.send((self.pipeline_id, msg)) {
                warn!("Failed to forward network message: {:?}", e);
            }
        }
    }
}
