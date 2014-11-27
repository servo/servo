/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use resource_task::{LoadResponse, LoadData};
use servo_util::task::spawn_named;
use http_loader;
use std::comm::{channel, Sender};

use http::headers::HeaderEnum;



use std::ascii::AsciiExt;

pub fn factory(load_data: LoadData, _start_chan: Sender<LoadResponse>) {
    spawn_named("ws_loader", proc() load(load_data, _start_chan))
}


fn load(load_data: LoadData, _start_chan: Sender<LoadResponse>) {
let(sen,rec)=channel();
http_loader::load(load_data, sen);
let response=rec.recv();

response.metadata.headers.as_ref().map(|headers| {
headers.iter().find(|h|
h.header_name().as_slice().to_ascii_lower().to_string() == "upgrade".to_string() && h.header_value().as_slice().to_ascii_lower().to_string() == "websocket".to_string()

);



});

}

 

