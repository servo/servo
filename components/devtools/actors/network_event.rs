/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

/// Liberally derived from the [Firefox JS implementation](http://mxr.mozilla.org/mozilla-central/source/toolkit/devtools/server/actors/webconsole.js).
/// Mediates interaction between the remote web console and equivalent functionality (object
/// inspection, JS evaluation, autocompletion) in Servo.

extern crate hyper;
extern crate url;

use actor::{Actor, ActorRegistry};
use protocol::JsonPacketStream;

use devtools_traits::DevtoolScriptControlMsg;
use msg::constellation_msg::PipelineId;

use collections::BTreeMap;
use core::cell::RefCell;
use rustc_serialize::json::{self, Json, ToJson};
use std::net::TcpStream;
use std::num::Float;
use std::sync::mpsc::{channel, Sender};

use url::Url;
use hyper::header::Headers;
use hyper::http::RawStatus;
use hyper::method::Method;

#[derive(RustcEncodable)]
pub struct HttpRequest {
    pub url: Url,
    //method: Method,
    //headers: Headers,
    pub body: Option<Vec<u8>>,
}

#[derive(RustcEncodable)]
pub struct NetworkEventActor {
    pub name: String,
    pub request: HttpRequest,
}

impl Actor for NetworkEventActor {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn handle_message(&self,
                      _registry: &ActorRegistry,
                      msg_type: &str,
                      msg: &json::Object,
                      stream: &mut TcpStream) -> Result<bool, ()> {
        Ok(match msg_type {

            "getRequestHeaders" => {
                //stream.write_json_packet(&msg);
                true
            }

            "getRequestCookies" => {
                true
            }

            _ => false
        })
    }
}
