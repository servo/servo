/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use resource_task::{ProgressMsg, Payload, Done, UrlChange, LoaderTask};

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
    assert!("http" == url.scheme);

    info!("requesting %s", url.to_str());

    let request = ~RequestWriter::new(Get, url.clone());
    let mut response = match request.read_response() {
        Ok(r) => r,
        Err(_) => {
            progress_chan.send(Done(Err(())));
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
            progress_chan.send(UrlChange(url.clone()));
            return load(url, progress_chan);
        }
        None => ()
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
