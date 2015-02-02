/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! A task that takes a URL and streams back the binary data.

use about_loader;
use data_loader;
use file_loader;
use http_loader;
use sniffer_task;
use sniffer_task::SnifferTask;

use util::task::spawn_named;

use hyper::header::common::UserAgent;
use hyper::header::Headers;
use hyper::http::RawStatus;
use hyper::method::Method;
use hyper::mime::{Mime, Attr};
use url::Url;

use std::borrow::ToOwned;
use std::sync::mpsc::{channel, Receiver, Sender};

pub enum ControlMsg {
    /// Request the data associated with a particular URL
    Load(LoadData),
    Exit
}

#[derive(Clone)]
pub struct LoadData {
    pub url: Url,
    pub method: Method,
    pub headers: Headers,
    pub data: Option<Vec<u8>>,
    pub cors: Option<ResourceCORSData>,
    pub consumer: Sender<LoadResponse>,
}

impl LoadData {
    pub fn new(url: Url, consumer: Sender<LoadResponse>) -> LoadData {
        LoadData {
            url: url,
            method: Method::Get,
            headers: Headers::new(),
            data: None,
            cors: None,
            consumer: consumer,
        }
    }
}

#[derive(Clone)]
pub struct ResourceCORSData {
    /// CORS Preflight flag
    pub preflight: bool,
    /// Origin of CORS Request
    pub origin: Url
}

/// Metadata about a loaded resource, such as is obtained from HTTP headers.
pub struct Metadata {
    /// Final URL after redirects.
    pub final_url: Url,

    /// MIME type / subtype.
    pub content_type: Option<(String, String)>,

    /// Character set.
    pub charset: Option<String>,

    /// Headers
    pub headers: Option<Headers>,

    /// HTTP Status
    pub status: Option<RawStatus>
}

impl Metadata {
    /// Metadata with defaults for everything optional.
    pub fn default(url: Url) -> Metadata {
        Metadata {
            final_url:    url,
            content_type: None,
            charset:      None,
            headers: None,
            // http://fetch.spec.whatwg.org/#concept-response-status-message
            status: Some(RawStatus(200, "OK".to_owned()))
        }
    }

    /// Extract the parts of a Mime that we care about.
    pub fn set_content_type(&mut self, content_type: Option<&Mime>) {
        match content_type {
            None => (),
            Some(&Mime(ref type_, ref subtype, ref parameters)) => {
                self.content_type = Some((type_.to_string(), subtype.to_string()));
                for &(ref k, ref v) in parameters.iter() {
                    if &Attr::Charset == k {
                        self.charset = Some(v.to_string());
                    }
                }
            }
        }
    }
}

/// Message sent in response to `Load`.  Contains metadata, and a port
/// for receiving the data.
///
/// Even if loading fails immediately, we send one of these and the
/// progress_port will provide the error.
pub struct LoadResponse {
    /// Metadata, such as from HTTP headers.
    pub metadata: Metadata,
    /// Port for reading data.
    pub progress_port: Receiver<ProgressMsg>,
}
/// A LoadResponse directed at a particular consumer
pub struct TargetedLoadResponse {
    pub load_response: LoadResponse,
    pub consumer: Sender<LoadResponse>,
}

// Data structure containing ports
pub struct ResponseSenders {
    pub immediate_consumer: Sender<TargetedLoadResponse>,
    pub eventual_consumer: Sender<LoadResponse>,
}

/// Messages sent in response to a `Load` message
#[derive(PartialEq,Show)]
pub enum ProgressMsg {
    /// Binary data - there may be multiple of these
    Payload(Vec<u8>),
    /// Indicates loading is complete, either successfully or not
    Done(Result<(), String>)
}

/// For use by loaders in responding to a Load message.
pub fn start_sending(senders: ResponseSenders, metadata: Metadata) -> Sender<ProgressMsg> {
    start_sending_opt(senders, metadata).ok().unwrap()
}

/// For use by loaders in responding to a Load message.
pub fn start_sending_opt(senders: ResponseSenders, metadata: Metadata) -> Result<Sender<ProgressMsg>, ()> {
    let (progress_chan, progress_port) = channel();
    let result = senders.immediate_consumer.send(TargetedLoadResponse {
        load_response: LoadResponse {
            metadata:      metadata,
            progress_port: progress_port,
        },
        consumer: senders.eventual_consumer
    });
    match result {
        Ok(_) => Ok(progress_chan),
        Err(_) => Err(())
    }
}

/// Convenience function for synchronously loading a whole resource.
pub fn load_whole_resource(resource_task: &ResourceTask, url: Url)
        -> Result<(Metadata, Vec<u8>), String> {
    let (start_chan, start_port) = channel();
    resource_task.send(ControlMsg::Load(LoadData::new(url, start_chan)));
    let response = start_port.recv().unwrap();

    let mut buf = vec!();
    loop {
        match response.progress_port.recv().unwrap() {
            ProgressMsg::Payload(data) => buf.push_all(data.as_slice()),
            ProgressMsg::Done(Ok(()))  => return Ok((response.metadata, buf)),
            ProgressMsg::Done(Err(e))  => return Err(e)
        }
    }
}

/// Handle to a resource task
pub type ResourceTask = Sender<ControlMsg>;

/// Create a ResourceTask
pub fn new_resource_task(user_agent: Option<String>) -> ResourceTask {
    let (setup_chan, setup_port) = channel();
    let sniffer_task = sniffer_task::new_sniffer_task();
    spawn_named("ResourceManager".to_owned(), move || {
        ResourceManager::new(setup_port, user_agent, sniffer_task).start();
    });
    setup_chan
}

struct ResourceManager {
    from_client: Receiver<ControlMsg>,
    user_agent: Option<String>,
    sniffer_task: SnifferTask,
}

impl ResourceManager {
    fn new(from_client: Receiver<ControlMsg>, user_agent: Option<String>, sniffer_task: SnifferTask) -> ResourceManager {
        ResourceManager {
            from_client: from_client,
            user_agent: user_agent,
            sniffer_task: sniffer_task,
        }
    }
}


impl ResourceManager {
    fn start(&self) {
        loop {
            match self.from_client.recv().unwrap() {
              ControlMsg::Load(load_data) => {
                self.load(load_data)
              }
              ControlMsg::Exit => {
                break
              }
            }
        }
    }

    fn load(&self, load_data: LoadData) {
        let mut load_data = load_data;
        self.user_agent.as_ref().map(|ua| load_data.headers.set(UserAgent(ua.clone())));
        let senders = ResponseSenders {
            immediate_consumer: self.sniffer_task.clone(),
            eventual_consumer: load_data.consumer.clone(),
        };

        debug!("resource_task: loading url: {}", load_data.url.serialize());
        match load_data.url.scheme.as_slice() {
            "file" => file_loader::factory(load_data, self.sniffer_task.clone()),
            "http" | "https" => http_loader::factory(load_data, self.sniffer_task.clone()),
            "data" => data_loader::factory(load_data, self.sniffer_task.clone()),
            "about" => about_loader::factory(load_data, self.sniffer_task.clone()),
            _ => {
                debug!("resource_task: no loader for scheme {}", load_data.url.scheme);
                start_sending(senders, Metadata::default(load_data.url))
                    .send(ProgressMsg::Done(Err("no loader for scheme".to_string())));
                return
            }
        }
    }
}

/// Load a URL asynchronously and iterate over chunks of bytes from the response.
pub fn load_bytes_iter(resource_task: &ResourceTask, url: Url) -> (Metadata, ProgressMsgPortIterator) {
    let (input_chan, input_port) = channel();
    resource_task.send(ControlMsg::Load(LoadData::new(url, input_chan)));

    let response = input_port.recv().unwrap();
    let iter = ProgressMsgPortIterator { progress_port: response.progress_port };
    (response.metadata, iter)
}

/// Iterator that reads chunks of bytes from a ProgressMsg port
pub struct ProgressMsgPortIterator {
    progress_port: Receiver<ProgressMsg>
}

impl Iterator for ProgressMsgPortIterator {
    type Item = Vec<u8>;

    fn next(&mut self) -> Option<Vec<u8>> {
        match self.progress_port.recv().unwrap() {
            ProgressMsg::Payload(data) => Some(data),
            ProgressMsg::Done(Ok(()))  => None,
            ProgressMsg::Done(Err(e))  => {
                error!("error receiving bytes: {}", e);
                None
            }
        }
    }
}

#[test]
fn test_exit() {
    let resource_task = new_resource_task(None);
    resource_task.send(ControlMsg::Exit);
}

#[test]
fn test_bad_scheme() {
    let resource_task = new_resource_task(None);
    let (start_chan, start) = channel();
    let url = Url::parse("bogus://whatever").unwrap();
    resource_task.send(ControlMsg::Load(LoadData::new(url, start_chan)));
    let response = start.recv().unwrap();
    match response.progress_port.recv().unwrap() {
      ProgressMsg::Done(result) => { assert!(result.is_err()) }
      _ => panic!("bleh")
    }
    resource_task.send(ControlMsg::Exit);
}
