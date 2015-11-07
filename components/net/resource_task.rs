/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! A task that takes a URL and streams back the binary data.

use about_loader;
use cookie;
use cookie_storage::CookieStorage;
use data_loader;
use devtools_traits::{DevtoolsControlMsg};
use file_loader;
use hsts::{HSTSList, preload_hsts_domains};
use http_loader::{self, Connector, create_http_connector};
use hyper::client::pool::Pool;
use hyper::header::{ContentType, Header, SetCookie};
use hyper::mime::{Mime, SubLevel, TopLevel};
use ipc_channel::ipc::{self, IpcReceiver, IpcSender};
use mime_classifier::{ApacheBugFlag, MIMEClassifier, NoSniffFlag};
use net_traits::ProgressMsg::Done;
use net_traits::{AsyncResponseTarget, Metadata, ProgressMsg, ResourceTask, ResponseAction};
use net_traits::{ControlMsg, CookieSource, LoadConsumer, LoadData, LoadResponse, ResourceId};
use std::borrow::ToOwned;
use std::boxed::FnBox;
use std::cell::Cell;
use std::collections::HashMap;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, RwLock};
use url::Url;
use util::opts;
use util::task::spawn_named;

pub enum ProgressSender {
    Channel(IpcSender<ProgressMsg>),
    Listener(AsyncResponseTarget),
}

impl ProgressSender {
    //XXXjdm return actual error
    pub fn send(&self, msg: ProgressMsg) -> Result<(), ()> {
        match *self {
            ProgressSender::Channel(ref c) => c.send(msg).map_err(|_| ()),
            ProgressSender::Listener(ref b) => {
                let action = match msg {
                    ProgressMsg::Payload(buf) => ResponseAction::DataAvailable(buf),
                    ProgressMsg::Done(status) => ResponseAction::ResponseComplete(status),
                };
                b.invoke_with_listener(action);
                Ok(())
            }
        }
    }
}

pub fn send_error(url: Url, err: String, start_chan: LoadConsumer) {
    let mut metadata: Metadata = Metadata::default(url);
    metadata.status = None;

    if let Ok(p) = start_sending_opt(start_chan, metadata) {
        p.send(Done(Err(err))).unwrap();
    }
}

/// For use by loaders in responding to a Load message that allows content sniffing.
pub fn start_sending_sniffed(start_chan: LoadConsumer, metadata: Metadata,
                             classifier: Arc<MIMEClassifier>, partial_body: &[u8])
                             -> ProgressSender {
    start_sending_sniffed_opt(start_chan, metadata, classifier, partial_body).ok().unwrap()
}

/// For use by loaders in responding to a Load message that allows content sniffing.
pub fn start_sending_sniffed_opt(start_chan: LoadConsumer, mut metadata: Metadata,
                                 classifier: Arc<MIMEClassifier>, partial_body: &[u8])
                                 -> Result<ProgressSender, ()> {
    if opts::get().sniff_mime_types {
        // TODO: should be calculated in the resource loader, from pull requeset #4094
        let mut no_sniff = NoSniffFlag::OFF;
        let mut check_for_apache_bug = ApacheBugFlag::OFF;

        if let Some(ref headers) = metadata.headers {
            if let Some(ref raw_content_type) = headers.get_raw("content-type") {
                if raw_content_type.len() > 0 {
                    let last_raw_content_type = &raw_content_type[raw_content_type.len() - 1];
                    check_for_apache_bug = apache_bug_predicate(last_raw_content_type)
                }
            }
            if let Some(ref raw_content_type_options) = headers.get_raw("X-content-type-options") {
                if raw_content_type_options.iter().any(|ref opt| *opt == b"nosniff") {
                    no_sniff = NoSniffFlag::ON
                }
            }
        }

        let supplied_type =
            metadata.content_type.map(|ContentType(Mime(toplevel, sublevel, _))| {
            (format!("{}", toplevel), format!("{}", sublevel))
        });
        let (toplevel, sublevel) = classifier.classify(no_sniff,
                                                       check_for_apache_bug,
                                                       &supplied_type,
                                                       &partial_body);
        let mime_tp: TopLevel = toplevel.parse().unwrap();
        let mime_sb: SubLevel = sublevel.parse().unwrap();
        metadata.content_type = Some(ContentType(Mime(mime_tp, mime_sb, vec![])));
    }

    start_sending_opt(start_chan, metadata)
}

fn apache_bug_predicate(last_raw_content_type: &[u8]) -> ApacheBugFlag {
    if last_raw_content_type == b"text/plain"
           || last_raw_content_type == b"text/plain; charset=ISO-8859-1"
           || last_raw_content_type == b"text/plain; charset=iso-8859-1"
           || last_raw_content_type == b"text/plain; charset=UTF-8" {
        ApacheBugFlag::ON
    } else {
        ApacheBugFlag::OFF
    }
}

/// For use by loaders in responding to a Load message.
fn start_sending_opt(start_chan: LoadConsumer, metadata: Metadata) -> Result<ProgressSender, ()> {
    match start_chan {
        LoadConsumer::Channel(start_chan) => {
            let (progress_chan, progress_port) = ipc::channel().unwrap();
            let result = start_chan.send(LoadResponse {
                metadata:      metadata,
                progress_port: progress_port,
            });
            match result {
                Ok(_) => Ok(ProgressSender::Channel(progress_chan)),
                Err(_) => Err(())
            }
        }
        LoadConsumer::Listener(target) => {
            target.invoke_with_listener(ResponseAction::HeadersAvailable(metadata));
            Ok(ProgressSender::Listener(target))
        }
    }
}

/// Create a ResourceTask
pub fn new_resource_task(user_agent: String,
                         devtools_chan: Option<Sender<DevtoolsControlMsg>>) -> ResourceTask {
    let hsts_preload = match preload_hsts_domains() {
        Some(list) => list,
        None => HSTSList::new()
    };

    let (setup_chan, setup_port) = ipc::channel().unwrap();
    spawn_named("ResourceManager".to_owned(), move || {
        let resource_manager = ResourceManager::new(
            user_agent, hsts_preload, devtools_chan
        );

        let mut channel_manager = ResourceChannelManager {
            from_client: setup_port,
            resource_manager: resource_manager
        };

        channel_manager.start();
    });
    setup_chan
}

struct ResourceChannelManager {
    from_client: IpcReceiver<ControlMsg>,
    resource_manager: ResourceManager
}

impl ResourceChannelManager {
    fn start(&mut self) {
        loop {
            match self.from_client.recv().unwrap() {
                ControlMsg::Load(load_data, consumer, id_sender) =>
                    self.resource_manager.load(load_data, consumer, id_sender),
                ControlMsg::SetCookiesForUrl(request, cookie_list, source) =>
                    self.resource_manager.set_cookies_for_url(request, cookie_list, source),
                ControlMsg::GetCookiesForUrl(url, consumer, source) => {
                    let cookie_jar = &self.resource_manager.cookie_storage;
                    let mut cookie_jar = cookie_jar.write().unwrap();
                    consumer.send(cookie_jar.cookies_for_url(&url, source)).unwrap();
                }
                ControlMsg::Cancel(res_id) => {
                    if let Some(cancel_sender) = self.resource_manager.cancel_load_map.get(&res_id) {
                        let _ = cancel_sender.send(CancelLoad);
                    }
                    self.resource_manager.cancel_load_map.remove(&res_id);
                }
                ControlMsg::Exit => break,
            }
        }
    }
}

/// A load cancellation message which is sent to the listener on cancellation
pub struct CancelLoad;

/// A listener which is basically a wrapped optional receiver which looks
/// for the `CancelLoad` message. Some of the loading processes always keep
/// an eye out for this message and stop loading stuff once they receive it.
pub struct CancellationListener {
    /// The receiver which receives the cancellation message
    receiver: Option<Receiver<CancelLoad>>,
    /// This lets us know whether the request has already been cancelled
    cancel_status: Cell<bool>,
    /// If we haven't initiated any cancel requests, then the loaders ask
    /// the listener to remove the stored `ResourceId` once they finish loading
    id_remover_sender: Option<Sender<()>>,
}

impl CancellationListener {
    pub fn new(receiver: Option<Receiver<CancelLoad>>,
               id_remover_sender: Option<Sender<()>>) -> CancellationListener {
        CancellationListener {
            receiver: receiver,
            cancel_status: Cell::new(false),
            id_remover_sender: id_remover_sender,
        }
    }

    pub fn is_cancelled(&self) -> bool {
        match self.receiver {
            Some(ref receiver) => match receiver.try_recv() {
                Ok(_) => {
                    self.cancel_status.set(true);
                    true
                },
                Err(_) => self.cancel_status.get(),
            },
            None => false,      // channel doesn't exist!
        }
    }

    pub fn remove_resource_id(&self) {
        if let Some(ref sender) = self.id_remover_sender {
            let _ = sender.send(());
        }
    }
}

pub struct ResourceManager {
    user_agent: String,
    cookie_storage: Arc<RwLock<CookieStorage>>,
    mime_classifier: Arc<MIMEClassifier>,
    devtools_chan: Option<Sender<DevtoolsControlMsg>>,
    hsts_list: Arc<RwLock<HSTSList>>,
    connector: Arc<Pool<Connector>>,
    cancel_load_map: HashMap<ResourceId, Sender<CancelLoad>>,
    next_resource_id: ResourceId,
}

impl ResourceManager {
    pub fn new(user_agent: String,
               hsts_list: HSTSList,
               devtools_channel: Option<Sender<DevtoolsControlMsg>>) -> ResourceManager {
        ResourceManager {
            user_agent: user_agent,
            cookie_storage: Arc::new(RwLock::new(CookieStorage::new())),
            mime_classifier: Arc::new(MIMEClassifier::new()),
            devtools_chan: devtools_channel,
            hsts_list: Arc::new(RwLock::new(hsts_list)),
            connector: create_http_connector(),
            cancel_load_map: HashMap::new(),
            next_resource_id: ResourceId(0),
        }
    }

    fn set_cookies_for_url(&mut self, request: Url, cookie_list: String, source: CookieSource) {
        let header = Header::parse_header(&[cookie_list.into_bytes()]);
        if let Ok(SetCookie(cookies)) = header {
            for bare_cookie in cookies {
                if let Some(cookie) = cookie::Cookie::new_wrapped(bare_cookie, &request, source) {
                    let cookie_jar = &self.cookie_storage;
                    let mut cookie_jar = cookie_jar.write().unwrap();
                    cookie_jar.push(cookie, source);
                }
            }
        }
    }

    fn load(&mut self,
            load_data: LoadData,
            consumer: LoadConsumer,
            id_sender: Option<IpcSender<ResourceId>>) {

        fn from_factory(factory: fn(LoadData, LoadConsumer, Arc<MIMEClassifier>, CancellationListener))
                        -> Box<FnBox(LoadData,
                                     LoadConsumer,
                                     Arc<MIMEClassifier>,
                                     CancellationListener) + Send> {
            box move |load_data, senders, classifier, cancel_listener| {
                factory(load_data, senders, classifier, cancel_listener)
            }
        }

        let current_res_id = self.next_resource_id;
        let (cancel_listener, id_remover_receiver) = id_sender.map_or_else(|| {
            (CancellationListener::new(None, None), None)
        }, |sender| {
            let (id_remover_sender, id_remover_receiver) = channel();
            let resource_id = self.next_resource_id;
            let _ = sender.send(resource_id);
            let (cancel_sender, cancel_receiver) = channel();
            self.cancel_load_map.insert(resource_id, cancel_sender);
            self.next_resource_id.0 += 1;
            (CancellationListener::new(Some(cancel_receiver), Some(id_remover_sender)),
             Some(id_remover_receiver))
        });

        let loader = match &*load_data.url.scheme {
            "file" => from_factory(file_loader::factory),
            "http" | "https" | "view-source" =>
                http_loader::factory(self.user_agent.clone(),
                                     self.hsts_list.clone(),
                                     self.cookie_storage.clone(),
                                     self.devtools_chan.clone(),
                                     self.connector.clone()),
            "data" => from_factory(data_loader::factory),
            "about" => from_factory(about_loader::factory),
            _ => {
                debug!("resource_task: no loader for scheme {}", load_data.url.scheme);
                send_error(load_data.url, "no loader for scheme".to_owned(), consumer);
                return
            }
        };
        debug!("resource_task: loading url: {}", load_data.url.serialize());

        loader.call_box((load_data,
                         consumer,
                         self.mime_classifier.clone(),
                         cancel_listener));

        if let Some(receiver) = id_remover_receiver {
            let _ = receiver.recv();
            self.cancel_load_map.remove(&current_res_id);
        }
    }
}
