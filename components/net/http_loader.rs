/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use resource_task::{Metadata, TargetedLoadResponse, LoadData, start_sending_opt, ResponseSenders};
use resource_task::ProgressMsg::{Payload, Done};

use log;
use std::collections::HashSet;
use hyper::client::Request;
use hyper::header::common::{ContentLength, ContentType, Host, Location};
use hyper::method::Method;
use hyper::status::StatusClass;
use std::error::Error;
use std::io::Reader;
use std::sync::mpsc::Sender;
use util::task::spawn_named;
use url::{Url, UrlParser};

use std::borrow::ToOwned;

pub fn factory(load_data: LoadData, start_chan: Sender<TargetedLoadResponse>) {
    spawn_named("http_loader".to_owned(), move || load(load_data, start_chan))
}

fn send_error(url: Url, err: String, senders: ResponseSenders) {
    let mut metadata = Metadata::default(url);
    metadata.status = None;

    match start_sending_opt(senders, metadata) {
        Ok(p) => p.send(Done(Err(err))).unwrap(),
        _ => {}
    };
}

fn load(load_data: LoadData, start_chan: Sender<TargetedLoadResponse>) {
    // FIXME: At the time of writing this FIXME, servo didn't have any central
    //        location for configuration. If you're reading this and such a
    //        repository DOES exist, please update this constant to use it.
    let max_redirects = 50u;
    let mut iters = 0u;
    let mut url = load_data.url.clone();
    let mut redirected_to = HashSet::new();

    let senders = ResponseSenders {
        immediate_consumer: start_chan,
        eventual_consumer: load_data.consumer
    };

    // Loop to handle redirects.
    loop {
        iters = iters + 1;

        if iters > max_redirects {
            send_error(url, "too many redirects".to_string(), senders);
            return;
        }

        if redirected_to.contains(&url) {
            send_error(url, "redirect loop".to_string(), senders);
            return;
        }

        redirected_to.insert(url.clone());

        match url.scheme.as_slice() {
            "http" | "https" => {}
            _ => {
                let s = format!("{} request, but we don't support that scheme", url.scheme);
                send_error(url, s, senders);
                return;
            }
        }

        info!("requesting {}", url.serialize());

        let mut req = match Request::new(load_data.method.clone(), url.clone()) {
            Ok(req) => req,
            Err(e) => {
                send_error(url, e.description().to_string(), senders);
                return;
            }
        };

        // Preserve the `host` header set automatically by Request.
        let host = req.headers().get::<Host>().unwrap().clone();
        *req.headers_mut() = load_data.headers.clone();
        req.headers_mut().set(host);
        // FIXME(seanmonstar): use AcceptEncoding from Hyper once available
        //if !req.headers.has::<AcceptEncoding>() {
            // We currently don't support HTTP Compression (FIXME #2587)
            req.headers_mut().set_raw("Accept-Encoding".to_owned(), vec![b"identity".to_vec()]);
        //}
        let writer = match load_data.data {
            Some(ref data) => {
                req.headers_mut().set(ContentLength(data.len() as u64));
                let mut writer = match req.start() {
                    Ok(w) => w,
                    Err(e) => {
                        send_error(url, e.description().to_string(), senders);
                        return;
                    }
                };
                match writer.write(data.as_slice()) {
                    Err(e) => {
                        send_error(url, e.desc.to_string(), senders);
                        return;
                    }
                    _ => {}
                };
                writer
            },
            None => {
                match load_data.method {
                    Method::Get | Method::Head => (),
                    _ => req.headers_mut().set(ContentLength(0))
                }
                match req.start() {
                    Ok(w) => w,
                    Err(e) => {
                        send_error(url, e.description().to_string(), senders);
                        return;
                    }
                }
            }
        };
        let mut response = match writer.send() {
            Ok(r) => r,
            Err(e) => {
                send_error(url, e.description().to_string(), senders);
                return;
            }
        };

        // Dump headers, but only do the iteration if info!() is enabled.
        info!("got HTTP response {}, headers:", response.status);
        if log_enabled!(log::INFO) {
            for header in response.headers.iter() {
                info!(" - {}", header);
            }
        }

        if response.status.class() == StatusClass::Redirection {
            match response.headers.get::<Location>() {
                Some(&Location(ref new_url)) => {
                    // CORS (http://fetch.spec.whatwg.org/#http-fetch, status section, point 9, 10)
                    match load_data.cors {
                        Some(ref c) => {
                            if c.preflight {
                                // The preflight lied
                                send_error(url, "Preflight fetch inconsistent with main fetch".to_string(), senders);
                                return;
                            } else {
                                // XXXManishearth There are some CORS-related steps here,
                                // but they don't seem necessary until credentials are implemented
                            }
                        }
                        _ => {}
                    }
                    let new_url = match UrlParser::new().base_url(&url).parse(new_url.as_slice()) {
                        Ok(u) => u,
                        Err(e) => {
                            send_error(url, e.to_string(), senders);
                            return;
                        }
                    };
                    info!("redirecting to {}", new_url);
                    url = new_url;
                    continue;
                }
                None => ()
            }
        }

        let mut metadata = Metadata::default(url);
        metadata.set_content_type(match response.headers.get() {
            Some(&ContentType(ref mime)) => Some(mime),
            None => None
        });
        metadata.headers = Some(response.headers.clone());
        metadata.status = Some(response.status_raw().clone());

        let progress_chan = match start_sending_opt(senders, metadata) {
            Ok(p) => p,
            _ => return
        };
        loop {
            let mut buf = Vec::with_capacity(1024);

            unsafe { buf.set_len(1024); }
            match response.read(buf.as_mut_slice()) {
                Ok(len) => {
                    unsafe { buf.set_len(len); }
                    if progress_chan.send(Payload(buf)).is_err() {
                        // The send errors when the receiver is out of scope,
                        // which will happen if the fetch has timed out (or has been aborted)
                        // so we don't need to continue with the loading of the file here.
                        return;
                    }
                }
                Err(_) => {
                    let _ = progress_chan.send(Done(Ok(())));
                    break;
                }
            }
        }

        // We didn't get redirected.
        break;
    }
}
