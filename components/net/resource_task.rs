/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! A task that takes a URL and streams back the binary data.

use about_loader;
use data_loader;
use file_loader;
use http_loader;
use cookie_storage::CookieStorage;
use cookie;
use mime_classifier::MIMEClassifier;

use net_traits::{ControlMsg, LoadData, LoadResponse, LoadConsumer};
use net_traits::{Metadata, ProgressMsg, ResourceTask, AsyncResponseTarget, ResponseAction, CookieSource};
use net_traits::ProgressMsg::Done;
use util::opts;
use util::task::spawn_named;
use util::resource_files::read_resource_file;
use url::Url;

use devtools_traits::{DevtoolsControlMsg};
use hyper::header::{ContentType, Header, SetCookie, UserAgent};
use hyper::mime::{Mime, TopLevel, SubLevel};

use rustc_serialize::json::{decode};

use regex::Regex;
use std::borrow::ToOwned;
use std::boxed::FnBox;
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{BufReader, Read};
use std::str::{from_utf8};
use std::sync::Arc;
use std::sync::mpsc::{channel, Receiver, Sender};
use time;

static mut HOST_TABLE: Option<*mut HashMap<String, String>> = None;
static IPV4_REGEX: Regex = regex!(
    r"^(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)$"
);
static IPV6_REGEX: Regex = regex!(r"^([a-fA-F0-9]{0,4}[:]?){1,8}(/\d{1,3})?$");

pub fn global_init() {
    //TODO: handle bad file path
    let path = match env::var("HOST_FILE") {
        Ok(host_file_path) => host_file_path,
        Err(_) => return,
    };

    let mut file = match File::open(&path) {
        Ok(f) => BufReader::new(f),
        Err(_) => return,
    };

    let mut lines = String::new();
    match file.read_to_string(&mut lines) {
        Ok(_) => (),
        Err(_) => return,
    };

    let host_table = Box::into_raw(parse_hostsfile(&lines));
    unsafe {
        HOST_TABLE = Some(host_table);
    }
}

pub enum ProgressSender {
    Channel(Sender<ProgressMsg>),
    Listener(Box<AsyncResponseTarget>),
}

impl ProgressSender {
    //XXXjdm return actual error
    pub fn send(&self, msg: ProgressMsg) -> Result<(), ()> {
        match *self {
            ProgressSender::Channel(ref c) => c.send(msg).map_err(|_| ()),
            ProgressSender::Listener(ref b) => {
                let action = match msg {
                    ProgressMsg::Payload(buf) => ResponseAction::DataAvailable(buf),
                    ProgressMsg::Done(status) => ResponseAction::ResponseComplete(status),
                };
                b.invoke_with_listener(action);
                Ok(())
            }
        }
    }
}

/// For use by loaders in responding to a Load message.
pub fn start_sending(start_chan: LoadConsumer, metadata: Metadata) -> ProgressSender {
    start_sending_opt(start_chan, metadata).ok().unwrap()
}

/// For use by loaders in responding to a Load message that allows content sniffing.
pub fn start_sending_sniffed(start_chan: LoadConsumer, metadata: Metadata,
                             classifier: Arc<MIMEClassifier>, partial_body: &Vec<u8>)
                             -> ProgressSender {
    start_sending_sniffed_opt(start_chan, metadata, classifier, partial_body).ok().unwrap()
}

/// For use by loaders in responding to a Load message that allows content sniffing.
pub fn start_sending_sniffed_opt(start_chan: LoadConsumer, mut metadata: Metadata,
                                 classifier: Arc<MIMEClassifier>, partial_body: &Vec<u8>)
                                 -> Result<ProgressSender, ()> {
    if opts::get().sniff_mime_types {
        // TODO: should be calculated in the resource loader, from pull requeset #4094
        let mut nosniff = false;
        let mut check_for_apache_bug = false;

        if let Some(ref headers) = metadata.headers {
            if let Some(ref raw_content_type) = headers.get_raw("content-type") {
                if raw_content_type.len() > 0 {
                    let ref last_raw_content_type = raw_content_type[raw_content_type.len() - 1];
                    check_for_apache_bug = last_raw_content_type == b"text/plain"
                                        || last_raw_content_type == b"text/plain; charset=ISO-8859-1"
                                        || last_raw_content_type == b"text/plain; charset=iso-8859-1"
                                        || last_raw_content_type == b"text/plain; charset=UTF-8";
                }
            }
            if let Some(ref raw_content_type_options) = headers.get_raw("X-content-type-options") {
                nosniff = raw_content_type_options.iter().any(|ref opt| *opt == b"nosniff");
            }
        }

        let supplied_type = metadata.content_type.map(|ContentType(Mime(toplevel, sublevel, _))| {
            (format!("{}", toplevel), format!("{}", sublevel))
        });
        metadata.content_type = classifier.classify(nosniff, check_for_apache_bug, &supplied_type,
                                                    &partial_body).map(|(toplevel, sublevel)| {
            let mime_tp: TopLevel = toplevel.parse().unwrap();
            let mime_sb: SubLevel = sublevel.parse().unwrap();
            ContentType(Mime(mime_tp, mime_sb, vec!()))
        });

    }

    start_sending_opt(start_chan, metadata)
}

/// For use by loaders in responding to a Load message.
pub fn start_sending_opt(start_chan: LoadConsumer, metadata: Metadata) -> Result<ProgressSender, ()> {
    match start_chan {
        LoadConsumer::Channel(start_chan) => {
            let (progress_chan, progress_port) = channel();
            let result = start_chan.send(LoadResponse {
                metadata:      metadata,
                progress_port: progress_port,
            });
            match result {
                Ok(_) => Ok(ProgressSender::Channel(progress_chan)),
                Err(_) => Err(())
            }
        }
        LoadConsumer::Listener(target) => {
            target.invoke_with_listener(ResponseAction::HeadersAvailable(metadata));
            Ok(ProgressSender::Listener(target))
        }
    }
}

fn preload_hsts_domains() -> Option<HSTSList> {
    match read_resource_file(&["hsts_preload.json"]) {
        Ok(bytes) => {
            match from_utf8(&bytes) {
                Ok(hsts_preload_content) => {
                    HSTSList::new_from_preload(hsts_preload_content)
                },
                Err(_) => None
            }
        },
        Err(_) => None
    }
}

/// Create a ResourceTask
pub fn new_resource_task(user_agent: Option<String>,
                         devtools_chan: Option<Sender<DevtoolsControlMsg>>) -> ResourceTask {
    let hsts_preload = preload_hsts_domains();

    let (setup_chan, setup_port) = channel();
    let setup_chan_clone = setup_chan.clone();
    spawn_named("ResourceManager".to_owned(), move || {
        ResourceManager::new(setup_port, user_agent, setup_chan_clone, hsts_preload, devtools_chan).start();
    });
    setup_chan
}

#[derive(RustcDecodable, RustcEncodable, Clone)]
pub struct HSTSEntry {
    pub host: String,
    pub include_subdomains: bool,
    pub max_age: Option<u64>,
    pub timestamp: Option<u64>
}

impl HSTSEntry {
    pub fn new(host: String, include_subdomains: bool, max_age: Option<u64>) -> Option<HSTSEntry> {
        if IPV4_REGEX.is_match(&host) || IPV6_REGEX.is_match(&host) {
            None
        } else {
            Some(HSTSEntry {
                host: host,
                include_subdomains: include_subdomains,
                max_age: max_age,
                timestamp: Some(time::get_time().sec as u64)
            })
        }
    }

    pub fn is_expired(&self) -> bool {
        match (self.max_age, self.timestamp) {
            (Some(max_age), Some(timestamp)) => {
                (time::get_time().sec as u64) - timestamp >= max_age
            },

            _ => false
        }
    }

    fn matches_domain(&self, host: &str) -> bool {
        !self.is_expired() && self.host == host
    }

    fn matches_subdomain(&self, host: &str) -> bool {
        !self.is_expired() && host.ends_with(&format!(".{}", self.host))
    }
}

#[derive(RustcDecodable, RustcEncodable)]
pub struct HSTSList {
    pub entries: Vec<HSTSEntry>
}

impl HSTSList {
    pub fn new_from_preload(preload_content: &str) -> Option<HSTSList> {
        match decode(preload_content) {
            Ok(list) => Some(list),
            Err(_) => None
        }
    }

    pub fn is_host_secure(&self, host: &str) -> bool {
        // TODO - Should this be faster than O(n)? The HSTS list is only a few
        // hundred or maybe thousand entries...
        //
        // Could optimise by searching for exact matches first (via a map or
        // something), then checking for subdomains.
        self.entries.iter().any(|e| {
            if e.include_subdomains {
                e.matches_subdomain(host) || e.matches_domain(host)
            } else {
                e.matches_domain(host)
            }
        })
    }

    fn has_domain(&self, host: String) -> bool {
        self.entries.iter().any(|e| {
            e.matches_domain(&host)
        })
    }

    fn has_subdomain(&self, host: String) -> bool {
        self.entries.iter().any(|e| {
            e.matches_subdomain(&host)
        })
    }

    pub fn push(&mut self, entry: HSTSEntry) {
        let have_domain = self.has_domain(entry.host.clone());
        let have_subdomain = self.has_subdomain(entry.host.clone());

        if !have_domain && !have_subdomain {
            self.entries.push(entry);
        } else if !have_subdomain {
            self.entries = self.entries.iter().fold(Vec::new(), |mut m, e| {
                if e.matches_domain(&entry.host) {
                    // Update the entry if there's an exact domain match.
                    m.push(entry.clone());
                } else {
                    // Ignore the new details if it's a subdomain match, or not
                    // a match at all. Just use the existing entry
                    m.push(e.clone());
                }

                m
            });
        }
    }
}

pub fn secure_load_data(load_data: &LoadData) -> LoadData {
    if let Some(h) = load_data.url.domain() {
        match &*load_data.url.scheme {
            "http" => {
                let mut secure_load_data = load_data.clone();
                let mut secure_url = load_data.url.clone();
                secure_url.scheme = "https".to_string();
                secure_load_data.url = secure_url;

                secure_load_data
            },
            _ => load_data.clone()
        }
    } else {
        load_data.clone()
    }
}

pub fn parse_hostsfile(hostsfile_content: &str) -> Box<HashMap<String, String>> {
    let mut host_table = HashMap::new();
    let lines: Vec<&str> = hostsfile_content.split('\n').collect();

    for line in lines.iter() {
        let ip_host: Vec<&str> = line.trim().split(|c: char| c == ' ' || c == '\t').collect();
        if ip_host.len() > 1 {
            if !IPV4_REGEX.is_match(ip_host[0]) && !IPV6_REGEX.is_match(ip_host[0]) { continue; }
            let address = ip_host[0].to_owned();

            for token in ip_host.iter().skip(1) {
                if token.as_bytes()[0] == b'#' {
                    break;
                }
                host_table.insert(token.to_owned().to_string(), address.clone());
            }
        }
    }
    box host_table
}

pub fn replace_hosts(mut load_data: LoadData, host_table: *mut HashMap<String, String>) -> LoadData {
    if let Some(h) = load_data.url.domain_mut() {
        unsafe {
            if let Some(ip) = (*host_table).get(h) {
                *h = ip.clone();
            }
        }
    }
    return load_data;
}

struct ResourceManager {
    from_client: Receiver<ControlMsg>,
    user_agent: Option<String>,
    cookie_storage: CookieStorage,
    resource_task: Sender<ControlMsg>,
    mime_classifier: Arc<MIMEClassifier>,
    devtools_chan: Option<Sender<DevtoolsControlMsg>>,
    hsts_list: Option<HSTSList>
}

impl ResourceManager {
    fn new(from_client: Receiver<ControlMsg>,
           user_agent: Option<String>,
           resource_task: Sender<ControlMsg>,
           hsts_list: Option<HSTSList>,
           devtools_channel: Option<Sender<DevtoolsControlMsg>>) -> ResourceManager {
        ResourceManager {
            from_client: from_client,
            user_agent: user_agent,
            cookie_storage: CookieStorage::new(),
            resource_task: resource_task,
            mime_classifier: Arc::new(MIMEClassifier::new()),
            devtools_chan: devtools_channel,
            hsts_list: hsts_list
        }
    }
}


impl ResourceManager {
    fn set_cookies_for_url(&mut self, request: Url, cookie_list: String, source: CookieSource) {
        let header = Header::parse_header(&[cookie_list.into_bytes()]);
        if let Ok(SetCookie(cookies)) = header {
            for bare_cookie in cookies.into_iter() {
                if let Some(cookie) = cookie::Cookie::new_wrapped(bare_cookie, &request, source) {
                    self.cookie_storage.push(cookie, source);
                }
            }
        }
    }

    fn start(&mut self) {
        loop {
            match self.from_client.recv().unwrap() {
              ControlMsg::Load(load_data, consumer) => {
                  self.load(load_data, consumer)
              }
              ControlMsg::SetCookiesForUrl(request, cookie_list, source) => {
                  self.set_cookies_for_url(request, cookie_list, source)
              }
              ControlMsg::GetCookiesForUrl(url, consumer, source) => {
                  consumer.send(self.cookie_storage.cookies_for_url(&url, source)).unwrap();
              }
              ControlMsg::Exit => {
                  break
              }
            }
        }
    }

    fn load(&mut self, mut load_data: LoadData, consumer: LoadConsumer) {
        unsafe {
            if let Some(host_table) = HOST_TABLE {
                load_data = replace_hosts(load_data, host_table);
            }
        }

        self.user_agent.as_ref().map(|ua| {
            load_data.preserved_headers.set(UserAgent(ua.clone()));
        });

        load_data = match (self.hsts_list.as_ref(), load_data.url.domain()) {
            (Some(ref l), Some(ref h)) => {
                if l.is_host_secure(h) {
                    secure_load_data(&load_data)
                } else {
                    load_data.clone()
                }
            },
            _ => load_data.clone()
        };

        fn from_factory(factory: fn(LoadData, LoadConsumer, Arc<MIMEClassifier>))
                        -> Box<FnBox(LoadData, LoadConsumer, Arc<MIMEClassifier>) + Send> {
            box move |load_data, senders, classifier| {
                factory(load_data, senders, classifier)
            }
        }

        let loader = match &*load_data.url.scheme {
            "file" => from_factory(file_loader::factory),
            "http" | "https" | "view-source" =>
                http_loader::factory(self.resource_task.clone(), self.devtools_chan.clone()),
            "data" => from_factory(data_loader::factory),
            "about" => from_factory(about_loader::factory),
            _ => {
                debug!("resource_task: no loader for scheme {}", load_data.url.scheme);
                start_sending(consumer, Metadata::default(load_data.url))
                    .send(ProgressMsg::Done(Err("no loader for scheme".to_string()))).unwrap();
                return
            }
        };
        debug!("resource_task: loading url: {}", load_data.url.serialize());

        loader.call_box((load_data, consumer, self.mime_classifier.clone()));
    }
}
