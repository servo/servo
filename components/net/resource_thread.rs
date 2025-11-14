/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! A thread that takes a URL and streams back the binary data.

use std::borrow::ToOwned;
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufReader};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Weak};
use std::thread;

use base::id::CookieStoreId;
use base::threadpool::ThreadPool;
use cookie::Cookie;
use crossbeam_channel::Sender;
use devtools_traits::DevtoolsControlMsg;
use embedder_traits::EmbedderProxy;
use hyper_serde::Serde;
use ipc_channel::ipc::{self, IpcReceiver, IpcReceiverSet, IpcSender};
use log::{debug, trace, warn};
use net_traits::blob_url_store::parse_blob_url;
use net_traits::filemanager_thread::FileTokenCheck;
use net_traits::pub_domains::public_suffix_list_size_of;
use net_traits::request::{Destination, RequestBuilder, RequestId};
use net_traits::response::{Response, ResponseInit};
use net_traits::{
    AsyncRuntime, CookieAsyncResponse, CookieData, CookieSource, CoreResourceMsg,
    CoreResourceThread, CustomResponseMediator, DiscardFetch, FetchChannels, FetchTaskTarget,
    ResourceFetchTiming, ResourceThreads, ResourceTimingType, WebSocketDomAction,
    WebSocketNetworkEvent,
};
use parking_lot::{Mutex, RwLock};
use profile_traits::mem::{
    ProcessReports, ProfilerChan as MemProfilerChan, Report, ReportKind, ReportsChan,
    perform_memory_report,
};
use profile_traits::path;
use profile_traits::time::ProfilerChan;
use rustc_hash::FxHashMap;
use rustls_pki_types::CertificateDer;
use rustls_pki_types::pem::PemObject;
use serde::{Deserialize, Serialize};
use servo_arc::Arc as ServoArc;
use servo_url::{ImmutableOrigin, ServoUrl};

use crate::async_runtime::{init_async_runtime, spawn_task};
use crate::connector::{
    CACertificates, CertificateErrorOverrideManager, create_http_client, create_tls_config,
};
use crate::cookie::ServoCookie;
use crate::cookie_storage::CookieStorage;
use crate::fetch::cors_cache::CorsCache;
use crate::fetch::fetch_params::FetchParams;
use crate::fetch::methods::{CancellationListener, FetchContext, WebSocketChannel, fetch};
use crate::filemanager_thread::FileManager;
use crate::hsts::{self, HstsList};
use crate::http_cache::HttpCache;
use crate::http_loader::{HttpState, http_redirect_fetch};
use crate::protocols::ProtocolRegistry;
use crate::request_interceptor::RequestInterceptor;
use crate::websocket_loader::create_handshake_request;

/// Load a file with CA certificate and produce a RootCertStore with the results.
fn load_root_cert_store_from_file(file_path: String) -> io::Result<Vec<CertificateDer<'static>>> {
    let mut pem = BufReader::new(File::open(file_path)?);

    let certs = CertificateDer::pem_reader_iter(&mut pem)
        .filter_map(|cert| {
            cert.inspect_err(|e| log::error!("Could not load certificate ({e}). Ignoring it."))
                .ok()
        })
        .collect();
    Ok(certs)
}

/// Returns a tuple of (public, private) senders to the new threads.
#[allow(clippy::too_many_arguments)]
pub fn new_resource_threads(
    devtools_sender: Option<Sender<DevtoolsControlMsg>>,
    time_profiler_chan: ProfilerChan,
    mem_profiler_chan: MemProfilerChan,
    embedder_proxy: EmbedderProxy,
    config_dir: Option<PathBuf>,
    certificate_path: Option<String>,
    ignore_certificate_errors: bool,
    protocols: Arc<ProtocolRegistry>,
) -> (ResourceThreads, ResourceThreads, Box<dyn AsyncRuntime>) {
    // Initialize the async runtime, and get a handle to it for use in clean shutdown.
    let async_runtime = init_async_runtime();

    let ca_certificates = certificate_path
        .and_then(|path| {
            Some(CACertificates::Override(
                load_root_cert_store_from_file(path).ok()?,
            ))
        })
        .unwrap_or_default();

    let (public_core, private_core) = new_core_resource_thread(
        devtools_sender,
        time_profiler_chan,
        mem_profiler_chan.clone(),
        embedder_proxy,
        config_dir.clone(),
        ca_certificates,
        ignore_certificate_errors,
        protocols,
    );
    (
        ResourceThreads::new(public_core),
        ResourceThreads::new(private_core),
        async_runtime,
    )
}

/// Create a CoreResourceThread
#[allow(clippy::too_many_arguments)]
pub fn new_core_resource_thread(
    devtools_sender: Option<Sender<DevtoolsControlMsg>>,
    time_profiler_chan: ProfilerChan,
    mem_profiler_chan: MemProfilerChan,
    embedder_proxy: EmbedderProxy,
    config_dir: Option<PathBuf>,
    ca_certificates: CACertificates<'static>,
    ignore_certificate_errors: bool,
    protocols: Arc<ProtocolRegistry>,
) -> (CoreResourceThread, CoreResourceThread) {
    let (public_setup_chan, public_setup_port) = ipc::channel().unwrap();
    let (private_setup_chan, private_setup_port) = ipc::channel().unwrap();
    let (report_chan, report_port) = ipc::channel().unwrap();

    thread::Builder::new()
        .name("ResourceManager".to_owned())
        .spawn(move || {
            let resource_manager = CoreResourceManager::new(
                devtools_sender,
                time_profiler_chan,
                embedder_proxy.clone(),
                ca_certificates.clone(),
                ignore_certificate_errors,
            );

            let mut channel_manager = ResourceChannelManager {
                resource_manager,
                config_dir,
                ca_certificates,
                ignore_certificate_errors,
                cancellation_listeners: Default::default(),
                cookie_listeners: Default::default(),
            };

            mem_profiler_chan.run_with_memory_reporting(
                || {
                    channel_manager.start(
                        public_setup_port,
                        private_setup_port,
                        report_port,
                        protocols,
                        embedder_proxy,
                    )
                },
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
    ca_certificates: CACertificates<'static>,
    ignore_certificate_errors: bool,
    cancellation_listeners: FxHashMap<RequestId, Weak<CancellationListener>>,
    cookie_listeners: FxHashMap<CookieStoreId, IpcSender<CookieAsyncResponse>>,
}

/// This returns a tuple HttpState and a private HttpState.
fn create_http_states(
    config_dir: Option<&Path>,
    ca_certificates: CACertificates<'static>,
    ignore_certificate_errors: bool,
    embedder_proxy: EmbedderProxy,
) -> (Arc<HttpState>, Arc<HttpState>) {
    let mut hsts_list = HstsList::default();
    let mut auth_cache = AuthCache::default();
    let mut cookie_jar = CookieStorage::new(150);
    if let Some(config_dir) = config_dir {
        base::read_json_from_file(&mut auth_cache, config_dir, "auth_cache.json");
        base::read_json_from_file(&mut hsts_list, config_dir, "hsts_list.json");
        base::read_json_from_file(&mut cookie_jar, config_dir, "cookie_jar.json");
    }

    let override_manager = CertificateErrorOverrideManager::new();
    let http_state = HttpState {
        hsts_list: RwLock::new(hsts_list),
        cookie_jar: RwLock::new(cookie_jar),
        auth_cache: RwLock::new(auth_cache),
        history_states: RwLock::new(FxHashMap::default()),
        http_cache: HttpCache::default(),
        client: create_http_client(create_tls_config(
            ca_certificates.clone(),
            ignore_certificate_errors,
            override_manager.clone(),
        )),
        override_manager,
        embedder_proxy: Mutex::new(embedder_proxy.clone()),
    };

    let override_manager = CertificateErrorOverrideManager::new();
    let private_http_state = HttpState {
        hsts_list: RwLock::new(HstsList::default()),
        cookie_jar: RwLock::new(CookieStorage::new(150)),
        auth_cache: RwLock::new(AuthCache::default()),
        history_states: RwLock::new(FxHashMap::default()),
        http_cache: HttpCache::default(),
        client: create_http_client(create_tls_config(
            ca_certificates,
            ignore_certificate_errors,
            override_manager.clone(),
        )),
        override_manager,
        embedder_proxy: Mutex::new(embedder_proxy),
    };

    (Arc::new(http_state), Arc::new(private_http_state))
}

impl ResourceChannelManager {
    fn start(
        &mut self,
        public_receiver: IpcReceiver<CoreResourceMsg>,
        private_receiver: IpcReceiver<CoreResourceMsg>,
        memory_reporter: IpcReceiver<ReportsChan>,
        protocols: Arc<ProtocolRegistry>,
        embedder_proxy: EmbedderProxy,
    ) {
        let (public_http_state, private_http_state) = create_http_states(
            self.config_dir.as_deref(),
            self.ca_certificates.clone(),
            self.ignore_certificate_errors,
            embedder_proxy,
        );

        let mut rx_set = IpcReceiverSet::new().unwrap();
        let private_id = rx_set.add(private_receiver).unwrap();
        let public_id = rx_set.add(public_receiver).unwrap();
        let reporter_id = rx_set.add(memory_reporter).unwrap();

        loop {
            for receiver in rx_set.select().unwrap().into_iter() {
                // Handles case where profiler thread shuts down before resource thread.
                if let ipc::IpcSelectionResult::ChannelClosed(..) = receiver {
                    continue;
                }
                let (id, data) = receiver.unwrap();
                // If message is memory report, get the size_of of public and private http caches
                if id == reporter_id {
                    if let Ok(msg) = data.to() {
                        self.process_report(msg, &public_http_state, &private_http_state);
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
                        if !self.process_msg(msg, group, Arc::clone(&protocols)) {
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
        perform_memory_report(|ops| {
            let mut reports = public_http_state.memory_reports("public", ops);
            reports.extend(private_http_state.memory_reports("private", ops));
            reports.extend(vec![
                Report {
                    path: path!["hsts-preload-list"],
                    kind: ReportKind::ExplicitJemallocHeapSize,
                    size: hsts::hsts_preload_size_of(ops),
                },
                Report {
                    path: path!["public-suffix-list"],
                    kind: ReportKind::ExplicitJemallocHeapSize,
                    size: public_suffix_list_size_of(ops),
                },
            ]);
            msg.send(ProcessReports::new(reports));
        })
    }

    fn cancellation_listener(&self, request_id: RequestId) -> Option<Arc<CancellationListener>> {
        self.cancellation_listeners
            .get(&request_id)
            .and_then(Weak::upgrade)
    }

    fn get_or_create_cancellation_listener(
        &mut self,
        request_id: RequestId,
    ) -> Arc<CancellationListener> {
        if let Some(listener) = self.cancellation_listener(request_id) {
            return listener;
        }

        // Clear away any cancellation listeners that are no longer valid.
        self.cancellation_listeners
            .retain(|_, listener| listener.strong_count() > 0);

        let cancellation_listener = Arc::new(Default::default());
        self.cancellation_listeners
            .insert(request_id, Arc::downgrade(&cancellation_listener));
        cancellation_listener
    }

    fn send_cookie_response(&self, store_id: CookieStoreId, data: CookieData) {
        let Some(sender) = self.cookie_listeners.get(&store_id) else {
            warn!(
                "Async cookie request made for store id that is non-existent {:?}",
                store_id
            );
            return;
        };
        let res = sender.send(CookieAsyncResponse { data });
        if res.is_err() {
            warn!("Unable to send cookie response to script thread");
        }
    }

    /// Returns false if the thread should exit.
    fn process_msg(
        &mut self,
        msg: CoreResourceMsg,
        http_state: &Arc<HttpState>,
        protocols: Arc<ProtocolRegistry>,
    ) -> bool {
        match msg {
            CoreResourceMsg::Fetch(request_builder, channels) => match channels {
                FetchChannels::ResponseMsg(sender) => {
                    let cancellation_listener =
                        self.get_or_create_cancellation_listener(request_builder.id);
                    self.resource_manager.fetch(
                        request_builder,
                        None,
                        sender,
                        http_state,
                        cancellation_listener,
                        protocols,
                    );
                },
                FetchChannels::WebSocket {
                    event_sender,
                    action_receiver,
                } => {
                    let cancellation_listener =
                        self.get_or_create_cancellation_listener(request_builder.id);

                    self.resource_manager.websocket_connect(
                        request_builder,
                        event_sender,
                        action_receiver,
                        http_state,
                        cancellation_listener,
                        protocols,
                    )
                },
                FetchChannels::Prefetch => self.resource_manager.fetch(
                    request_builder,
                    None,
                    DiscardFetch,
                    http_state,
                    Arc::new(Default::default()),
                    protocols,
                ),
            },
            CoreResourceMsg::Cancel(request_ids) => {
                for cancellation_listener in request_ids
                    .into_iter()
                    .filter_map(|request_id| self.cancellation_listener(request_id))
                {
                    cancellation_listener.cancel();
                }
            },
            CoreResourceMsg::DeleteCookies(request, sender) => {
                http_state
                    .cookie_jar
                    .write()
                    .clear_storage(request.as_ref());
                if let Some(sender) = sender {
                    let _ = sender.send(());
                }
                return true;
            },
            CoreResourceMsg::DeleteCookie(request, name) => {
                http_state
                    .cookie_jar
                    .write()
                    .delete_cookie_with_name(&request, name);
                return true;
            },
            CoreResourceMsg::DeleteCookieAsync(cookie_store_id, url, name) => {
                http_state
                    .cookie_jar
                    .write()
                    .delete_cookie_with_name(&url, name);
                self.send_cookie_response(cookie_store_id, CookieData::Delete(Ok(())));
            },
            CoreResourceMsg::FetchRedirect(request_builder, res_init, sender) => {
                let cancellation_listener =
                    self.get_or_create_cancellation_listener(request_builder.id);
                self.resource_manager.fetch(
                    request_builder,
                    Some(res_init),
                    sender,
                    http_state,
                    cancellation_listener,
                    protocols,
                )
            },
            CoreResourceMsg::SetCookieForUrl(request, cookie, source) => self
                .resource_manager
                .set_cookie_for_url(&request, cookie.into_inner().to_owned(), source, http_state),
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
            CoreResourceMsg::SetCookieForUrlAsync(cookie_store_id, url, cookie, source) => {
                self.resource_manager.set_cookie_for_url(
                    &url,
                    cookie.into_inner().to_owned(),
                    source,
                    http_state,
                );
                self.send_cookie_response(cookie_store_id, CookieData::Set(Ok(())));
            },
            CoreResourceMsg::GetCookiesForUrl(url, consumer, source) => {
                let mut cookie_jar = http_state.cookie_jar.write();
                cookie_jar.remove_expired_cookies_for_url(&url);
                consumer
                    .send(cookie_jar.cookies_for_url(&url, source))
                    .unwrap();
            },
            CoreResourceMsg::GetCookieDataForUrlAsync(cookie_store_id, url, name) => {
                let mut cookie_jar = http_state.cookie_jar.write();
                cookie_jar.remove_expired_cookies_for_url(&url);
                let cookie = cookie_jar
                    .query_cookies(&url, name)
                    .into_iter()
                    .map(Serde)
                    .next();
                self.send_cookie_response(cookie_store_id, CookieData::Get(cookie));
            },
            CoreResourceMsg::GetAllCookieDataForUrlAsync(cookie_store_id, url, name) => {
                let mut cookie_jar = http_state.cookie_jar.write();
                cookie_jar.remove_expired_cookies_for_url(&url);
                let cookies = cookie_jar
                    .query_cookies(&url, name)
                    .into_iter()
                    .map(Serde)
                    .collect();
                self.send_cookie_response(cookie_store_id, CookieData::GetAll(cookies));
            },
            CoreResourceMsg::NewCookieListener(cookie_store_id, sender, _url) => {
                // TODO: Use the URL for setting up the actual monitoring
                self.cookie_listeners.insert(cookie_store_id, sender);
            },
            CoreResourceMsg::RemoveCookieListener(cookie_store_id) => {
                self.cookie_listeners.remove(&cookie_store_id);
            },
            CoreResourceMsg::NetworkMediator(mediator_chan, origin) => {
                self.resource_manager
                    .sw_managers
                    .insert(origin, mediator_chan);
            },
            CoreResourceMsg::GetCookiesDataForUrl(url, consumer, source) => {
                let mut cookie_jar = http_state.cookie_jar.write();
                cookie_jar.remove_expired_cookies_for_url(&url);
                let cookies = cookie_jar
                    .cookies_data_for_url(&url, source)
                    .map(Serde)
                    .collect();
                consumer.send(cookies).unwrap();
            },
            CoreResourceMsg::GetHistoryState(history_state_id, consumer) => {
                let history_states = http_state.history_states.read();
                consumer
                    .send(history_states.get(&history_state_id).cloned())
                    .unwrap();
            },
            CoreResourceMsg::SetHistoryState(history_state_id, structured_data) => {
                let mut history_states = http_state.history_states.write();
                history_states.insert(history_state_id, structured_data);
            },
            CoreResourceMsg::RemoveHistoryStates(states_to_remove) => {
                let mut history_states = http_state.history_states.write();
                for history_state in states_to_remove {
                    history_states.remove(&history_state);
                }
            },
            CoreResourceMsg::ClearCache => {
                http_state.http_cache.clear();
            },
            CoreResourceMsg::ToFileManager(msg) => self.resource_manager.filemanager.handle(msg),
            CoreResourceMsg::Exit(sender) => {
                if let Some(ref config_dir) = self.config_dir {
                    let auth_cache = http_state.auth_cache.read();
                    base::write_json_to_file(&*auth_cache, config_dir, "auth_cache.json");
                    let jar = http_state.cookie_jar.read();
                    base::write_json_to_file(&*jar, config_dir, "cookie_jar.json");
                    let hsts = http_state.hsts_list.read();
                    base::write_json_to_file(&*hsts, config_dir, "hsts_list.json");
                }
                self.resource_manager.exit();
                let _ = sender.send(());
                return false;
            },
        }
        true
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AuthCacheEntry {
    pub user_name: String,
    pub password: String,
}

impl Default for AuthCache {
    fn default() -> Self {
        Self {
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
    devtools_sender: Option<Sender<DevtoolsControlMsg>>,
    sw_managers: HashMap<ImmutableOrigin, IpcSender<CustomResponseMediator>>,
    filemanager: FileManager,
    request_interceptor: RequestInterceptor,
    thread_pool: Arc<ThreadPool>,
    ca_certificates: CACertificates<'static>,
    ignore_certificate_errors: bool,
}

impl CoreResourceManager {
    pub fn new(
        devtools_sender: Option<Sender<DevtoolsControlMsg>>,
        _profiler_chan: ProfilerChan,
        embedder_proxy: EmbedderProxy,
        ca_certificates: CACertificates<'static>,
        ignore_certificate_errors: bool,
    ) -> CoreResourceManager {
        let num_threads = thread::available_parallelism()
            .map(|i| i.get())
            .unwrap_or(servo_config::pref!(threadpools_fallback_worker_num) as usize)
            .min(servo_config::pref!(threadpools_resource_workers_max).max(1) as usize);
        let pool = ThreadPool::new(num_threads, "CoreResourceThreadPool".to_string());
        let pool_handle = Arc::new(pool);
        CoreResourceManager {
            devtools_sender,
            sw_managers: Default::default(),
            filemanager: FileManager::new(embedder_proxy.clone(), Arc::downgrade(&pool_handle)),
            request_interceptor: RequestInterceptor::new(embedder_proxy),
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
        cookie: Cookie<'static>,
        source: CookieSource,
        http_state: &Arc<HttpState>,
    ) {
        if let Some(cookie) = ServoCookie::new_wrapped(cookie, request, source) {
            let mut cookie_jar = http_state.cookie_jar.write();
            cookie_jar.push(cookie, request, source)
        }
    }

    fn fetch<Target: 'static + FetchTaskTarget + Send>(
        &self,
        request_builder: RequestBuilder,
        res_init_: Option<ResponseInit>,
        mut sender: Target,
        http_state: &Arc<HttpState>,
        cancellation_listener: Arc<CancellationListener>,
        protocols: Arc<ProtocolRegistry>,
    ) {
        let http_state = http_state.clone();
        let dc = self.devtools_sender.clone();
        let filemanager = self.filemanager.clone();
        let request_interceptor = self.request_interceptor.clone();

        let timing_type = match request_builder.destination {
            Destination::Document => ResourceTimingType::Navigation,
            _ => ResourceTimingType::Resource,
        };

        let request = request_builder.build();
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

        let ca_certificates = self.ca_certificates.clone();
        let ignore_certificate_errors = self.ignore_certificate_errors;

        spawn_task(async move {
            // XXXManishearth: Check origin against pipeline id (also ensure that the mode is allowed)
            // todo load context / mimesniff in fetch
            // todo referrer policy?
            // todo service worker stuff
            let context = FetchContext {
                state: http_state,
                user_agent: servo_config::pref!(user_agent),
                devtools_chan: dc.map(|dc| Arc::new(Mutex::new(dc))),
                filemanager: Arc::new(Mutex::new(filemanager)),
                file_token,
                request_interceptor: Arc::new(Mutex::new(request_interceptor)),
                cancellation_listener,
                timing: ServoArc::new(Mutex::new(ResourceFetchTiming::new(request.timing_type()))),
                protocols,
                websocket_chan: None,
                ca_certificates,
                ignore_certificate_errors,
            };

            match res_init_ {
                Some(res_init) => {
                    let response = Response::from_init(res_init, timing_type);

                    let mut fetch_params = FetchParams::new(request);
                    http_redirect_fetch(
                        &mut fetch_params,
                        &mut CorsCache::default(),
                        response,
                        true,
                        &mut sender,
                        &mut None,
                        &context,
                    )
                    .await;
                },
                None => {
                    fetch(request, &mut sender, &context).await;
                },
            };

            // Remove token after fetch.
            if let Some(id) = blob_url_file_id.as_ref() {
                context
                    .filemanager
                    .lock()
                    .invalidate_token(&context.file_token, id);
            }
        });
    }

    /// <https://websockets.spec.whatwg.org/#concept-websocket-establish>
    fn websocket_connect(
        &self,
        mut request: RequestBuilder,
        event_sender: IpcSender<WebSocketNetworkEvent>,
        action_receiver: IpcReceiver<WebSocketDomAction>,
        http_state: &Arc<HttpState>,
        cancellation_listener: Arc<CancellationListener>,
        protocols: Arc<ProtocolRegistry>,
    ) {
        let http_state = http_state.clone();
        let dc = self.devtools_sender.clone();
        let filemanager = self.filemanager.clone();
        let request_interceptor = self.request_interceptor.clone();

        let ca_certificates = self.ca_certificates.clone();
        let ignore_certificate_errors = self.ignore_certificate_errors;

        spawn_task(async move {
            let mut event_sender = event_sender;

            // Let requestURL be a copy of url, with its scheme set to "http", if url’s scheme is
            // "ws"; otherwise to "https"
            let scheme = match request.url.scheme() {
                "ws" => "http",
                _ => "https",
            };
            request
                .url
                .as_mut_url()
                .set_scheme(scheme)
                .unwrap_or_else(|_| panic!("Can't set scheme to {scheme}"));

            match create_handshake_request(request, http_state.clone()) {
                Ok(request) => {
                    let context = FetchContext {
                        state: http_state,
                        user_agent: servo_config::pref!(user_agent),
                        devtools_chan: dc.map(|dc| Arc::new(Mutex::new(dc))),
                        filemanager: Arc::new(Mutex::new(filemanager)),
                        file_token: FileTokenCheck::NotRequired,
                        request_interceptor: Arc::new(Mutex::new(request_interceptor)),
                        cancellation_listener,
                        timing: ServoArc::new(Mutex::new(ResourceFetchTiming::new(
                            request.timing_type(),
                        ))),
                        protocols: protocols.clone(),
                        websocket_chan: Some(Arc::new(Mutex::new(WebSocketChannel::new(
                            event_sender.clone(),
                            Some(action_receiver),
                        )))),
                        ca_certificates,
                        ignore_certificate_errors,
                    };
                    fetch(request, &mut event_sender, &context).await;
                },
                Err(e) => {
                    trace!("unable to create websocket handshake request {:?}", e);
                    let _ = event_sender.send(WebSocketNetworkEvent::Fail);
                },
            }
        });
    }
}
