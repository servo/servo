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
    Visited(~str,Chan<HistoryResponseMsg>),
    Exit(Chan<()>),
}

#[deriving(Clone)]
pub enum HistoryResponseMsg {
    NotVisitedStie,
    VisitedSite
}

#[deriving(Clone)]
pub struct HistoryCacheTask {
    chan: SharedChan<Msg>,
}

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
    HistoryCacheTask {
        chan:chan,
    }
}

struct HistoryCache {
    /// A handle to the resource task for fetching the url history data
    resource_task: ResourceTask,
    /// The port on which we'll receive client requests
    port: Port<Msg>,
    /// A copy of the shared chan to give to child tasks
    chan: SharedChan<Msg>,
    need_exit: Option<Chan<()>>,
}

impl HistoryCache {
    pub fn run(&mut self) {

        loop {
            let msg: Msg = self.port.recv();

            match msg {
                Record(url) => { println!("Visited Record");},
                Request(url) => { println!("Request"); },
                Visited(url, chan) => { println!("for Response"); },
                Exit(response) => {
                    self.need_exit = Some(response);
                },
            }
  
            let need_exit = replace(&mut self.need_exit, None);

            match need_exit {
              Some(response) => {
                  let mut can_exit = true;

                  if can_exit {
                      response.send(());
                      break;
                  } else {
                      self.need_exit = Some(response);
                  }
              }
              None => ()
            }
 
        }
    }
}

impl HistoryCacheTask {
    pub fn send(&self, msg: Msg) {
        self.chan.send(msg);
    }

    pub fn exit(&self) {
        let (response_port, response_chan) = Chan::new();
        self.send(Exit(response_chan));
        response_port.recv();
    }
}
