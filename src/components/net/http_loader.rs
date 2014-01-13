/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use resource_task::{Metadata, Payload, Done, LoadResponse, LoaderTask, start_sending};

use std::vec;
use extra::url::Url;
use http::client::RequestWriter;
use http::method::Get;
use http::headers::HeaderEnum;
use std::io::Reader;

pub fn factory() -> LoaderTask {
    let f: LoaderTask = proc(url, start_chan) {
        spawn(proc() load(url, start_chan))
    };
    f
}

fn load(mut url: Url, start_chan: Chan<LoadResponse>) {
    // Loop to handle redirects.
    loop {
        assert!("http" == url.scheme);

        info!("requesting {:s}", url.to_str());

        let request = ~RequestWriter::new(Get, url.clone());
        let mut response = match request.read_response() {
            Ok(r) => r,
            Err(_) => {
                start_sending(start_chan, Metadata::default(url)).send(Done(Err(())));
                return;
            }
        };

        // Dump headers, but only do the iteration if info!() is enabled.
        info!("got HTTP response {:s}, headers:", response.status.to_str());
        info!("{:?}",
            for header in response.headers.iter() {
                info!(" - {:s}: {:s}", header.header_name(), header.header_value());
            });

        // FIXME: detect redirect loops
        if 3 == (response.status.code() / 100) {
            match response.headers.location {
                Some(new_url) => {
                    info!("redirecting to {:s}", new_url.to_str());
                    url = new_url;
                    continue;
                }
                None => ()
            }
        }

        let mut metadata = Metadata::default(url);
        metadata.set_content_type(&response.headers.content_type);

        let progress_chan = start_sending(start_chan, metadata);
        loop {
            let mut buf = vec::with_capacity(1024);

            unsafe { buf.set_len(1024); }
            match response.read(buf) {
                Some(len) => {
                    unsafe { buf.set_len(len); }
                    progress_chan.send(Payload(buf));
                }
                None => {
                    progress_chan.send(Done(Ok(())));
                    break;
                }
            }
        }

        // We didn't get redirected.
        break;
    }
}
