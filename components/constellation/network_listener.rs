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
    pub res_init: ResponseInit,
    pub req_init: RequestInit,
    pub pipeline_id: PipelineId,
    pub resource_threads: ResourceThreads,
    pub sender: Sender<(PipelineId, FetchResponseMsg)>,
    pub should_send: bool,
}

impl NetworkListener {
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
        ROUTER.add_route(ipc_receiver.to_opaque(), box move |message| {
            let msg = message.to();
            match msg {
                Ok(FetchResponseMsg::ProcessResponse(res)) => listener.check_redirect(res),
                Ok(msg_) => listener.send(msg_),
                _ => {}
            };
        });

        if let Err(e) = self.resource_threads.sender().send(
                            CoreResourceMsg::FetchRedirect(
                            self.req_init.clone(),
                            self.res_init.clone(),
                            ipc_sender)) {
            warn!("Exit resource thread ({})", e);
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
                    Some(ref headers) => {
                        if let Some(_) = headers.get::<Location>() {
                            self.req_init.url_list.push(metadata.final_url.clone());
                            self.initiate_fetch();
                        } else {
                            // when response does not redirect
                            self.should_send = true;
                            self.send(FetchResponseMsg::ProcessResponse(Ok(res_metadata.clone())));
                        }
                    },
                    None => warn!("No headers received!")
                };
            },
            Err(e) => self.send(FetchResponseMsg::ProcessResponse(Err(e))),
        };
    }

    fn send(&mut self, msg: FetchResponseMsg) {
        if self.should_send {
            match self.sender.send((self.pipeline_id, msg)) {
                Err(e) => warn!("Failed to send message: {:?}", e),
                _ => {}
            }
        }
    }
}
