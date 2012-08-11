/*!

A task that takes a URL and streams back the binary data

*/

export ControlMsg, Load, Exit;
export ProgressMsg, Payload, Done;
export ResourceTask, ResourceManager, LoaderTaskFactory;

import comm::{chan, port};
import task::{spawn, spawn_listener};
import std::net::url;
import std::net::url::url;
import result::{result, ok, err};

enum ControlMsg {
    /// Request the data associated with a particular URL
    Load(url, chan<ProgressMsg>),
    Exit
}

/// Messages sent in response to a `Load` message
enum ProgressMsg {
    /// Binary data - there may be multiple of these
    Payload(~[u8]),
    /// Indicates loading is complete, either successfully or not
    Done(result<(), ()>)
}

/// Handle to a resource task
type ResourceTask = chan<ControlMsg>;

/**
Creates a task to load a specific resource

The ResourceManager delegates loading to a different type of loader task for
each URL scheme
*/
type LoaderTaskFactory = fn~(+url: url, chan<ProgressMsg>);

/// Create a ResourceTask with the default loaders
fn ResourceTask() -> ResourceTask {
    let loaders = ~[
        (~"file", file_loader::factory),
        (~"http", http_loader::factory)
    ];
    create_resource_task_with_loaders(loaders)
}

fn create_resource_task_with_loaders(+loaders: ~[(~str, LoaderTaskFactory)]) -> ResourceTask {
    do spawn_listener |from_client| {
        // TODO: change copy to move once we can move into closures
        ResourceManager(from_client, copy loaders).start()
    }
}

class ResourceManager {
    let from_client: port<ControlMsg>;
    /// Per-scheme resource loaders
    let loaders: ~[(~str, LoaderTaskFactory)];

    new(from_client: port<ControlMsg>, -loaders: ~[(~str, LoaderTaskFactory)]) {
        self.from_client = from_client;
        self.loaders = loaders;
    }

    fn start() {
        loop {
            match self.from_client.recv() {
              Load(url, progress_chan) => {
                self.load(copy url, progress_chan)
              }
              Exit => {
                break
              }
            }
        }
    }

    fn load(+url: url, progress_chan: chan<ProgressMsg>) {

        match self.get_loader_factory(url) {
          some(loader_factory) => {
            #debug("resource_task: loading url: %s", url::to_str(url));
            loader_factory(url, progress_chan);
          }
          none => {
            #debug("resource_task: no loader for scheme %s", url.scheme);
            progress_chan.send(Done(err(())));
          }
        }
    }

    fn get_loader_factory(url: url) -> option<LoaderTaskFactory> {
        for self.loaders.each |scheme_loader| {
            let (scheme, loader_factory) = copy scheme_loader;
            if scheme == url.scheme {
                return some(loader_factory);
            }
        }
        return none;
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
    let progress = port();
    resource_task.send(Load(url::from_str(~"bogus://whatever").get(), progress.chan()));
    match check progress.recv() {
      Done(result) => { assert result.is_err() }
    }
    resource_task.send(Exit);
}

#[test]
#[allow(non_implicitly_copyable_typarams)]
fn should_delegate_to_scheme_loader() {
    let payload = ~[1, 2, 3];
    let loader_factory = fn~(+_url: url, progress_chan: chan<ProgressMsg>, copy payload) {
        progress_chan.send(Payload(copy payload));
        progress_chan.send(Done(ok(())));
    };
    let loader_factories = ~[(~"snicklefritz", loader_factory)];
    let resource_task = create_resource_task_with_loaders(loader_factories);
    let progress = port();
    resource_task.send(Load(url::from_str(~"snicklefritz://heya").get(), progress.chan()));
    assert progress.recv() == Payload(payload);
    assert progress.recv() == Done(ok(()));
    resource_task.send(Exit);
}
