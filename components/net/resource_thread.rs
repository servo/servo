/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! A thread that takes a URL and streams back the binary data.

use about_loader;
use cookie;
use cookie_storage::CookieStorage;
use data_loader;
use devtools_traits::{DevtoolsControlMsg};
use file_loader;
use hsts::HstsList;
use http_loader::{self, Connector, create_http_connector, HttpState};
use hyper::client::pool::Pool;
use hyper::header::{ContentType, Header, SetCookie};
use hyper::mime::{Mime, SubLevel, TopLevel};
use ipc_channel::ipc::{self, IpcReceiver, IpcSender};
use mime_classifier::{ApacheBugFlag, MIMEClassifier, NoSniffFlag};
use net_traits::LoadContext;
use net_traits::ProgressMsg::Done;
use net_traits::{AsyncResponseTarget, Metadata, ProgressMsg, ResourceThread, ResponseAction};
use net_traits::{ControlMsg, CookieSource, LoadConsumer, LoadData, LoadResponse, ResourceId};
use net_traits::{WebSocketCommunicate, WebSocketConnectData};
use std::borrow::ToOwned;
use std::boxed::FnBox;
use std::cell::Cell;
use std::collections::HashMap;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, RwLock};
use url::Url;
use util::prefs;
use util::thread::spawn_named;
use websocket_loader;

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
                             classifier: Arc<MIMEClassifier>, partial_body: &[u8],
                             context: LoadContext)
                             -> ProgressSender {
    start_sending_sniffed_opt(start_chan, metadata, classifier, partial_body, context).ok().unwrap()
}

/// For use by loaders in responding to a Load message that allows content sniffing.
pub fn start_sending_sniffed_opt(start_chan: LoadConsumer, mut metadata: Metadata,
                                 classifier: Arc<MIMEClassifier>, partial_body: &[u8],
                                 context: LoadContext)
                                 -> Result<ProgressSender, ()> {
    if prefs::get_pref("network.mime.sniff").as_boolean().unwrap_or(false) {
        // TODO: should be calculated in the resource loader, from pull requeset #4094
        let mut no_sniff = NoSniffFlag::OFF;
        let mut check_for_apache_bug = ApacheBugFlag::OFF;

        if let Some(ref headers) = metadata.headers {
            if let Some(ref content_type) = headers.get_raw("content-type").and_then(|c| c.last()) {
                check_for_apache_bug = ApacheBugFlag::from_content_type(content_type)
            }
            if let Some(ref raw_content_type_options) = headers.get_raw("X-content-type-options") {
                if raw_content_type_options.iter().any(|ref opt| *opt == b"nosniff") {
                    no_sniff = NoSniffFlag::ON
                }
            }
        }

        let supplied_type =
            metadata.content_type.as_ref().map(|&ContentType(Mime(ref toplevel, ref sublevel, _))| {
            (format!("{}", toplevel), format!("{}", sublevel))
        });
        let (toplevel, sublevel) = classifier.classify(context,
                                                       no_sniff,
                                                       check_for_apache_bug,
                                                       &supplied_type,
                                                       &partial_body);
        let mime_tp: TopLevel = toplevel.parse().unwrap();
        let mime_sb: SubLevel = sublevel.parse().unwrap();
        metadata.content_type = Some(ContentType(Mime(mime_tp, mime_sb, vec![])));
    }

    start_sending_opt(start_chan, metadata)
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

/// Create a ResourceThread
pub fn new_resource_thread(user_agent: String,
                         devtools_chan: Option<Sender<DevtoolsControlMsg>>) -> ResourceThread {
    let hsts_preload = HstsList::from_servo_preload();
    let (setup_chan, setup_port) = ipc::channel().unwrap();
    let setup_chan_clone = setup_chan.clone();
    spawn_named("ResourceManager".to_owned(), move || {
        let resource_manager = ResourceManager::new(
            user_agent, hsts_preload, devtools_chan
        );

        let mut channel_manager = ResourceChannelManager {
            from_client: setup_port,
            resource_manager: resource_manager
        };
        channel_manager.start(setup_chan_clone);
    });
    setup_chan
}

struct ResourceChannelManager {
    from_client: IpcReceiver<ControlMsg>,
    resource_manager: ResourceManager
}

impl ResourceChannelManager {
    fn start(&mut self, control_sender: ResourceThread) {
        loop {
            match self.from_client.recv().unwrap() {
                ControlMsg::Load(load_data, consumer, id_sender) =>
                    self.resource_manager.load(load_data, consumer, id_sender, control_sender.clone()),
                ControlMsg::WebsocketConnect(connect, connect_data) =>
                    self.resource_manager.websocket_connect(connect, connect_data),
                ControlMsg::SetCookiesForUrl(request, cookie_list, source) =>
                    self.resource_manager.set_cookies_for_url(request, cookie_list, source),
                ControlMsg::GetCookiesForUrl(url, consumer, source) => {
                    let cookie_jar = &self.resource_manager.cookie_jar;
                    let mut cookie_jar = cookie_jar.write().unwrap();
                    consumer.send(cookie_jar.cookies_for_url(&url, source)).unwrap();
                }
                ControlMsg::Cancel(res_id) => {
                    if let Some(cancel_sender) = self.resource_manager.cancel_load_map.get(&res_id) {
                        let _ = cancel_sender.send(());
                    }
                    self.resource_manager.cancel_load_map.remove(&res_id);
                }
                ControlMsg::Synchronize(sender) => {
                    let _ = sender.send(());
                }
                ControlMsg::Exit => break,
            }
        }
    }
}

/// The optional resources required by the `CancellationListener`
pub struct CancellableResource {
    /// The receiver which receives a message on load cancellation
    cancel_receiver: Receiver<()>,
    /// The `CancellationListener` is unique to this `ResourceId`
    resource_id: ResourceId,
    /// If we haven't initiated any cancel requests, then the loaders ask
    /// the listener to remove the `ResourceId` in the `HashMap` of
    /// `ResourceManager` once they finish loading
    resource_thread: ResourceThread,
}

impl CancellableResource {
    pub fn new(receiver: Receiver<()>, res_id: ResourceId, res_thread: ResourceThread) -> CancellableResource {
        CancellableResource {
            cancel_receiver: receiver,
            resource_id: res_id,
            resource_thread: res_thread,
        }
    }
}

/// A listener which is basically a wrapped optional receiver which looks
/// for the load cancellation message. Some of the loading processes always keep
/// an eye out for this message and stop loading stuff once they receive it.
pub struct CancellationListener {
    /// We'll be needing the resources only if we plan to cancel it
    cancel_resource: Option<CancellableResource>,
    /// This lets us know whether the request has already been cancelled
    cancel_status: Cell<bool>,
}

impl CancellationListener {
    pub fn new(resources: Option<CancellableResource>) -> CancellationListener {
        CancellationListener {
            cancel_resource: resources,
            cancel_status: Cell::new(false),
        }
    }

    pub fn is_cancelled(&self) -> bool {
        match self.cancel_resource {
            Some(ref resource) => {
                match resource.cancel_receiver.try_recv() {
                    Ok(_) => {
                        self.cancel_status.set(true);
                        true
                    },
                    Err(_) => self.cancel_status.get(),
                }
            },
            None => false,      // channel doesn't exist!
        }
    }
}

impl Drop for CancellationListener {
    fn drop(&mut self) {
        if let Some(ref resource) = self.cancel_resource {
            // Ensure that the resource manager stops tracking this request now that it's terminated.
            let _ = resource.resource_thread.send(ControlMsg::Cancel(resource.resource_id));
        }
    }
}

pub struct AuthCacheEntry {
    pub user_name: String,
    pub password: String,
}

pub struct ResourceManager {
    user_agent: String,
    cookie_jar: Arc<RwLock<CookieStorage>>,
    auth_cache: Arc<RwLock<HashMap<Url, AuthCacheEntry>>>,
    mime_classifier: Arc<MIMEClassifier>,
    devtools_chan: Option<Sender<DevtoolsControlMsg>>,
    hsts_list: Arc<RwLock<HstsList>>,
    connector: Arc<Pool<Connector>>,
    cancel_load_map: HashMap<ResourceId, Sender<()>>,
    next_resource_id: ResourceId,
}

impl ResourceManager {
    pub fn new(user_agent: String,
               hsts_list: HstsList,
               devtools_channel: Option<Sender<DevtoolsControlMsg>>) -> ResourceManager {
        ResourceManager {
            user_agent: user_agent,
            cookie_jar: Arc::new(RwLock::new(CookieStorage::new())),
            auth_cache: Arc::new(RwLock::new(HashMap::new())),
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
                    let cookie_jar = &self.cookie_jar;
                    let mut cookie_jar = cookie_jar.write().unwrap();
                    cookie_jar.push(cookie, source);
                }
            }
        }
    }

    fn load(&mut self,
            load_data: LoadData,
            consumer: LoadConsumer,
            id_sender: Option<IpcSender<ResourceId>>,
            resource_thread: ResourceThread) {

        fn from_factory(factory: fn(LoadData, LoadConsumer, Arc<MIMEClassifier>, CancellationListener))
                        -> Box<FnBox(LoadData,
                                     LoadConsumer,
                                     Arc<MIMEClassifier>,
                                     CancellationListener) + Send> {
            box move |load_data, senders, classifier, cancel_listener| {
                factory(load_data, senders, classifier, cancel_listener)
            }
        }

        let cancel_resource = id_sender.map(|sender| {
            let current_res_id = self.next_resource_id;
            let _ = sender.send(current_res_id);
            let (cancel_sender, cancel_receiver) = channel();
            self.cancel_load_map.insert(current_res_id, cancel_sender);
            self.next_resource_id.0 += 1;
            CancellableResource::new(cancel_receiver, current_res_id, resource_thread)
        });

        let cancel_listener = CancellationListener::new(cancel_resource);
        let loader = match &*load_data.url.scheme {
            "file" => from_factory(file_loader::factory),
            "http" | "https" | "view-source" => {
                let http_state = HttpState {
                    hsts_list: self.hsts_list.clone(),
                    cookie_jar: self.cookie_jar.clone(),
                    auth_cache: self.auth_cache.clone()
                };
                http_loader::factory(self.user_agent.clone(),
                                     http_state,
                                     self.devtools_chan.clone(),
                                     self.connector.clone())
            },
            "data" => from_factory(data_loader::factory),
            "about" => from_factory(about_loader::factory),
            _ => {
                debug!("resource_thread: no loader for scheme {}", load_data.url.scheme);
                send_error(load_data.url, "no loader for scheme".to_owned(), consumer);
                return
            }
        };
        debug!("resource_thread: loading url: {}", load_data.url.serialize());

        loader.call_box((load_data,
                         consumer,
                         self.mime_classifier.clone(),
                         cancel_listener));
    }

    fn websocket_connect(&self,
                         connect: WebSocketCommunicate,
                         connect_data: WebSocketConnectData) {
        websocket_loader::init(connect, connect_data, self.cookie_jar.clone());
    }
}
