/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Liberally derived from the [Firefox JS implementation]
//! (http://mxr.mozilla.org/mozilla-central/source/toolkit/devtools/server/actors/webconsole.js).
//! Handles interaction with the remote web console on network events (HTTP requests, responses) in Servo.

extern crate hyper;

use actor::{Actor, ActorRegistry, ActorMessageStatus};
use protocol::JsonPacketStream;
use rustc_serialize::json;
use std::net::TcpStream;
use url::Url;
use hyper::header::Headers;
use hyper::http::RawStatus;
use hyper::method::Method;

struct HttpRequest {
    url: String,
    method: Method,
    headers: Headers,
    body: Option<Vec<u8>>,
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
struct GetRequestHeadersReply {
    from: String,
    headers: Vec<String>,
    headerSize: u8,
    rawHeaders: String
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
                let msg = GetRequestHeadersReply {
                    from: self.name(),
                    headers: Vec::new(),
                    headerSize: 10,
                    rawHeaders: "Raw headers".to_string(),
                };
                stream.write_json_packet(&msg);
                ActorMessageStatus::Processed
            }
            "getRequestCookies" => {
                ActorMessageStatus::Ignored
            }
            "getRequestPostData" => {
                ActorMessageStatus::Ignored
            }
            "getResponseHeaders" => {
                ActorMessageStatus::Ignored
            }
            "getResponseCookies" => {
                ActorMessageStatus::Ignored
            }
            "getResponseContent" => {
                ActorMessageStatus::Ignored
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
                body: None
            },
            response: HttpResponse {
                headers: None,
                status: None,
                body: None,
            }
        }
    }

    pub fn add_request(&mut self, url: Url, method: Method, headers: Headers, body: Option<Vec<u8>>) {
        self.request.url = url.serialize();
        self.request.method = method.clone();
        self.request.headers = headers.clone();
        self.request.body = body;
    }

    pub fn add_response(&mut self, headers: Option<Headers>, status: Option<RawStatus>, body: Option<Vec<u8>>) {
        self.response.headers = headers.clone();
        self.response.status = status.clone();
        self.response.body = body.clone();
    }

    pub fn event_actor(&self) -> EventActor {
        // TODO: Send the correct values for startedDateTime, isXHR, private
        EventActor {
            actor: self.name(),
            url: self.request.url.clone(),
            method: format!("{}", self.request.method),
            startedDateTime: "2015-04-22T20:47:08.545Z".to_string(),
            isXHR: false,
            private: false,
        }
    }

    pub fn response_start(&self) -> ResponseStartMsg {
        // TODO: Send the correct values for all these fields.
        //       This is a fake message.
        ResponseStartMsg {
            httpVersion: "HTTP/1.1".to_string(),
            remoteAddress: "63.245.217.43".to_string(),
            remotePort: 443,
            status: "200".to_string(),
            statusText: "OK".to_string(),
            headersSize: 337,
            discardResponseBody: true
        }
    }
}
