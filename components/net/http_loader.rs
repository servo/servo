/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use http_cache::{MemoryCache, Uncacheable, CachedContentPending, NewCacheEntry, Revalidate};
use http_cache::{CachedPendingResource, UncachedPendingResource, ResourceResponseTarget};
use http_cache::{UncachedInProgressResource, CachedInProgressResource, ResourceProgressTarget};
use http_cache::{ExpiryDate, Etag};
use resource_task::{Metadata, Payload, Done, LoadResponse, LoadData, start_sending_opt};

use log;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use http::client::{RequestWriter, NetworkStream};
use http::headers::HeaderEnum;
use http::status::NotModified;
use std::io::Reader;
use servo_util::task::spawn_named;
use url::Url;

pub fn factory<'a>(cache: Arc<Mutex<MemoryCache>>)
                   -> proc(load_data: LoadData, start_chan: Sender<LoadResponse>): 'a {
    proc(load_data: LoadData, start_chan: Sender<LoadResponse>) {
        spawn_named("http_loader", proc() load(load_data, start_chan, cache.clone()))
    }
}

fn start_sending_http_opt(start_chan: ResourceResponseTarget, metadata: Metadata)
                          -> Result<ResourceProgressTarget, ()> {
    match start_chan {
        CachedPendingResource(key, cache) => {
            {
                let mut cache = cache.lock();
                cache.process_metadata(&key, metadata);
            }
            Ok(CachedInProgressResource(key, cache))
        }
        UncachedPendingResource(start_chan) =>
            start_sending_opt(start_chan, metadata).map(|chan| {
                UncachedInProgressResource(chan)
            })
    }
}

pub fn send_error_direct(url: Url, err: String, start_chan: Sender<LoadResponse>) {
    match start_sending_opt(start_chan, Metadata::default(url)) {
        Ok(p) => p.send(Done(Err(err))),
        _ => {}
    };
}

fn send_error(url: Url, err: String, start_chan: &ResourceResponseTarget) {
    match *start_chan {
        CachedPendingResource(ref key, ref cache) => {
            let mut cache = cache.lock();
            cache.doom_request(key, err);
        }
        UncachedPendingResource(ref start_chan) => send_error_direct(url, err, start_chan.clone()),
    }
}

fn load(mut load_data: LoadData, start_chan: Sender<LoadResponse>, cache: Arc<Mutex<MemoryCache>>) {
    // FIXME: At the time of writing this FIXME, servo didn't have any central
    //        location for configuration. If you're reading this and such a
    //        repository DOES exist, please update this constant to use it.
    let max_redirects = 50u;
    let mut iters = 0u;
    let mut url = load_data.url.clone();
    let mut redirected_to = HashSet::new();

    info!("checking cache for {}", url);
    let cache_result = {
        let mut cache = cache.lock();
        cache.process_pending_request(&load_data, start_chan.clone())
    };

    let revalidating = match cache_result {
        Revalidate(ref _key, ExpiryDate(ref last_fetched)) => {
            load_data.headers.if_modified_since = Some(last_fetched.clone());
            true
        }

        Revalidate(ref _key, Etag(ref etag)) => {
            load_data.headers.if_none_match = Some(etag.opaque_tag.clone());
            true
        }

        _ => false
    };

    let start_chan = match cache_result {
        Uncacheable(reason) => {
            info!("request for {} can't be cached: {}", url, reason);
            UncachedPendingResource(start_chan)
        }
        CachedContentPending => return,
        NewCacheEntry(key) => {
            info!("new cache entry for {}", url);
            CachedPendingResource(key, cache)
        }
        Revalidate(key, _) => {
            info!("revalidating {}", url);
            CachedPendingResource(key, cache)
        }
    };

    // Loop to handle redirects.
    loop {
        iters = iters + 1;

        if iters > max_redirects {
            send_error(url, "too many redirects".to_string(), &start_chan);
            return;
        }

        if redirected_to.contains(&url) {
            send_error(url, "redirect loop".to_string(), &start_chan);
            return;
        }

        redirected_to.insert(url.clone());

        match url.scheme.as_slice() {
            "http" | "https" => {}
            _ => {
                let s = format!("{:s} request, but we don't support that scheme", url.scheme);
                send_error(url, s, &start_chan);
                return;
            }
        }

        info!("requesting {:s}", url.serialize());

        let request = RequestWriter::<NetworkStream>::new(load_data.method.clone(), url.clone());
        let mut writer = match request {
            Ok(w) => box w,
            Err(e) => {
                send_error(url, e.desc.to_string(), &start_chan);
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
                        send_error(url, e.desc.to_string(), &start_chan);
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
                send_error(url, e.desc.to_string(), &start_chan);
                return;
            }
        };

        // Dump headers, but only do the iteration if info!() is enabled.
        info!("got HTTP response {:s}, headers:", response.status.to_string());
        if log_enabled!(log::INFO) {
            for header in response.headers.iter() {
                info!(" - {:s}: {:s}", header.header_name(), header.header_value());
            }
        }

        if revalidating {
            let (key, cache) = match start_chan {
                CachedPendingResource(ref key, ref cache) => (key, cache),
                UncachedPendingResource(..) => unreachable!(),
            };

            let mut cache = cache.lock();
            if response.status == NotModified && revalidating {
                cache.process_not_modified(key, &response.headers);
                return;
            }

            cache.process_revalidation_failed(key);
        }

        if 3 == (response.status.code() / 100) {
            match response.headers.location {
                Some(new_url) => {
                    // CORS (http://fetch.spec.whatwg.org/#http-fetch, status section, point 9, 10)
                    match load_data.cors {
                        Some(ref c) => {
                            if c.preflight {
                                // The preflight lied
                                send_error(url, "Preflight fetch inconsistent with main fetch".to_string(), &start_chan);
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

        let progress_chan = match start_sending_http_opt(start_chan, metadata) {
            Ok(p) => p,
            _ => return
        };
        loop {
            let mut buf = Vec::with_capacity(1024);

            unsafe { buf.set_len(1024); }
            match response.read(buf.as_mut_slice()) {
                Ok(len) => {
                    unsafe { buf.set_len(len); }
                    match progress_chan {
                        CachedInProgressResource(ref key, ref cache) => {
                            let mut cache = cache.lock();
                            cache.process_payload(key, buf);
                        }
                        UncachedInProgressResource(ref progress_chan) => {
                            if progress_chan.send_opt(Payload(buf)).is_err() {
                                // The send errors when the receiver is out of scope,
                                // which will happen if the fetch has timed out (or has been aborted)
                                // so we don't need to continue with the loading of the file here.
                                return;
                            }
                        }
                    }
                }
                Err(_) => {
                    match progress_chan {
                        CachedInProgressResource(ref key, ref cache) => {
                            let mut cache = cache.lock();
                            cache.process_done(key);
                        }
                        UncachedInProgressResource(ref progress_chan) => {
                            let _ = progress_chan.send_opt(Done(Ok(())));
                        }
                    }
                    break;
                }
            }
        }

        // We didn't get redirected.
        break;
    }
}
