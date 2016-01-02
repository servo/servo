/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Liberally derived from the [Firefox JS implementation]
//! (http://mxr.mozilla.org/mozilla-central/source/toolkit/devtools/server/actors/webconsole.js).
//! Handles interaction with the remote web console on network events (HTTP requests, responses) in Servo.

extern crate hyper;

use actor::{Actor, ActorMessageStatus, ActorRegistry};
use devtools_traits::HttpRequest as DevtoolsHttpRequest;
use devtools_traits::HttpResponse as DevtoolsHttpResponse;
use hyper::header::Headers;
use hyper::header::{ContentType, Cookie};
use hyper::http::RawStatus;
use hyper::method::Method;
use protocol::JsonPacketStream;
use rustc_serialize::json;
use std::net::TcpStream;
use time;
use time::Tm;

struct HttpRequest {
    url: String,
    method: Method,
    headers: Headers,
    body: Option<Vec<u8>>,
    startedDateTime: Tm
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
}

#[derive(RustcEncodable)]
pub struct EventActor {
    pub actor: String,
    pub url: String,
    pub method: String,
    pub startedDateTime: String,
    pub isXHR: bool,
    pub private: bool
}

#[derive(RustcEncodable)]
pub struct ResponseCookiesMsg {
    pub cookies: u32,
}

#[derive(RustcEncodable)]
pub struct ResponseStartMsg {
    pub httpVersion: String,
    pub remoteAddress: String,
    pub remotePort: u32,
    pub status: String,
    pub statusText: String,
    pub headersSize: u32,
    pub discardResponseBody: bool,
}

#[derive(RustcEncodable)]
pub struct ResponseContentMsg {
    pub mimeType: String,
    pub contentSize: u32,
    pub transferredSize: u32,
    pub discardResponseBody: bool,
}


#[derive(RustcEncodable)]
pub struct ResponseHeadersMsg {
    pub headers: u32,
    pub headersSize: u32,
}


#[derive(RustcEncodable)]
pub struct RequestCookiesMsg {
    pub cookies: u32,
}

#[derive(RustcEncodable)]
struct GetRequestHeadersReply {
    from: String,
    headers: Vec<String>,
    headerSize: u8,
    rawHeaders: String
}

#[derive(RustcEncodable)]
struct GetResponseHeadersReply {
    from: String,
    headers: Vec<String>,
    headerSize: u8,
    rawHeaders: String
}

#[derive(RustcEncodable)]
struct GetResponseContentReply {
    from: String,
    content: Option<Vec<u8>>,
    contentDiscarded: bool,
}

#[derive(RustcEncodable)]
struct GetRequestPostDataReply {
    from: String,
    postData: Option<Vec<u8>>,
    postDataDiscarded: bool
}

#[derive(RustcEncodable)]
struct GetRequestCookiesReply {
    from: String,
    cookies: Vec<u8>
}

#[derive(RustcEncodable)]
struct GetResponseCookiesReply {
    from: String,
    cookies: Vec<u8>
}

#[derive(RustcEncodable)]
struct Timings {
    blocked: u32,
    dns: u32,
    connect: u32,
    send: u32,
    wait: u32,
    receive: u32,
}

#[derive(RustcEncodable)]
struct GetEventTimingsReply {
    from: String,
    timings: Timings,
    totalTime: u32,
}

#[derive(RustcEncodable)]
struct GetSecurityInfoReply {
    from: String,
    seuritInfo: String,
}


impl Actor for NetworkEventActor {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn handle_message(&self,
                      _registry: &ActorRegistry,
                      msg_type: &str,
                      _msg: &json::Object,
                      stream: &mut TcpStream) -> Result<ActorMessageStatus, ()> {
        Ok(match msg_type {
            "getRequestHeaders" => {
                // TODO: Pass the correct values for headers, headerSize, rawHeaders
                let headersSize = self.request.headers.len() as u8;
                let mut headerNames = Vec::new();
                let mut rawHeadersString = "".to_owned();
                for item in self.request.headers.iter() {
                    let name = item.name();
                    let value = item.value_string();
                    headerNames.push(name.to_owned());
                    rawHeadersString = rawHeadersString + name + ":" + &value + "\r\n";
                }
                let msg = GetRequestHeadersReply {
                    from: self.name(),
                    headers: headerNames,
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
                if let Some(ref headers) = self.response.headers {
                    let headersSize = headers.len() as u8;
                    let mut headerNames = Vec::new();
                    let mut rawHeadersString = "".to_owned();
                    for item in headers.iter()  {
                        let name = item.name();
                        let value = item.value_string();
                        headerNames.push(name.to_owned());
                        rawHeadersString = rawHeadersString + name + ":" + &value + "\r\n";
                    }
                    let msg = GetResponseHeadersReply {
                        from: self.name(),
                        headers: headerNames,
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
                    contentDiscarded: false,
                };
                stream.write_json_packet(&msg);
                ActorMessageStatus::Processed
            }
            "getEventTimings" => {
                // TODO: This is a fake timings msg
                let timingsObj = Timings {
                    blocked: 0,
                    dns: 0,
                    connect: 0,
                    send: 0,
                    wait: 0,
                    receive: 0,
                };
                // TODO: Send the correct values for all these fields.
                let msg = GetEventTimingsReply {
                    from: self.name(),
                    timings: timingsObj,
                    totalTime: 0,
                };
                stream.write_json_packet(&msg);
                ActorMessageStatus::Processed
            }
            "getSecurityInfo" => {
                // TODO: Send the correct values for securityInfo.
                let msg = GetSecurityInfoReply {
                    from: self.name(),
                    seuritInfo: "".to_owned(),
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
            },
            response: HttpResponse {
                headers: None,
                status: None,
                body: None,
            }
        }
    }

    pub fn add_request(&mut self, request: DevtoolsHttpRequest) {
        self.request.url = request.url.serialize();
        self.request.method = request.method.clone();
        self.request.headers = request.headers.clone();
        self.request.body = request.body;
        self.request.startedDateTime = request.startedDateTime;
    }

    pub fn add_response(&mut self, response: DevtoolsHttpResponse) {
        self.response.headers = response.headers.clone();
        self.response.status = response.status.clone();
        self.response.body = response.body.clone();
     }

    pub fn event_actor(&self) -> EventActor {
        // TODO: Send the correct values for startedDateTime, isXHR, private
        EventActor {
            actor: self.name(),
            url: self.request.url.clone(),
            method: format!("{}", self.request.method),
            startedDateTime: format!("{}", self.request.startedDateTime.rfc3339()),
            isXHR: false,
            private: false,
        }
    }

    pub fn response_start(&self) -> ResponseStartMsg {
        // TODO: Send the correct values for all these fields.
        let hSizeOption = self.response.headers.as_ref().map(|headers| headers.len() as u32);
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
            discardResponseBody: false,
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
            cookies: cookies_size as u32,
        }
    }

    pub fn response_headers(&self) -> ResponseHeadersMsg {

        let mut headers_size = 0;
        let mut headers_byte_count = 0;
        if let Some(ref headers) = self.response.headers {
            headers_size = headers.len() as u32;
            for item in headers.iter()  {
                headers_byte_count += item.name().len() + item.value_string().len();
            }

        }
        ResponseHeadersMsg {
            headers: headers_size,
            headersSize: headers_byte_count as u32,
        }
    }

    pub fn request_cookies(&self) -> RequestCookiesMsg {

        let mut cookies_size = 0;
        if let Some(ref headers) = self.response.headers {
            cookies_size = match headers.get() {
                Some(&Cookie(ref cookie)) => cookie.len(),
                None => 0
            };
        }
        RequestCookiesMsg {
            cookies: cookies_size as u32,
        }
    }

}
