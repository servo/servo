/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use hyper::header::Location;
use ipc_channel::ipc;
use msg::constellation_msg::PipelineId;
use net_traits::{FetchMetadata, FetchResponseMsg, FilteredMetadata, NetworkError};
use net_traits::request::RequestInit;
use net_traits::response::ResponseInit;
use script_traits::{ConstellationControlMsg, ScriptMsg};

pub struct NetworkListener {
    pub res_init: ResponseInit,
    pub req_init: RequestInit,
    pub pipeline_id: PipelineId,
}

impl NetworkListener {
    pub fn notify(&self, message: Result<FetchMetadata, NetworkError>) {
        let mut result = None;
        let id = self.pipeline_id;
        match message {
            Ok(res_metadata) => {
                let res_data = match res_metadata {
                    FetchMetadata::Filtered
                    {
                       filtered,
                       unsafe_: unsafe_metadata
                    } => Some((filtered, unsafe_metadata)),
                    _ => None,
                };
                match res_data {
                    Some((FilteredMetadata::OpaqueRedirect, _)) => {
                        if let Some((filtered, unsafe_metadata)) = res_data {
                            match unsafe_metadata.headers.clone() {
                                Some(headers) => {
                                    if headers.has::<Location>() {
                                        let (sender, _) = ipc::channel().expect("Failed to create IPC channel!");
                                        result = Some(sender.send(ScriptMsg::InitiateNavigateRequest(
                                                                      self.req_init.clone(),
                                                                      self.res_init.clone(),
                                                                      self.pipeline_id)));
                                    } else {
                                        // when response does not redirect
                                        let (sender, _) = ipc::channel().expect("Failed to create IPC channel!");
                                        result = Some(sender.send(
                                                        ConstellationControlMsg::NavigationResponse(
                                                        id,
                                                        FetchResponseMsg::ProcessResponse(
                                                        Ok(FetchMetadata::Filtered {
                                                        filtered: filtered,
                                                        unsafe_: unsafe_metadata
                                                    })))));
                                    }
                                },
                                None => warn!("No headers received!")
                            };
                        };
                    },
                    _ => {}
                };
            },
            Err(e) => {
                let (sender, _) = ipc::channel().expect("Failed to create IPC channel!");
                result = Some(sender.send(
                            ConstellationControlMsg::NavigationResponse(
                                id,
                            FetchResponseMsg::ProcessResponse(Err(e)))));
            }
        };

        match result {
            Some(Err(e)) => warn!("Failed to send message: {:?}", e),
            _ => {}
        };
    }
}
