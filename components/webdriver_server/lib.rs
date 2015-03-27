/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![crate_name = "webdriver_server"]
#![crate_type = "rlib"]

#![feature(net)]

extern crate webdriver;

use webdriver::command::WebDriverMessage;
use webdriver::error::WebDriverResult;
use webdriver::response::WebDriverResponse;
use webdriver::server::{self, WebDriverHandler, Session};

use std::net::IpAddr;

pub fn start_server(port: u16) {
    server::start(IpAddr::new_v4(0, 0, 0, 0), port, Handler);
}

struct Handler;

impl WebDriverHandler for Handler {
    fn handle_command(&mut self, _session: &Option<Session>, _msg: &WebDriverMessage) -> WebDriverResult<WebDriverResponse> {
        Ok(WebDriverResponse::Void)
    }

    fn delete_session(&mut self, _session: &Option<Session>) {
    }
}
