/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crossbeam_channel::Sender;
use devtools_traits::{
    ChromeToDevtoolsControlMsg, DevtoolsControlMsg, HttpRequest as DevtoolsHttpRequest,
    HttpResponse as DevtoolsHttpResponse, NetworkEvent, SecurityInfoUpdate,
};
use http::{HeaderMap, Method};
use hyper_serde::Serde;
use log::error;
use net_traits::http_status::HttpStatus;
use net_traits::request::{Destination, Request};
use net_traits::response::{CacheState, Response};
use net_traits::{DebugVec, FetchMetadata};
use servo_base::id::{BrowsingContextId, PipelineId};
use servo_url::ServoUrl;

use crate::fetch::methods::FetchContext;

#[allow(clippy::too_many_arguments)]
pub(crate) fn prepare_devtools_request(
    request_id: String,
    url: ServoUrl,
    method: Method,
    headers: HeaderMap,
    body: Option<Vec<u8>>,
    pipeline_id: PipelineId,
    connect_time: Duration,
    send_time: Duration,
    destination: Destination,
    is_xhr: bool,
    browsing_context_id: BrowsingContextId,
) -> ChromeToDevtoolsControlMsg {
    let started_date_time = SystemTime::now();
    let request = DevtoolsHttpRequest {
        url,
        method,
        headers,
        body: body.map(DebugVec::from),
        pipeline_id,
        started_date_time,
        time_stamp: started_date_time
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64,
        connect_time,
        send_time,
        destination,
        is_xhr,
        browsing_context_id,
    };
    let net_event = NetworkEvent::HttpRequestUpdate(request);

    ChromeToDevtoolsControlMsg::NetworkEvent(request_id, net_event)
}

pub(crate) fn send_request_to_devtools(
    msg: ChromeToDevtoolsControlMsg,
    devtools_chan: &Sender<DevtoolsControlMsg>,
) {
    if matches!(msg, ChromeToDevtoolsControlMsg::NetworkEvent(_, ref network_event) if !network_event.forward_to_devtools())
    {
        return;
    }
    if let Err(e) = devtools_chan.send(DevtoolsControlMsg::FromChrome(msg)) {
        error!("DevTools send failed: {e}");
    }
}

pub(crate) fn send_response_to_devtools(
    request: &Request,
    context: &FetchContext,
    response: &Response,
    body_data: Option<Vec<u8>>,
) {
    let meta = match response.metadata() {
        Ok(FetchMetadata::Unfiltered(m)) => m,
        Ok(FetchMetadata::Filtered { unsafe_, .. }) => unsafe_,
        Err(_) => {
            log::warn!("No metadata available, skipping devtools response.");
            return;
        },
    };
    send_response_values_to_devtools(
        meta.headers.map(Serde::into_inner),
        meta.status,
        body_data,
        response.cache_state,
        request,
        context.devtools_chan.clone(),
    );
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn send_response_values_to_devtools(
    headers: Option<HeaderMap>,
    status: HttpStatus,
    body: Option<Vec<u8>>,
    cache_state: CacheState,
    request: &Request,
    devtools_chan: Option<Sender<DevtoolsControlMsg>>,
) {
    if let (Some(devtools_chan), Some(pipeline_id), Some(webview_id)) = (
        devtools_chan,
        request.pipeline_id,
        request.target_webview_id,
    ) {
        let browsing_context_id = webview_id.into();
        let from_cache = matches!(cache_state, CacheState::Local | CacheState::Validated);

        let devtoolsresponse = DevtoolsHttpResponse {
            headers,
            status,
            body: body.map(DebugVec::from),
            from_cache,
            pipeline_id,
            browsing_context_id,
        };
        let net_event_response = NetworkEvent::HttpResponse(devtoolsresponse);

        let msg =
            ChromeToDevtoolsControlMsg::NetworkEvent(request.id.0.to_string(), net_event_response);

        let _ = devtools_chan.send(DevtoolsControlMsg::FromChrome(msg));
    }
}

pub(crate) fn send_security_info_to_devtools(
    request: &Request,
    context: &FetchContext,
    response: &Response,
) {
    let meta = match response.metadata() {
        Ok(FetchMetadata::Unfiltered(m)) => m,
        Ok(FetchMetadata::Filtered { unsafe_, .. }) => unsafe_,
        Err(_) => {
            log::warn!("No metadata available, skipping devtools security info.");
            return;
        },
    };

    if let (Some(devtools_chan), Some(security_info), Some(webview_id)) = (
        context.devtools_chan.clone(),
        meta.tls_security_info,
        request.target_webview_id,
    ) {
        let update = NetworkEvent::SecurityInfo(SecurityInfoUpdate {
            browsing_context_id: webview_id.into(),
            security_info: Some(security_info),
        });

        let msg = ChromeToDevtoolsControlMsg::NetworkEvent(request.id.0.to_string(), update);

        let _ = devtools_chan.send(DevtoolsControlMsg::FromChrome(msg));
    }
}

pub(crate) fn send_early_httprequest_to_devtools(request: &Request, context: &FetchContext) {
    // Do not forward data requests to devtools
    if request.url().scheme() == "data" {
        return;
    }
    if let (Some(devtools_chan), Some(browsing_context_id), Some(pipeline_id)) = (
        context.devtools_chan.as_ref(),
        request.target_webview_id.map(|id| id.into()),
        request.pipeline_id,
    ) {
        // Build the partial DevtoolsHttpRequest
        let devtools_request = DevtoolsHttpRequest {
            url: request.current_url(),
            method: request.method.clone(),
            headers: request.headers.clone(),
            body: None,
            pipeline_id,
            started_date_time: SystemTime::now(),
            time_stamp: 0,
            connect_time: Duration::from_millis(0),
            send_time: Duration::from_millis(0),
            destination: request.destination,
            is_xhr: false,
            browsing_context_id,
        };

        let msg = ChromeToDevtoolsControlMsg::NetworkEvent(
            request.id.0.to_string(),
            NetworkEvent::HttpRequest(devtools_request),
        );

        send_request_to_devtools(msg, devtools_chan);
    }
}
