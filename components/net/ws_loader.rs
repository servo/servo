/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use resource_task::{Done, LoadResponse, LoadData, start_sending};
use http_loader;
use http::headers::HeaderEnum;
use servo_util::task::spawn_named;
use std::ascii::AsciiExt;
use std::comm::{channel, Sender};

pub fn factory(load_data: LoadData, start_chan: Sender<LoadResponse>) {
    spawn_named("ws_loader", proc() load(load_data, start_chan))
}

fn load(load_data: LoadData, start_chan: Sender<LoadResponse>) {    
    let(sen, rec) = channel();
    http_loader::load(load_data, sen);
    let response=rec.recv();
    let mut flag: int = 0;

    response.metadata.headers.as_ref().map(|headers| {
        let header = headers.iter().find(|h|
            h.header_name().as_slice().to_ascii_lower() == "upgrade".to_string()
        );        
        match header {
            Some(h) => {    if h.header_value().as_slice().to_ascii_lower() == "websocket".to_string()
                            {
                                flag = flag + 1
                            }
                       },
            None => {}
        }
    });

    let progress_chan = start_sending(start_chan, response.metadata);
    if flag == 1
    {
       progress_chan.send(Done(Ok(()))); 
    } else {
       progress_chan.send(Done(Err("invalid upgrade header value".to_string())));
    }
}
