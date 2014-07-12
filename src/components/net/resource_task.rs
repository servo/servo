/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! A task that takes a URL and streams back the binary data.

use file_loader;
use http_loader;
use data_loader;

use std::comm::{channel, Receiver, Sender};
use std::task::TaskBuilder;
use http::headers::content_type::MediaType;
use ResponseHeaderCollection = http::headers::response::HeaderCollection;
use RequestHeaderCollection = http::headers::request::HeaderCollection;
use http::method::{Method, Get};
use url::Url;

use StatusOk = http::status::Ok;
use http::status::Status;

#[cfg(test)]
use std::from_str::FromStr;

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
    pub data: Option<Vec<u8>>
}

impl LoadData {
    pub fn new(url: Url) -> LoadData {
        LoadData {
            url: url,
            method: Get,
            headers: RequestHeaderCollection::new(),
            data: None
        }
    }
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

pub type LoaderTask = proc(load_data: LoadData, Sender<LoadResponse>);

/**
Creates a task to load a specific resource

The ResourceManager delegates loading to a different type of loader task for
each URL scheme
*/
type LoaderTaskFactory = extern "Rust" fn() -> LoaderTask;

/// Create a ResourceTask with the default loaders
pub fn new_resource_task() -> ResourceTask {
    let loaders = vec!(
        ("file".to_string(), file_loader::factory),
        ("http".to_string(), http_loader::factory),
        ("data".to_string(), data_loader::factory),
    );
    create_resource_task_with_loaders(loaders)
}

fn create_resource_task_with_loaders(loaders: Vec<(String, LoaderTaskFactory)>) -> ResourceTask {
    let (setup_chan, setup_port) = channel();
    let builder = TaskBuilder::new().named("ResourceManager");
    builder.spawn(proc() {
        let (chan, port) = channel();
        setup_chan.send(chan);
        ResourceManager::new(port, loaders).start();
    });
    setup_port.recv()
}

struct ResourceManager {
    from_client: Receiver<ControlMsg>,
    /// Per-scheme resource loaders
    loaders: Vec<(String, LoaderTaskFactory)>,
}


impl ResourceManager {
    fn new(from_client: Receiver<ControlMsg>, loaders: Vec<(String, LoaderTaskFactory)>)
           -> ResourceManager {
        ResourceManager {
            from_client : from_client,
            loaders : loaders,
        }
    }
}


impl ResourceManager {
    fn start(&self) {
        loop {
            match self.from_client.recv() {
              Load(load_data, start_chan) => {
                self.load(load_data.clone(), start_chan)
              }
              Exit => {
                break
              }
            }
        }
    }

    fn load(&self, load_data: LoadData, start_chan: Sender<LoadResponse>) {
        match self.get_loader_factory(&load_data) {
            Some(loader_factory) => {
                debug!("resource_task: loading url: {:s}", load_data.url.to_str());
                loader_factory(load_data, start_chan);
            }
            None => {
                debug!("resource_task: no loader for scheme {:s}", load_data.url.scheme);
                start_sending(start_chan, Metadata::default(load_data.url)).send(Done(Err("no loader for scheme".to_string())));
            }
        }
    }

    fn get_loader_factory(&self, load_data: &LoadData) -> Option<LoaderTask> {
        for scheme_loader in self.loaders.iter() {
            match *scheme_loader {
                (ref scheme, ref loader_factory) => {
                    if (*scheme) == load_data.url.scheme {
                        return Some((*loader_factory)());
                    }
                }
            }
        }
        return None;
    }
}

#[test]
fn test_exit() {
    let resource_task = new_resource_task();
    resource_task.send(Exit);
}

#[test]
fn test_bad_scheme() {
    let resource_task = new_resource_task();
    let (start_chan, start) = channel();
    resource_task.send(Load(LoadData::new(FromStr::from_str("bogus://whatever").unwrap()), start_chan));
    let response = start.recv();
    match response.progress_port.recv() {
      Done(result) => { assert!(result.is_err()) }
      _ => fail!("bleh")
    }
    resource_task.send(Exit);
}

#[cfg(test)]
static snicklefritz_payload: [u8, ..3] = [1, 2, 3];

#[cfg(test)]
fn snicklefritz_loader_factory() -> LoaderTask {
    let f: LoaderTask = proc(load_data: LoadData, start_chan: Sender<LoadResponse>) {
        let progress_chan = start_sending(start_chan, Metadata::default(load_data.url));
        progress_chan.send(Payload(Vec::from_slice(snicklefritz_payload)));
        progress_chan.send(Done(Ok(())));
    };
    f
}

#[test]
fn should_delegate_to_scheme_loader() {
    let loader_factories = vec!(("snicklefritz".to_string(), snicklefritz_loader_factory));
    let resource_task = create_resource_task_with_loaders(loader_factories);
    let (start_chan, start) = channel();
    resource_task.send(Load(LoadData::new(FromStr::from_str("snicklefritz://heya").unwrap()), start_chan));

    let response = start.recv();
    let progress = response.progress_port;

    assert!(progress.recv() == Payload(Vec::from_slice(snicklefritz_payload)));
    assert!(progress.recv() == Done(Ok(())));
    resource_task.send(Exit);
}
