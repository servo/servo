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

use base::generic_channel::{
    self, CallbackSetter, GenericReceiver, GenericReceiverSet, GenericSelectionResult,
};
use base::id::CookieStoreId;
use cookie::Cookie;
use crossbeam_channel::Sender;
use devtools_traits::DevtoolsControlMsg;
use embedder_traits::GenericEmbedderProxy;
use hyper_serde::Serde;
use ipc_channel::ipc::IpcSender;
use log::{debug, trace, warn};
use net_traits::blob_url_store::parse_blob_url;
use net_traits::filemanager_thread::FileTokenCheck;
use net_traits::pub_domains::public_suffix_list_size_of;
use net_traits::request::{Destination, PreloadEntry, PreloadId, RequestBuilder, RequestId};
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
use tokio::sync::Mutex as TokioMutex;

use crate::async_runtime::{init_async_runtime, spawn_task};
use crate::connector::{
    CACertificates, CertificateErrorOverrideManager, create_http_client, create_tls_config,
};
use crate::cookie::ServoCookie;
use crate::cookie_storage::CookieStorage;
use crate::embedder::NetToEmbedderMsg;
use crate::fetch::cors_cache::CorsCache;
use crate::fetch::fetch_params::{FetchParams, SharedPreloadedResources};
use crate::fetch::methods::{
    CancellationListener, FetchContext, SharedInflightKeepAliveRecords, WebSocketChannel, fetch,
};
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
#[expect(clippy::too_many_arguments)]
pub fn new_resource_threads(
    devtools_sender: Option<Sender<DevtoolsControlMsg>>,
    time_profiler_chan: ProfilerChan,
    mem_profiler_chan: MemProfilerChan,
    embedder_proxy: GenericEmbedderProxy<NetToEmbedderMsg>,
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
#[expect(clippy::too_many_arguments)]
pub fn new_core_resource_thread(
    devtools_sender: Option<Sender<DevtoolsControlMsg>>,
    time_profiler_chan: ProfilerChan,
    mem_profiler_chan: MemProfilerChan,
    embedder_proxy: GenericEmbedderProxy<NetToEmbedderMsg>,
    config_dir: Option<PathBuf>,
    ca_certificates: CACertificates<'static>,
    ignore_certificate_errors: bool,
    protocols: Arc<ProtocolRegistry>,
) -> (CoreResourceThread, CoreResourceThread) {
    let (public_setup_chan, public_setup_port) = generic_channel::channel().unwrap();
    let (private_setup_chan, private_setup_port) = generic_channel::channel().unwrap();
    let (report_chan, report_port) = generic_channel::channel().unwrap();

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
                CoreResourceMsg::CollectMemoryReport,
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
    embedder_proxy: GenericEmbedderProxy<NetToEmbedderMsg>,
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
        embedder_proxy: embedder_proxy.clone(),
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
        embedder_proxy,
    };

    (Arc::new(http_state), Arc::new(private_http_state))
}

impl ResourceChannelManager {
    fn start(
        &mut self,
        public_receiver: GenericReceiver<CoreResourceMsg>,
        private_receiver: GenericReceiver<CoreResourceMsg>,
        memory_reporter: GenericReceiver<CoreResourceMsg>,
        protocols: Arc<ProtocolRegistry>,
        embedder_proxy: GenericEmbedderProxy<NetToEmbedderMsg>,
    ) {
        let (public_http_state, private_http_state) = create_http_states(
            self.config_dir.as_deref(),
            self.ca_certificates.clone(),
            self.ignore_certificate_errors,
            embedder_proxy,
        );

        let mut rx_set = GenericReceiverSet::new();
        let private_id = rx_set.add(private_receiver);
        let public_id = rx_set.add(public_receiver);
        let reporter_id = rx_set.add(memory_reporter);

        loop {
            for received in rx_set.select().into_iter() {
                // Handles case where profiler thread shuts down before resource thread.
                match received {
                    GenericSelectionResult::ChannelClosed(_) => continue,
                    GenericSelectionResult::Error(error) => {
                        log::error!("Found selection error: {error}")
                    },
                    GenericSelectionResult::MessageReceived(id, msg) => {
                        if id == reporter_id {
                            if let CoreResourceMsg::CollectMemoryReport(report_chan) = msg {
                                self.process_report(
                                    report_chan,
                                    &public_http_state,
                                    &private_http_state,
                                );
                                continue;
                            } else {
                                log::error!("memory reporter should only send CollectMemoryReport");
                            }
                        } else {
                            let group = if id == private_id {
                                &private_http_state
                            } else {
                                assert_eq!(id, public_id);
                                &public_http_state
                            };
                            if !self.process_msg(msg, group, Arc::clone(&protocols)) {
                                return;
                            }
                        }
                    },
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
            CoreResourceMsg::DeleteCookiesForSites(sites, sender) => {
                http_state
                    .cookie_jar
                    .write()
                    .delete_cookies_for_sites(&sites);
                let _ = sender.send(());
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
            CoreResourceMsg::ListCookies(sender) => {
                let mut cookie_jar = http_state.cookie_jar.write();
                cookie_jar.remove_all_expired_cookies();
                let _ = sender.send(cookie_jar.cookie_site_descriptors());
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
            CoreResourceMsg::GetCacheEntries(sender) => {
                let _ = sender.send(http_state.http_cache.cache_entry_descriptors());
            },
            CoreResourceMsg::ClearCache(sender) => {
                http_state.http_cache.clear();
                if let Some(sender) = sender {
                    let _ = sender.send(());
                }
            },
            CoreResourceMsg::ToFileManager(msg) => self.resource_manager.filemanager.handle(msg),
            CoreResourceMsg::StorePreloadedResponse(preload_id, response) => self
                .resource_manager
                .handle_preloaded_response(preload_id, response),
            CoreResourceMsg::TotalSizeOfInFlightKeepAliveRecords(pipeline_id, sender) => {
                let total = self
                    .resource_manager
                    .in_flight_keep_alive_records
                    .lock()
                    .get(&pipeline_id)
                    .map(|records| {
                        records
                            .iter()
                            .map(|record| record.keep_alive_body_length)
                            .sum()
                    })
                    .unwrap_or_default();
                let _ = sender.send(total);
            },
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
            // Ignore this message as we handle it only in the reporter chan
            CoreResourceMsg::CollectMemoryReport(_) => {},
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
    ca_certificates: CACertificates<'static>,
    ignore_certificate_errors: bool,
    preloaded_resources: SharedPreloadedResources,
    /// <https://fetch.spec.whatwg.org/#concept-fetch-record>
    in_flight_keep_alive_records: SharedInflightKeepAliveRecords,
}

impl CoreResourceManager {
    pub fn new(
        devtools_sender: Option<Sender<DevtoolsControlMsg>>,
        _profiler_chan: ProfilerChan,
        embedder_proxy: GenericEmbedderProxy<NetToEmbedderMsg>,
        ca_certificates: CACertificates<'static>,
        ignore_certificate_errors: bool,
    ) -> CoreResourceManager {
        CoreResourceManager {
            devtools_sender,
            sw_managers: Default::default(),
            filemanager: FileManager::new(embedder_proxy.clone()),
            request_interceptor: RequestInterceptor::new(embedder_proxy),
            ca_certificates,
            ignore_certificate_errors,
            preloaded_resources: Default::default(),
            in_flight_keep_alive_records: Default::default(),
        }
    }

    fn handle_preloaded_response(&self, preload_id: PreloadId, response: Response) {
        let mut preloaded_resources = self.preloaded_resources.lock().unwrap();
        if let Some(entry) = preloaded_resources.get_mut(&preload_id) {
            entry.with_response(response);
        }
    }

    /// Exit the core resource manager.
    pub fn exit(&mut self) {
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
        let devtools_chan = self.devtools_sender.clone();
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
        let in_flight_keep_alive_records = self.in_flight_keep_alive_records.clone();
        let preloaded_resources = self.preloaded_resources.clone();
        if let Some(ref preload_id) = request.preload_id {
            let mut preloaded_resources = self.preloaded_resources.lock().unwrap();
            let entry = PreloadEntry::new(request.integrity_metadata.clone());
            preloaded_resources.insert(preload_id.clone(), entry);
        }

        spawn_task(async move {
            // XXXManishearth: Check origin against pipeline id (also ensure that the mode is allowed)
            // todo load context / mimesniff in fetch
            // todo referrer policy?
            // todo service worker stuff
            let context = FetchContext {
                state: http_state,
                user_agent: servo_config::pref!(user_agent),
                devtools_chan,
                filemanager,
                file_token,
                request_interceptor: Arc::new(TokioMutex::new(request_interceptor)),
                cancellation_listener,
                timing: ServoArc::new(Mutex::new(ResourceFetchTiming::new(request.timing_type()))),
                protocols,
                websocket_chan: None,
                ca_certificates,
                ignore_certificate_errors,
                preloaded_resources,
                in_flight_keep_alive_records,
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
                    .invalidate_token(&context.file_token, id);
            }
        });
    }

    /// <https://websockets.spec.whatwg.org/#concept-websocket-establish>
    fn websocket_connect(
        &self,
        mut request: RequestBuilder,
        event_sender: IpcSender<WebSocketNetworkEvent>,
        action_receiver: CallbackSetter<WebSocketDomAction>,
        http_state: &Arc<HttpState>,
        cancellation_listener: Arc<CancellationListener>,
        protocols: Arc<ProtocolRegistry>,
    ) {
        let http_state = http_state.clone();
        let devtools_chan = self.devtools_sender.clone();
        let filemanager = self.filemanager.clone();
        let request_interceptor = self.request_interceptor.clone();

        let ca_certificates = self.ca_certificates.clone();
        let ignore_certificate_errors = self.ignore_certificate_errors;
        let in_flight_keep_alive_records = self.in_flight_keep_alive_records.clone();
        let preloaded_resources = self.preloaded_resources.clone();

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
                        devtools_chan,
                        filemanager,
                        file_token: FileTokenCheck::NotRequired,
                        request_interceptor: Arc::new(TokioMutex::new(request_interceptor)),
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
                        preloaded_resources,
                        in_flight_keep_alive_records,
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
