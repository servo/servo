use core::fmt;
use std::rc::Rc;

use indexmap::{IndexMap, IndexSet};
use serde::{Deserialize, Serialize};
use servo_base::id::BrowsingContextId;
use servo_webdriver::bidi::{
    self, ErrorCode, script::PreloadScript as PreloadScriptId,
    session::Subscription as SubscriptionId,
};
use tokio::{
    sync::{
        RwLock,
        mpsc::{self, UnboundedReceiver, UnboundedSender},
        oneshot::{self, Sender},
    },
    task,
};
use uuid::Uuid;

use crate::bidi::{ActiveSessions, connection::Connection};

/// A session can be both http and bidi.
pub struct Session {
    /// A session has a session ID, which is the string representation
    /// of a UUID used to uniquely identify the session. This is set
    /// when creating the session.
    ///
    /// ## Why `Option`?
    ///
    /// The WebDriver specication includes the concept of static commands
    /// (commands executed without an active session). A value of `None`
    /// corresponds to cases where `session` is `null`  in the specification.
    id: Option<SessionId>,
    /// WebDriver BiDi extends the session concept from WebDriver.
    ///
    /// BiDi-specific components are grouped in this sub-struct via composition.
    bidi: Option<SessionBidi>,
    // TODO: http: Option<SessionHttp> for http specific components later
    active_sessions: Rc<RwLock<ActiveSessions>>,
    send: UnboundedSender<SessionMessage>,
    recv: UnboundedReceiver<SessionMessage>,
}

impl Session {
    pub fn new(active_sessions: Rc<RwLock<ActiveSessions>>) -> Self {
        let (send, recv) = mpsc::unbounded_channel();
        let bidi = Some(SessionBidi::new());
        Self {
            id: None,
            bidi,
            active_sessions,
            send,
            recv,
        }
    }

    async fn register(&self) {
        let proxy = SessionProxy {
            send: self.send.clone(),
        };
        {
            self.active_sessions.write().await.insert(self.id, proxy);
        }
    }

    pub async fn start(&mut self) {}

    /// <https://w3c.github.io/webdriver/#dfn-create-a-session>
    pub fn create_a_session(&self, capabilities: (), flags: ()) -> Result<Self, ErrorCode> {
        // 1. Let session id be the result of generating a UUID.
        let session_id = Uuid::new_v4();

        // 2. let session be a new session with session ID session id, and HTTP flags contains "http".
        // NOTE: we do not set HTTP flag in BiDi.
        // let session = BidiSession {
        //     connections: IndexSet::new(),
        // };

        todo!()

        // match self.id {
        //     Some(_) => Err(ErrorCode::SessionNotCreated),
        //     None => Ok(Self { id: Some(self.id) }),
        // }
    }
}

/// Unregister self from `active_sessions` when drop.
impl Drop for Session {
    fn drop(&mut self) {
        let active_sessions = self.active_sessions.clone();
        let id = self.id;
        task::spawn_local(async move { active_sessions.write().await.remove(&id) });
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct SessionId(pub Uuid);

impl fmt::Display for SessionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Uuid> for SessionId {
    fn from(value: Uuid) -> Self {
        Self(value)
    }
}

impl From<SessionId> for Uuid {
    fn from(value: SessionId) -> Self {
        value.0
    }
}

pub enum SessionMessage {
    /// Associate a WebSocket connection to a session.
    Connection(Connection),
    IsBidi(Sender<bool>),
}

/// In rust we have single ownership rule.
/// So only session itself owns the data, while others only channel to it.
pub struct SessionProxy {
    send: UnboundedSender<SessionMessage>,
}

impl SessionProxy {
    /// Ask the session whether it is a BiDi session.
    async fn is_bidi(&self) -> bool {
        let (send, recv) = oneshot::channel();
        self.send.send(SessionMessage::IsBidi(send));
        recv.await.unwrap()
    }

    pub(crate) async fn associate(&self, connection: Connection) {
        self.send.send(SessionMessage::Connection(connection));
    }
}

/// BiDi-specific components of a session.
pub struct SessionBidi {
    /// A BiDi session has subscriptions which is a list of subscription.
    ///
    /// <https://www.w3.org/TR/webdriver-bidi/#event-subscriptions>
    subscriptions: Vec<Subscription>,
    /// A BiDi session has a known subscription ids which is a set of all subscription
    /// ids that have been issued to the local end but which have not yet been unsubscribed.
    /// <https://www.w3.org/TR/webdriver-bidi/#event-known-subscription-ids>
    known_subscription_ids: IndexSet<SubscriptionId>,
    /// A BiDi session has a set of session WebSocket connections whose
    /// elements are WebSocket connections. This is initially empty.
    ///
    /// <https://www.w3.org/TR/webdriver-bidi/#session-websocket-connections>
    session_websocket_connections: IndexSet<Connection>,
    // TODO: sandbox map
    /// A BiDi session has a user context to accept insecure certificates
    /// override map, which is a map between user contexts and boolean.
    ///
    /// <https://www.w3.org/TR/webdriver-bidi/#user-context-to-accept-insecure-certificates-override-map>
    user_context_to_accept_insecure_certificates_override_map: IndexMap<(), bool>,
    /// A BiDi session has a user context to proxy configuration map,
    /// which is a map between user contexts and proxy configuration.
    ///
    /// <https://www.w3.org/TR/webdriver-bidi/#user-context-to-proxy-configuration-map>
    user_context_to_proxy_configuration_map: IndexMap<(), ()>,
    /// A BiDi session has a emulated network conditions ...
    ///
    /// <https://www.w3.org/TR/webdriver-bidi/#session-emulated-network-conditions>
    emulated_network_conditions: EmulatedNetworkConditions,
    /// A BiDi session has a screencast recordings map which is a map in which the keys are UUIDs,
    /// and the values are screencast recording.
    ///
    /// <https://www.w3.org/TR/webdriver-bidi/#screencast-recordings-map>
    screencast_recordings_mao: IndexMap<Uuid, ScreencastRecording>,
    /// A BiDi session has an emulated user agent ...
    ///
    /// <https://www.w3.org/TR/webdriver-bidi/#session-emulated-maxtouchpoints>
    emulated_user_agent: EmulatedUserAgent,
    /// A BiDi session has emulated maxTouchPoints, ...
    ///
    /// <https://www.w3.org/TR/webdriver-bidi/#session-emulated-maxtouchpoints>
    emulated_max_touch_points: EmulatedMaxTouchPoints,
    /// A BiDi session has a extra headers ...
    ///
    /// <https://www.w3.org/TR/webdriver-bidi/#session-extra-headers>
    extra_headers: ExtraHeaders,
    /// A BiDi session has network collectors which is a map between network.Collector
    /// and a collector. It is initially empty.
    ///
    /// <https://www.w3.org/TR/webdriver-bidi/#network-collectors>
    network_collectors: IndexMap<bidi::network::Collector, Collector>,
    // TODO: intercept map
    // TODO: blocked request map
    /// A BiDi session has a preload script map which is a map in which the keys are UUIDs,
    /// and the values are structs with ...
    ///
    /// <https://www.w3.org/TR/webdriver-bidi/#preload-script-map>
    preload_script_map: IndexMap<PreloadScriptId, PreloadScript>,
    /// A BiDi Session has a log event buffer which is a map from navigable id to a list
    /// of log events for that context that have not been emitted.
    ///
    /// <https://www.w3.org/TR/webdriver-bidi/#log-event-buffer>
    log_event_buffer: IndexMap<BrowsingContextId, Vec<()>>,
    /// Receive connections from
    recv: UnboundedReceiver<Connection>,
}

impl SessionBidi {
    fn new() -> Self {
        todo!()
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#remove-user-context-subscriptions>
    fn remove_user_context_subscriptions(&self) {
        todo!()
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#buffer-a-log-event>
    fn buffer_a_log_event(&self) {
        todo!()
    }
}

/// A subscription is a struct consisting of a subscription id (a string), event names
/// (a set of event names), top-level traversable ids (a set of IDs of top-level traversables)
/// and user context ids (a set of IDs of user contexts).
///
/// <https://www.w3.org/TR/webdriver-bidi/#event-subscription>
pub struct Subscription {
    subscription_id: SubscriptionId,
    event_names: IndexSet<String>,
    top_level_traversable_ids: IndexSet<BrowsingContextId>,
    user_context_ids: IndexSet<()>,
}

impl Subscription {
    /// <https://www.w3.org/TR/webdriver-bidi/#subscription-global>
    pub fn is_global(&self) -> bool {
        self.top_level_traversable_ids.is_empty() && self.user_context_ids.is_empty()
    }
}

/// A BiDi session has a emulated network conditions which is a struct with an item
/// named default network conditions, which is an emulated network conditions struct
/// or null, an item named user context network conditions, which is a weak map between
/// user contexts and emulated network conditions struct, and a item named navigable
/// network conditions, which is a weak map between navigables and emulated network
/// conditions struct.
///
/// <https://www.w3.org/TR/webdriver-bidi/#session-emulated-network-conditions>
pub struct EmulatedNetworkConditions {
    default_network_conditions: Option<EmulatedNetworkConditionsStruct>,
    user_context_network_conditions: IndexMap<(), EmulatedNetworkConditionsStruct>,
    navigable_network_conditions: IndexMap<BrowsingContextId, EmulatedNetworkConditionsStruct>,
}

/// An emulated network conditions struct is a struct with:
///
/// - item named offline which is a boolean or null.
///
/// <https://www.w3.org/TR/webdriver-bidi/#emulated-network-conditions-struct>
pub struct EmulatedNetworkConditionsStruct {
    offline: Option<bool>,
}

/// A BiDi session has a screencast recordings map which is a map in which the keys
/// are UUIDs, and the values are screencast recording, which is a struct with an
/// item named stream, which is a screencast stream, an item named path, which is
/// a string, an item named state, which is one of "recording", "stopping", "stopped",
/// an item named writeError, which is a string or null.
///
/// <https://www.w3.org/TR/webdriver-bidi/#screencast-recording>
pub struct ScreencastRecording {
    stream: ScreencastStream,
    path: String,
    state: ScreencastRecordingState,
    write_error: Option<String>,
}

pub enum ScreencastRecordingState {
    Recording,
    Stopping,
    Stopped,
}

/// A screencast stream is an abstract stream of the viewport of a top-level traversable,
/// consisting of a video track containing the rendered visual output of the top-level
/// traversable’s document’s viewport, and optionally an audio track containing the audio
/// output of the top-level traversable’s document.
///
/// <https://www.w3.org/TR/webdriver-bidi/#screencast-stream>
pub struct ScreencastStream;

/// A BiDi session has an emulated user agent which is a struct with an item named
/// default user agent, which is a string or null, an item named user context user
/// agent, which is a weak map between user contexts and string, and an item named
/// navigable user agent, which is a weak map between navigables and string.
///
/// <https://www.w3.org/TR/webdriver-bidi/#session-emulated-maxtouchpoints>
pub struct EmulatedUserAgent {
    default_user_agent: Option<String>,
    user_context_user_agent: IndexMap<(), String>,
    navigable_user_agent: IndexMap<BrowsingContextId, String>,
}

/// A BiDi session has emulated maxTouchPoints, which is a struct with an item named
/// default, which is an integer or null, initially null; an item named user contexts,
/// which is a weak map between user contexts and integer, initially empty; and an
/// item named navigables, which is a weak map between navigables and integer,
/// initially empty.
///
/// <https://www.w3.org/TR/webdriver-bidi/#session-emulated-maxtouchpoints>
pub struct EmulatedMaxTouchPoints {
    default: Option<usize>,
    user_contexts: IndexMap<(), usize>,
    navigables: IndexMap<BrowsingContextId, usize>,
}

/// A BiDi session has a extra headers which is a struct with an item named default headers,
/// which is a header list (initially set to an empty header list), an item named user context
/// headers, which is a weak map between user contexts and header lists, and a item named
/// navigable headers, which is a weak map between navigables and header lists.
///
/// <https://www.w3.org/TR/webdriver-bidi/#session-extra-headers>
pub struct ExtraHeaders {
    // TODO: type
    default_headers: Vec<String>,
    user_context_headers: IndexMap<(), Vec<String>>,
    navigable_headers: IndexMap<BrowsingContextId, Vec<String>>,
}

pub struct Collector;

/// A BiDi session has a preload script map which is a map in which the keys are UUIDs,
/// and the values are structs with an item named function declaration, which is a string,
/// an item named arguments, which is a list, an item named contexts, which is a list or null,
/// an item named sandbox, which is a string or null, and an item named user contexts,
/// which is a set.
///
/// <https://www.w3.org/TR/webdriver-bidi/#preload-script-map>
pub struct PreloadScript {
    function_declaration: String,
    arguments: Vec<String>,
    contexts: Option<Vec<String>>,
    sandbox: Option<String>,
    user_contexts: IndexSet<()>,
}
