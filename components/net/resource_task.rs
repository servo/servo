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
use cookie_storage::{CookieStorage, CookieSource};
use cookie;

use util::task::spawn_named;

use hyper::header::UserAgent;
use hyper::header::{Headers, Header, SetCookie};
use hyper::http::RawStatus;
use hyper::method::Method;
use hyper::mime::{Mime, Attr};
use url::Url;

use std::borrow::{ToOwned, IntoCow};
use std::collections::HashMap;
use std::env;
use std::mem;
use std::old_io::{BufferedReader, File};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thunk::Invoke;

#[cfg(test)]
use std::old_io::{Listener, Acceptor, TimedOut};
#[cfg(test)]
use std::old_io::net::tcp::TcpListener;

static mut HOST_TABLE: Option<*mut HashMap<String, String>> = None;

pub fn global_init() {
    if let Ok(host_file_path) = env::var_string("HOST_FILE") {
        //TODO: handle bad file path
        let path = Path::new(host_file_path);
        let mut file = BufferedReader::new(File::open(&path));
        if let Ok(lines) = file.read_to_string(){
            unsafe {
                let host_table: *mut HashMap<String, String> =  mem::transmute(parse_hostsfile(lines.as_slice()));
                HOST_TABLE = Some(host_table);
            }
        }
    }
}

pub enum ControlMsg {
    /// Request the data associated with a particular URL
    Load(LoadData),
    /// Store a set of cookies for a given originating URL
    SetCookiesForUrl(Url, String, CookieSource),
    /// Retrieve the stored cookies for a given URL
    GetCookiesForUrl(Url, Sender<Option<String>>, CookieSource),
    Exit
}

#[derive(Clone)]
pub struct LoadData {
    pub url: Url,
    pub method: Method,
    /// Headers that will apply to the initial request only
    pub headers: Headers,
    /// Headers that will apply to the initial request and any redirects
    pub preserved_headers: Headers,
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
            preserved_headers: Headers::new(),
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
#[derive(Clone)]
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
    pub status: Option<RawStatus>,
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
            status: Some(RawStatus(200, "OK".into_cow())),
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
#[derive(PartialEq,Debug)]
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
    resource_task.send(ControlMsg::Load(LoadData::new(url, start_chan))).unwrap();
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
    let setup_chan_clone = setup_chan.clone();
    spawn_named("ResourceManager".to_owned(), move || {
        ResourceManager::new(setup_port, user_agent, sniffer_task, setup_chan_clone).start();
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
                if token[].as_bytes()[0] == b'#' {
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
    sniffer_task: SnifferTask,
    cookie_storage: CookieStorage,
    resource_task: Sender<ControlMsg>,
}

impl ResourceManager {
    fn new(from_client: Receiver<ControlMsg>, user_agent: Option<String>, sniffer_task: SnifferTask,
           resource_task: Sender<ControlMsg>) -> ResourceManager {
        ResourceManager {
            from_client: from_client,
            user_agent: user_agent,
            sniffer_task: sniffer_task,
            cookie_storage: CookieStorage::new(),
            resource_task: resource_task,
        }
    }
}


impl ResourceManager {
    fn start(&mut self) {
        loop {
            match self.from_client.recv().unwrap() {
              ControlMsg::Load(load_data) => {
                self.load(load_data)
              }
              ControlMsg::SetCookiesForUrl(request, cookie_list, source) => {
                let header = Header::parse_header([cookie_list.into_bytes()].as_slice());
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

    fn load(&mut self, mut load_data: LoadData) {
        unsafe {
            if let Some(host_table) = HOST_TABLE {
                load_data = replace_hosts(load_data, host_table);
            }
        }

        self.user_agent.as_ref().map(|ua| load_data.headers.set(UserAgent(ua.clone())));
        let senders = ResponseSenders {
            immediate_consumer: self.sniffer_task.clone(),
            eventual_consumer: load_data.consumer.clone(),
        };

        fn from_factory(factory: fn(LoadData, Sender<TargetedLoadResponse>))
                        -> Box<Invoke<(LoadData, Sender<TargetedLoadResponse>)> + Send> {
            box move |&:(load_data, start_chan)| {
                factory(load_data, start_chan)
            }
        }

        let loader = match load_data.url.scheme.as_slice() {
            "file" => from_factory(file_loader::factory),
            "http" | "https" | "view-source" => http_loader::factory(self.resource_task.clone()),
            "data" => from_factory(data_loader::factory),
            "about" => from_factory(about_loader::factory),
            _ => {
                debug!("resource_task: no loader for scheme {}", load_data.url.scheme);
                start_sending(senders, Metadata::default(load_data.url))
                    .send(ProgressMsg::Done(Err("no loader for scheme".to_string()))).unwrap();
                return
            }
        };
        debug!("resource_task: loading url: {}", load_data.url.serialize());

        loader.invoke((load_data, self.sniffer_task.clone()));
    }
}

/// Load a URL asynchronously and iterate over chunks of bytes from the response.
pub fn load_bytes_iter(resource_task: &ResourceTask, url: Url) -> (Metadata, ProgressMsgPortIterator) {
    let (input_chan, input_port) = channel();
    resource_task.send(ControlMsg::Load(LoadData::new(url, input_chan))).unwrap();

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

#[test]
fn test_parse_hostsfile() {
    let mock_hosts_file_content = "127.0.0.1 foo.bar.com\n127.0.0.2 servo.test.server";
    let hosts_table = parse_hostsfile(mock_hosts_file_content);
    assert_eq!(2, (*hosts_table).len());
    assert_eq!("127.0.0.1".to_owned(), *hosts_table.get(&"foo.bar.com".to_owned()).unwrap());
    assert_eq!("127.0.0.2".to_owned(), *hosts_table.get(&"servo.test.server".to_owned()).unwrap());
}

#[test]
fn test_parse_malformed_hostsfile() {
    let mock_hosts_file_content = "malformed file\n127.0.0.1 foo.bar.com\nservo.test.server 127.0.0.1";
    let hosts_table = parse_hostsfile(mock_hosts_file_content);
    assert_eq!(1, (*hosts_table).len());
    assert_eq!("127.0.0.1".to_owned(), *hosts_table.get(&"foo.bar.com".to_owned()).unwrap());
}

#[test]
fn test_parse_hostsfile_with_line_comment() {
    let mock_hosts_file_content = "# this is a line comment\n127.0.0.1 foo.bar.com\n# anothercomment";
    let hosts_table = parse_hostsfile(mock_hosts_file_content);
    assert_eq!(1, (*hosts_table).len());
    assert_eq!("127.0.0.1".to_owned(), *hosts_table.get(&"foo.bar.com".to_owned()).unwrap());
}

#[test]
fn test_parse_hostsfile_with_end_of_line_comment() {
    let mock_hosts_file_content = "127.0.0.1 foo.bar.com # line ending comment\n127.0.0.2 servo.test.server #comment";
    let hosts_table = parse_hostsfile(mock_hosts_file_content);
    assert_eq!(2, (*hosts_table).len());
    assert_eq!("127.0.0.1".to_owned(), *hosts_table.get(&"foo.bar.com".to_owned()).unwrap());
    assert_eq!("127.0.0.2".to_owned(), *hosts_table.get(&"servo.test.server".to_owned()).unwrap());
}

#[test]
fn test_parse_hostsfile_with_2_hostnames_for_1_address() {
    let mock_hosts_file_content = "127.0.0.1 foo.bar.com baz.bar.com";
    let hosts_table = parse_hostsfile(mock_hosts_file_content);
    assert_eq!(2, (*hosts_table).len());
    assert_eq!("127.0.0.1".to_owned(), *hosts_table.get(&"foo.bar.com".to_owned()).unwrap());
    assert_eq!("127.0.0.1".to_owned(), *hosts_table.get(&"baz.bar.com".to_owned()).unwrap());
}

#[test]
fn test_parse_hostsfile_with_4_hostnames_for_1_address() {
    let mock_hosts_file_content = "127.0.0.1 moz.foo.com moz.bar.com moz.baz.com moz.moz.com";
    let hosts_table = parse_hostsfile(mock_hosts_file_content);
    assert_eq!(4, (*hosts_table).len());
    assert_eq!("127.0.0.1".to_owned(), *hosts_table.get(&"moz.foo.com".to_owned()).unwrap());
    assert_eq!("127.0.0.1".to_owned(), *hosts_table.get(&"moz.bar.com".to_owned()).unwrap());
    assert_eq!("127.0.0.1".to_owned(), *hosts_table.get(&"moz.baz.com".to_owned()).unwrap());
    assert_eq!("127.0.0.1".to_owned(), *hosts_table.get(&"moz.moz.com".to_owned()).unwrap());
}

#[test]
fn test_parse_hostsfile_with_tabs_instead_spaces() {
    let mock_hosts_file_content = "127.0.0.1\tfoo.bar.com\n127.0.0.2\tservo.test.server";
    let hosts_table = parse_hostsfile(mock_hosts_file_content);
    assert_eq!(2, (*hosts_table).len());
    assert_eq!("127.0.0.1".to_owned(), *hosts_table.get(&"foo.bar.com".to_owned()).unwrap());
    assert_eq!("127.0.0.2".to_owned(), *hosts_table.get(&"servo.test.server".to_owned()).unwrap());
}

#[test]
fn test_parse_hostsfile_with_valid_ipv4_addresses()
{
    let mock_hosts_file_content = "255.255.255.255 foo.bar.com\n169.0.1.201 servo.test.server\n192.168.5.0 servo.foo.com";
    let hosts_table = parse_hostsfile(mock_hosts_file_content);
    assert_eq!(3, (*hosts_table).len());
}

#[test]
fn test_parse_hostsfile_with_invalid_ipv4_addresses()
{
    let mock_hosts_file_content = "256.255.255.255 foo.bar.com\n169.0.1000.201 servo.test.server \
                                   \n192.168.5.500 servo.foo.com\n192.abc.100.2 test.servo.com";
    let hosts_table = parse_hostsfile(mock_hosts_file_content);
    assert_eq!(0, (*hosts_table).len());
}

#[test]
fn test_parse_hostsfile_with_valid_ipv6_addresses()
{
    let mock_hosts_file_content = "2001:0db8:0000:0000:0000:ff00:0042:8329 foo.bar.com\n\
                                   2001:db8:0:0:0:ff00:42:8329 moz.foo.com\n\
                                   2001:db8::ff00:42:8329 foo.moz.com moz.moz.com\n\
                                   0000:0000:0000:0000:0000:0000:0000:0001 bar.moz.com\n\
                                   ::1 foo.bar.baz baz.foo.com\n\
                                   2001:0DB8:85A3:0042:1000:8A2E:0370:7334 baz.bar.moz\n\
                                   2002:0DB8:85A3:0042:1000:8A2E:0370:7334/96 baz2.bar.moz\n\
                                   2002:0DB8:85A3:0042:1000:8A2E:0370:7334/128 baz3.bar.moz\n\
                                   :: unspecified.moz.com\n\
                                   ::/128 unspecified.address.com";
    let hosts_table = parse_hostsfile(mock_hosts_file_content);
    assert_eq!(12, (*hosts_table).len());
}

#[test]
fn test_parse_hostsfile_with_invalid_ipv6_addresses()
{
    let mock_hosts_file_content = "12001:0db8:0000:0000:0000:ff00:0042:8329 foo.bar.com\n\
                                   2001:zdb8:0:0:0:gg00:42:t329 moz.foo.com\n\
                                   2001:db8::ff00:42:8329:1111:1111:42 foo.moz.com moz.moz.com\n\
                                   2002:0DB8:85A3:0042:1000:8A2E:0370:7334/1289 baz3.bar.moz";
    let hosts_table = parse_hostsfile(mock_hosts_file_content);
    assert_eq!(0, (*hosts_table).len());
}

#[test]
fn test_parse_hostsfile_with_end_of_line_whitespace()
{
    let mock_hosts_file_content = "127.0.0.1 foo.bar.com \n\
                                   2001:db8:0:0:0:ff00:42:8329 moz.foo.com\n \
                                   127.0.0.2 servo.test.server ";
    let hosts_table = parse_hostsfile(mock_hosts_file_content);
    assert_eq!(3, (*hosts_table).len());
    assert_eq!("127.0.0.1".to_owned(), *hosts_table.get(&"foo.bar.com".to_owned()).unwrap());
    assert_eq!("2001:db8:0:0:0:ff00:42:8329".to_owned(), *hosts_table.get(&"moz.foo.com".to_owned()).unwrap());
    assert_eq!("127.0.0.2".to_owned(), *hosts_table.get(&"servo.test.server".to_owned()).unwrap());
}

#[test]
fn test_replace_hosts() {
    let mut host_table_box = box HashMap::new();
    host_table_box.insert("foo.bar.com".to_owned(), "127.0.0.1".to_owned());
    host_table_box.insert("servo.test.server".to_owned(), "127.0.0.2".to_owned());

    let host_table: *mut HashMap<String, String> = unsafe {mem::transmute(host_table_box)};

    //Start the TCP server
    let mut listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.socket_name().unwrap().port;
    let mut acceptor = listener.listen().unwrap();

    //Start the resource task and make a request to our TCP server
    let resource_task = new_resource_task(None);
    let (start_chan, _) = channel();
    let mut raw_url: String = "http://foo.bar.com:".to_string();
    raw_url = raw_url + port.to_string().as_slice();
    let url = Url::parse(raw_url.as_slice()).unwrap();
    resource_task.send(ControlMsg::Load(replace_hosts(LoadData::new(url, start_chan), host_table)));

    match acceptor.accept() {
        Ok(..) => assert!(true, "received request"),
        Err(ref e) if e.kind == TimedOut => { assert!(false, "timed out!");  },
        Err(_) => assert!(false, "error")
    }

    resource_task.send(ControlMsg::Exit);
    drop(acceptor);
}
