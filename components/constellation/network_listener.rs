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

pub enum NetworkListenerMsg {
    ResponseMetadata(PipelineId,
                     (Result<FetchMetadata, NetworkError>,
                      Vec<Vec<u8>>,
                      Option<Result<(), NetworkError>>))
}

pub struct NetworkListener {
    pub res_init: ResponseInit,
    pub req_init: RequestInit,
    pub pipeline_id: PipelineId,
    pub resource_threads: ResourceThreads,
    pub sender: Sender<NetworkListenerMsg>,
    /// chunks which constitute the response body
    pub chunks: Vec<Vec<u8>>,
    pub metadata: Option<FetchMetadata>,
    pub eof: Option<Result<(), NetworkError>>,
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
            chunks: vec![],
            metadata: None,
            eof: None,
        };
        ROUTER.add_route(ipc_receiver.to_opaque(), box move |message| {
            let msg = message.to();
            match msg {
                Ok(FetchResponseMsg::ProcessResponse(res)) => listener.check_redirect(res),
                Ok(FetchResponseMsg::ProcessResponseChunk(chunk)) => listener.chunks.push(chunk),
                Ok(FetchResponseMsg::ProcessResponseEOF(res_eof)) => {
                    listener.eof = Some(res_eof);
                    listener.send();
                },
                _ => {}
            };
        });

        if let Err(e) = self.resource_threads.sender().send(
                            CoreResourceMsg::FetchRedirect(
                                self.req_init.clone(),
                                self.res_init.clone(),
                                ipc_sender)) {
            warn!("Exit resource thread ({})", e);
        };
    }

    fn check_redirect(&mut self, message: Result<(FetchMetadata), NetworkError>) {
        match message {
            Ok(res_metadata) => {
                let metadata = match res_metadata {
                    FetchMetadata::Filtered { ref unsafe_, .. } => unsafe_,
                    FetchMetadata::Unfiltered(ref m) => m,
                };

                match metadata.headers.clone() {
                    Some(headers) => {
                        if headers.has::<Location>() {
                            match headers.get::<Location>() {
                                Some(&Location(ref loc)) => self.req_init.url_list.push(loc.clone()),
                                None => unreachable!()
                            };

                            self.initiate_fetch();
                        } else {
                            // when response does not redirect
                            self.metadata = Some(res_metadata.clone());
                            // attempt to send
                            self.send();
                        }
                    },
                    None => warn!("No headers received!")
                };
            },
            Err(net_error) => {
                if let Err(e) = self.sender.send(
                    NetworkListenerMsg::ResponseMetadata(
                        self.pipeline_id.clone(),
                        (Err(net_error),
                         self.chunks.clone(),
                         self.eof.clone())
                    )) {
                    warn!("Failed to send message: {:?}", e);
                }
            }
        };
    }

    fn send(&mut self) {
        if let (Some(metadata), Some(_)) = (self.metadata.clone(), self.eof.clone()) {
            let msg = NetworkListenerMsg::ResponseMetadata(
                          self.pipeline_id,
                          (Ok((metadata).clone()),
                           self.chunks.clone(),
                           self.eof.clone())
                      );

            match self.sender.send(msg) {
                Err(e) => warn!("Failed to send message: {:?}", e),
                _ => {}
            }
        }
    }
}
