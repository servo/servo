use std::comm::{Chan, Port, SharedChan};
use std::task::spawn;
use std::to_str::ToStr;
use std::util::replace;
use extra::url::Url;
use std::hashmap::HashMap;

pub enum Msg {
    UrlRecord(Url),
    Visited(~str,Chan<bool>),
    Exit(Chan<()>),
}

#[deriving(Clone)]
pub struct HistoryTask {
    chan: SharedChan<Msg>,
}

pub fn HistoryTask() -> HistoryTask {
    HistoryTask_()
}

pub fn HistoryTask_()
                       -> HistoryTask {
    let (port, chan) = SharedChan::new();
    let chan_clone = chan.clone();

    do spawn {
        let mut cache = History {
            visited_site: HashMap::new(),
            current_site: None,
            current_url_path: ~[],
            port: port,
            chan: chan_clone,
            need_exit: None
        };
        cache.run();
    }
    HistoryTask {
        chan:chan,
    }
}

struct History {
    /// The data on which we'll store visited site.
    visited_site: HashMap<~str, Url>,
    current_site: Option<Url>,
    current_url_path: ~[~str],
    /// The port on which we'll receive client requests
    port: Port<Msg>,
    /// A copy of the shared chan to give to child tasks
    chan: SharedChan<Msg>,
    need_exit: Option<Chan<()>>,
}

impl History {
    pub fn run(&mut self) {

        loop {
            let msg: Msg = self.port.recv();

            match msg {
                UrlRecord(url) => { self.record(url); },
                Visited(url, chan) => {
                    let visited = self.is_visited(url);
                    chan.send(visited);
                },
                Exit(response) => {
                    self.need_exit = Some(response);
                },
            }
  
            let need_exit = replace(&mut self.need_exit, None);

            match need_exit {
              Some(response) => {
                  let can_exit = true;

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

    pub fn record(&mut self, url: Url) {
        let path = url.path.to_owned().clone();
        let mut current_url_path:~[~str] = ~[]; 
        let mut file_name = ~"";

        let mut url_it = path.chars();
 
        for ch in url_it {
            if ch == '/' {
                if file_name.len() > 0 {
                    if !file_name.equals(&~"..") {
                        current_url_path.push(file_name.clone());
                    } else {
                        current_url_path.pop();
                    }

                    file_name = ~"";
                }
            } else {
                file_name.push_char(ch);
            }
        }

        let slash = ~"/";
        
        let full_path = { if current_url_path.len() > 0 {
                              slash + current_url_path.connect("/")+ slash + file_name
                          } else {
                              slash + file_name
                          } };

        let url = Url { scheme: url.scheme.clone(),
                        user: url.user.clone(),
                        host: url.host.clone(),
                        port: url.port.clone(),
                        path: full_path,
                        query: ~[],
                        fragment: None };
 

        if !self.visited_site.contains_key(&url.to_str()) {
            self.visited_site.insert(url.to_str().clone(), url.clone());
        }
        self.current_site = Some(url.clone());
        self.current_url_path = current_url_path;
    }

    pub fn is_visited(&mut self, url: &str) -> bool {
        let mut result = false;      

        match self.current_site {
            Some(ref site) => {
                let full_path = self.compute_url_path(url);
                let url = Url { scheme: site.scheme.clone(),
                                user: site.user.clone(),
                                host: site.host.clone(),
                                port: site.port.clone(),
                                path: full_path,
                                query: ~[],
                                fragment: None };
                result = self.visited_site.contains_key(&url.to_str()); 
            },
            None => {}
        }

        result
    }

    pub fn compute_url_path(&self, link_url: &str) -> ~str {
        let mut current_url_path:~[~str] = self.current_url_path.clone();
        let mut link_url_path:~[~str] = ~[];
        let mut link_url_it = link_url.chars();
        let mut file_name = ~"";

        for ch in link_url_it {
           if ch == '/' {
                if file_name.len() > 0 {
                    link_url_path.push(file_name.clone());
                    file_name = ~"";
                }
            } else {
                file_name.push_char(ch);
            }
        }

        let mut path_it = link_url_path.iter();
        let mut position = 0;

        for folder in path_it {
            if folder.equals(&~"..") {
                current_url_path.pop();
                position = -1;
            } else {
                 if position >= 0 && current_url_path.len() != 0 { 
                     if folder.equals(&current_url_path[position]) {
                         position += 1;
                     } else {
                         current_url_path.push(folder.to_owned().clone());
                     }
                 } else {
                     current_url_path.push(folder.to_owned().clone());
                 }
            }
        }
 
        let slash = ~"/";   
        let full_path = { if current_url_path.len() > 0 {
                              slash + current_url_path.connect("/")+ slash + file_name
                          } else {
                              slash + file_name
                          } };

        full_path
    }
}

impl HistoryTask {
    pub fn send(&self, msg: Msg) {
        self.chan.send(msg);
    }

    pub fn exit(&self) {
        let (response_port, response_chan) = Chan::new();
        self.send(Exit(response_chan));
        response_port.recv();
    }

    pub fn record(&self, url: Option<Url>) {
        match url {
            Some(url) => self.send(UrlRecord(url)),
            None => {}
        }
    }

    pub fn url_is_visited(&self, url: &str) -> bool {
        let (response_port, response_chan) = Chan::new();
        self.send(Visited(url.to_owned().clone(),response_chan));
        response_port.recv()
    }
}
