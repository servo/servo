/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use content_security_policy::Destination;
use embedder_traits::{EmbedderMsg, EmbedderProxy, WebResourceRequest, WebResourceResponseMsg};
use ipc_channel::ipc;
use log::error;
use net_traits::http_status::HttpStatus;
use net_traits::request::Request;
use net_traits::response::{Response, ResponseBody};
use net_traits::NetworkError;

use crate::fetch::methods::FetchContext;

#[derive(Clone)]
pub struct RequestInterceptor {
    embedder_proxy: EmbedderProxy,
}

impl RequestInterceptor {
    pub fn new(embedder_proxy: EmbedderProxy) -> RequestInterceptor {
        RequestInterceptor { embedder_proxy }
    }

    pub fn intercept_request(
        &self,
        request: &mut Request,
        response: &mut Option<Response>,
        context: &FetchContext,
    ) {
        let (sender, receiver) = ipc::channel().unwrap();
        let is_for_main_frame = matches!(request.destination, Destination::Document);
        let web_resource_request = WebResourceRequest {
            method: request.method.clone(),
            url: request.url().into_url(),
            headers: request.headers.clone(),
            is_for_main_frame,
            is_redirect: request.redirect_count > 0,
        };

        self.embedder_proxy.send(EmbedderMsg::WebResourceRequested(
            request.target_webview_id,
            web_resource_request,
            sender,
        ));

        // TODO: use done_chan and run in CoreResourceThreadPool.
        let mut accumulated_body = Vec::new();
        while let Ok(message) = receiver.recv() {
            match message {
                WebResourceResponseMsg::Start(webresource_response) => {
                    let timing = context.timing.lock().unwrap().clone();
                    let mut response_override =
                        Response::new(webresource_response.url.into(), timing);
                    response_override.headers = webresource_response.headers;
                    response_override.status = HttpStatus::new(
                        webresource_response.status_code,
                        webresource_response.status_message,
                    );
                    *response = Some(response_override);
                },
                WebResourceResponseMsg::SendBodyData(data) => {
                    accumulated_body.push(data);
                },
                WebResourceResponseMsg::FinishLoad => {
                    if accumulated_body.is_empty() {
                        break;
                    }
                    let Some(response) = response.as_mut() else {
                        error!("Received unexpected FinishLoad message");
                        break;
                    };
                    *response.body.lock().unwrap() =
                        ResponseBody::Done(accumulated_body.into_iter().flatten().collect());
                    break;
                },
                WebResourceResponseMsg::CancelLoad => {
                    *response = Some(Response::network_error(NetworkError::LoadCancelled));
                    break;
                },
                WebResourceResponseMsg::DoNotIntercept => break,
            }
        }
    }
}
