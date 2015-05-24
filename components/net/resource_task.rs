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
use net_traits::{Metadata, ProgressMsg, ResourceTask, AsyncResponseTarget, ResponseAction};
use net_traits::ProgressMsg::Done;
use util::opts;
use util::task::spawn_named;

use devtools_traits::{DevtoolsControlMsg};
use hyper::header::{ContentType, Header, SetCookie, UserAgent};
use hyper::mime::{Mime, TopLevel, SubLevel};

use std::borrow::ToOwned;
use std::boxed::{self, FnBox};
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{BufReader, Read};
use std::str::FromStr;
use std::sync::Arc;
use std::sync::mpsc::{channel, Receiver, Sender};


static mut HOST_TABLE: Option<*mut HashMap<String, String>> = None;

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

    unsafe {
        let host_table = boxed::into_raw(parse_hostsfile(&lines));
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
                let ref last_raw_content_type = raw_content_type[raw_content_type.len() - 1];
                check_for_apache_bug = last_raw_content_type == b"text/plain"
                                    || last_raw_content_type == b"text/plain; charset=ISO-8859-1"
                                    || last_raw_content_type == b"text/plain; charset=iso-8859-1"
                                    || last_raw_content_type == b"text/plain; charset=UTF-8";
            }
            if let Some(ref raw_content_type_options) = headers.get_raw("content-type-options") {
                for options in raw_content_type_options.iter() {
                    nosniff = nosniff || options == b"nosniff";
                }
            }
        }

        let supplied_type = metadata.content_type.map(|ContentType(Mime(toplevel, sublevel, _))| {
            (format!("{}", toplevel), format!("{}", sublevel))
        });
        metadata.content_type = classifier.classify(nosniff, check_for_apache_bug, &supplied_type, &partial_body).map(|(toplevel, sublevel)| {
            let mime_tp: TopLevel = FromStr::from_str(&toplevel).unwrap();
            let mime_sb: SubLevel = FromStr::from_str(&sublevel).unwrap();
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

/// Create a ResourceTask
pub fn new_resource_task(user_agent: Option<String>, devtools_chan: Option<Sender<DevtoolsControlMsg>>) -> ResourceTask {
    let (setup_chan, setup_port) = channel();
    let setup_chan_clone = setup_chan.clone();
    spawn_named("ResourceManager".to_owned(), move || {
        ResourceManager::new(setup_port, user_agent, setup_chan_clone, devtools_chan).start();
    });
    setup_chan
}

pub fn parse_hostsfile(hostsfile_content: &str) -> Box<HashMap<String, String>> {
    let ipv4_regex = regex!(r"^(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)$");
    let ipv6_regex = regex!(r"^([a-fA-F0-9]{0,4}[:]?){1,8}(/\d{1,3})?$");
    let mut host_table = HashMap::new();
    let lines: Vec<&str> = hostsfile_content.split('\n').collect();

    for line in lines.iter() {
        let ip_host: Vec<&str> = line.trim().split(|c: char| c == ' ' || c == '\t').collect();
        if ip_host.len() > 1 {
            if !ipv4_regex.is_match(ip_host[0]) && !ipv6_regex.is_match(ip_host[0]) { continue; }
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
    devtools_chan: Option<Sender<DevtoolsControlMsg>>
}

impl ResourceManager {
    fn new(from_client: Receiver<ControlMsg>, user_agent: Option<String>,
           resource_task: Sender<ControlMsg>, devtools_channel: Option<Sender<DevtoolsControlMsg>>) -> ResourceManager {
        ResourceManager {
            from_client: from_client,
            user_agent: user_agent,
            cookie_storage: CookieStorage::new(),
            resource_task: resource_task,
            mime_classifier: Arc::new(MIMEClassifier::new()),
            devtools_chan: devtools_channel
        }
    }
}


impl ResourceManager {
    fn start(&mut self) {
        loop {
            match self.from_client.recv().unwrap() {
              ControlMsg::Load(load_data, consumer) => {
                self.load(load_data, consumer)
              }
              ControlMsg::SetCookiesForUrl(request, cookie_list, source) => {
                let header = Header::parse_header(&[cookie_list.into_bytes()]);
                if let Some(SetCookie(cookies)) = header {
                  for bare_cookie in cookies.into_iter() {
                    if let Some(cookie) = cookie::Cookie::new_wrapped(bare_cookie, &request, source) {
                      self.cookie_storage.push(cookie, source);
                    }
                  }
                }
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

        self.user_agent.as_ref().map(|ua| load_data.headers.set(UserAgent(ua.clone())));

        fn from_factory(factory: fn(LoadData, LoadConsumer, Arc<MIMEClassifier>))
                        -> Box<FnBox(LoadData, LoadConsumer, Arc<MIMEClassifier>) + Send> {
            box move |load_data, senders, classifier| {
                factory(load_data, senders, classifier)
            }
        }

        let loader = match &*load_data.url.scheme {
            "file" => from_factory(file_loader::factory),
            "http" | "https" | "view-source" => http_loader::factory(self.resource_task.clone(), self.devtools_chan.clone()),
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
