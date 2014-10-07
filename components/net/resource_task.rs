/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! A task that takes a URL and streams back the binary data.

use about_loader;
use data_loader;
use file_loader;
use http_loader;

use std::comm::{channel, Receiver, Sender};
use std::task::TaskBuilder;
use http::headers::content_type::MediaType;
use http::headers::response::HeaderCollection as ResponseHeaderCollection;
use http::headers::request::HeaderCollection as RequestHeaderCollection;
use http::method::{Method, Get};
use url::Url;

use http::status::Ok as StatusOk;
use http::status::Status;


pub enum ControlMsg {
    /// Request the data associated with a particular URL
    Load(LoadData, Sender<LoadResponse>),
    Exit
}

#[deriving(Clone)]
pub struct LoadData {
    pub url: Url,
    pub method: Method,
    pub headers: RequestHeaderCollection,
    pub data: Option<Vec<u8>>,
    pub cors: Option<ResourceCORSData>
}

impl LoadData {
    pub fn new(url: Url) -> LoadData {
        LoadData {
            url: url,
            method: Get,
            headers: RequestHeaderCollection::new(),
            data: None,
            cors: None
        }
    }
}

#[deriving(Clone)]
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
    pub headers: Option<ResponseHeaderCollection>,

    /// HTTP Status
    pub status: Status
}

impl Metadata {
    /// Metadata with defaults for everything optional.
    pub fn default(url: Url) -> Metadata {
        Metadata {
            final_url:    url,
            content_type: None,
            charset:      None,
            headers: None,
            status: StatusOk // http://fetch.spec.whatwg.org/#concept-response-status-message
        }
    }

    /// Extract the parts of a MediaType that we care about.
    pub fn set_content_type(&mut self, content_type: &Option<MediaType>) {
        match *content_type {
            None => (),
            Some(MediaType { type_:      ref type_,
                             subtype:    ref subtype,
                             parameters: ref parameters }) => {
                self.content_type = Some((type_.clone(), subtype.clone()));
                for &(ref k, ref v) in parameters.iter() {
                    if "charset" == k.as_slice() {
                        self.charset = Some(v.clone());
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

/// Messages sent in response to a `Load` message
#[deriving(PartialEq,Show)]
pub enum ProgressMsg {
    /// Binary data - there may be multiple of these
    Payload(Vec<u8>),
    /// Indicates loading is complete, either successfully or not
    Done(Result<(), String>)
}

/// For use by loaders in responding to a Load message.
pub fn start_sending(start_chan: Sender<LoadResponse>, metadata: Metadata) -> Sender<ProgressMsg> {
    start_sending_opt(start_chan, metadata).ok().unwrap()
}

/// For use by loaders in responding to a Load message.
pub fn start_sending_opt(start_chan: Sender<LoadResponse>, metadata: Metadata) -> Result<Sender<ProgressMsg>, ()> {
    let (progress_chan, progress_port) = channel();
    let result = start_chan.send_opt(LoadResponse {
        metadata:      metadata,
        progress_port: progress_port,
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
    resource_task.send(Load(LoadData::new(url), start_chan));
    let response = start_port.recv();

    let mut buf = vec!();
    loop {
        match response.progress_port.recv() {
            Payload(data) => buf.push_all(data.as_slice()),
            Done(Ok(()))  => return Ok((response.metadata, buf)),
            Done(Err(e))  => return Err(e)
        }
    }
}

/// Handle to a resource task
pub type ResourceTask = Sender<ControlMsg>;

/// Create a ResourceTask
pub fn new_resource_task(user_agent: Option<String>) -> ResourceTask {
    let (setup_chan, setup_port) = channel();
    let builder = TaskBuilder::new().named("ResourceManager");
    builder.spawn(proc() {
        ResourceManager::new(setup_port, user_agent).start();
    });
    setup_chan
}

struct ResourceManager {
    from_client: Receiver<ControlMsg>,
    user_agent: Option<String>,
}


impl ResourceManager {
    fn new(from_client: Receiver<ControlMsg>, user_agent: Option<String>) -> ResourceManager {
        ResourceManager {
            from_client: from_client,
            user_agent: user_agent,
        }
    }
}


impl ResourceManager {
    fn start(&self) {
        loop {
            match self.from_client.recv() {
              Load(load_data, start_chan) => {
                self.load(load_data, start_chan)
              }
              Exit => {
                break
              }
            }
        }
    }

    fn load(&self, load_data: LoadData, start_chan: Sender<LoadResponse>) {
        let mut load_data = load_data;
        load_data.headers.user_agent = self.user_agent.clone();

        let loader = match load_data.url.scheme.as_slice() {
            "file" => file_loader::factory,
            "http" | "https" => http_loader::factory,
            "data" => data_loader::factory,
            "about" => about_loader::factory,
            _ => {
                debug!("resource_task: no loader for scheme {:s}", load_data.url.scheme);
                start_sending(start_chan, Metadata::default(load_data.url))
                    .send(Done(Err("no loader for scheme".to_string())));
                return
            }
        };
        debug!("resource_task: loading url: {:s}", load_data.url.serialize());
        loader(load_data, start_chan);
    }
}

/// Load a URL asynchronously and iterate over chunks of bytes from the response.
pub fn load_bytes_iter(resource_task: &ResourceTask, url: Url) -> (Metadata, ProgressMsgPortIterator) {
    let (input_chan, input_port) = channel();
    resource_task.send(Load(LoadData::new(url), input_chan));

    let response = input_port.recv();
    let iter = ProgressMsgPortIterator { progress_port: response.progress_port };
    (response.metadata, iter)
}

/// Iterator that reads chunks of bytes from a ProgressMsg port
pub struct ProgressMsgPortIterator {
    progress_port: Receiver<ProgressMsg>
}

impl Iterator<Vec<u8>> for ProgressMsgPortIterator {
    fn next(&mut self) -> Option<Vec<u8>> {
        match self.progress_port.recv() {
            Payload(data) => Some(data),
            Done(Ok(()))  => None,
            Done(Err(e))  => {
                error!("error receiving bytes: {}", e);
                None
            }
        }
    }
}

#[test]
fn test_exit() {
    let resource_task = new_resource_task(None);
    resource_task.send(Exit);
}

#[test]
fn test_bad_scheme() {
    let resource_task = new_resource_task(None);
    let (start_chan, start) = channel();
    let url = Url::parse("bogus://whatever").unwrap();
    resource_task.send(Load(LoadData::new(url), start_chan));
    let response = start.recv();
    match response.progress_port.recv() {
      Done(result) => { assert!(result.is_err()) }
      _ => fail!("bleh")
    }
    resource_task.send(Exit);
}
