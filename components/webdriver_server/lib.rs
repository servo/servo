/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![crate_name = "webdriver_server"]
#![crate_type = "rlib"]

#![feature(net)]
#![feature(rustc_private)]

#[macro_use]
extern crate log;

extern crate webdriver;
extern crate msg;
extern crate url;
extern crate util;
extern crate "rustc-serialize" as rustc_serialize;
extern crate uuid;

use msg::constellation_msg::{ConstellationChan, LoadData};
use msg::constellation_msg::Msg as ConstellationMsg;
use std::sync::mpsc::channel;

use url::Url;
use webdriver::command::{WebDriverMessage, WebDriverCommand};
use webdriver::command::GetParameters;
use webdriver::response::{
    WebDriverResponse, NewSessionResponse, ValueResponse};
use webdriver::server::{self, WebDriverHandler, Session};
use webdriver::error::{WebDriverResult, WebDriverError, ErrorStatus};
use util::task::spawn_named;
use uuid::Uuid;

use std::borrow::ToOwned;
use std::net::IpAddr;
use rustc_serialize::json::{Json, ToJson};
use std::collections::BTreeMap;

pub fn start_server(port: u16, constellation_chan: ConstellationChan) {
    let handler = Handler::new(constellation_chan);

    spawn_named("WebdriverHttpServer".to_owned(), move || {
        server::start(IpAddr::new_v4(0, 0, 0, 0), port, handler);
    });
}

struct WebdriverSession {
    id: Uuid
}

struct Handler {
    session: Option<WebdriverSession>,
    constellation_chan: ConstellationChan
}

impl WebdriverSession {
    pub fn new() -> WebdriverSession {
        WebdriverSession {
            id: Uuid::new_v4()
        }
    }
}

impl Handler {
    pub fn new(constellation_chan: ConstellationChan) -> Handler {
        Handler {
            session: None,
            constellation_chan: constellation_chan
        }
    }

    fn handle_new_session(&mut self) -> WebDriverResult<WebDriverResponse> {
        if self.session.is_none() {
            let session = WebdriverSession::new();
            let rv = Ok(WebDriverResponse::NewSession(
                NewSessionResponse::new(
                    session.id.to_string(),
                    Json::Object(BTreeMap::new()))));
            self.session = Some(session);
            rv
        } else {
            Err(WebDriverError::new(ErrorStatus::UnknownError,
                                    "Session already created"))
        }
    }

    fn handle_get(&self, parameters: &GetParameters) -> WebDriverResult<WebDriverResponse> {
        let url = match Url::parse(&parameters.url[..]) {
            Ok(url) => url,
            Err(_) => return Err(WebDriverError::new(ErrorStatus::InvalidArgument,
                                               "Invalid URL"))
        };

        let (sender, reciever) = channel();
        let ConstellationChan(ref const_chan) = self.constellation_chan;
        const_chan.send(ConstellationMsg::GetRootPipeline(sender)).unwrap();

        let pipeline_id = reciever.recv().unwrap().unwrap();

        let load_data = LoadData::new(url);
        const_chan.send(ConstellationMsg::LoadUrl(pipeline_id, load_data)).unwrap();
        //TODO: Now we ought to wait until we get a load event
        Ok(WebDriverResponse::Void)
    }

    fn handle_get_window_handle(&self) -> WebDriverResult<WebDriverResponse> {
        // For now we assume there's only one window so just use the session
        // id as the window id
        let handle = self.session.as_ref().unwrap().id.to_string();
        Ok(WebDriverResponse::Generic(ValueResponse::new(handle.to_json())))
    }
}

impl WebDriverHandler for Handler {
    fn handle_command(&mut self, _session: &Option<Session>, msg: &WebDriverMessage) -> WebDriverResult<WebDriverResponse> {

        match msg.command {
            WebDriverCommand::NewSession => self.handle_new_session(),
            WebDriverCommand::Get(ref parameters) => self.handle_get(parameters),
            WebDriverCommand::GetWindowHandle => self.handle_get_window_handle(),
            _ => Err(WebDriverError::new(ErrorStatus::UnsupportedOperation,
                                         "Command not implemented"))
        }
    }

    fn delete_session(&mut self, _session: &Option<Session>) {
        self.session = None;
    }
}
