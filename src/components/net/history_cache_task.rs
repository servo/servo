use resource_task;
use resource_task::ResourceTask;
use servo_util::url::{UrlMap, url_map};

use std::comm::{Chan, Port, SharedChan};
use std::task::spawn;
use std::to_str::ToStr;
use std::util::replace;
use std::result;
use extra::arc::{Arc,MutexArc};
use extra::url::Url;


pub enum Msg {
    Record(Url),
    Request(Url),
    Visited(bool),
    Exit(Chan<()>),
}

pub type HistoryCacheTask = SharedChan<Msg>;

pub fn HistoryCacheTask(resource_task: ResourceTask) -> HistoryCacheTask {
    HistoryCacheTask_(resource_task)
}

pub fn HistoryCacheTask_(resource_task: ResourceTask)
                       -> HistoryCacheTask {
    let (port, chan) = SharedChan::new();
    let chan_clone = chan.clone();

    do spawn {
        let mut cache = HistoryCache {
            resource_task: resource_task.clone(),
            port: port,
            chan: chan_clone,
            need_exit: None
        };
        cache.run();
    }

    chan
}

struct HistoryCache {
    /// A handle to the resource task for fetching the image binaries
    resource_task: ResourceTask,
    /// The port on which we'll receive client requests
    port: Port<Msg>,
    /// A copy of the shared chan to give to child tasks
    chan: SharedChan<Msg>,
    need_exit: Option<Chan<()>>,
}

impl HistoryCache {
    pub fn run(&mut self) {
        //let mut msg_handlers: ~[~fn(msg: &Msg)] = ~[];

        loop {
            let msg: Msg = self.port.recv();

            match msg {
                Record(url) => { println!("Visited Record");},
                Request(url) => { println!("Request"); },
                Exit(response) => {
                    self.need_exit = Some(response);
                },
                _ => {}
            }
  
            let need_exit = replace(&mut self.need_exit, None);

            match need_exit {
              Some(response) => {
                // save file for exit
              }
              None => ()
            }
 
        }
    }
}
