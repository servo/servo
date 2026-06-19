mod callback;
mod connection;
mod error;
mod listener;
mod modules;
mod remote_end;
mod session;

use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    net::{SocketAddr, SocketAddrV4},
    rc::Rc,
    thread::{self},
};

use async_tungstenite::tungstenite;
use crossbeam_channel::{Receiver, Sender, unbounded};
use devtools_traits::WorkerId;
use embedder_traits::{EmbedderMsg, GenericEmbedderProxy};
use futures_util::StreamExt;
use net_traits::ResourceThreads;
use servo_base::{
    generic_channel::GenericSender,
    id::{BrowsingContextId, PainterId, PipelineId, WebViewId},
};
use tokio::{
    sync::{
        RwLock,
        mpsc::{self, UnboundedReceiver, UnboundedSender},
    },
    task,
};
use webdriver_traits::{
    ScriptToWebDriverMessage, WebDriverMessage, WebDriverToConstellationMessage,
    WebDriverToScriptMessage,
    bidi::{
        CommandData, ErrorCode, browser, browsing_context,
        script::{BaseRealmInfo, RealmInfo, WindowRealmInfo, WindowRealmInfoType},
    },
};

use crate::bidi::{
    connection::Connection,
    listener::Listener,
    remote_end::RemoteEnd,
    session::{
        SessionOldOwning,
        common::{SessionId, SessionMessage},
        proxy::SessionProxy,
    },
};

// TODO: this should later be renamed to `WebDriverServer`
// after classic is merged.
pub struct WebDriverBidiThread {
    port: u16,
    embedder_proxy: GenericEmbedderProxy<EmbedderMsg>,
    constellation_sender: Sender<WebDriverToConstellationMessage>,
    resource_threads: ResourceThreads,
    // Remote end states are shared across all sessions.
    // Though this is a single threaded
    remote_end_state: Rc<RemoteEndState>,
}

impl WebDriverBidiThread {
    pub fn start(
        embedder_proxy: GenericEmbedderProxy<EmbedderMsg>,
        resource_threads: ResourceThreads,
    ) -> (
        UnboundedSender<WebDriverMessage>,
        Receiver<WebDriverToConstellationMessage>,
    ) {
        let (c2w_sender, c2w_receiver) = mpsc::unbounded_channel();
        let (w2c_sender, w2c_receiver) = unbounded();

        thread::Builder::new()
            .name("WebDriverBiDi".to_string())
            .spawn(move || {
                WebDriverBidiThread::new(0, embedder_proxy, resource_threads, w2c_sender)
                    .run(c2w_receiver);
            })
            .expect("Thread spawning failed");

        (c2w_sender, w2c_receiver)
    }

    fn new(
        port: u16,
        embedder_proxy: GenericEmbedderProxy<EmbedderMsg>,
        resource_threads: ResourceThreads,
        constellation_sender: Sender<WebDriverToConstellationMessage>,
    ) -> Self {
        Self {
            port,
            embedder_proxy,
            resource_threads,
            constellation_sender,
            remote_end_state: Default::default(),
        }
    }

    fn run(&self, receiver: UnboundedReceiver<WebDriverMessage>) {
        let address = SocketAddr::V4(SocketAddrV4::new("0.0.0.0".parse().unwrap(), self.port));
        tokio::runtime::LocalRuntime::new()
            .expect("Runtime creation failed")
            .block_on(async move {
                let remote_end_state = &self.remote_end_state;
                let embedder_proxy = &self.embedder_proxy;
                let resource_threads = &self.resource_threads;
                let constellation_sender = &self.constellation_sender;

                let (_, sender) = SessionOldOwning::start_static(
                    remote_end_state.clone(),
                    embedder_proxy.clone(),
                    resource_threads.clone(),
                    constellation_sender.clone(),
                );
                Listener::start(address, remote_end_state.clone(), sender);

                let forward = Self::handle_thread_message(remote_end_state.clone(), receiver);

                forward.await
            });
    }

    /// Handle thread messages from constellation/script/...
    async fn handle_thread_message(
        remote_end_state: Rc<RemoteEndState>,
        mut receiver: UnboundedReceiver<WebDriverMessage>,
    ) {
        while let Some(msg) = receiver.recv().await {
            match msg {
                WebDriverMessage::FromConstellation(constellation_to_web_driver_message) => todo!(),
                WebDriverMessage::FromScript(msg) => match msg {
                    ScriptToWebDriverMessage::LogEntryAdded(items, entry_added) => todo!(),
                    ScriptToWebDriverMessage::RealmCreated(
                        (browsing_context_id, pipeline_id, worker_id, webview_id),
                        generic_sender,
                    ) => {
                        // realm
                        remote_end_state.realms.write().await.insert(
                            RealmId(pipeline_id, worker_id),
                            // TODO: faked, replace with true info
                            RealmInfo::WindowRealmInfo(WindowRealmInfo {
                                r#type: WindowRealmInfoType::Window,
                                base_realm_info: BaseRealmInfo {
                                    realm: "".to_string(),
                                    origin: "".to_string(),
                                },
                                context: browsing_context_id.to_string(),
                                user_context: None,
                                sandbox: None,
                            }),
                        );

                        remote_end_state.navigables.write().await.insert(
                            browsing_context_id,
                            Navigable {
                                id: browsing_context_id,
                                // unknown here
                                original_opener: None,
                                sender: generic_sender.clone(),
                                webview_id: Some(webview_id),
                                active_document: pipeline_id,
                            },
                        );
                    },
                    ScriptToWebDriverMessage::Message { channel, data } => todo!(),
                    ScriptToWebDriverMessage::FileDialogOpened(file_dialog_opened) => todo!(),
                },
            }

            // TODO: should not directly forward
            // forward to each session
            // let msg = Rc::new(msg);
            // for session in remote_end_state.active_sessions.read().await.values() {
            //     if let Err(e) = session.sender.send(SessionMessage::WebDriver(msg.clone())) {
            //         log::warn!("Sending constellation message to session failed: {e:?}");
            //     }
            // }
        }
    }
}

/// Global state of a remote end is grouped here to be shared
/// across sessions.
#[derive(Default)]
pub struct RemoteEndState {
    /// The active sessions of a remote end.
    active_sessions: RwLock<ActiveSessions>,

    /// The navigables of a remote end.
    navigables: RwLock<HashMap<BrowsingContextId, Navigable>>,

    /// The navigables of a remote end.
    realms: RwLock<HashMap<RealmId, RealmInfo>>,
}

type ActiveSessions = HashMap<SessionId, SessionProxy>;

impl RemoteEndState {
    /// <https://www.w3.org/TR/webdriver-bidi/#cleanup-remote-end-state>
    pub(crate) fn cleanup_remote_end_state(&self) {
        // 1. TODO: blocked by network module
        // 2. TODO: blocked by network module
        // 3. TODO: blocked by network module
        // 4. SKIP: implementation-defined
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#get-a-navigable>
    /// Deviation: in spec the output is null iff input is null,
    /// to avoid unwrapping option, we disallow the null case here.
    pub(crate) async fn get_a_navigable(
        &self,
        navigable_id: BrowsingContextId,
    ) -> Result<Navigable, ErrorCode> {
        // 1. SKIP: disallow null
        // 2.
        let Some(navigable) = self.navigables.read().await.get(&navigable_id).cloned() else {
            return Err(ErrorCode::NoSuchFrame);
        };
        // 3. SKIP: done in last step
        // 4.
        Ok(navigable)
    }

    pub(crate) fn top_level_traversables(&self) -> Vec<TopLevelTraversable> {
        todo!()
    }
}

// TODO: have a correct representation and storage of hierarchy

#[derive(Clone)]
pub(crate) struct Navigable {
    pub(crate) id: BrowsingContextId,
    pub(crate) original_opener: Option<BrowsingContextId>,
    pub(crate) sender: GenericSender<WebDriverToScriptMessage>,
    pub(crate) webview_id: Option<WebViewId>,
    pub(crate) active_document: PipelineId,
}

impl Navigable {
    /// <https://www.w3.org/TR/webdriver-bidi/#get-the-navigable-info>
    pub(crate) fn get_the_navigable_info(&self, max_depth: Option<u64>) -> browsing_context::Info {
        // TODO:
        todo!()
    }

    pub(crate) fn send_to_script(&self, message: WebDriverToScriptMessage) {
        if let Err(e) = self.sender.send(message) {
            log::warn!("WebDriver to script channel closed: {e:?}");
        };
    }

    /// Checks whether the navigable is a top-level traversable.
    pub(crate) fn is_top_level_traversable(&self) -> bool {
        self.webview_id.is_some()
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) struct RealmId(pub(crate) PipelineId, pub(crate) Option<WorkerId>);

pub struct TopLevelTraversable {
    pub(crate) webview_id: WebViewId,
}

impl TopLevelTraversable {
    fn associated_client_window(&self) -> ClientWindow {
        todo!()
    }
}

pub struct ClientWindow {
    id: PainterId,
}

// TODO: this should be flatten to be field on thread/webdriver
struct Hierarchy {
    client_windows: HashMap<PainterId, ()>,
    /// The realms of a remote end.
    realms: HashMap<RealmId, ()>,
    /// The navigables of a remote end.
    navigables: HashMap<BrowsingContextId, ()>,
    /// The top level traversables of a remote end.
    traversables: HashMap<WebViewId, ()>,
}

// like
//
// struct FlattenToRefactorTo {
//     connections: HashMap<ConnectionId, RefCell<Connecition>>, // refcell because we are single threads
//     unassociated: HashSet<ConnectionId>,
//
//     // here the new session keep states only, and does not hold connection
//     active_sessions: HashMap<SessionId, Session>,
// }
//
// struct Session {
//     connections: Vec<ConnectionId>,
//     bidi: bool,
// }
//
// struct Navigable {
//    navigable_id: BrowsingContextId,
//    top_level_traversable: WebViewId,
// }

pub struct WebDriverThread {
    remote_end: Rc<RemoteEnd>,
}
