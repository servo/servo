/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use content_security_policy::Destination;
use embedder_traits::{
    GenericEmbedderProxy, WebResourceLoadCompleted, WebResourceLoadError, WebResourceLoadId,
    WebResourceRequest, WebResourceResponse, WebResourceResponseMsg, WebResourceResponseReceived,
};
use log::error;
use net_traits::NetworkError;
use net_traits::http_status::HttpStatus;
use net_traits::request::Request;
use net_traits::response::{Response, ResponseBody};

use crate::embedder::NetToEmbedderMsg;
use crate::fetch::methods::FetchContext;

#[derive(Clone)]
pub struct RequestInterceptor {
    embedder_proxy: GenericEmbedderProxy<NetToEmbedderMsg>,
}

impl RequestInterceptor {
    pub fn new(embedder_proxy: GenericEmbedderProxy<NetToEmbedderMsg>) -> RequestInterceptor {
        RequestInterceptor { embedder_proxy }
    }

    pub async fn intercept_request(
        &self,
        request: &mut Request,
        response: &mut Option<Response>,
        context: &FetchContext,
    ) {
        let (sender, mut receiver) = tokio::sync::mpsc::unbounded_channel();
        let web_resource_request = web_resource_request(request);

        self.embedder_proxy
            .send(NetToEmbedderMsg::WebResourceRequested(
                request.target_webview_id,
                web_resource_request,
                sender,
            ));

        // TODO: use done_chan and run in CoreResourceThreadPool.
        let mut accumulated_body = Vec::new();
        while let Some(message) = receiver.recv().await {
            match message {
                WebResourceResponseMsg::Start(webresource_response) => {
                    let timing = context.timing.inner().clone();
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
                    *response.body.lock() =
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

    pub fn notify_response_received(&self, request: &Request, response: &Response) {
        if response.is_network_error() {
            return;
        }
        let actual = response.actual_response();
        let Some(url) = actual.url().map(|url| url.as_url().clone()) else {
            return;
        };
        let Some(status_code) = actual.status.try_code() else {
            return;
        };
        let observed = WebResourceResponseReceived {
            load_id: web_resource_load_id(request),
            request: web_resource_request(request),
            response: WebResourceResponse::new(url)
                .headers(actual.headers.clone())
                .status_code(status_code)
                .status_message(actual.status.message().to_vec()),
        };
        self.embedder_proxy
            .send(NetToEmbedderMsg::WebResourceResponseReceived(
                request.target_webview_id,
                Box::new(observed),
            ));
    }

    pub fn notify_load_completed(&self, request: &Request, response: &Response) {
        let completed = WebResourceLoadCompleted {
            load_id: web_resource_load_id(request),
            request: web_resource_request(request),
            error: response
                .get_network_error()
                .map(|error| WebResourceLoadError {
                    message: format!("{error:?}"),
                    is_cancelled: matches!(error, NetworkError::LoadCancelled),
                }),
        };
        self.embedder_proxy
            .send(NetToEmbedderMsg::WebResourceLoadCompleted(
                request.target_webview_id,
                Box::new(completed),
            ));
    }
}

fn web_resource_request(request: &Request) -> WebResourceRequest {
    WebResourceRequest {
        method: request.method.clone(),
        url: request.url().into_url(),
        headers: request.headers.clone(),
        destination: request.destination,
        referrer_url: request.referrer.to_url().map(|url| url.as_url().clone()),
        is_for_main_frame: matches!(request.destination, Destination::Document),
        is_redirect: request.redirect_count > 0,
    }
}

fn web_resource_load_id(request: &Request) -> WebResourceLoadId {
    WebResourceLoadId {
        request_id: request.id.0,
        redirect_index: request.redirect_count,
    }
}

#[cfg(test)]
mod tests {
    use crossbeam_channel::{Receiver, unbounded};
    use embedder_traits::{EventLoopWaker, GenericEmbedderProxy};
    use http::{HeaderValue, StatusCode};
    use net_traits::blob_url_store::UrlWithBlobClaim;
    use net_traits::request::{Referrer, RequestBuilder};
    use net_traits::{NetworkError, ResourceFetchTiming, ResourceTimingType};
    use servo_url::ServoUrl;

    use super::*;

    #[derive(Clone)]
    struct TestEventLoopWaker;

    impl EventLoopWaker for TestEventLoopWaker {
        fn wake(&self) {}

        fn clone_box(&self) -> Box<dyn EventLoopWaker> {
            Box::new(self.clone())
        }
    }

    fn interceptor() -> (RequestInterceptor, Receiver<NetToEmbedderMsg>) {
        let (sender, receiver) = unbounded();
        let embedder_proxy = GenericEmbedderProxy {
            sender,
            event_loop_waker: Box::new(TestEventLoopWaker),
        };
        (RequestInterceptor::new(embedder_proxy), receiver)
    }

    fn request(url: &ServoUrl) -> Request {
        RequestBuilder::new(
            None,
            UrlWithBlobClaim::new(url.clone(), None),
            Referrer::NoReferrer,
        )
        .build()
    }

    #[test]
    fn response_notification_preserves_request_identity_and_metadata() {
        let (interceptor, receiver) = interceptor();
        let url = ServoUrl::parse("https://example.com/download").unwrap();
        let mut request = request(&url);
        request.redirect_count = 2;
        let mut response = Response::new(
            url.clone(),
            ResourceFetchTiming::new(ResourceTimingType::Resource),
        );
        response.status = HttpStatus::new(StatusCode::OK, b"OK".to_vec());
        response.headers.insert(
            "content-type",
            HeaderValue::from_static("application/octet-stream"),
        );

        interceptor.notify_response_received(&request, &response);

        let NetToEmbedderMsg::WebResourceResponseReceived(target_webview_id, observed) =
            receiver.try_recv().unwrap()
        else {
            panic!("expected a web resource response notification");
        };
        assert_eq!(target_webview_id, request.target_webview_id);
        assert_eq!(observed.load_id.request_id, request.id.0);
        assert_eq!(observed.load_id.redirect_index, 2);
        assert_eq!(observed.response.url, url.into_url());
        assert_eq!(observed.response.status_code, StatusCode::OK);
        assert_eq!(
            observed.response.headers.get("content-type"),
            Some(&HeaderValue::from_static("application/octet-stream"))
        );
        assert!(receiver.try_recv().is_err());
    }

    #[test]
    fn network_errors_do_not_emit_response_notifications() {
        let (interceptor, receiver) = interceptor();
        let url = ServoUrl::parse("https://example.com/failure").unwrap();
        let request = request(&url);
        let response = Response::network_error(NetworkError::ResourceLoadError(
            "expected test failure".to_owned(),
        ));

        interceptor.notify_response_received(&request, &response);

        assert!(receiver.try_recv().is_err());
    }

    #[test]
    fn completion_notification_reports_success_and_failure() {
        let (interceptor, receiver) = interceptor();
        let url = ServoUrl::parse("https://example.com/resource").unwrap();
        let request = request(&url);
        let response = Response::new(url, ResourceFetchTiming::new(ResourceTimingType::Resource));

        interceptor.notify_load_completed(&request, &response);

        let NetToEmbedderMsg::WebResourceLoadCompleted(_, completed) = receiver.try_recv().unwrap()
        else {
            panic!("expected a web resource completion notification");
        };
        assert_eq!(completed.load_id.request_id, request.id.0);
        assert!(completed.error.is_none());

        let response = Response::network_error(NetworkError::LoadCancelled);
        interceptor.notify_load_completed(&request, &response);
        let NetToEmbedderMsg::WebResourceLoadCompleted(_, completed) = receiver.try_recv().unwrap()
        else {
            panic!("expected a web resource failure notification");
        };
        let error = completed.error.expect("expected a load error");
        assert!(error.is_cancelled);
        assert_eq!(error.message, "Load cancelled");
    }
}
