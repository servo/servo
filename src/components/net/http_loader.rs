/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use resource_task::{Metadata, Payload, Done, LoadResponse, LoaderTask, start_sending};

use std::cell::Cell;
use std::vec;
use extra::url::Url;
use http::client::RequestWriter;
use http::method::Get;
use http::headers::HeaderEnum;
use std::rt::io::Reader;

pub fn factory() -> LoaderTask {
    let f: LoaderTask = |url, start_chan| {
        let url = Cell::new(url);
        let start_chan = Cell::new(start_chan);
        spawn(|| load(url.take(), start_chan.take()))
    };
    f
}

fn load(url: Url, start_chan: Chan<LoadResponse>) {
    assert!("http" == url.scheme);

    info!("requesting %s", url.to_str());

    let request = ~RequestWriter::new(Get, url.clone());
    let mut response = match request.read_response() {
        Ok(r) => r,
        Err(_) => {
            start_sending(start_chan, Metadata::default(url)).send(Done(Err(())));
            return;
        }
    };

    info!("got HTTP response %s, headers:", response.status.to_str())

    let is_redirect = 3 == (response.status.code() / 100);
    let mut redirect: Option<Url> = None;
    for header in response.headers.iter() {
        let name  = header.header_name();
        let value = header.header_value();
        info!(" - %s: %s", name, value);
        if is_redirect && ("Location" == name) {
            redirect = Some(FromStr::from_str(value).expect("Failed to parse redirect URL"));
        }
    }

    // FIXME: detect redirect loops
    match redirect {
        Some(url) => {
            info!("redirecting to %s", url.to_str());
            return load(url, start_chan);
        }
        None => ()
    }

    let mut metadata = Metadata::default(url);
    // We will set other fields here.

    let progress_chan = start_sending(start_chan, metadata);
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
