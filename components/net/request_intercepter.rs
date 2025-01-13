use content_security_policy::Destination;
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use embedder_traits::{
    EmbedderMsg, EmbedderProxy, HttpBodyData, WebResourceRequest, WebResourceResponseMsg,
};
use ipc_channel::ipc;
use net_traits::{
    request::Request,
    response::{Response, ResponseBody},
    NetworkError,
};

use crate::fetch::methods::FetchContext;

#[derive(Clone)]
pub struct RequestIntercepter {
    embedder_proxy: EmbedderProxy,
    is_redirect: bool,
}

impl RequestIntercepter {
    pub fn new(embedder_proxy: EmbedderProxy) -> RequestIntercepter {
        RequestIntercepter {
            embedder_proxy,
            is_redirect: false,
        }
    }

    pub fn set_redirect(&mut self, redirect: bool) {
        self.is_redirect = redirect;
    }

    pub fn intercept_request(
        &self,
        request: &mut Request,
        response: &mut Option<Response>,
        context: &FetchContext,
    ) {
        let (tx, rx) = ipc::channel().unwrap();
        let is_iframe = matches!(request.destination, Destination::IFrame);
        let req = WebResourceRequest::new(request.url())
            .method(request.method.clone())
            .headers(request.headers.clone())
            .is_redirect(self.is_redirect)
            .is_main_frame(!is_iframe);

        self.embedder_proxy.send((
            request.target_browsing_context_id,
            EmbedderMsg::WebResourceRequested(req, tx),
        ));
        let mut response_received = false;

        // TODO: use done_chan and run in CoreResourceThreadPool.
        while let Ok(msg) = rx.recv() {
            match msg {
                WebResourceResponseMsg::Start(webresource_response) => {
                    response_received = true;
                    let timing = context.timing.lock().unwrap().clone();
                    let res = Response::new(webresource_response.url.clone(), timing);
                    *res.body.lock().unwrap() = ResponseBody::Receiving(Vec::new());
                    *response = Some(res);
                },
                WebResourceResponseMsg::Body(data) => {
                    if !response_received {
                        panic!("Receive body before initializing a Response!");
                    }
                    if let Some(ref mut res) = *response {
                        match data {
                            HttpBodyData::Chunk(chunk) => {
                                if let ResponseBody::Receiving(ref mut body) =
                                    *res.body.lock().unwrap()
                                {
                                    body.extend_from_slice(&chunk);
                                } else {
                                    panic!("Receive Playload Message when Response body is not in Receiving state!");
                                }
                            },
                            HttpBodyData::Done => {
                                let mut res_body = res.body.lock().unwrap();
                                if let ResponseBody::Receiving(ref mut body) = *res_body {
                                    let completed_body = std::mem::take(body);
                                    *res_body = ResponseBody::Done(completed_body);
                                } else {
                                    panic!("Receive Done Message when Response body is not in Receiving state!");
                                }
                                break;
                            },
                            HttpBodyData::Cancelled => {
                                *response =
                                    Some(Response::network_error(NetworkError::LoadCancelled));
                                break;
                            },
                        }
                    }
                },
                WebResourceResponseMsg::None => {
                    // Will not intercept the response. Continue.
                    break;
                },
            }
        }
    }
}
