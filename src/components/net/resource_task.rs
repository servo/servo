/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! A task that takes a URL and streams back the binary data.

use file_loader;
use http_loader;
use data_loader;

use std::comm::{channel, Receiver, Sender};
use std::task;
use http::headers::content_type::MediaType;
use url::Url;

#[cfg(test)]
use std::from_str::FromStr;

pub enum ControlMsg {
    /// Request the data associated with a particular URL
    Load(Url, Sender<LoadResponse>),
    Exit
}

/// Metadata about a loaded resource, such as is obtained from HTTP headers.
pub struct Metadata {
    /// Final URL after redirects.
    pub final_url: Url,

    /// MIME type / subtype.
    pub content_type: Option<(~str, ~str)>,

    /// Character set.
    pub charset: Option<~str>,
}

impl Metadata {
    /// Metadata with defaults for everything optional.
    pub fn default(url: Url) -> Metadata {
        Metadata {
            final_url:    url,
            content_type: None,
            charset:      None,
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
#[deriving(Eq,Show)]
pub enum ProgressMsg {
    /// Binary data - there may be multiple of these
    Payload(Vec<u8>),
    /// Indicates loading is complete, either successfully or not
    Done(Result<(), ()>)
}

/// For use by loaders in responding to a Load message.
pub fn start_sending(start_chan: Sender<LoadResponse>,
                     metadata:   Metadata) -> Sender<ProgressMsg> {
    let (progress_chan, progress_port) = channel();
    start_chan.send(LoadResponse {
        metadata:      metadata,
        progress_port: progress_port,
    });
    progress_chan
}

/// Convenience function for synchronously loading a whole resource.
pub fn load_whole_resource(resource_task: &ResourceTask, url: Url)
        -> Result<(Metadata, Vec<u8>), ()> {
    let (start_chan, start_port) = channel();
    resource_task.send(Load(url, start_chan));
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

pub type LoaderTask = proc(url: Url, Sender<LoadResponse>);

/**
Creates a task to load a specific resource

The ResourceManager delegates loading to a different type of loader task for
each URL scheme
*/
type LoaderTaskFactory = extern "Rust" fn() -> LoaderTask;

/// Create a ResourceTask with the default loaders
pub fn ResourceTask() -> ResourceTask {
    let loaders = ~[
        ("file".to_owned(), file_loader::factory),
        ("http".to_owned(), http_loader::factory),
        ("data".to_owned(), data_loader::factory),
    ];
    create_resource_task_with_loaders(loaders)
}

fn create_resource_task_with_loaders(loaders: ~[(~str, LoaderTaskFactory)]) -> ResourceTask {
    let (setup_chan, setup_port) = channel();
    let builder = task::task().named("ResourceManager");
    builder.spawn(proc() {
        let (chan, port) = channel();
        setup_chan.send(chan);
        ResourceManager(port, loaders).start();
    });
    setup_port.recv()
}

struct ResourceManager {
    from_client: Receiver<ControlMsg>,
    /// Per-scheme resource loaders
    loaders: ~[(~str, LoaderTaskFactory)],
}


fn ResourceManager(from_client: Receiver<ControlMsg>,
                   loaders: ~[(~str, LoaderTaskFactory)]) -> ResourceManager {
    ResourceManager {
        from_client : from_client,
        loaders : loaders,
    }
}


impl ResourceManager {
    fn start(&self) {
        loop {
            match self.from_client.recv() {
              Load(url, start_chan) => {
                self.load(url.clone(), start_chan)
              }
              Exit => {
                break
              }
            }
        }
    }

    fn load(&self, url: Url, start_chan: Sender<LoadResponse>) {
        match self.get_loader_factory(&url) {
            Some(loader_factory) => {
                debug!("resource_task: loading url: {:s}", url.to_str());
                loader_factory(url, start_chan);
            }
            None => {
                debug!("resource_task: no loader for scheme {:s}", url.scheme);
                start_sending(start_chan, Metadata::default(url)).send(Done(Err(())));
            }
        }
    }

    fn get_loader_factory(&self, url: &Url) -> Option<LoaderTask> {
        for scheme_loader in self.loaders.iter() {
            match *scheme_loader {
                (ref scheme, ref loader_factory) => {
	            if (*scheme) == url.scheme {
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
    let resource_task = ResourceTask();
    resource_task.send(Exit);
}

#[test]
fn test_bad_scheme() {
    let resource_task = ResourceTask();
    let (start_chan, start) = channel();
    resource_task.send(Load(FromStr::from_str("bogus://whatever").unwrap(), start_chan));
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
    let f: LoaderTask = proc(url: Url, start_chan: Sender<LoadResponse>) {
        let progress_chan = start_sending(start_chan, Metadata::default(url));
        progress_chan.send(Payload(Vec::from_slice(snicklefritz_payload)));
        progress_chan.send(Done(Ok(())));
    };
    f
}

#[test]
fn should_delegate_to_scheme_loader() {
    let loader_factories = ~[("snicklefritz".to_owned(), snicklefritz_loader_factory)];
    let resource_task = create_resource_task_with_loaders(loader_factories);
    let (start_chan, start) = channel();
    resource_task.send(Load(FromStr::from_str("snicklefritz://heya").unwrap(), start_chan));

    let response = start.recv();
    let progress = response.progress_port;

    assert!(progress.recv() == Payload(Vec::from_slice(snicklefritz_payload)));
    assert!(progress.recv() == Done(Ok(())));
    resource_task.send(Exit);
}
