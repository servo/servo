/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! A thread that takes a URL and streams back the binary data.

use crate::connector::{create_http_client, create_ssl_connector_builder};
use crate::cookie;
use crate::cookie_storage::CookieStorage;
use crate::fetch::cors_cache::CorsCache;
use crate::fetch::methods::{fetch, CancellationListener, FetchContext};
use crate::filemanager_thread::FileManager;
use crate::hsts::HstsList;
use crate::http_cache::HttpCache;
use crate::http_loader::{http_redirect_fetch, HttpState, HANDLE};
use crate::storage_thread::StorageThreadFactory;
use crate::websocket_loader;
use crossbeam_channel::Sender;
use devtools_traits::DevtoolsControlMsg;
use embedder_traits::resources::{self, Resource};
use embedder_traits::EmbedderProxy;
use hyper_serde::Serde;
use ipc_channel::ipc::{self, IpcReceiver, IpcReceiverSet, IpcSender};
use malloc_size_of::{MallocSizeOf, MallocSizeOfOps};
use net_traits::request::{Destination, RequestBuilder};
use net_traits::response::{Response, ResponseInit};
use net_traits::storage_thread::StorageThreadMsg;
use net_traits::WebSocketNetworkEvent;
use net_traits::{CookieSource, CoreResourceMsg, CoreResourceThread};
use net_traits::{CustomResponseMediator, FetchChannels};
use net_traits::{FetchResponseMsg, ResourceThreads, WebSocketDomAction};
use net_traits::{ResourceFetchTiming, ResourceTimingType};
use profile_traits::mem::ProfilerChan as MemProfilerChan;
use profile_traits::mem::{Report, ReportKind, ReportsChan};
use profile_traits::time::ProfilerChan;
use serde::{Deserialize, Serialize};
use servo_config::opts;
use servo_url::ServoUrl;
use std::borrow::{Cow, ToOwned};
use std::collections::HashMap;
use std::error::Error;
use std::fs::{self, File};
use std::io::prelude::*;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, RwLock};
use std::thread;

/// Returns a tuple of (public, private) senders to the new threads.
pub fn new_resource_threads(
    user_agent: Cow<'static, str>,
    devtools_chan: Option<Sender<DevtoolsControlMsg>>,
    time_profiler_chan: ProfilerChan,
    mem_profiler_chan: MemProfilerChan,
    embedder_proxy: EmbedderProxy,
    config_dir: Option<PathBuf>,
) -> (ResourceThreads, ResourceThreads) {
    let (public_core, private_core) = new_core_resource_thread(
        user_agent,
        devtools_chan,
        time_profiler_chan,
        mem_profiler_chan,
        embedder_proxy,
        config_dir.clone(),
    );
    let storage: IpcSender<StorageThreadMsg> = StorageThreadFactory::new(config_dir);
    (
        ResourceThreads::new(public_core, storage.clone()),
        ResourceThreads::new(private_core, storage),
    )
}

/// Create a CoreResourceThread
pub fn new_core_resource_thread(
    user_agent: Cow<'static, str>,
    devtools_chan: Option<Sender<DevtoolsControlMsg>>,
    time_profiler_chan: ProfilerChan,
    mem_profiler_chan: MemProfilerChan,
    embedder_proxy: EmbedderProxy,
    config_dir: Option<PathBuf>,
) -> (CoreResourceThread, CoreResourceThread) {
    let (public_setup_chan, public_setup_port) = ipc::channel().unwrap();
    let (private_setup_chan, private_setup_port) = ipc::channel().unwrap();
    let (report_chan, report_port) = ipc::channel().unwrap();

    thread::Builder::new()
        .name("ResourceManager".to_owned())
        .spawn(move || {
            let resource_manager = CoreResourceManager::new(
                user_agent,
                devtools_chan,
                time_profiler_chan,
                embedder_proxy,
            );

            let mut channel_manager = ResourceChannelManager {
                resource_manager: resource_manager,
                config_dir: config_dir,
            };

            mem_profiler_chan.run_with_memory_reporting(
                || (channel_manager.start(public_setup_port, private_setup_port, report_port)),
                String::from("network-cache-reporter"),
                report_chan,
                |report_chan| report_chan,
            );
        })
        .expect("Thread spawning failed");
    (public_setup_chan, private_setup_chan)
}

struct ResourceChannelManager {
    resource_manager: CoreResourceManager,
    config_dir: Option<PathBuf>,
}

fn create_http_states(config_dir: Option<&Path>) -> (Arc<HttpState>, Arc<HttpState>) {
    let mut hsts_list = HstsList::from_servo_preload();
    let mut auth_cache = AuthCache::new();
    let http_cache = HttpCache::new();
    let mut cookie_jar = CookieStorage::new(150);
    if let Some(config_dir) = config_dir {
        read_json_from_file(&mut auth_cache, config_dir, "auth_cache.json");
        read_json_from_file(&mut hsts_list, config_dir, "hsts_list.json");
        read_json_from_file(&mut cookie_jar, config_dir, "cookie_jar.json");
    }

    let certs = match opts::get().certificate_path {
        Some(ref path) => fs::read_to_string(path).expect("Couldn't not find certificate file"),
        None => resources::read_string(Resource::SSLCertificates),
    };

    let ssl_connector_builder = create_ssl_connector_builder(&certs);
    let http_state = HttpState {
        cookie_jar: RwLock::new(cookie_jar),
        auth_cache: RwLock::new(auth_cache),
        http_cache: RwLock::new(http_cache),
        hsts_list: RwLock::new(hsts_list),
        history_states: RwLock::new(HashMap::new()),
        client: create_http_client(ssl_connector_builder, HANDLE.lock().unwrap().executor()),
    };

    let private_ssl_client = create_ssl_connector_builder(&certs);
    let private_http_state = HttpState::new(private_ssl_client);

    (Arc::new(http_state), Arc::new(private_http_state))
}

impl ResourceChannelManager {
    #[allow(unsafe_code)]
    fn start(
        &mut self,
        public_receiver: IpcReceiver<CoreResourceMsg>,
        private_receiver: IpcReceiver<CoreResourceMsg>,
        memory_reporter: IpcReceiver<ReportsChan>,
    ) {
        let (public_http_state, private_http_state) =
            create_http_states(self.config_dir.as_ref().map(Deref::deref));

        let mut rx_set = IpcReceiverSet::new().unwrap();
        let private_id = rx_set.add(private_receiver).unwrap();
        let public_id = rx_set.add(public_receiver).unwrap();
        let reporter_id = rx_set.add(memory_reporter).unwrap();

        loop {
            for receiver in rx_set.select().unwrap().into_iter() {
                // Handles case where profiler thread shuts down before resource thread.
                match receiver {
                    ipc::IpcSelectionResult::ChannelClosed(..) => continue,
                    _ => {},
                }
                let (id, data) = receiver.unwrap();
                // If message is memory report, get the size_of of public and private http caches
                if id == reporter_id {
                    if let Ok(msg) = data.to() {
                        self.process_report(msg, &private_http_state, &public_http_state);
                        continue;
                    }
                } else {
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
    }

    fn process_report(
        &mut self,
        msg: ReportsChan,
        public_http_state: &Arc<HttpState>,
        private_http_state: &Arc<HttpState>,
    ) {
        let mut ops = MallocSizeOfOps::new(servo_allocator::usable_size, None, None);
        let public_cache = public_http_state.http_cache.read().unwrap();
        let private_cache = private_http_state.http_cache.read().unwrap();

        let public_report = Report {
            path: path!["memory-cache", "public"],
            kind: ReportKind::ExplicitJemallocHeapSize,
            size: public_cache.size_of(&mut ops),
        };

        let private_report = Report {
            path: path!["memory-cache", "private"],
            kind: ReportKind::ExplicitJemallocHeapSize,
            size: private_cache.size_of(&mut ops),
        };

        msg.send(vec![public_report, private_report]);
    }

    /// Returns false if the thread should exit.
    fn process_msg(&mut self, msg: CoreResourceMsg, http_state: &Arc<HttpState>) -> bool {
        match msg {
            CoreResourceMsg::Fetch(req_init, channels) => match channels {
                FetchChannels::ResponseMsg(sender, cancel_chan) => {
                    self.resource_manager
                        .fetch(req_init, None, sender, http_state, cancel_chan)
                },
                FetchChannels::WebSocket {
                    event_sender,
                    action_receiver,
                } => self.resource_manager.websocket_connect(
                    req_init,
                    event_sender,
                    action_receiver,
                    http_state,
                ),
            },
            CoreResourceMsg::FetchRedirect(req_init, res_init, sender, cancel_chan) => self
                .resource_manager
                .fetch(req_init, Some(res_init), sender, http_state, cancel_chan),
            CoreResourceMsg::SetCookieForUrl(request, cookie, source) => self
                .resource_manager
                .set_cookie_for_url(&request, cookie.into_inner(), source, http_state),
            CoreResourceMsg::SetCookiesForUrl(request, cookies, source) => {
                for cookie in cookies {
                    self.resource_manager.set_cookie_for_url(
                        &request,
                        cookie.into_inner(),
                        source,
                        http_state,
                    );
                }
            },
            CoreResourceMsg::GetCookiesForUrl(url, consumer, source) => {
                let mut cookie_jar = http_state.cookie_jar.write().unwrap();
                consumer
                    .send(cookie_jar.cookies_for_url(&url, source))
                    .unwrap();
            },
            CoreResourceMsg::NetworkMediator(mediator_chan) => {
                self.resource_manager.swmanager_chan = Some(mediator_chan)
            },
            CoreResourceMsg::GetCookiesDataForUrl(url, consumer, source) => {
                let mut cookie_jar = http_state.cookie_jar.write().unwrap();
                let cookies = cookie_jar
                    .cookies_data_for_url(&url, source)
                    .map(Serde)
                    .collect();
                consumer.send(cookies).unwrap();
            },
            CoreResourceMsg::GetHistoryState(history_state_id, consumer) => {
                let history_states = http_state.history_states.read().unwrap();
                consumer
                    .send(history_states.get(&history_state_id).cloned())
                    .unwrap();
            },
            CoreResourceMsg::SetHistoryState(history_state_id, history_state) => {
                let mut history_states = http_state.history_states.write().unwrap();
                history_states.insert(history_state_id, history_state);
            },
            CoreResourceMsg::RemoveHistoryStates(states_to_remove) => {
                let mut history_states = http_state.history_states.write().unwrap();
                for history_state in states_to_remove {
                    history_states.remove(&history_state);
                }
            },
            CoreResourceMsg::Synchronize(sender) => {
                let _ = sender.send(());
            },
            CoreResourceMsg::ToFileManager(msg) => self.resource_manager.filemanager.handle(msg),
            CoreResourceMsg::Exit(sender) => {
                if let Some(ref config_dir) = self.config_dir {
                    match http_state.auth_cache.read() {
                        Ok(auth_cache) => {
                            write_json_to_file(&*auth_cache, config_dir, "auth_cache.json")
                        },
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
            },
        }
        true
    }
}

pub fn read_json_from_file<T>(data: &mut T, config_dir: &Path, filename: &str)
where
    T: for<'de> Deserialize<'de>,
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
        Err(why) => panic!(
            "couldn't read from {}: {}",
            display,
            Error::description(&why)
        ),
        Ok(_) => println!("successfully read from {}", display),
    }

    match serde_json::from_str(&string_buffer) {
        Ok(decoded_buffer) => *data = decoded_buffer,
        Err(why) => warn!("Could not decode buffer{}", why),
    }
}

pub fn write_json_to_file<T>(data: &T, config_dir: &Path, filename: &str)
where
    T: Serialize,
{
    let json_encoded: String;
    match serde_json::to_string_pretty(&data) {
        Ok(d) => json_encoded = d,
        Err(_) => return,
    }
    let path = config_dir.join(filename);
    let display = path.display();

    let mut file = match File::create(&path) {
        Err(why) => panic!("couldn't create {}: {}", display, Error::description(&why)),
        Ok(file) => file,
    };

    match file.write_all(json_encoded.as_bytes()) {
        Err(why) => panic!(
            "couldn't write to {}: {}",
            display,
            Error::description(&why)
        ),
        Ok(_) => println!("successfully wrote to {}", display),
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AuthCacheEntry {
    pub user_name: String,
    pub password: String,
}

impl AuthCache {
    pub fn new() -> AuthCache {
        AuthCache {
            version: 1,
            entries: HashMap::new(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AuthCache {
    pub version: u32,
    pub entries: HashMap<String, AuthCacheEntry>,
}

pub struct CoreResourceManager {
    user_agent: Cow<'static, str>,
    devtools_chan: Option<Sender<DevtoolsControlMsg>>,
    swmanager_chan: Option<IpcSender<CustomResponseMediator>>,
    filemanager: FileManager,
}

impl CoreResourceManager {
    pub fn new(
        user_agent: Cow<'static, str>,
        devtools_channel: Option<Sender<DevtoolsControlMsg>>,
        _profiler_chan: ProfilerChan,
        embedder_proxy: EmbedderProxy,
    ) -> CoreResourceManager {
        CoreResourceManager {
            user_agent: user_agent,
            devtools_chan: devtools_channel,
            swmanager_chan: None,
            filemanager: FileManager::new(embedder_proxy),
        }
    }

    fn set_cookie_for_url(
        &mut self,
        request: &ServoUrl,
        cookie: cookie_rs::Cookie<'static>,
        source: CookieSource,
        http_state: &Arc<HttpState>,
    ) {
        if let Some(cookie) = cookie::Cookie::new_wrapped(cookie, request, source) {
            let mut cookie_jar = http_state.cookie_jar.write().unwrap();
            cookie_jar.push(cookie, request, source)
        }
    }

    fn fetch(
        &self,
        request_builder: RequestBuilder,
        res_init_: Option<ResponseInit>,
        mut sender: IpcSender<FetchResponseMsg>,
        http_state: &Arc<HttpState>,
        cancel_chan: Option<IpcReceiver<()>>,
    ) {
        let http_state = http_state.clone();
        let ua = self.user_agent.clone();
        let dc = self.devtools_chan.clone();
        let filemanager = self.filemanager.clone();

        let timing_type = match request_builder.destination {
            Destination::Document => ResourceTimingType::Navigation,
            _ => ResourceTimingType::Resource,
        };

        thread::Builder::new()
            .name(format!("fetch thread for {}", request_builder.url))
            .spawn(move || {
                let mut request = request_builder.build();
                // XXXManishearth: Check origin against pipeline id (also ensure that the mode is allowed)
                // todo load context / mimesniff in fetch
                // todo referrer policy?
                // todo service worker stuff
                let context = FetchContext {
                    state: http_state,
                    user_agent: ua,
                    devtools_chan: dc,
                    filemanager: filemanager,
                    cancellation_listener: Arc::new(Mutex::new(CancellationListener::new(
                        cancel_chan,
                    ))),
                    timing: Arc::new(Mutex::new(ResourceFetchTiming::new(request.timing_type()))),
                };

                match res_init_ {
                    Some(res_init) => {
                        let response = Response::from_init(res_init, timing_type);
                        http_redirect_fetch(
                            &mut request,
                            &mut CorsCache::new(),
                            response,
                            true,
                            &mut sender,
                            &mut None,
                            &context,
                        );
                    },
                    None => fetch(&mut request, &mut sender, &context),
                };
            })
            .expect("Thread spawning failed");
    }

    fn websocket_connect(
        &self,
        request: RequestBuilder,
        event_sender: IpcSender<WebSocketNetworkEvent>,
        action_receiver: IpcReceiver<WebSocketDomAction>,
        http_state: &Arc<HttpState>,
    ) {
        websocket_loader::init(request, event_sender, action_receiver, http_state.clone());
    }
}
