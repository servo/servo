/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Liberally derived from the [Firefox JS implementation](http://mxr.mozilla.org/mozilla-central/source/toolkit/devtools/server/actors/webconsole.js).
//! Handles interaction with the remote web console on network events (HTTP requests, responses) in Servo.

use std::cell::RefCell;
use std::time::{Duration, UNIX_EPOCH};

use base64::engine::Engine;
use base64::engine::general_purpose::STANDARD;
use chrono::{Local, LocalResult, TimeZone};
use devtools_traits::{HttpRequest, HttpResponse};
use headers::{ContentLength, HeaderMapExt};
use http::HeaderMap;
use net::cookie::ServoCookie;
use net_traits::fetch::headers::extract_mime_type_as_dataurl_mime;
use net_traits::{CookieSource, TlsSecurityInfo};
use serde::Serialize;
use serde_json::{Map, Value};
use servo_url::ServoUrl;

use crate::StreamId;
use crate::actor::{Actor, ActorEncode, ActorError, ActorRegistry};
use crate::actors::browsing_context::BrowsingContextActor;
use crate::actors::long_string::LongStringActor;
use crate::actors::watcher::WatcherActor;
use crate::network_handler::Cause;
use crate::protocol::ClientRequest;

#[derive(Default)]
pub struct NetworkEventActor {
    name: String,
    request: RefCell<Option<NetworkEventRequest>>,
    resource_id: u64,
    response: RefCell<Option<NetworkEventResponse>>,
    security_info: RefCell<TlsSecurityInfo>,
    pub watcher: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkEventResource {
    #[serde(rename = "browsingContextID")]
    browsing_context_id: u32,
    inner_window_id: u64,
    resource_id: u64,
    resource_updates: ResourceUpdates,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkEventMsg {
    actor: String,
    #[serde(rename = "browsingContextID")]
    browsing_context_id: u32,
    cause: Cause,
    #[serde(rename = "isXHR")]
    is_xhr: bool,
    method: String,
    private: bool,
    resource_id: u64,
    started_date_time: String,
    time_stamp: i64,
    url: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GetRequestHeadersReply {
    from: String,
    headers: Vec<HeaderWrapper>,
    header_size: usize,
    raw_headers: String,
}

#[derive(Serialize)]
struct GetCookiesReply {
    from: String,
    cookies: Vec<CookieWrapper>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GetRequestPostDataReply {
    from: String,
    post_data: Option<Vec<u8>>,
    post_data_discarded: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GetResponseHeadersReply {
    from: String,
    headers: Vec<HeaderWrapper>,
    header_size: usize,
    raw_headers: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GetResponseContentReply {
    from: String,
    content: Option<ResponseContent>,
    content_discarded: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GetEventTimingsReply {
    from: String,
    offsets: Timings,
    server_timings: Vec<()>,
    timings: Timings,
    total_time: usize,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GetSecurityInfoReply {
    from: String,
    security_info: SecurityInfo,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct RequestFields {
    event_timings_available: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    remote_address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    remote_port: Option<u16>,
    request_cookies_available: bool,
    request_headers_available: bool,
    total_time: f64,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ResponseFields {
    #[serde(flatten)]
    cache_details: CacheDetails,
    response_content_available: bool,
    response_cookies_available: bool,
    response_headers_available: bool,
    response_start_available: bool,
    status: String,
    status_text: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct SecurityFields {
    security_state: String,
    security_info_available: bool,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceUpdates {
    http_version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(flatten)]
    request: Option<RequestFields>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(flatten)]
    response: Option<ResponseFields>,
    #[serde(flatten)]
    security: SecurityFields,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ResponseContent {
    body_size: usize,
    content_charset: String,
    decoded_body_size: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    encoding: Option<String>,
    headers_size: usize,
    is_content_encoded: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    mime_type: Option<String>,
    size: usize,
    text: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    transferred_size: Option<u64>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CacheDetails {
    from_cache: bool,
    from_service_worker: bool,
}

#[derive(Clone, Default, Serialize)]
pub struct Timings {
    blocked: usize,
    dns: usize,
    connect: usize,
    send: usize,
    wait: usize,
    receive: usize,
}

impl Timings {
    fn total(&self) -> usize {
        self.dns + self.connect + self.send + self.wait + self.receive
    }
}

#[derive(Serialize, Default)]
#[serde(rename_all = "camelCase")]
struct CertificateIdentity {
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    common_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    organization: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    organizational_unit: Option<String>,
}

#[derive(Serialize, Default)]
#[serde(rename_all = "camelCase")]
struct CertificateValidity {
    #[serde(skip_serializing_if = "Option::is_none")]
    start: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    end: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    lifetime: Option<String>,
    expired: bool,
}

#[derive(Serialize, Default)]
#[serde(rename_all = "camelCase")]
struct CertificateFingerprint {
    #[serde(skip_serializing_if = "Option::is_none")]
    sha256: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    sha1: Option<String>,
}

#[derive(Serialize, Default)]
#[serde(rename_all = "camelCase")]
struct SecurityCertificate {
    subject: CertificateIdentity,
    issuer: CertificateIdentity,
    validity: CertificateValidity,
    fingerprint: CertificateFingerprint,
    #[serde(skip_serializing_if = "Option::is_none")]
    serial_number: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    is_built_in_root: Option<bool>,
}

#[derive(Serialize, Default)]
#[serde(rename_all = "camelCase")]
struct SecurityInfo {
    state: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    weakness_reasons: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    protocol_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    cipher_suite: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    kea_group_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    signature_scheme_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    alpn_protocol: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    certificate_transparency: Option<String>,
    hsts: bool,
    hpkp: bool,
    used_ech: bool,
    used_delegated_credentials: bool,
    used_ocsp: bool,
    used_private_dns: bool,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    certificate_chain: Vec<String>,
    cert: SecurityCertificate,
}

impl From<&TlsSecurityInfo> for SecurityInfo {
    fn from(info: &TlsSecurityInfo) -> Self {
        Self {
            state: info.state.to_string(),
            weakness_reasons: info.weakness_reasons.clone(),
            protocol_version: info.protocol_version.clone(),
            cipher_suite: info.cipher_suite.clone(),
            kea_group_name: info.kea_group_name.clone(),
            signature_scheme_name: info.signature_scheme_name.clone(),
            alpn_protocol: info.alpn_protocol.clone(),
            certificate_transparency: info
                .certificate_transparency
                .clone()
                .or_else(|| Some("unknown".to_string())),
            hsts: info.hsts,
            hpkp: info.hpkp,
            used_ech: info.used_ech,
            used_delegated_credentials: info.used_delegated_credentials,
            used_ocsp: info.used_ocsp,
            used_private_dns: info.used_private_dns,
            ..Default::default()
        }
    }
}

struct NetworkEventRequest {
    offsets: Timings,
    timings: Timings,
    request: HttpRequest,
    total_time: Duration,
}

struct NetworkEventResponse {
    cache_details: CacheDetails,
    response: HttpResponse,
}

#[derive(Serialize)]
pub struct CookieWrapper {
    name: String,
    value: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    domain: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    expires: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    http_only: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    secure: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    same_site: Option<String>,
}

#[derive(Serialize)]
struct HeaderWrapper {
    name: String,
    value: String,
}

impl Actor for NetworkEventActor {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn handle_message(
        &self,
        client_request: ClientRequest,
        registry: &ActorRegistry,
        msg_type: &str,
        _msg: &Map<String, Value>,
        _id: StreamId,
    ) -> Result<(), ActorError> {
        match msg_type {
            "getRequestHeaders" => {
                let request = self.request.borrow();
                let request = request.as_ref().ok_or(ActorError::Internal)?;

                let headers = get_header_list(&request.request.headers);
                let raw_headers = get_raw_headers(&headers);

                let msg = GetRequestHeadersReply {
                    from: self.name(),
                    headers,
                    header_size: raw_headers.len(),
                    raw_headers,
                };
                client_request.reply_final(&msg)?
            },

            "getRequestCookies" => {
                let request = self.request.borrow();
                let request = request.as_ref().ok_or(ActorError::Internal)?;

                let msg = GetCookiesReply {
                    from: self.name(),
                    cookies: get_cookies_from_headers(
                        &request.request.headers,
                        &request.request.url,
                    ),
                };

                client_request.reply_final(&msg)?
            },

            "getRequestPostData" => {
                let request = self.request.borrow();
                let request = request.as_ref().ok_or(ActorError::Internal)?;

                let msg = GetRequestPostDataReply {
                    from: self.name(),
                    post_data: request.request.body.as_ref().map(|b| b.0.clone()),
                    post_data_discarded: request.request.body.is_none(),
                };
                client_request.reply_final(&msg)?
            },

            "getResponseHeaders" => {
                let response = self.response.borrow();
                let response = response.as_ref().ok_or(ActorError::Internal)?;

                let list = response
                    .response
                    .headers
                    .as_ref()
                    .map(get_header_list)
                    .unwrap_or_default();
                let raw_headers = get_raw_headers(&list);

                let msg = GetResponseHeadersReply {
                    from: self.name(),
                    headers: list,
                    header_size: raw_headers.len(),
                    raw_headers,
                };
                client_request.reply_final(&msg)?;
            },

            "getResponseCookies" => {
                let request = self.request.borrow();
                let request = request.as_ref().ok_or(ActorError::Internal)?;
                let response = self.response.borrow();
                let response = response.as_ref().ok_or(ActorError::Internal)?;

                let msg = GetCookiesReply {
                    from: self.name(),
                    cookies: get_cookies_from_headers(
                        response
                            .response
                            .headers
                            .as_ref()
                            .ok_or(ActorError::Internal)?,
                        &request.request.url,
                    ),
                };
                client_request.reply_final(&msg)?
            },

            "getResponseContent" => {
                let response = self.response.borrow();
                let response = response.as_ref().ok_or(ActorError::Internal)?;

                let headers = response.response.headers.as_ref();
                let list = headers.map(get_header_list).unwrap_or_default();
                let raw_headers = get_raw_headers(&list);

                let mime_type = headers
                    .and_then(extract_mime_type_as_dataurl_mime)
                    .map(|url| url.to_string());
                let transferred_size = headers
                    .and_then(|header| header.typed_get::<ContentLength>())
                    .map(|content_length_header| content_length_header.0);

                let content = response.response.body.as_ref().map(|body| {
                    let (encoding, text) = if mime_type.is_some() {
                        // Queue a LongStringActor for this body
                        let body_string = String::from_utf8_lossy(body).to_string();
                        let long_string = LongStringActor::new(registry, body_string);
                        let value = long_string.long_string_obj();
                        registry.register_later(long_string);
                        (None, serde_json::to_value(value).unwrap())
                    } else {
                        let b64 = STANDARD.encode(&body.0);
                        (Some("base64".into()), serde_json::to_value(b64).unwrap())
                    };
                    let is_content_encoded = encoding.is_some();

                    ResponseContent {
                        body_size: body.len(),
                        content_charset: "".into(),
                        decoded_body_size: body.len(),
                        encoding,
                        headers_size: raw_headers.len(),
                        is_content_encoded,
                        mime_type,
                        size: body.len(),
                        text,
                        transferred_size,
                    }
                });

                let msg = GetResponseContentReply {
                    from: self.name(),
                    content,
                    content_discarded: response.response.body.is_none(),
                };
                client_request.reply_final(&msg)?
            },

            "getEventTimings" => {
                let request = self.request.borrow();
                let request = request.as_ref().ok_or(ActorError::Internal)?;

                let offsets = request.offsets.clone();
                let timings = request.timings.clone();
                let total_time = timings.total();

                let msg = GetEventTimingsReply {
                    from: self.name(),
                    offsets,
                    server_timings: vec![],
                    timings,
                    total_time,
                };
                client_request.reply_final(&msg)?
            },

            "getSecurityInfo" => {
                let security_info = &*self.security_info.borrow();

                let msg = GetSecurityInfoReply {
                    from: self.name(),
                    security_info: security_info.into(),
                };
                client_request.reply_final(&msg)?
            },

            _ => return Err(ActorError::UnrecognizedPacketType),
        };
        Ok(())
    }
}

impl NetworkEventActor {
    pub fn new(name: String, resource_id: u64, watcher: String) -> NetworkEventActor {
        NetworkEventActor {
            name,
            resource_id,
            watcher,
            ..Default::default()
        }
    }

    pub fn add_request(&self, request: HttpRequest) {
        self.request.replace(Some(NetworkEventRequest {
            // TODO: Fill the rest of the fields correctly for offsets and timings
            offsets: Default::default(),
            timings: Timings {
                connect: request.connect_time.as_millis() as usize,
                send: request.send_time.as_millis() as usize,
                ..Default::default()
            },
            total_time: request.connect_time + request.send_time,
            request,
        }));
    }

    pub fn add_response(&self, response: HttpResponse) {
        if response.body.is_none() {
            return;
        }
        self.response.replace(Some(NetworkEventResponse {
            cache_details: CacheDetails {
                from_cache: response.from_cache,
                from_service_worker: false,
            },
            response,
        }));
    }

    pub fn add_security_info(&self, security_info: Option<TlsSecurityInfo>) {
        self.security_info
            .replace(security_info.unwrap_or_default());
    }

    fn request_fields(&self) -> Option<RequestFields> {
        let request = self.request.borrow();
        let request = request.as_ref()?;
        let url = request.request.url.as_url();
        let cookies = get_cookies_from_headers(&request.request.headers, &request.request.url);

        Some(RequestFields {
            event_timings_available: true,
            remote_address: url.host_str().map(|a| a.into()),
            remote_port: url.port(),
            request_cookies_available: !cookies.is_empty(),
            request_headers_available: !request.request.headers.is_empty(),
            total_time: request.total_time.as_secs_f64(),
        })
    }

    fn response_fields(&self) -> Option<ResponseFields> {
        let response = self.response.borrow();
        let response = response.as_ref()?;
        let url = self.request.borrow().as_ref()?.request.url.clone();
        let headers = response.response.headers.as_ref();
        let cookies = headers.map(|headers| get_cookies_from_headers(headers, &url));
        let status = &response.response.status;

        Some(ResponseFields {
            cache_details: response.cache_details.clone(),
            response_content_available: response
                .response
                .body
                .as_ref()
                .is_some_and(|body| !body.is_empty()),
            response_cookies_available: cookies.is_some(),
            response_headers_available: headers.is_some(),
            response_start_available: true,
            status: status.code().to_string(),
            status_text: String::from_utf8_lossy(status.message()).to_string(),
        })
    }

    fn security_fields(&self) -> SecurityFields {
        let security_info = self.security_info.borrow();

        SecurityFields {
            security_state: security_info.state.to_string(),
            security_info_available: true,
        }
    }

    pub fn resource_updates(&self, registry: &ActorRegistry) -> NetworkEventResource {
        let watcher = registry.find::<WatcherActor>(&self.watcher);
        let browsing_context =
            registry.find::<BrowsingContextActor>(&watcher.browsing_context_actor);

        NetworkEventResource {
            resource_id: self.resource_id,
            resource_updates: ResourceUpdates {
                // TODO: Set correct value
                http_version: "HTTP/1.1".into(),
                request: self.request_fields(),
                response: self.response_fields(),
                security: self.security_fields(),
            },
            browsing_context_id: browsing_context.browsing_context_id.value(),
            inner_window_id: 0,
        }
    }
}

fn get_cookies_from_headers(headers: &HeaderMap, url: &ServoUrl) -> Vec<CookieWrapper> {
    headers
        .get_all("set-cookie")
        .iter()
        .filter_map(|cookie| {
            let cookie_str = std::str::from_utf8(cookie.as_bytes()).ok()?;
            ServoCookie::from_cookie_string(cookie_str, url, CookieSource::HTTP)
        })
        .map(|cookie| {
            let cookie = &cookie.cookie;
            CookieWrapper {
                name: cookie.name().into(),
                value: cookie.value().into(),
                path: cookie.path().map(|p| p.into()),
                domain: cookie.domain().map(|d| d.into()),
                expires: cookie.expires().map(|e| format!("{e:?}")),
                http_only: cookie.http_only(),
                secure: cookie.secure(),
                same_site: cookie.same_site().map(|s| s.to_string()),
            }
        })
        .collect()
}

fn get_header_list(headers: &HeaderMap) -> Vec<HeaderWrapper> {
    headers
        .iter()
        .map(|(name, value)| HeaderWrapper {
            name: name.as_str().into(),
            value: value.to_str().unwrap_or_default().into(),
        })
        .collect()
}

fn get_raw_headers(headers: &[HeaderWrapper]) -> String {
    headers
        .iter()
        .map(|header| format!("{}:{}", header.name, header.value))
        .collect::<Vec<_>>()
        .join("\r\n")
}

impl ActorEncode<NetworkEventMsg> for NetworkEventActor {
    fn encode(&self, registry: &ActorRegistry) -> NetworkEventMsg {
        let request = self.request.borrow();
        let request = &request.as_ref().expect("There should be a request").request;

        let started_datetime_rfc3339 = match Local.timestamp_millis_opt(
            request
                .started_date_time
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as i64,
        ) {
            LocalResult::None => "".to_owned(),
            LocalResult::Single(date_time) => date_time.to_rfc3339().to_string(),
            LocalResult::Ambiguous(date_time, _) => date_time.to_rfc3339().to_string(),
        };

        let watcher = registry.find::<WatcherActor>(&self.watcher);
        let browsing_context =
            registry.find::<BrowsingContextActor>(&watcher.browsing_context_actor);

        NetworkEventMsg {
            actor: self.name(),
            browsing_context_id: browsing_context.browsing_context_id.value(),
            cause: Cause {
                type_: request.destination.as_str().to_string(),
                loading_document_uri: None, // Set if available
            },
            is_xhr: request.is_xhr,
            method: format!("{}", request.method),
            private: false,
            resource_id: self.resource_id,
            started_date_time: started_datetime_rfc3339,
            time_stamp: request.time_stamp,
            url: request.url.to_string(),
        }
    }
}
