/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Liberally derived from the [Firefox JS implementation](http://mxr.mozilla.org/mozilla-central/source/toolkit/devtools/server/actors/webconsole.js).
//! Handles interaction with the remote web console on network events (HTTP requests, responses) in Servo.

use std::net::TcpStream;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use chrono::{Local, LocalResult, TimeZone};
use devtools_traits::{HttpRequest as DevtoolsHttpRequest, HttpResponse as DevtoolsHttpResponse};
use headers::{ContentType, Cookie, HeaderMapExt};
use http::{HeaderMap, Method, header};
use net_traits::http_status::HttpStatus;
use serde::Serialize;
use serde_json::{Map, Value};

use crate::StreamId;
use crate::actor::{Actor, ActorMessageStatus, ActorRegistry};
use crate::network_handler::Cause;
use crate::protocol::JsonPacketStream;

struct HttpRequest {
    url: String,
    method: Method,
    headers: HeaderMap,
    body: Option<Vec<u8>>,
    started_date_time: SystemTime,
    time_stamp: i64,
    connect_time: Duration,
    send_time: Duration,
}

struct HttpResponse {
    headers: Option<HeaderMap>,
    status: HttpStatus,
    body: Option<Vec<u8>>,
}

pub struct NetworkEventActor {
    pub name: String,
    request: HttpRequest,
    response: HttpResponse,
    is_xhr: bool,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EventActor {
    pub actor: String,
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
    pub cookies: usize,
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
    pub cookies: usize,
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
    cookies: Vec<u8>,
}

#[derive(Serialize)]
struct GetResponseCookiesReply {
    from: String,
    cookies: Vec<u8>,
}

#[derive(Serialize)]
struct Timings {
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
        _registry: &ActorRegistry,
        msg_type: &str,
        _msg: &Map<String, Value>,
        stream: &mut TcpStream,
        _id: StreamId,
    ) -> Result<ActorMessageStatus, ()> {
        Ok(match msg_type {
            "getRequestHeaders" => {
                let mut headers = Vec::new();
                let mut raw_headers_string = "".to_owned();
                let mut headers_size = 0;
                for (name, value) in self.request.headers.iter() {
                    let value = &value.to_str().unwrap().to_string();
                    raw_headers_string = raw_headers_string + name.as_str() + ":" + value + "\r\n";
                    headers_size += name.as_str().len() + value.len();
                    headers.push(Header {
                        name: name.as_str().to_owned(),
                        value: value.to_owned(),
                    });
                }
                let msg = GetRequestHeadersReply {
                    from: self.name(),
                    headers,
                    header_size: headers_size,
                    raw_headers: raw_headers_string,
                };
                let _ = stream.write_json_packet(&msg);
                ActorMessageStatus::Processed
            },
            "getRequestCookies" => {
                let mut cookies = Vec::new();

                for cookie in self.request.headers.get_all(header::COOKIE) {
                    if let Ok(cookie_value) = String::from_utf8(cookie.as_bytes().to_vec()) {
                        cookies = cookie_value.into_bytes();
                    }
                }

                let msg = GetRequestCookiesReply {
                    from: self.name(),
                    cookies,
                };
                let _ = stream.write_json_packet(&msg);
                ActorMessageStatus::Processed
            },
            "getRequestPostData" => {
                let msg = GetRequestPostDataReply {
                    from: self.name(),
                    post_data: self.request.body.clone(),
                    post_data_discarded: false,
                };
                let _ = stream.write_json_packet(&msg);
                ActorMessageStatus::Processed
            },
            "getResponseHeaders" => {
                if let Some(ref response_headers) = self.response.headers {
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
                    let _ = stream.write_json_packet(&msg);
                }
                ActorMessageStatus::Processed
            },
            "getResponseCookies" => {
                let mut cookies = Vec::new();
                // TODO: This seems quite broken
                for cookie in self.request.headers.get_all(header::SET_COOKIE) {
                    if let Ok(cookie_value) = String::from_utf8(cookie.as_bytes().to_vec()) {
                        cookies = cookie_value.into_bytes();
                    }
                }

                let msg = GetResponseCookiesReply {
                    from: self.name(),
                    cookies,
                };
                let _ = stream.write_json_packet(&msg);
                ActorMessageStatus::Processed
            },
            "getResponseContent" => {
                let msg = GetResponseContentReply {
                    from: self.name(),
                    content: self.response.body.clone(),
                    content_discarded: self.response.body.is_none(),
                };
                let _ = stream.write_json_packet(&msg);
                ActorMessageStatus::Processed
            },
            "getEventTimings" => {
                // TODO: This is a fake timings msg
                let timings_obj = Timings {
                    blocked: 0,
                    dns: 0,
                    connect: self.request.connect_time.as_millis() as u64,
                    send: self.request.send_time.as_millis() as u64,
                    wait: 0,
                    receive: 0,
                };
                let total = timings_obj.connect + timings_obj.send;
                // TODO: Send the correct values for all these fields.
                let msg = GetEventTimingsReply {
                    from: self.name(),
                    timings: timings_obj,
                    total_time: total,
                };
                let _ = stream.write_json_packet(&msg);
                ActorMessageStatus::Processed
            },
            "getSecurityInfo" => {
                // TODO: Send the correct values for securityInfo.
                let msg = GetSecurityInfoReply {
                    from: self.name(),
                    security_info: SecurityInfo {
                        state: "insecure".to_owned(),
                    },
                };
                let _ = stream.write_json_packet(&msg);
                ActorMessageStatus::Processed
            },
            _ => ActorMessageStatus::Ignored,
        })
    }
}

impl NetworkEventActor {
    pub fn new(name: String) -> NetworkEventActor {
        NetworkEventActor {
            name,
            request: HttpRequest {
                url: String::new(),
                method: Method::GET,
                headers: HeaderMap::new(),
                body: None,
                started_date_time: SystemTime::now(),
                time_stamp: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs() as i64,
                send_time: Duration::ZERO,
                connect_time: Duration::ZERO,
            },
            response: HttpResponse {
                headers: None,
                status: HttpStatus::default(),
                body: None,
            },
            is_xhr: false,
        }
    }

    pub fn add_request(&mut self, request: DevtoolsHttpRequest) {
        request.url.as_str().clone_into(&mut self.request.url);

        self.request.method = request.method.clone();
        self.request.headers = request.headers.clone();
        self.request.body = request.body;
        self.request.started_date_time = request.started_date_time;
        self.request.time_stamp = request.time_stamp;
        self.request.connect_time = request.connect_time;
        self.request.send_time = request.send_time;
        self.is_xhr = request.is_xhr;
    }

    pub fn add_response(&mut self, response: DevtoolsHttpResponse) {
        self.response.headers.clone_from(&response.headers);
        self.response.status = response.status;
        self.response.body = response.body;
    }

    pub fn event_actor(&self) -> EventActor {
        // TODO: Send the correct values for startedDateTime, isXHR, private

        let started_datetime_rfc3339 = match Local.timestamp_millis_opt(
            self.request
                .started_date_time
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as i64,
        ) {
            LocalResult::None => "".to_owned(),
            LocalResult::Single(date_time) => date_time.to_rfc3339().to_string(),
            LocalResult::Ambiguous(date_time, _) => date_time.to_rfc3339().to_string(),
        };

        let cause_type = match self.request.url.as_str() {
            // Adjust based on request data
            url if url.ends_with(".css") => "stylesheet",
            url if url.ends_with(".js") => "script",
            url if url.ends_with(".png") || url.ends_with(".jpg") => "img",
            _ => "document",
        };

        EventActor {
            actor: self.name(),
            url: self.request.url.clone(),
            method: format!("{}", self.request.method),
            started_date_time: started_datetime_rfc3339,
            time_stamp: self.request.time_stamp,
            is_xhr: self.is_xhr,
            private: false,
            cause: Cause {
                type_: cause_type.to_string(),
                loading_document_uri: None, // Set if available
            },
        }
    }

    pub fn response_start(&self) -> ResponseStartMsg {
        // TODO: Send the correct values for all these fields.
        let h_size_option = self.response.headers.as_ref().map(|headers| headers.len());
        let h_size = h_size_option.unwrap_or(0);
        let status = &self.response.status;
        // TODO: Send the correct values for remoteAddress and remotePort and http_version.
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

    pub fn response_content(&self) -> ResponseContentMsg {
        let mut m_string = "".to_owned();
        if let Some(ref headers) = self.response.headers {
            m_string = match headers.typed_get::<ContentType>() {
                Some(ct) => ct.to_string(),
                _ => "".to_owned(),
            };
        }
        // TODO: Set correct values when response's body is sent to the devtools in http_loader.
        ResponseContentMsg {
            mime_type: m_string,
            content_size: 0,
            transferred_size: 0,
            discard_response_body: true,
        }
    }

    pub fn response_cookies(&self) -> ResponseCookiesMsg {
        let mut cookies_size = 0;
        if let Some(ref headers) = self.response.headers {
            cookies_size = match headers.typed_get::<Cookie>() {
                Some(ref cookie) => cookie.len(),
                _ => 0,
            };
        }
        ResponseCookiesMsg {
            cookies: cookies_size,
        }
    }

    pub fn response_headers(&self) -> ResponseHeadersMsg {
        let mut headers_size = 0;
        let mut headers_byte_count = 0;
        if let Some(ref headers) = self.response.headers {
            headers_size = headers.len();
            for (name, value) in headers.iter() {
                headers_byte_count += name.as_str().len() + value.len();
            }
        }
        ResponseHeadersMsg {
            headers: headers_size,
            headers_size: headers_byte_count,
        }
    }

    pub fn request_headers(&self) -> RequestHeadersMsg {
        let size = self.request.headers.iter().fold(0, |acc, (name, value)| {
            acc + name.as_str().len() + value.len()
        });
        RequestHeadersMsg {
            headers: self.request.headers.len(),
            headers_size: size,
        }
    }

    pub fn request_cookies(&self) -> RequestCookiesMsg {
        let cookies_size = match self.request.headers.typed_get::<Cookie>() {
            Some(ref cookie) => cookie.len(),
            _ => 0,
        };
        RequestCookiesMsg {
            cookies: cookies_size,
        }
    }

    pub fn total_time(&self) -> Duration {
        self.request.connect_time + self.request.send_time
    }
}
