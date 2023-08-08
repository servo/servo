/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! A thread that takes a URL and streams back the binary data.

use crate::connector::{
    create_http_client, create_tls_config, CACertificates, CertificateErrorOverrideManager,
};
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
use embedder_traits::EmbedderProxy;
use hyper_serde::Serde;
use ipc_channel::ipc::{self, IpcReceiver, IpcReceiverSet, IpcSender};
use malloc_size_of::{MallocSizeOf, MallocSizeOfOps};
use net_traits::blob_url_store::parse_blob_url;
use net_traits::filemanager_thread::FileTokenCheck;
use net_traits::request::{Destination, RequestBuilder};
use net_traits::response::{Response, ResponseInit};
use net_traits::storage_thread::StorageThreadMsg;
use net_traits::DiscardFetch;
use net_traits::FetchTaskTarget;
use net_traits::WebSocketNetworkEvent;
use net_traits::{CookieSource, CoreResourceMsg, CoreResourceThread};
use net_traits::{CustomResponseMediator, FetchChannels};
use net_traits::{ResourceFetchTiming, ResourceTimingType};
use net_traits::{ResourceThreads, WebSocketDomAction};
use profile_traits::mem::ProfilerChan as MemProfilerChan;
use profile_traits::mem::{Report, ReportKind, ReportsChan};
use profile_traits::time::ProfilerChan;
use rustls::RootCertStore;
use serde::{Deserialize, Serialize};
use servo_arc::Arc as ServoArc;
use servo_url::{ImmutableOrigin, ServoUrl};
use std::borrow::{Cow, ToOwned};
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, prelude::*, BufReader};
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::time::Duration;

/// Load a file with CA certificate and produce a RootCertStore with the results.
fn load_root_cert_store_from_file(file_path: String) -> io::Result<RootCertStore> {
    let mut root_cert_store = RootCertStore::empty();

    let mut pem = BufReader::new(File::open(file_path)?);
    let certs = rustls_pemfile::certs(&mut pem)?;
    root_cert_store.add_parsable_certificates(&certs);
    Ok(root_cert_store)
}

/// Returns a tuple of (public, private) senders to the new threads.
pub fn new_resource_threads(
    user_agent: Cow<'static, str>,
    devtools_sender: Option<Sender<DevtoolsControlMsg>>,
    time_profiler_chan: ProfilerChan,
    mem_profiler_chan: MemProfilerChan,
    embedder_proxy: EmbedderProxy,
    config_dir: Option<PathBuf>,
    certificate_path: Option<String>,
    ignore_certificate_errors: bool,
) -> (ResourceThreads, ResourceThreads) {
    let ca_certificates = match certificate_path {
        Some(path) => match load_root_cert_store_from_file(path) {
            Ok(root_cert_store) => CACertificates::Override(root_cert_store),
            Err(error) => {
                warn!("Could not load CA file. Falling back to defaults. {error:?}");
                CACertificates::Default
            },
        },
        None => CACertificates::Default,
    };

    let (public_core, private_core) = new_core_resource_thread(
        user_agent,
        devtools_sender,
        time_profiler_chan,
        mem_profiler_chan,
        embedder_proxy,
        config_dir.clone(),
        ca_certificates,
        ignore_certificate_errors,
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
    devtools_sender: Option<Sender<DevtoolsControlMsg>>,
    time_profiler_chan: ProfilerChan,
    mem_profiler_chan: MemProfilerChan,
    embedder_proxy: EmbedderProxy,
    config_dir: Option<PathBuf>,
    ca_certificates: CACertificates,
    ignore_certificate_errors: bool,
) -> (CoreResourceThread, CoreResourceThread) {
    let (public_setup_chan, public_setup_port) = ipc::channel().unwrap();
    let (private_setup_chan, private_setup_port) = ipc::channel().unwrap();
    let (report_chan, report_port) = ipc::channel().unwrap();

    thread::Builder::new()
        .name("ResourceManager".to_owned())
        .spawn(move || {
            let resource_manager = CoreResourceManager::new(
                user_agent,
                devtools_sender,
                time_profiler_chan,
                embedder_proxy,
                ca_certificates.clone(),
                ignore_certificate_errors,
            );

            let mut channel_manager = ResourceChannelManager {
                resource_manager,
                config_dir,
                ca_certificates,
                ignore_certificate_errors,
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
    ca_certificates: CACertificates,
    ignore_certificate_errors: bool,
}

fn create_http_states(
    config_dir: Option<&Path>,
    ca_certificates: CACertificates,
    ignore_certificate_errors: bool,
) -> (Arc<HttpState>, Arc<HttpState>) {
    let mut hsts_list = HstsList::from_servo_preload();
    let mut auth_cache = AuthCache::new();
    let http_cache = HttpCache::new();
    let mut cookie_jar = CookieStorage::new(150);
    if let Some(config_dir) = config_dir {
        read_json_from_file(&mut auth_cache, config_dir, "auth_cache.json");
        read_json_from_file(&mut hsts_list, config_dir, "hsts_list.json");
        read_json_from_file(&mut cookie_jar, config_dir, "cookie_jar.json");
    }

    let override_manager = CertificateErrorOverrideManager::new();
    let http_state = HttpState {
        hsts_list: RwLock::new(hsts_list),
        cookie_jar: RwLock::new(cookie_jar),
        auth_cache: RwLock::new(auth_cache),
        history_states: RwLock::new(HashMap::new()),
        http_cache: RwLock::new(http_cache),
        http_cache_state: Mutex::new(HashMap::new()),
        client: create_http_client(create_tls_config(
            ca_certificates.clone(),
            ignore_certificate_errors,
            override_manager.clone(),
        )),
        override_manager,
    };

    let override_manager = CertificateErrorOverrideManager::new();
    let private_http_state = HttpState {
        hsts_list: RwLock::new(HstsList::from_servo_preload()),
        cookie_jar: RwLock::new(CookieStorage::new(150)),
        auth_cache: RwLock::new(AuthCache::new()),
        history_states: RwLock::new(HashMap::new()),
        http_cache: RwLock::new(HttpCache::new()),
        http_cache_state: Mutex::new(HashMap::new()),
        client: create_http_client(create_tls_config(
            ca_certificates,
            ignore_certificate_errors,
            override_manager.clone(),
        )),
        override_manager,
    };

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
        let (public_http_state, private_http_state) = create_http_states(
            self.config_dir.as_ref().map(Deref::deref),
            self.ca_certificates.clone(),
            self.ignore_certificate_errors,
        );

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
                FetchChannels::Prefetch => {
                    self.resource_manager
                        .fetch(req_init, None, DiscardFetch, http_state, None)
                },
            },
            CoreResourceMsg::DeleteCookies(request) => {
                http_state
                    .cookie_jar
                    .write()
                    .unwrap()
                    .clear_storage(&request);
                return true;
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
                cookie_jar.remove_expired_cookies_for_url(&url);
                consumer
                    .send(cookie_jar.cookies_for_url(&url, source))
                    .unwrap();
            },
            CoreResourceMsg::NetworkMediator(mediator_chan, origin) => {
                self.resource_manager
                    .sw_managers
                    .insert(origin, mediator_chan);
            },
            CoreResourceMsg::GetCookiesDataForUrl(url, consumer, source) => {
                let mut cookie_jar = http_state.cookie_jar.write().unwrap();
                cookie_jar.remove_expired_cookies_for_url(&url);
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
            CoreResourceMsg::SetHistoryState(history_state_id, structured_data) => {
                let mut history_states = http_state.history_states.write().unwrap();
                history_states.insert(history_state_id, structured_data);
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
            CoreResourceMsg::ClearCache => {
                http_state.http_cache.write().unwrap().clear();
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
                self.resource_manager.exit();
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
            warn!("couldn't open {}: {}", display, why);
            return;
        },
        Ok(file) => file,
    };

    let mut string_buffer: String = String::new();
    match file.read_to_string(&mut string_buffer) {
        Err(why) => panic!("couldn't read from {}: {}", display, why),
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
        Err(why) => panic!("couldn't create {}: {}", display, why),
        Ok(file) => file,
    };

    match file.write_all(json_encoded.as_bytes()) {
        Err(why) => panic!("couldn't write to {}: {}", display, why),
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
    devtools_sender: Option<Sender<DevtoolsControlMsg>>,
    sw_managers: HashMap<ImmutableOrigin, IpcSender<CustomResponseMediator>>,
    filemanager: FileManager,
    thread_pool: Arc<CoreResourceThreadPool>,
    ca_certificates: CACertificates,
    ignore_certificate_errors: bool,
}

/// The state of the thread-pool used by CoreResource.
struct ThreadPoolState {
    /// The number of active workers.
    active_workers: u32,
    /// Whether the pool can spawn additional work.
    active: bool,
}

impl ThreadPoolState {
    pub fn new() -> ThreadPoolState {
        ThreadPoolState {
            active_workers: 0,
            active: true,
        }
    }

    /// Is the pool still able to spawn new work?
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// How many workers are currently active?
    pub fn active_workers(&self) -> u32 {
        self.active_workers
    }

    /// Prevent additional work from being spawned.
    pub fn switch_to_inactive(&mut self) {
        self.active = false;
    }

    /// Add to the count of active workers.
    pub fn increment_active(&mut self) {
        self.active_workers += 1;
    }

    /// Substract from the count of active workers.
    pub fn decrement_active(&mut self) {
        self.active_workers -= 1;
    }
}

/// Threadpool used by Fetch and file operations.
pub struct CoreResourceThreadPool {
    pool: rayon::ThreadPool,
    state: Arc<Mutex<ThreadPoolState>>,
}

impl CoreResourceThreadPool {
    pub fn new(num_threads: usize) -> CoreResourceThreadPool {
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(num_threads)
            .build()
            .unwrap();
        let state = Arc::new(Mutex::new(ThreadPoolState::new()));
        CoreResourceThreadPool { pool: pool, state }
    }

    /// Spawn work on the thread-pool, if still active.
    ///
    /// There is no need to give feedback to the caller,
    /// because if we do not perform work,
    /// it is because the system as a whole is exiting.
    pub fn spawn<OP>(&self, work: OP)
    where
        OP: FnOnce() + Send + 'static,
    {
        {
            let mut state = self.state.lock().unwrap();
            if state.is_active() {
                state.increment_active();
            } else {
                // Don't spawn any work.
                return;
            }
        }

        let state = self.state.clone();

        self.pool.spawn(move || {
            {
                let mut state = state.lock().unwrap();
                if !state.is_active() {
                    // Decrement number of active workers and return,
                    // without doing any work.
                    return state.decrement_active();
                }
            }
            // Perform work.
            work();
            {
                // Decrement number of active workers.
                let mut state = state.lock().unwrap();
                state.decrement_active();
            }
        });
    }

    /// Prevent further work from being spawned,
    /// and wait until all workers are done,
    /// or a timeout of roughly one second has been reached.
    pub fn exit(&self) {
        {
            let mut state = self.state.lock().unwrap();
            state.switch_to_inactive();
        }
        let mut rounds = 0;
        loop {
            rounds += 1;
            {
                let state = self.state.lock().unwrap();
                let still_active = state.active_workers();

                if still_active == 0 || rounds == 10 {
                    if still_active > 0 {
                        debug!("Exiting CoreResourceThreadPool with {:?} still working(should be zero)", still_active);
                    }
                    break;
                }
            }
            thread::sleep(Duration::from_millis(100));
        }
    }
}

impl CoreResourceManager {
    pub fn new(
        user_agent: Cow<'static, str>,
        devtools_sender: Option<Sender<DevtoolsControlMsg>>,
        _profiler_chan: ProfilerChan,
        embedder_proxy: EmbedderProxy,
        ca_certificates: CACertificates,
        ignore_certificate_errors: bool,
    ) -> CoreResourceManager {
        let pool = CoreResourceThreadPool::new(16);
        let pool_handle = Arc::new(pool);
        CoreResourceManager {
            user_agent: user_agent,
            devtools_sender,
            sw_managers: Default::default(),
            filemanager: FileManager::new(embedder_proxy, Arc::downgrade(&pool_handle)),
            thread_pool: pool_handle,
            ca_certificates,
            ignore_certificate_errors,
        }
    }

    /// Exit the core resource manager.
    pub fn exit(&mut self) {
        // Prevents further work from being spawned on the pool,
        // blocks until all workers in the pool are done,
        // or a short timeout has been reached.
        self.thread_pool.exit();

        debug!("Exited CoreResourceManager");
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

    fn fetch<Target: 'static + FetchTaskTarget + Send>(
        &self,
        request_builder: RequestBuilder,
        res_init_: Option<ResponseInit>,
        mut sender: Target,
        http_state: &Arc<HttpState>,
        cancel_chan: Option<IpcReceiver<()>>,
    ) {
        let http_state = http_state.clone();
        let ua = self.user_agent.clone();
        let dc = self.devtools_sender.clone();
        let filemanager = self.filemanager.clone();

        let timing_type = match request_builder.destination {
            Destination::Document => ResourceTimingType::Navigation,
            _ => ResourceTimingType::Resource,
        };

        let mut request = request_builder.build();
        let url = request.current_url();

        // In the case of a valid blob URL, acquiring a token granting access to a file,
        // regardless if the URL is revoked after token acquisition.
        //
        // TODO: to make more tests pass, acquire this token earlier,
        // probably in a separate message flow.
        //
        // In such a setup, the token would not be acquired here,
        // but could instead be contained in the actual CoreResourceMsg::Fetch message.
        //
        // See https://github.com/servo/servo/issues/25226
        let (file_token, blob_url_file_id) = match url.scheme() {
            "blob" => {
                if let Ok((id, _)) = parse_blob_url(&url) {
                    (self.filemanager.get_token_for_file(&id), Some(id))
                } else {
                    (FileTokenCheck::ShouldFail, None)
                }
            },
            _ => (FileTokenCheck::NotRequired, None),
        };

        HANDLE.lock().unwrap().as_ref().unwrap().spawn(async move {
            // XXXManishearth: Check origin against pipeline id (also ensure that the mode is allowed)
            // todo load context / mimesniff in fetch
            // todo referrer policy?
            // todo service worker stuff
            let context = FetchContext {
                state: http_state,
                user_agent: ua,
                devtools_chan: dc.map(|dc| Arc::new(Mutex::new(dc))),
                filemanager: Arc::new(Mutex::new(filemanager)),
                file_token,
                cancellation_listener: Arc::new(Mutex::new(CancellationListener::new(cancel_chan))),
                timing: ServoArc::new(Mutex::new(ResourceFetchTiming::new(request.timing_type()))),
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
                    )
                    .await;
                },
                None => {
                    fetch(&mut request, &mut sender, &context).await;
                },
            };

            // Remove token after fetch.
            if let Some(id) = blob_url_file_id.as_ref() {
                context
                    .filemanager
                    .lock()
                    .unwrap()
                    .invalidate_token(&context.file_token, id);
            }
        });
    }

    fn websocket_connect(
        &self,
        request: RequestBuilder,
        event_sender: IpcSender<WebSocketNetworkEvent>,
        action_receiver: IpcReceiver<WebSocketDomAction>,
        http_state: &Arc<HttpState>,
    ) {
        websocket_loader::init(
            request,
            event_sender,
            action_receiver,
            http_state.clone(),
            self.ca_certificates.clone(),
            self.ignore_certificate_errors,
        );
    }
}
