/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! A thread that takes a URL and streams back the binary data.
use connector::{create_http_connector, create_ssl_client};
use cookie;
use cookie_rs;
use cookie_storage::CookieStorage;
use devtools_traits::DevtoolsControlMsg;
use fetch::cors_cache::CorsCache;
use fetch::methods::{FetchContext, fetch};
use filemanager_thread::{FileManager, TFDProvider};
use hsts::HstsList;
use http_loader::{HttpState, http_redirect_fetch};
use hyper_serde::Serde;
use ipc_channel::ipc::{self, IpcReceiver, IpcReceiverSet, IpcSender};
use net_traits::{CookieSource, CoreResourceThread};
use net_traits::{CoreResourceMsg, FetchResponseMsg};
use net_traits::{CustomResponseMediator, ResourceId};
use net_traits::{ResourceThreads, WebSocketCommunicate, WebSocketConnectData};
use net_traits::request::{Request, RequestInit};
use net_traits::response::{Response, ResponseInit};
use net_traits::storage_thread::StorageThreadMsg;
use profile_traits::time::ProfilerChan;
use serde::{Deserialize, Serialize};
use serde_json;
use servo_config::opts;
use servo_config::resource_files::resources_dir_path;
use servo_url::ServoUrl;
use std::borrow::{Cow, ToOwned};
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::sync::mpsc::Sender;
use std::thread;
use storage_thread::StorageThreadFactory;
use websocket_loader;

const TFD_PROVIDER: &'static TFDProvider = &TFDProvider;

/// Returns a tuple of (public, private) senders to the new threads.
pub fn new_resource_threads(user_agent: Cow<'static, str>,
                            devtools_chan: Option<Sender<DevtoolsControlMsg>>,
                            profiler_chan: ProfilerChan,
                            config_dir: Option<PathBuf>)
                            -> (ResourceThreads, ResourceThreads) {
    let (public_core, private_core) = new_core_resource_thread(
        user_agent,
        devtools_chan,
        profiler_chan,
        config_dir.clone());
    let storage: IpcSender<StorageThreadMsg> = StorageThreadFactory::new(config_dir);
    (ResourceThreads::new(public_core, storage.clone()),
     ResourceThreads::new(private_core, storage))
}


/// Create a CoreResourceThread
pub fn new_core_resource_thread(user_agent: Cow<'static, str>,
                                devtools_chan: Option<Sender<DevtoolsControlMsg>>,
                                profiler_chan: ProfilerChan,
                                config_dir: Option<PathBuf>)
                                -> (CoreResourceThread, CoreResourceThread) {
    let (public_setup_chan, public_setup_port) = ipc::channel().unwrap();
    let (private_setup_chan, private_setup_port) = ipc::channel().unwrap();
    thread::Builder::new().name("ResourceManager".to_owned()).spawn(move || {
        let resource_manager = CoreResourceManager::new(
            user_agent, devtools_chan, profiler_chan
        );

        let mut channel_manager = ResourceChannelManager {
            resource_manager: resource_manager,
            config_dir: config_dir,
        };
        channel_manager.start(public_setup_port,
                              private_setup_port);
    }).expect("Thread spawning failed");
    (public_setup_chan, private_setup_chan)
}

struct ResourceChannelManager {
    resource_manager: CoreResourceManager,
    config_dir: Option<PathBuf>,
}

fn create_http_states(config_dir: Option<&Path>) -> (Arc<HttpState>, Arc<HttpState>) {
    let mut hsts_list = HstsList::from_servo_preload();
    let mut auth_cache = AuthCache::new();
    let mut cookie_jar = CookieStorage::new(150);
    if let Some(config_dir) = config_dir {
        read_json_from_file(&mut auth_cache, config_dir, "auth_cache.json");
        read_json_from_file(&mut hsts_list, config_dir, "hsts_list.json");
        read_json_from_file(&mut cookie_jar, config_dir, "cookie_jar.json");
    }

    let ca_file = match opts::get().certificate_path {
        Some(ref path) => PathBuf::from(path),
        None => resources_dir_path()
            .expect("Need certificate file to make network requests")
            .join("certs"),
    };

    let ssl_client = create_ssl_client(&ca_file);
    let http_state = HttpState {
        cookie_jar: RwLock::new(cookie_jar),
        auth_cache: RwLock::new(auth_cache),
        hsts_list: RwLock::new(hsts_list),
        ssl_client: ssl_client.clone(),
        connector: create_http_connector(ssl_client),
    };

    let private_ssl_client = create_ssl_client(&ca_file);
    let private_http_state = HttpState::new(private_ssl_client);

    (Arc::new(http_state), Arc::new(private_http_state))
}

impl ResourceChannelManager {
    #[allow(unsafe_code)]
    fn start(&mut self,
             public_receiver: IpcReceiver<CoreResourceMsg>,
             private_receiver: IpcReceiver<CoreResourceMsg>) {
        let (public_http_state, private_http_state) =
            create_http_states(self.config_dir.as_ref().map(Deref::deref));

        let mut rx_set = IpcReceiverSet::new().unwrap();
        let private_id = rx_set.add(private_receiver).unwrap();
        let public_id = rx_set.add(public_receiver).unwrap();

        loop {
            for (id, data) in rx_set.select().unwrap().into_iter().map(|m| m.unwrap()) {
                let group = if id == private_id {
                    &private_http_state
                } else {
                    assert_eq!(id, public_id);
                    &public_http_state
                };
                if let Ok(msg) = data.to() {
                    if !self.process_msg(msg, group) {
                        return;
                    }
                }
            }
        }
    }


    /// Returns false if the thread should exit.
    fn process_msg(&mut self,
                   msg: CoreResourceMsg,
                   http_state: &Arc<HttpState>) -> bool {
        match msg {
            CoreResourceMsg::Fetch(req_init, sender) =>
                self.resource_manager.fetch(req_init, None, sender, http_state),
            CoreResourceMsg::FetchRedirect(req_init, res_init, sender) =>
                self.resource_manager.fetch(req_init, Some(res_init), sender, http_state),
            CoreResourceMsg::WebsocketConnect(connect, connect_data) =>
                self.resource_manager.websocket_connect(connect, connect_data, http_state),
            CoreResourceMsg::SetCookieForUrl(request, cookie, source) =>
                self.resource_manager.set_cookie_for_url(&request, cookie.into_inner(), source, http_state),
            CoreResourceMsg::SetCookiesForUrl(request, cookies, source) => {
                for cookie in cookies {
                    self.resource_manager.set_cookie_for_url(&request, cookie.into_inner(), source, http_state);
                }
            }
            CoreResourceMsg::GetCookiesForUrl(url, consumer, source) => {
                let mut cookie_jar = http_state.cookie_jar.write().unwrap();
                consumer.send(cookie_jar.cookies_for_url(&url, source)).unwrap();
            }
            CoreResourceMsg::NetworkMediator(mediator_chan) => {
                self.resource_manager.swmanager_chan = Some(mediator_chan)
            }
            CoreResourceMsg::GetCookiesDataForUrl(url, consumer, source) => {
                let mut cookie_jar = http_state.cookie_jar.write().unwrap();
                let cookies = cookie_jar.cookies_data_for_url(&url, source).map(Serde).collect();
                consumer.send(cookies).unwrap();
            }
            CoreResourceMsg::Cancel(res_id) => {
                if let Some(cancel_sender) = self.resource_manager.cancel_load_map.get(&res_id) {
                    let _ = cancel_sender.send(());
                }
                self.resource_manager.cancel_load_map.remove(&res_id);
            }
            CoreResourceMsg::Synchronize(sender) => {
                let _ = sender.send(());
            }
            CoreResourceMsg::ToFileManager(msg) => self.resource_manager.filemanager.handle(msg, TFD_PROVIDER),
            CoreResourceMsg::Exit(sender) => {
                if let Some(ref config_dir) = self.config_dir {
                    match http_state.auth_cache.read() {
                        Ok(auth_cache) => write_json_to_file(&*auth_cache, config_dir, "auth_cache.json"),
                        Err(_) => warn!("Error writing auth cache to disk"),
                    }
                    match http_state.cookie_jar.read() {
                        Ok(jar) => write_json_to_file(&*jar, config_dir, "cookie_jar.json"),
                        Err(_) => warn!("Error writing cookie jar to disk"),
                    }
                    match http_state.hsts_list.read() {
                        Ok(hsts) => write_json_to_file(&*hsts, config_dir, "hsts_list.json"),
                        Err(_) => warn!("Error writing hsts list to disk"),
                    }
                }
                let _ = sender.send(());
                return false;
            }
        }
        true
    }
}

pub fn read_json_from_file<T>(data: &mut T, config_dir: &Path, filename: &str)
    where T: Deserialize
{
    let path = config_dir.join(filename);
    let display = path.display();

    let mut file = match File::open(&path) {
        Err(why) => {
            warn!("couldn't open {}: {}", display, Error::description(&why));
            return;
        },
        Ok(file) => file,
    };

    let mut string_buffer: String = String::new();
    match file.read_to_string(&mut string_buffer) {
        Err(why) => {
            panic!("couldn't read from {}: {}", display,
                                                Error::description(&why))
        },
        Ok(_) => println!("successfully read from {}", display),
    }

    match serde_json::from_str(&string_buffer) {
        Ok(decoded_buffer) => *data = decoded_buffer,
        Err(why) => warn!("Could not decode buffer{}", why),
    }
}

pub fn write_json_to_file<T>(data: &T, config_dir: &Path, filename: &str)
    where T: Serialize
{
    let json_encoded: String;
    match serde_json::to_string_pretty(&data) {
        Ok(d) => json_encoded = d,
        Err(_) => return,
    }
    let path = config_dir.join(filename);
    let display = path.display();

    let mut file = match File::create(&path) {
        Err(why) => panic!("couldn't create {}: {}",
                           display,
                           Error::description(&why)),
        Ok(file) => file,
    };

    match file.write_all(json_encoded.as_bytes()) {
        Err(why) => {
            panic!("couldn't write to {}: {}", display,
                                               Error::description(&why))
        },
        Ok(_) => println!("successfully wrote to {}", display),
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct AuthCacheEntry {
    pub user_name: String,
    pub password: String,
}

impl AuthCache {
    pub fn new() -> AuthCache {
        AuthCache {
            version: 1,
            entries: HashMap::new()
        }
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct AuthCache {
    pub version: u32,
    pub entries: HashMap<String, AuthCacheEntry>,
}

pub struct CoreResourceManager {
    user_agent: Cow<'static, str>,
    devtools_chan: Option<Sender<DevtoolsControlMsg>>,
    swmanager_chan: Option<IpcSender<CustomResponseMediator>>,
    filemanager: FileManager,
    cancel_load_map: HashMap<ResourceId, Sender<()>>,
}

impl CoreResourceManager {
    pub fn new(user_agent: Cow<'static, str>,
               devtools_channel: Option<Sender<DevtoolsControlMsg>>,
               _profiler_chan: ProfilerChan) -> CoreResourceManager {
        CoreResourceManager {
            user_agent: user_agent,
            devtools_chan: devtools_channel,
            swmanager_chan: None,
            filemanager: FileManager::new(),
            cancel_load_map: HashMap::new(),
        }
    }

    fn set_cookie_for_url(&mut self, request: &ServoUrl,
                          cookie: cookie_rs::Cookie<'static>,
                          source: CookieSource,
                          http_state: &Arc<HttpState>) {
        if let Some(cookie) = cookie::Cookie::new_wrapped(cookie, request, source) {
            let mut cookie_jar = http_state.cookie_jar.write().unwrap();
            cookie_jar.push(cookie, request, source)
        }
    }

    fn fetch(&self,
             req_init: RequestInit,
             res_init_: Option<ResponseInit>,
             mut sender: IpcSender<FetchResponseMsg>,
             http_state: &Arc<HttpState>) {
        let http_state = http_state.clone();
        let ua = self.user_agent.clone();
        let dc = self.devtools_chan.clone();
        let filemanager = self.filemanager.clone();

        thread::Builder::new().name(format!("fetch thread for {}", req_init.url)).spawn(move || {
            let mut request = Request::from_init(req_init);
            // XXXManishearth: Check origin against pipeline id (also ensure that the mode is allowed)
            // todo load context / mimesniff in fetch
            // todo referrer policy?
            // todo service worker stuff
            let context = FetchContext {
                state: http_state,
                user_agent: ua,
                devtools_chan: dc,
                filemanager: filemanager,
            };

            match res_init_ {
                Some(res_init) => {
                    let response = Response::from_init(res_init);
                    http_redirect_fetch(&mut request,
                                        &mut CorsCache::new(),
                                        response,
                                        true,
                                        &mut sender,
                                        &mut None,
                                        &context);
                },
                None => fetch(&mut request, &mut sender, &context),
            };
        }).expect("Thread spawning failed");
    }

    fn websocket_connect(&self,
                         connect: WebSocketCommunicate,
                         connect_data: WebSocketConnectData,
                         http_state: &Arc<HttpState>) {
        websocket_loader::init(connect, connect_data, http_state.clone());
    }
}
