/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Liberally derived from the [Firefox JS implementation](http://mxr.mozilla.org/mozilla-central/source/toolkit/devtools/server/actors/webconsole.js).
//! Handles interaction with the remote web console on network events (HTTP requests, responses) in Servo.

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use chrono::{Local, LocalResult, TimeZone};
use devtools_traits::{HttpRequest as DevtoolsHttpRequest, HttpResponse as DevtoolsHttpResponse};
use headers::{ContentLength, ContentType, Cookie, HeaderMapExt};
use http::{HeaderMap, Method};
use net::cookie::ServoCookie;
use net_traits::CookieSource;
use net_traits::request::Destination as RequestDestination;
use serde::Serialize;
use serde_json::{Map, Value};
use servo_url::ServoUrl;

use crate::StreamId;
use crate::actor::{Actor, ActorError, ActorRegistry};
use crate::network_handler::Cause;
use crate::protocol::ClientRequest;

pub struct NetworkEventActor {
    pub name: String,
    pub resource_id: u64,
    pub is_xhr: bool,
    pub request_url: String,
    pub request_method: Method,
    pub request_started: SystemTime,
    pub request_time_stamp: i64,
    pub request_destination: RequestDestination,
    pub request_headers_raw: Option<HeaderMap>,
    pub request_body: Option<Vec<u8>>,
    pub request_cookies: Option<RequestCookiesMsg>,
    pub request_headers: Option<RequestHeadersMsg>,
    pub response_headers_raw: Option<HeaderMap>,
    pub response_body: Option<Vec<u8>>,
    pub response_content: Option<ResponseContentMsg>,
    pub response_start: Option<ResponseStartMsg>,
    pub response_cookies: Option<ResponseCookiesMsg>,
    pub response_headers: Option<ResponseHeadersMsg>,
    pub total_time: Duration,
    pub security_state: String,
    pub event_timing: Option<Timings>,
    pub watcher_name: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkEventResource {
    pub resource_id: u64,
    pub resource_updates: Map<String, Value>,
    pub browsing_context_id: u64,
    pub inner_window_id: u64,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EventActor {
    pub actor: String,
    pub resource_id: u64,
    pub url: String,
    pub method: String,
    pub started_date_time: String,
    pub time_stamp: i64,
    #[serde(rename = "isXHR")]
    pub is_xhr: bool,
    pub private: bool,
    pub cause: Cause,
}

#[derive(Serialize)]
pub struct ResponseCookiesMsg {
    pub cookies: Vec<ResponseCookieObj>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResponseStartMsg {
    pub http_version: String,
    pub remote_address: String,
    pub remote_port: u32,
    pub status: String,
    pub status_text: String,
    pub headers_size: usize,
    pub discard_response_body: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResponseContentMsg {
    pub mime_type: String,
    pub content_size: u32,
    pub transferred_size: u32,
    pub discard_response_body: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResponseHeadersMsg {
    pub headers: usize,
    pub headers_size: usize,
}

#[derive(Serialize)]
pub struct RequestCookiesMsg {
    pub cookies: Vec<RequestCookieObj>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestHeadersMsg {
    headers: usize,
    headers_size: usize,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GetRequestHeadersReply {
    from: String,
    headers: Vec<Header>,
    header_size: usize,
    raw_headers: String,
}

#[derive(Serialize)]
struct Header {
    name: String,
    value: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GetResponseHeadersReply {
    from: String,
    headers: Vec<Header>,
    header_size: usize,
    raw_headers: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GetResponseContentReply {
    from: String,
    content: Option<Vec<u8>>,
    content_discarded: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GetRequestPostDataReply {
    from: String,
    post_data: Option<Vec<u8>>,
    post_data_discarded: bool,
}

#[derive(Serialize)]
struct GetRequestCookiesReply {
    from: String,
    cookies: Vec<RequestCookieObj>,
}

#[derive(Serialize)]
struct GetResponseCookiesReply {
    from: String,
    cookies: Vec<ResponseCookieObj>,
}
#[derive(Clone, Serialize)]
pub struct ResponseCookieObj {
    pub name: String,
    pub value: String,
    pub path: Option<String>,
    pub domain: Option<String>,
    pub expires: Option<String>,
    #[serde(rename = "httpOnly")]
    pub http_only: Option<bool>,
    pub secure: Option<bool>,
    #[serde(rename = "sameSite")]
    pub same_site: Option<String>,
}

#[derive(Clone, Serialize)]
pub struct RequestCookieObj {
    pub name: String,
    pub value: String,
}

#[derive(Clone, Default, Serialize)]
pub struct Timings {
    blocked: u32,
    dns: u32,
    connect: u64,
    send: u64,
    wait: u32,
    receive: u32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GetEventTimingsReply {
    from: String,
    timings: Timings,
    total_time: u64,
}

#[derive(Serialize)]
struct SecurityInfo {
    state: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GetSecurityInfoReply {
    from: String,
    security_info: SecurityInfo,
}

impl Actor for NetworkEventActor {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn handle_message(
        &self,
        request: ClientRequest,
        _registry: &ActorRegistry,
        msg_type: &str,
        _msg: &Map<String, Value>,
        _id: StreamId,
    ) -> Result<(), ActorError> {
        match msg_type {
            "getRequestHeaders" => {
                let mut headers = Vec::new();
                let mut raw_headers_string = "".to_owned();
                let mut headers_size = 0;
                if let Some(ref headers_map) = self.request_headers_raw {
                    for (name, value) in headers_map.iter() {
                        let value = &value.to_str().unwrap().to_string();
                        raw_headers_string =
                            raw_headers_string + name.as_str() + ":" + value + "\r\n";
                        headers_size += name.as_str().len() + value.len();
                        headers.push(Header {
                            name: name.as_str().to_owned(),
                            value: value.to_owned(),
                        });
                    }
                }

                let msg = GetRequestHeadersReply {
                    from: self.name(),
                    headers,
                    header_size: headers_size,
                    raw_headers: raw_headers_string,
                };
                request.reply_final(&msg)?
            },
            "getRequestCookies" => {
                let cookies = self
                    .request_cookies
                    .as_ref()
                    .map(|msg| msg.cookies.clone())
                    .unwrap_or_default();
                let msg = GetRequestCookiesReply {
                    from: self.name(),
                    cookies,
                };
                request.reply_final(&msg)?
            },
            "getRequestPostData" => {
                let msg = GetRequestPostDataReply {
                    from: self.name(),
                    post_data: self.request_body.clone(),
                    post_data_discarded: self.request_body.is_none(),
                };
                request.reply_final(&msg)?
            },
            "getResponseHeaders" => {
                if let Some(ref response_headers) = self.response_headers_raw {
                    let mut headers = vec![];
                    let mut raw_headers_string = "".to_owned();
                    let mut headers_size = 0;
                    for (name, value) in response_headers.iter() {
                        headers.push(Header {
                            name: name.as_str().to_owned(),
                            value: value.to_str().unwrap().to_owned(),
                        });
                        headers_size += name.as_str().len() + value.len();
                        raw_headers_string.push_str(name.as_str());
                        raw_headers_string.push(':');
                        raw_headers_string.push_str(value.to_str().unwrap());
                        raw_headers_string.push_str("\r\n");
                    }
                    let msg = GetResponseHeadersReply {
                        from: self.name(),
                        headers,
                        header_size: headers_size,
                        raw_headers: raw_headers_string,
                    };
                    request.reply_final(&msg)?;
                } else {
                    // FIXME: what happens when there are no response headers?
                    return Err(ActorError::Internal);
                }
            },
            "getResponseCookies" => {
                let cookies = self
                    .response_cookies
                    .as_ref()
                    .map(|msg| msg.cookies.clone())
                    .unwrap_or_default();
                let msg = GetResponseCookiesReply {
                    from: self.name(),
                    cookies,
                };
                request.reply_final(&msg)?
            },
            "getResponseContent" => {
                let msg = GetResponseContentReply {
                    from: self.name(),
                    content: self.response_body.clone(),
                    content_discarded: self.response_body.is_none(),
                };
                request.reply_final(&msg)?
            },
            "getEventTimings" => {
                // TODO: This is a fake timings msg
                let timings_obj = self.event_timing.clone().unwrap_or_default();
                // Might use the one on self
                let total = timings_obj.connect + timings_obj.send;
                // TODO: Send the correct values for all these fields.
                let msg = GetEventTimingsReply {
                    from: self.name(),
                    timings: timings_obj,
                    total_time: total,
                };
                request.reply_final(&msg)?
            },
            "getSecurityInfo" => {
                // TODO: Send the correct values for securityInfo.
                let msg = GetSecurityInfoReply {
                    from: self.name(),
                    security_info: SecurityInfo {
                        state: "insecure".to_owned(),
                    },
                };
                request.reply_final(&msg)?
            },
            _ => return Err(ActorError::UnrecognizedPacketType),
        };
        Ok(())
    }
}

impl NetworkEventActor {
    pub fn new(name: String, resource_id: u64, watcher_name: String) -> NetworkEventActor {
        NetworkEventActor {
            name,
            resource_id,
            is_xhr: false,
            request_url: String::new(),
            request_method: Method::GET,
            request_started: SystemTime::now(),
            request_time_stamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as i64,
            request_destination: RequestDestination::None,
            request_headers_raw: None,
            request_body: None,
            request_cookies: None,
            request_headers: None,
            response_headers_raw: None,
            response_body: None,
            response_content: None,
            response_start: None,
            response_cookies: None,
            response_headers: None,
            total_time: Duration::ZERO,
            security_state: "insecure".to_owned(),
            event_timing: None,
            watcher_name,
        }
    }

    pub fn add_request(&mut self, request: DevtoolsHttpRequest) {
        self.is_xhr = request.is_xhr;
        self.request_cookies = Self::request_cookies(&request);
        self.request_headers = Some(Self::request_headers(&request));
        self.total_time = Self::total_time(&request);
        self.event_timing = Some(Self::event_timing(&request));
        self.request_url = request.url.to_string();
        self.request_method = request.method;
        self.request_started = request.started_date_time;
        self.request_time_stamp = request.time_stamp;
        self.request_destination = request.destination;
        self.request_body = request.body.clone();
        self.request_headers_raw = Some(request.headers.clone());
    }

    pub fn add_response(&mut self, response: DevtoolsHttpResponse) {
        self.response_headers = Some(Self::response_headers(&response));
        self.response_cookies = ServoUrl::parse(&self.request_url)
            .ok()
            .as_ref()
            .and_then(|url| Self::response_cookies(&response, url));
        self.response_start = Some(Self::response_start(&response));
        self.response_content = Self::response_content(&response);
        self.response_body = response.body.clone();
        self.response_headers_raw = response.headers.clone();
    }

    pub fn event_actor(&self) -> EventActor {
        // TODO: Send the correct values for startedDateTime, isXHR, private

        let started_datetime_rfc3339 = match Local.timestamp_millis_opt(
            self.request_started
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as i64,
        ) {
            LocalResult::None => "".to_owned(),
            LocalResult::Single(date_time) => date_time.to_rfc3339().to_string(),
            LocalResult::Ambiguous(date_time, _) => date_time.to_rfc3339().to_string(),
        };

        EventActor {
            actor: self.name(),
            resource_id: self.resource_id,
            url: self.request_url.clone(),
            method: format!("{}", self.request_method),
            started_date_time: started_datetime_rfc3339,
            time_stamp: self.request_time_stamp,
            is_xhr: self.is_xhr,
            private: false,
            cause: Cause {
                type_: self.request_destination.as_str().to_string(),
                loading_document_uri: None, // Set if available
            },
        }
    }

    pub fn response_start(response: &DevtoolsHttpResponse) -> ResponseStartMsg {
        // TODO: Send the correct values for all these fields.
        let h_size = response.headers.as_ref().map(|h| h.len()).unwrap_or(0);
        let status = &response.status;

        // TODO: Send the correct values for remoteAddress and remotePort and http_version
        ResponseStartMsg {
            http_version: "HTTP/1.1".to_owned(),
            remote_address: "63.245.217.43".to_owned(),
            remote_port: 443,
            status: status.code().to_string(),
            status_text: String::from_utf8_lossy(status.message()).to_string(),
            headers_size: h_size,
            discard_response_body: false,
        }
    }

    pub fn response_content(response: &DevtoolsHttpResponse) -> Option<ResponseContentMsg> {
        let _body = response.body.as_ref()?;

        let mime_type = response
            .headers
            .as_ref()
            .and_then(|h| h.typed_get::<ContentType>())
            .map(|ct| ct.to_string())
            .unwrap_or_default();

        let transferred_size = response
            .headers
            .as_ref()
            .and_then(|hdrs| hdrs.typed_get::<ContentLength>())
            .map(|cl| cl.0);

        let content_size = response.body.as_ref().map(|body| body.len() as u64);

        Some(ResponseContentMsg {
            mime_type,
            content_size: content_size.unwrap_or(0) as u32,
            transferred_size: transferred_size.unwrap_or(0) as u32,
            discard_response_body: true,
        })
    }

    pub fn response_cookies(
        response: &DevtoolsHttpResponse,
        url: &ServoUrl,
    ) -> Option<ResponseCookiesMsg> {
        let headers = response.headers.as_ref()?;
        let cookies = headers
            .get_all("set-cookie")
            .iter()
            .filter_map(|cookie| {
                let cookie_str = String::from_utf8(cookie.as_bytes().to_vec()).ok()?;
                ServoCookie::from_cookie_string(cookie_str, url, CookieSource::HTTP)
            })
            .map(|servo_cookie| {
                let c = &servo_cookie.cookie;
                ResponseCookieObj {
                    name: c.name().to_string(),
                    value: c.value().to_string(),
                    path: c.path().map(|p| p.to_string()),
                    domain: c.domain().map(|d| d.to_string()),
                    expires: c.expires().map(|dt| format!("{:?}", dt)),
                    http_only: c.http_only(),
                    secure: c.secure(),
                    same_site: c.same_site().map(|s| s.to_string()),
                }
            })
            .collect::<Vec<_>>();
        Some(ResponseCookiesMsg { cookies })
    }

    pub fn response_headers(response: &DevtoolsHttpResponse) -> ResponseHeadersMsg {
        let mut header_size = 0;
        let mut headers_byte_count = 0;
        if let Some(ref headers) = response.headers {
            for (name, value) in headers.iter() {
                header_size += 1;
                headers_byte_count += name.as_str().len() + value.len();
            }
        }
        ResponseHeadersMsg {
            headers: header_size,
            headers_size: headers_byte_count,
        }
    }

    pub fn request_headers(request: &DevtoolsHttpRequest) -> RequestHeadersMsg {
        let size = request.headers.iter().fold(0, |acc, (name, value)| {
            acc + name.as_str().len() + value.len()
        });
        RequestHeadersMsg {
            headers: request.headers.len(),
            headers_size: size,
        }
    }

    pub fn request_cookies(request: &DevtoolsHttpRequest) -> Option<RequestCookiesMsg> {
        let header_value = request.headers.typed_get::<Cookie>()?;
        let cookies = header_value
            .iter()
            .map(|cookie| RequestCookieObj {
                name: cookie.0.to_string(),
                value: cookie.1.to_string(),
            })
            .collect::<Vec<_>>();
        Some(RequestCookiesMsg { cookies })
    }

    pub fn total_time(request: &DevtoolsHttpRequest) -> Duration {
        request.connect_time + request.send_time
    }

    pub fn event_timing(request: &DevtoolsHttpRequest) -> Timings {
        Timings {
            blocked: 0,
            dns: 0,
            connect: request.connect_time.as_millis() as u64,
            send: request.send_time.as_millis() as u64,
            wait: 0,
            receive: 0,
        }
    }

    fn insert_serialized_map<T: Serialize>(map: &mut Map<String, Value>, obj: &Option<T>) {
        if let Some(value) = obj {
            if let Ok(Value::Object(serialized)) = serde_json::to_value(value) {
                for (key, val) in serialized {
                    map.insert(key, val);
                }
            }
        }
    }

    pub fn resource_updates(&self) -> NetworkEventResource {
        let mut resource_updates = Map::new();

        resource_updates.insert(
            "requestCookiesAvailable".to_owned(),
            Value::Bool(self.request_cookies.is_some()),
        );

        resource_updates.insert(
            "requestHeadersAvailable".to_owned(),
            Value::Bool(self.request_headers.is_some()),
        );

        resource_updates.insert(
            "responseHeadersAvailable".to_owned(),
            Value::Bool(self.response_headers.is_some()),
        );
        resource_updates.insert(
            "responseCookiesAvailable".to_owned(),
            Value::Bool(self.response_cookies.is_some()),
        );
        resource_updates.insert(
            "responseStartAvailable".to_owned(),
            Value::Bool(self.response_start.is_some()),
        );
        resource_updates.insert(
            "responseContentAvailable".to_owned(),
            Value::Bool(self.response_content.is_some()),
        );

        resource_updates.insert(
            "totalTime".to_string(),
            Value::from(self.total_time.as_secs_f64()),
        );

        resource_updates.insert(
            "securityState".to_string(),
            Value::String(self.security_state.clone()),
        );
        resource_updates.insert(
            "eventTimingsAvailable".to_owned(),
            Value::Bool(self.event_timing.is_some()),
        );

        Self::insert_serialized_map(&mut resource_updates, &self.response_content);
        Self::insert_serialized_map(&mut resource_updates, &self.response_headers);
        Self::insert_serialized_map(&mut resource_updates, &self.response_cookies);
        Self::insert_serialized_map(&mut resource_updates, &self.request_headers);
        Self::insert_serialized_map(&mut resource_updates, &self.request_cookies);
        Self::insert_serialized_map(&mut resource_updates, &self.response_start);
        Self::insert_serialized_map(&mut resource_updates, &self.event_timing);

        // TODO: Set the correct values for these fields
        NetworkEventResource {
            resource_id: self.resource_id,
            resource_updates,
            browsing_context_id: 0,
            inner_window_id: 0,
        }
    }
}
