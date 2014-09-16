/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use resource_task::{Metadata, Payload, Done, LoadResponse, LoadData, start_sending_opt};

use std::collections::hashmap::HashSet;
use http::client::{RequestWriter, NetworkStream};
use http::headers::HeaderEnum;
use std::io::Reader;
use servo_util::task::spawn_named;
use url::Url;

pub fn factory(load_data: LoadData, start_chan: Sender<LoadResponse>) {
    spawn_named("http_loader", proc() load(load_data, start_chan))
}

fn send_error(url: Url, err: String, start_chan: Sender<LoadResponse>) {
    match start_sending_opt(start_chan, Metadata::default(url)) {
        Ok(p) => p.send(Done(Err(err))),
        _ => {}
    };
}

fn load(load_data: LoadData, start_chan: Sender<LoadResponse>) {
    // FIXME: At the time of writing this FIXME, servo didn't have any central
    //        location for configuration. If you're reading this and such a
    //        repository DOES exist, please update this constant to use it.
    let max_redirects = 50u;
    let mut iters = 0u;
    let mut url = load_data.url.clone();
    let mut redirected_to = HashSet::new();

    // Loop to handle redirects.
    loop {
        iters = iters + 1;

        if iters > max_redirects {
            send_error(url, "too many redirects".to_string(), start_chan);
            return;
        }

        if redirected_to.contains(&url) {
            send_error(url, "redirect loop".to_string(), start_chan);
            return;
        }

        redirected_to.insert(url.clone());

        match url.scheme.as_slice() {
            "http" | "https" => {}
            _ => {
                let s = format!("{:s} request, but we don't support that scheme", url.scheme);
                send_error(url, s, start_chan);
                return;
            }
        }

        info!("requesting {:s}", url.serialize());

        let request = RequestWriter::<NetworkStream>::new(load_data.method.clone(), url.clone());
        let mut writer = match request {
            Ok(w) => box w,
            Err(e) => {
                send_error(url, e.desc.to_string(), start_chan);
                return;
            }
        };

        // Preserve the `host` header set automatically by RequestWriter.
        let host = writer.headers.host.clone();
        writer.headers = load_data.headers.clone();
        writer.headers.host = host;
        if writer.headers.accept_encoding.is_none() {
            // We currently don't support HTTP Compression (FIXME #2587)
            writer.headers.accept_encoding = Some(String::from_str("identity".as_slice()))
        }
        match load_data.data {
            Some(ref data) => {
                writer.headers.content_length = Some(data.len());
                match writer.write(data.as_slice()) {
                    Err(e) => {
                        send_error(url, e.desc.to_string(), start_chan);
                        return;
                    }
                    _ => {}
                }
            },
            _ => {}
        }
        let mut response = match writer.read_response() {
            Ok(r) => r,
            Err((_, e)) => {
                send_error(url, e.desc.to_string(), start_chan);
                return;
            }
        };

        // Dump headers, but only do the iteration if info!() is enabled.
        info!("got HTTP response {:s}, headers:", response.status.to_string());
        info!("{:?}",
            for header in response.headers.iter() {
                info!(" - {:s}: {:s}", header.header_name(), header.header_value());
            });

        if 3 == (response.status.code() / 100) {
            match response.headers.location {
                Some(new_url) => {
                    // CORS (http://fetch.spec.whatwg.org/#http-fetch, status section, point 9, 10)
                    match load_data.cors {
                        Some(ref c) => {
                            if c.preflight {
                                // The preflight lied
                                send_error(url, "Preflight fetch inconsistent with main fetch".to_string(), start_chan);
                                return;
                            } else {
                                // XXXManishearth There are some CORS-related steps here,
                                // but they don't seem necessary until credentials are implemented
                            }
                        }
                        _ => {}
                    }
                    info!("redirecting to {:s}", new_url.serialize());
                    url = new_url;
                    continue;
                }
                None => ()
            }
        }

        let mut metadata = Metadata::default(url);
        metadata.set_content_type(&response.headers.content_type);
        metadata.headers = Some(response.headers.clone());
        metadata.status = response.status.clone();

        let progress_chan = match start_sending_opt(start_chan, metadata) {
            Ok(p) => p,
            _ => return
        };
        loop {
            let mut buf = Vec::with_capacity(1024);

            unsafe { buf.set_len(1024); }
            match response.read(buf.as_mut_slice()) {
                Ok(len) => {
                    unsafe { buf.set_len(len); }
                    if progress_chan.send_opt(Payload(buf)).is_err() {
                        // The send errors when the receiver is out of scope,
                        // which will happen if the fetch has timed out (or has been aborted)
                        // so we don't need to continue with the loading of the file here.
                        return;
                    }
                }
                Err(_) => {
                    let _ = progress_chan.send_opt(Done(Ok(())));
                    break;
                }
            }
        }

        // We didn't get redirected.
        break;
    }
}
