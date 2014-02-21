/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use resource_task::{Metadata, Payload, Done, LoadResponse, LoaderTask, start_sending};

use std::vec;
use std::hashmap::HashSet;
use extra::url::Url;
use http::client::RequestWriter;
use http::method::Get;
use http::headers::HeaderEnum;
use std::io::Reader;
use servo_util::task::spawn_named;

pub fn factory() -> LoaderTask {
    let f: LoaderTask = proc(url, start_chan) {
        spawn_named("http_loader", proc() load(url, start_chan))
    };
    f
}

fn send_error(url: Url, start_chan: Chan<LoadResponse>) {
    start_sending(start_chan, Metadata::default(url)).send(Done(Err(())));
}

fn load(mut url: Url, start_chan: Chan<LoadResponse>) {
    // FIXME: At the time of writing this FIXME, servo didn't have any central
    //        location for configuration. If you're reading this and such a
    //        repository DOES exist, please update this constant to use it.
    let max_redirects = 50u;
    let mut iters = 0u;

    let mut redirected_to = HashSet::new();

    // Loop to handle redirects.
    loop {
        iters = iters + 1;

        if iters > max_redirects {
            info!("too many redirects");
            send_error(url, start_chan);
            return;
        }

        if redirected_to.contains(&url) {
            info!("redirect loop");
            send_error(url, start_chan);
            return;
        }

        redirected_to.insert(url.clone());

        assert!("http" == url.scheme);

        info!("requesting {:s}", url.to_str());

        let request = ~RequestWriter::new(Get, url.clone());
        let mut response = match request.read_response() {
            Ok(r) => r,
            Err(_) => {
                send_error(url, start_chan);
                return;
            }
        };

        // Dump headers, but only do the iteration if info!() is enabled.
        info!("got HTTP response {:s}, headers:", response.status.to_str());
        info!("{:?}",
            for header in response.headers.iter() {
                info!(" - {:s}: {:s}", header.header_name(), header.header_value());
            });

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
