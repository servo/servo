/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use http_loader::send_error_direct;
use resource_task::{Metadata, ProgressMsg, LoadResponse, LoadData, Payload, Done, start_sending_opt};

use servo_util::time::parse_http_timestamp;

use http::headers::HeaderEnum;
use http::method::Get;
use http::status::Ok as StatusOk;

use std::collections::HashMap;
use std::comm::Sender;
use std::iter::Map;
use std::num::FromStrRadix;
use std::str::CharSplits;
use std::sync::{Arc, Mutex};
use std::time::duration::{MAX, Duration};
use time;
use time::Timespec;
use url::Url;

//TODO: Store an Arc<Vec<u8>> instead?
//TODO: Cache non-GET requests?
//TODO: Doom responses with network errors
//TODO: Send Err responses for doomed entries
//TODO: Enable forced eviction of a request instead of retrieving the cached response
//TODO: Evict items based on expiration time
//TODO: Use If-Modified-Since, Etag, etc.
//TODO: Doom incomplete entries

#[deriving(Clone, Hash, PartialEq, Eq)]
pub struct CacheKey {
    url: Url,
    request_headers: Vec<(String, String)>,
}

impl CacheKey {
    fn new(load_data: LoadData) -> CacheKey {
        CacheKey {
            url: load_data.url.clone(),
            request_headers: load_data.headers
                                      .iter()
                                      .map(|header| (header.header_name(), header.header_value()))
                                      .collect(),
        }
    }

    pub fn url(&self) -> Url {
        self.url.clone()
    }
}

enum PendingConsumers {
    AwaitingHeaders(Vec<Sender<LoadResponse>>),
    AwaitingBody(Vec<Sender<ProgressMsg>>),
}

struct PendingResource {
    metadata: Option<Metadata>,
    body: Vec<u8>,
    consumers: PendingConsumers,
    expires: Duration,
    doomed: bool,
}

struct CachedResource {
    metadata: Metadata,
    body: Vec<u8>,
    expires: Duration,
}

pub struct MemoryCache {
    complete_entries: HashMap<CacheKey, CachedResource>,
    pending_entries: HashMap<CacheKey, PendingResource>,
    base_time: Timespec,
}

pub enum ResourceResponseTarget {
    CachedPendingResource(CacheKey, Arc<Mutex<MemoryCache>>),
    UncachedPendingResource(Sender<LoadResponse>),
}

pub enum ResourceProgressTarget {
    CachedInProgressResource(CacheKey, Arc<Mutex<MemoryCache>>),
    UncachedInProgressResource(Sender<ProgressMsg>),
}

pub enum CacheOperationResult {
    Uncacheable(&'static str),
    CachedContentPending,
    NewCacheEntry(CacheKey),
}

fn split_header(header: &str) -> Map<&str, &str, CharSplits<char>> {
    header.split(',')
          .map(|v| v.trim())
}

fn response_is_cacheable(metadata: &Metadata) -> bool {
    if metadata.status != StatusOk {
        return false;
    }

    if metadata.headers.is_none() {
        return true;
    }

    fn any_token_matches(header: &str, tokens: &[&str]) -> bool {
        split_header(header).any(|token| tokens.iter().any(|&s| s == token))
    }

    let headers = metadata.headers.as_ref().unwrap();
    match headers.cache_control {
        Some(ref cache_control) => {
            if any_token_matches(cache_control[], &["no-cache", "no-store", "max-age=0"]) {
                return false;
            }
        }
        None => ()
    }

    match headers.pragma {
        Some(ref pragma) => {
            if any_token_matches(pragma[], &["no-cache"]) {
                return false;
            }
        }
        None => ()
    }

    return true;
}

fn get_response_expiry(metadata: &Metadata) -> Duration {
    metadata.headers.as_ref().and_then(|headers| {
        headers.cache_control.as_ref().and_then(|cache_control| {
            for token in split_header(cache_control[]) {
                let mut parts = token.split('=');
                if parts.next().unwrap() == "max-age" {
                    return parts.next()
                                .and_then(|val| FromStrRadix::from_str_radix(val, 10))
                                .map(|secs| Duration::seconds(secs));
                }
            }
            None
        }).or_else(|| {
            headers.expires.as_ref().and_then(|expires| {
                parse_http_timestamp(expires[]).map(|t| {
                    Duration::seconds(t.to_timespec().sec)
                })
            })
        })
    }).unwrap_or(MAX)
}

impl MemoryCache {
    pub fn new() -> MemoryCache {
        MemoryCache {
            complete_entries: HashMap::new(),
            pending_entries: HashMap::new(),
            base_time: time::now().to_timespec(),
        }
    }

    pub fn doom_request(&mut self, key: &CacheKey, err: String) {
        info!("dooming entry for {}", key.url);
        let resource = self.pending_entries.remove(key).unwrap();
        match resource.consumers {
            AwaitingHeaders(ref consumers) => {
                for consumer in consumers.iter() {
                    send_error_direct(key.url.clone(), err.clone(), consumer.clone());
                }
            }
            AwaitingBody(ref consumers) => {
                for consumer in consumers.iter() {
                    let _ = consumer.send_opt(Done(Ok(())));
                }
            }
        }
    }

    pub fn process_metadata(&mut self, key: &CacheKey, metadata: Metadata) {
        info!("storing metadata for {}", key.url);
        let resource = self.pending_entries.get_mut(key).unwrap();
        let chans: Vec<Sender<ProgressMsg>>;
        match resource.consumers {
            AwaitingHeaders(ref consumers) => {
                chans = consumers.iter()
                                 .map(|chan| start_sending_opt(chan.clone(), metadata.clone()))
                                 .take_while(|chan| chan.is_ok())
                                 .map(|chan| chan.unwrap())
                                 .collect();
            }
            AwaitingBody(_) => panic!("obtained headers for {} but awaiting body?", key.url)
        }

        if !response_is_cacheable(&metadata) {
            resource.doomed = true;
        }

        resource.expires = get_response_expiry(&metadata);
        resource.metadata = Some(metadata);
        resource.consumers = AwaitingBody(chans);
    }

    pub fn process_payload(&mut self, key: &CacheKey, payload: Vec<u8>) {
        info!("storing partial response for {}", key.url);
        let resource = self.pending_entries.get_mut(key).unwrap();
        resource.body.push_all(payload.as_slice());
        match resource.consumers {
            AwaitingBody(ref consumers) => {
                for consumer in consumers.iter() {
                    //FIXME: maybe remove consumer on failure to avoid extra clones?
                    let _ = consumer.send_opt(Payload(payload.clone()));
                }
            }
            AwaitingHeaders(_) => panic!("obtained body for {} but awaiting headers?", key.url)
        }
    }

    pub fn process_done(&mut self, key: &CacheKey) {
        info!("finished fetching {}", key.url);
        let resource = self.pending_entries.remove(key).unwrap();
        match resource.consumers {
            AwaitingHeaders(_) => panic!("saw Done for {} but awaiting headers?", key.url),
            AwaitingBody(ref consumers) => {
                for consumer in consumers.iter() {
                    let _ = consumer.send_opt(Done(Ok(())));
                }
            }
        }

        if resource.doomed {
            info!("completing dooming of {}", key.url);
            return;
        }

        let complete = CachedResource {
            metadata: resource.metadata.unwrap(),
            body: resource.body,
            expires: resource.expires,
        };
        self.complete_entries.insert(key.clone(), complete);
    }

    pub fn process_pending_request(&mut self, load_data: &LoadData, start_chan: Sender<LoadResponse>)
                                   -> CacheOperationResult {
        if load_data.method != Get {
            return Uncacheable("Only GET requests can be cached.");
        }

        let key = CacheKey::new(load_data.clone());
        let expired = self.complete_entries.get(&key).map(|resource| {
            self.base_time + resource.expires >= time::now().to_timespec()
        });

        match expired {
            Some(true) => {
                info!("evicting existing entry for {}", load_data.url);
                self.complete_entries.remove(&key);
            }

            Some(false) => {
                self.send_complete_entry(key, start_chan);
                return CachedContentPending;
            }

            None => ()
        }

        let new_entry = match self.pending_entries.get(&key) {
            Some(resource) if resource.doomed => return Uncacheable("Cache entry already doomed"),
            Some(_) => false,
            None => true,
        };

        if new_entry {
            self.add_pending_cache_entry(key.clone(), start_chan);
            NewCacheEntry(key)
        } else {
            self.send_partial_entry(key, start_chan);
            CachedContentPending
        }
    }

    fn add_pending_cache_entry(&mut self, key: CacheKey, start_chan: Sender<LoadResponse>) {
        let resource = PendingResource {
            metadata: None,
            body: vec!(),
            consumers: AwaitingHeaders(vec!(start_chan)),
            expires: MAX,
            doomed: false,
        };
        info!("creating cache entry for {}", key.url);
        self.pending_entries.insert(key, resource);
    }

    fn send_complete_entry(&self, key: CacheKey, start_chan: Sender<LoadResponse>) {
        info!("returning full cache body for {}", key.url);
        let resource = self.complete_entries.get(&key).unwrap();
        let progress_chan = start_sending_opt(start_chan, resource.metadata.clone());
        match progress_chan {
            Ok(chan) => {
                let _ = chan.send_opt(Payload(resource.body.clone()));
                let _ = chan.send_opt(Done(Ok(())));
            }
            Err(_) => ()
        }
    }

    fn send_partial_entry(&mut self, key: CacheKey, start_chan: Sender<LoadResponse>) {
        info!("returning partial cache data for {}", key.url);

        let resource = self.pending_entries.get_mut(&key).unwrap();

        let metadata = resource.metadata.clone();
        match metadata {
            Some(metadata) => {
                info!("headers available for {}", key.url);
                let progress_chan = start_sending_opt(start_chan, metadata);
                match progress_chan {
                    Ok(chan) => {
                        match resource.consumers {
                            AwaitingHeaders(_) =>
                                panic!("cached metadata, but consumers awaiting headers"),
                            AwaitingBody(ref mut consumers) => consumers.push(chan.clone()),
                        }

                        if !resource.body.is_empty() {
                            info!("partial body available for {}", key.url);
                            let _ = chan.send_opt(Payload(resource.body.clone()));
                        }
                    }

                    Err(_) => ()
                }
            }

            None => {
                match resource.consumers {
                    AwaitingHeaders(ref mut consumers) => consumers.push(start_chan),
                    AwaitingBody(_) => panic!("no cached metadata, but consumers awaiting body"),
                }
            }
        }
    }
}
