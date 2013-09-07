/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use resource_task::{ProgressMsg, Payload, Done, LoaderTask};

use std::cell::Cell;
use std::vec;
use extra::url::Url;
use http::client::RequestWriter;
use http::method::Get;
use http::headers::HeaderEnum;
use std::rt::io::Reader;

pub fn factory() -> LoaderTask {
	let f: LoaderTask = |url, progress_chan| {
        let url = Cell::new(url);
        let progress_chan = Cell::new(progress_chan);
        spawn(|| load(url.take(), progress_chan.take()))
	};
	f
}

fn load(url: Url, progress_chan: Chan<ProgressMsg>) {
	assert!(url.scheme == ~"http");

    info!("requesting %s", url.to_str());

    let request = ~RequestWriter::new(Get, url.clone());
    let mut response = match request.read_response() {
        Ok(r) => r,
        Err(_) => {
            progress_chan.send(Done(Err(())));
            return;
        }
    };

    for header in response.headers.iter() {
        info!(" - %s: %s", header.header_name(), header.header_value());
    }

    loop {
        let mut buf = vec::with_capacity(1024);

        unsafe { vec::raw::set_len(&mut buf, 1024) };
        match response.read(buf) {
            Some(len) => {
                unsafe { vec::raw::set_len(&mut buf, len) };
            }
            None => {
                progress_chan.send(Done(Ok(())));
                return;
            }
        }
        progress_chan.send(Payload(buf));
    }
}
