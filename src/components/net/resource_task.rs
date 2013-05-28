/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! A task that takes a URL and streams back the binary data.

use file_loader;
use http_loader;

use core::cell::Cell;
use core::comm::{Chan, Port, SharedChan};
use std::net::url::{Url, to_str};
use util::spawn_listener;

pub enum ControlMsg {
    /// Request the data associated with a particular URL
    Load(Url, Chan<ProgressMsg>),
    Exit
}

/// Messages sent in response to a `Load` message
#[deriving(Eq)]
pub enum ProgressMsg {
    /// Binary data - there may be multiple of these
    Payload(~[u8]),
    /// Indicates loading is complete, either successfully or not
    Done(Result<(), ()>)
}

/// Handle to a resource task
pub type ResourceTask = SharedChan<ControlMsg>;

/**
Creates a task to load a specific resource

The ResourceManager delegates loading to a different type of loader task for
each URL scheme
*/
type LoaderTaskFactory = ~fn() -> ~fn(url: Url, Chan<ProgressMsg>);

pub type LoaderTask = ~fn(url: Url, Chan<ProgressMsg>);

/// Create a ResourceTask with the default loaders
pub fn ResourceTask() -> ResourceTask {
    let file_loader_factory: LoaderTaskFactory = file_loader::factory;
    let http_loader_factory: LoaderTaskFactory = http_loader::factory;
    let loaders = ~[
        (~"file", file_loader_factory),
        (~"http", http_loader_factory)
    ];
    create_resource_task_with_loaders(loaders)
}

fn create_resource_task_with_loaders(loaders: ~[(~str, LoaderTaskFactory)]) -> ResourceTask {
    let loaders_cell = Cell(loaders);
    let chan = do spawn_listener |from_client| {
        // TODO: change copy to move once we can move out of closures
        ResourceManager(from_client, loaders_cell.take()).start()
    };
    SharedChan::new(chan)
}

pub struct ResourceManager {
    from_client: Port<ControlMsg>,
    /// Per-scheme resource loaders
    loaders: ~[(~str, LoaderTaskFactory)],
}


pub fn ResourceManager(from_client: Port<ControlMsg>, 
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
              Load(url, progress_chan) => {
                self.load(url.clone(), progress_chan)
              }
              Exit => {
                break
              }
            }
        }
    }

    fn load(&self, url: Url, progress_chan: Chan<ProgressMsg>) {

        match self.get_loader_factory(&url) {
            Some(loader_factory) => {
                debug!("resource_task: loading url: %s", to_str(&url));
                loader_factory(url, progress_chan);
            }
            None => {
                debug!("resource_task: no loader for scheme %s", url.scheme);
                progress_chan.send(Done(Err(())));
            }
        }
    }

    fn get_loader_factory(&self, url: &Url) -> Option<LoaderTask> {
        for self.loaders.each |scheme_loader| {
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
#[allow(non_implicitly_copyable_typarams)]
fn test_bad_scheme() {
    let resource_task = ResourceTask();
    let progress = Port();
    resource_task.send(Load(url::from_str(~"bogus://whatever").get(), progress.chan()));
    match progress.recv() {
      Done(result) => { assert!(result.is_err()) }
      _ => fail
    }
    resource_task.send(Exit);
}

#[test]
#[allow(non_implicitly_copyable_typarams)]
fn should_delegate_to_scheme_loader() {
    let payload = ~[1, 2, 3];
    let loader_factory = |_url: Url, progress_chan: Chan<ProgressMsg>| {
        progress_chan.send(Payload(copy payload));
        progress_chan.send(Done(Ok(())));
    };
    let loader_factories = ~[(~"snicklefritz", loader_factory)];
    let resource_task = create_resource_task_with_loaders(loader_factories);
    let progress = Port();
    resource_task.send(Load(url::from_str(~"snicklefritz://heya").get(), progress.chan()));
    assert!(progress.recv() == Payload(payload));
    assert!(progress.recv() == Done(Ok(())));
    resource_task.send(Exit);
}
