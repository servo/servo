/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Liberally derived from the [Firefox JS implementation]
//! (http://mxr.mozilla.org/mozilla-central/source/toolkit/devtools/server/actors/webconsole.js).
//! Handles interaction with the remote web console on network events (HTTP requests, responses) in Servo.

use actor::{Actor, ActorMessageStatus, ActorRegistry};
use devtools_traits::HttpRequest as DevtoolsHttpRequest;
use devtools_traits::HttpResponse as DevtoolsHttpResponse;
use encoding::all::UTF_8;
use encoding::types::{DecoderTrap, Encoding};
use hyper::header::{ContentType, Cookie};
use hyper::header::Headers;
use hyper::http::RawStatus;
use hyper::method::Method;
use protocol::JsonPacketStream;
use serde_json::{Map, Value};
use std::borrow::Cow;
use std::net::TcpStream;
use time;
use time::Tm;

struct HttpRequest {
    url: String,
    method: Method,
    headers: Headers,
    body: Option<Vec<u8>>,
    startedDateTime: Tm,
    timeStamp: i64,
    connect_time: u64,
    send_time: u64,
}

struct HttpResponse {
    headers: Option<Headers>,
    status: Option<RawStatus>,
    body: Option<Vec<u8>>
}

pub struct NetworkEventActor {
    pub name: String,
    request: HttpRequest,
    response: HttpResponse,
    is_xhr: bool,
}

#[derive(Serialize)]
pub struct EventActor {
    pub actor: String,
    pub url: String,
    pub method: String,
    pub startedDateTime: String,
    pub timeStamp: i64,
    pub isXHR: bool,
    pub private: bool
}

#[derive(Serialize)]
pub struct ResponseCookiesMsg {
    pub cookies: usize,
}

#[derive(Serialize)]
pub struct ResponseStartMsg {
    pub httpVersion: String,
    pub remoteAddress: String,
    pub remotePort: u32,
    pub status: String,
    pub statusText: String,
    pub headersSize: usize,
    pub discardResponseBody: bool,
}

#[derive(Serialize)]
pub struct ResponseContentMsg {
    pub mimeType: String,
    pub contentSize: u32,
    pub transferredSize: u32,
    pub discardResponseBody: bool,
}


#[derive(Serialize)]
pub struct ResponseHeadersMsg {
    pub headers: usize,
    pub headersSize: usize,
}


#[derive(Serialize)]
pub struct RequestCookiesMsg {
    pub cookies: usize,
}

#[derive(Serialize)]
pub struct RequestHeadersMsg {
    headers: usize,
    headersSize: usize,
}

#[derive(Serialize)]
struct GetRequestHeadersReply {
    from: String,
    headers: Vec<Header>,
    headerSize: usize,
    rawHeaders: String
}

#[derive(Serialize)]
struct Header {
    name: String,
    value: String,
}

#[derive(Serialize)]
struct GetResponseHeadersReply {
    from: String,
    headers: Vec<Header>,
    headerSize: usize,
    rawHeaders: String
}

#[derive(Serialize)]
struct GetResponseContentReply {
    from: String,
    content: Option<Vec<u8>>,
    contentDiscarded: bool,
}

#[derive(Serialize)]
struct GetRequestPostDataReply {
    from: String,
    postData: Option<Vec<u8>>,
    postDataDiscarded: bool
}

#[derive(Serialize)]
struct GetRequestCookiesReply {
    from: String,
    cookies: Vec<u8>
}

#[derive(Serialize)]
struct GetResponseCookiesReply {
    from: String,
    cookies: Vec<u8>
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
struct GetEventTimingsReply {
    from: String,
    timings: Timings,
    totalTime: u64,
}

#[derive(Serialize)]
struct SecurityInfo {
    state: String,
}

#[derive(Serialize)]
struct GetSecurityInfoReply {
    from: String,
    securityInfo: SecurityInfo,
}

impl Actor for NetworkEventActor {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn handle_message(&self,
                      _registry: &ActorRegistry,
                      msg_type: &str,
                      _msg: &Map<String, Value>,
                      stream: &mut TcpStream) -> Result<ActorMessageStatus, ()> {
        Ok(match msg_type {
            "getRequestHeaders" => {
                let mut headers = Vec::new();
                let mut rawHeadersString = "".to_owned();
                let mut headersSize = 0;
                for item in self.request.headers.iter() {
                    let name = item.name();
                    let value = item.value_string();
                    rawHeadersString = rawHeadersString + name + ":" + &value + "\r\n";
                    headersSize += name.len() + value.len();
                    headers.push(Header { name: name.to_owned(), value: value.to_owned() });
                }
                let msg = GetRequestHeadersReply {
                    from: self.name(),
                    headers: headers,
                    headerSize: headersSize,
                    rawHeaders: rawHeadersString,
                };
                stream.write_json_packet(&msg);
                ActorMessageStatus::Processed
            }
            "getRequestCookies" => {
                let mut cookies = Vec::new();
                if let Some(req_cookies) = self.request.headers.get_raw("Cookie") {
                    for cookie in &*req_cookies {
                        if let Ok(cookie_value) = String::from_utf8(cookie.clone()) {
                            cookies = cookie_value.into_bytes();
                        }
                    }
                }

                let msg = GetRequestCookiesReply {
                    from: self.name(),
                    cookies: cookies,
                };
                stream.write_json_packet(&msg);
                ActorMessageStatus::Processed
            }
            "getRequestPostData" => {
                let msg = GetRequestPostDataReply {
                    from: self.name(),
                    postData: self.request.body.clone(),
                    postDataDiscarded: false,
                };
                stream.write_json_packet(&msg);
                ActorMessageStatus::Processed
            }
            "getResponseHeaders" => {
                if let Some(ref response_headers) = self.response.headers {
                    let mut headers = vec![];
                    let mut rawHeadersString = "".to_owned();
                    let mut headersSize = 0;
                    for item in response_headers.iter() {
                        let name = item.name();
                        let value = item.value_string();
                        headers.push(Header {
                            name: name.to_owned(),
                            value: value.clone(),
                        });
                        headersSize += name.len() + value.len();
                        rawHeadersString.push_str(name);
                        rawHeadersString.push_str(":");
                        rawHeadersString.push_str(&value);
                        rawHeadersString.push_str("\r\n");
                    }
                    let msg = GetResponseHeadersReply {
                        from: self.name(),
                        headers: headers,
                        headerSize: headersSize,
                        rawHeaders: rawHeadersString,
                    };
                    stream.write_json_packet(&msg);
                }
                ActorMessageStatus::Processed
            }
            "getResponseCookies" => {
                let mut cookies = Vec::new();
                if let Some(res_cookies) = self.request.headers.get_raw("set-cookie") {
                    for cookie in &*res_cookies {
                        if let Ok(cookie_value) = String::from_utf8(cookie.clone()) {
                            cookies = cookie_value.into_bytes();
                        }
                    }
                }

                let msg = GetResponseCookiesReply {
                    from: self.name(),
                    cookies: cookies,
                };
                stream.write_json_packet(&msg);
                ActorMessageStatus::Processed
            }
            "getResponseContent" => {
                let msg = GetResponseContentReply {
                    from: self.name(),
                    content: self.response.body.clone(),
                    contentDiscarded: self.response.body.is_none(),
                };
                stream.write_json_packet(&msg);
                ActorMessageStatus::Processed
            }
            "getEventTimings" => {
                // TODO: This is a fake timings msg
                let timingsObj = Timings {
                    blocked: 0,
                    dns: 0,
                    connect: self.request.connect_time,
                    send: self.request.send_time,
                    wait: 0,
                    receive: 0,
                };
                let total = timingsObj.connect + timingsObj.send;
                // TODO: Send the correct values for all these fields.
                let msg = GetEventTimingsReply {
                    from: self.name(),
                    timings: timingsObj,
                    totalTime: total,
                };
                stream.write_json_packet(&msg);
                ActorMessageStatus::Processed
            }
            "getSecurityInfo" => {
                // TODO: Send the correct values for securityInfo.
                let msg = GetSecurityInfoReply {
                    from: self.name(),
                    securityInfo: SecurityInfo {
                        state: "insecure".to_owned()
                    },
                };
                stream.write_json_packet(&msg);
                ActorMessageStatus::Processed
            }
            _ => ActorMessageStatus::Ignored
        })
    }
}

impl NetworkEventActor {
    pub fn new(name: String) -> NetworkEventActor {
        NetworkEventActor {
            name: name,
            request: HttpRequest {
                url: String::new(),
                method: Method::Get,
                headers: Headers::new(),
                body: None,
                startedDateTime: time::now(),
                timeStamp: time::get_time().sec,
                send_time: 0,
                connect_time: 0,
            },
            response: HttpResponse {
                headers: None,
                status: None,
                body: None,
            },
            is_xhr: false,
        }
    }

    pub fn add_request(&mut self, request: DevtoolsHttpRequest) {
        self.request.url = request.url.as_str().to_owned();
        self.request.method = request.method.clone();
        self.request.headers = request.headers.clone();
        self.request.body = request.body;
        self.request.startedDateTime = request.startedDateTime;
        self.request.timeStamp = request.timeStamp;
        self.request.connect_time = request.connect_time;
        self.request.send_time = request.send_time;
        self.is_xhr = request.is_xhr;
    }

    pub fn add_response(&mut self, response: DevtoolsHttpResponse) {
        self.response.headers = response.headers.clone();
        self.response.status = response.status.as_ref().map(|&(s, ref st)| {
            let status_text = UTF_8.decode(st, DecoderTrap::Replace).unwrap();
            RawStatus(s, Cow::from(status_text))
        });
        self.response.body = response.body.clone();
    }

    pub fn event_actor(&self) -> EventActor {
        // TODO: Send the correct values for startedDateTime, isXHR, private
        EventActor {
            actor: self.name(),
            url: self.request.url.clone(),
            method: format!("{}", self.request.method),
            startedDateTime: format!("{}", self.request.startedDateTime.rfc3339()),
            timeStamp: self.request.timeStamp,
            isXHR: self.is_xhr,
            private: false,
        }
    }

    pub fn response_start(&self) -> ResponseStartMsg {
        // TODO: Send the correct values for all these fields.
        let hSizeOption = self.response.headers.as_ref().map(|headers| headers.len());
        let hSize = hSizeOption.unwrap_or(0);
        let (status_code, status_message) = self.response.status.as_ref().
                map_or((0, "".to_owned()), |&RawStatus(ref code, ref text)| (*code, text.clone().into_owned()));
        // TODO: Send the correct values for remoteAddress and remotePort and http_version.
        ResponseStartMsg {
            httpVersion: "HTTP/1.1".to_owned(),
            remoteAddress: "63.245.217.43".to_owned(),
            remotePort: 443,
            status: status_code.to_string(),
            statusText: status_message,
            headersSize: hSize,
            discardResponseBody: false
        }
    }

    pub fn response_content(&self) -> ResponseContentMsg {
        let mut mString = "".to_owned();
        if let Some(ref headers) = self.response.headers {
            mString = match headers.get() {
                Some(&ContentType(ref mime)) => mime.to_string(),
                None => "".to_owned()
            };
        }
        // TODO: Set correct values when response's body is sent to the devtools in http_loader.
        ResponseContentMsg {
            mimeType: mString,
            contentSize: 0,
            transferredSize: 0,
            discardResponseBody: true,
        }
    }

    pub fn response_cookies(&self) -> ResponseCookiesMsg {
        let mut cookies_size = 0;
        if let Some(ref headers) = self.response.headers {
            cookies_size = match headers.get() {
                Some(&Cookie(ref cookie)) => cookie.len(),
                None => 0
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
            for item in headers.iter()  {
                headers_byte_count += item.name().len() + item.value_string().len();
            }

        }
        ResponseHeadersMsg {
            headers: headers_size,
            headersSize: headers_byte_count,
        }
    }

    pub fn request_headers(&self) -> RequestHeadersMsg {
        let size = self.request
                       .headers
                       .iter()
                       .fold(0, |acc, h| acc + h.name().len() + h.value_string().len());
        RequestHeadersMsg {
            headers: self.request.headers.len(),
            headersSize: size,
        }
    }

    pub fn request_cookies(&self) -> RequestCookiesMsg {
        let cookies_size = match self.request.headers.get() {
            Some(&Cookie(ref cookie)) => cookie.len(),
            None => 0
        };
        RequestCookiesMsg {
            cookies: cookies_size,
        }
    }

    pub fn total_time(&self) -> u64 {
        self.request.connect_time + self.request.send_time
    }
}
