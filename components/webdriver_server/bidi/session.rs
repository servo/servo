use core::fmt;
use std::rc::Rc;

use indexmap::{IndexMap, IndexSet};
use serde::{Deserialize, Serialize};
use servo_base::id::BrowsingContextId;
use tokio::{
    sync::{
        RwLock,
        mpsc::{self, UnboundedReceiver, UnboundedSender},
    },
    task,
};
use uuid::Uuid;
use webdriver_traits::{
    ScriptToWebDriverMessage,
    bidi::{
        self, ErrorCode, Event, EventData, LogEvent, script::PreloadScript as PreloadScriptId,
        session::Subscription as SubscriptionId,
    },
};

use crate::bidi::{ActiveSessions, connection::Connection};

/// A session can be both http and bidi.
pub struct Session {
    /// ## Why `Option`?
    ///
    /// The WebDriver specication includes the concept of static commands
    /// (commands executed without an active session). A value of `None`
    /// corresponds to cases where `session` is `null`  in the specification.
    session_id: Option<SessionId>,
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
        let session = Self {
            session_id: None,
            bidi,
            active_sessions,
            send: send.clone(),
            recv,
        };
        session.register();
        session
    }

    // TODO: static session should not register and unregister

    /// Register self to `active_sessions`.
    ///
    /// This should be called when new session is created,
    /// regardless of static session or `create_a_session`.
    fn register(&self) {
        let proxy = SessionProxy {
            bidi_flag: self.bidi.is_some(),
            send: self.send.clone(),
        };
        task::spawn_local({
            let id = self.session_id;
            let active_sessions = self.active_sessions.clone();
            async move {
                active_sessions.write().await.insert(id, proxy);
            }
        });
        // TODO: also send new session message to constellation
    }

    /// Unregister self from `active_sessions`.
    ///
    /// This should be called in `Drop`.
    fn unregister(&self) {
        task::spawn_local({
            let active_sessions = self.active_sessions.clone();
            let id = self.session_id;
            async move { active_sessions.write().await.remove(&id) }
        });
    }

    pub async fn start(&mut self) {
        // TODO: poll connections
        while let Some(msg) = self.recv.recv().await {
            match msg {
                SessionMessage::Connection(connection) => {
                    if let Some(bidi) = &mut self.bidi {
                        bidi.connections.push(connection);
                    } else {
                        // TODO: what to do if failed?
                    }
                },
                SessionMessage::Script(msg) => match msg {
                    ScriptToWebDriverMessage::EntryAdded(related_navigables, body) => {
                        let body = LogEvent::EntryAdded(body);
                        // 1.16.
                        if let Some(bidi) = &mut self.bidi {
                            if bidi.event_is_enabled("log.entryAdded", &related_navigables) {
                                bidi.emit_an_event(&Event {
                                    event_data: EventData::LogEvent(body),
                                    extensible: Default::default(),
                                })
                                .await;
                            } else {
                                bidi.buffer_a_log_event(&related_navigables, body);
                            }
                        }
                    },
                },
            }
        }
    }

    /// <https://w3c.github.io/webdriver/#dfn-create-a-session>
    pub fn create_a_session(&self, capabilities: (), flags: ()) -> Result<Self, ErrorCode> {
        // Step 1.
        let session_id = Uuid::new_v4();

        // Step 2.
        // NOTE: the bidi spec says "A session created this way will not be accessible via HTTP."
        let session = Self {
            session_id: Some(session_id.into()),
            // TODO: bidi
            bidi: None,
            active_sessions: todo!(),
            send: todo!(),
            recv: todo!(),
        };

        session.register();
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        self.unregister();
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
    Script(ScriptToWebDriverMessage),
}

/// In rust we have single ownership rule.
/// So only session itself owns the data, while others only channel to it.
pub struct SessionProxy {
    bidi_flag: bool,
    send: UnboundedSender<SessionMessage>,
}

impl SessionProxy {
    pub(crate) async fn associate(&self, connection: Connection) {
        self.send.send(SessionMessage::Connection(connection));
    }
}

/// BiDi-specific components of a session.
pub struct SessionBidi {
    /// <https://www.w3.org/TR/webdriver-bidi/#event-subscriptions>
    subscriptions: Vec<Subscription>,
    /// <https://www.w3.org/TR/webdriver-bidi/#event-known-subscription-ids>
    known_subscription_ids: IndexSet<SubscriptionId>,
    /// <https://www.w3.org/TR/webdriver-bidi/#session-websocket-connections>
    ///  
    /// Deviation: we cannot use (ordered) set since `IndexSet` cannot `iter_mut`.
    connections: Vec<Connection>,
    // TODO: sandbox map
    /// <https://www.w3.org/TR/webdriver-bidi/#user-context-to-accept-insecure-certificates-override-map>
    user_context_to_accept_insecure_certificates_override_map: IndexMap<(), bool>,
    /// <https://www.w3.org/TR/webdriver-bidi/#user-context-to-proxy-configuration-map>
    user_context_to_proxy_configuration_map: IndexMap<(), ()>,
    /// <https://www.w3.org/TR/webdriver-bidi/#session-emulated-network-conditions>
    emulated_network_conditions: EmulatedNetworkConditions,
    /// <https://www.w3.org/TR/webdriver-bidi/#screencast-recordings-map>
    screencast_recordings_mao: IndexMap<Uuid, ScreencastRecording>,
    /// <https://www.w3.org/TR/webdriver-bidi/#session-emulated-maxtouchpoints>
    emulated_user_agent: EmulatedUserAgent,
    /// <https://www.w3.org/TR/webdriver-bidi/#session-emulated-maxtouchpoints>
    emulated_max_touch_points: EmulatedMaxTouchPoints,
    /// A BiDi session has a extra headers ...
    ///
    /// <https://www.w3.org/TR/webdriver-bidi/#session-extra-headers>
    extra_headers: ExtraHeaders,
    /// <https://www.w3.org/TR/webdriver-bidi/#network-collectors>
    network_collectors: IndexMap<bidi::network::Collector, Collector>,
    // TODO: intercept map
    // TODO: blocked request map
    /// <https://www.w3.org/TR/webdriver-bidi/#preload-script-map>
    preload_script_map: IndexMap<PreloadScriptId, PreloadScript>,
    /// <https://www.w3.org/TR/webdriver-bidi/#log-event-buffer>
    log_event_buffer: IndexMap<BrowsingContextId, Vec<LogEvent>>,
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
    fn buffer_a_log_event(&mut self, navigable_ids: &[BrowsingContextId], event: LogEvent) {
        // 1.
        let buffer = &mut self.log_event_buffer;
        // 2-3. SKIP since we can only use naviable id directly
        // 4.
        for navigable_id in navigable_ids {
            // NOTE: the spec self-contradicts here,
            // we choose to follow the `log_event_buffer` type rather than the steps.
            buffer.entry(*navigable_id).or_default().push(event.clone());
        }
        todo!()
    }

    /// <https://w3c.org/TR/webdriver-bidi/#event-is-enabled>
    fn event_is_enabled(&self, event_name: &str, navigables: &[BrowsingContextId]) -> bool {
        // 1. TODO
        // 2. TODO
        // 2.1. TODO
        // 2.2. TODO
        // 2.3. TODO
        // 2.4. TODO
        // 3.
        false
    }

    /// <https://w3c.org/TR/webdriver-bidi/#emit-an-event>
    async fn emit_an_event(&mut self, body: &Event) {
        // 1. SKIP Assert
        // 2.
        let serialized = serde_json::to_string(&body).expect("Event serialization failed");
        // 3.
        for connection in self.connections.iter_mut() {
            // 3.1.
            connection.send(serialized.clone()).await;
        }
    }
}

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

/// <https://www.w3.org/TR/webdriver-bidi/#session-emulated-network-conditions>
pub struct EmulatedNetworkConditions {
    default_network_conditions: Option<EmulatedNetworkConditionsStruct>,
    user_context_network_conditions: IndexMap<(), EmulatedNetworkConditionsStruct>,
    navigable_network_conditions: IndexMap<BrowsingContextId, EmulatedNetworkConditionsStruct>,
}

/// <https://www.w3.org/TR/webdriver-bidi/#emulated-network-conditions-struct>
pub struct EmulatedNetworkConditionsStruct {
    offline: Option<bool>,
}

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

/// <https://www.w3.org/TR/webdriver-bidi/#screencast-stream>
pub struct ScreencastStream;

/// <https://www.w3.org/TR/webdriver-bidi/#session-emulated-maxtouchpoints>
pub struct EmulatedUserAgent {
    default_user_agent: Option<String>,
    user_context_user_agent: IndexMap<(), String>,
    navigable_user_agent: IndexMap<BrowsingContextId, String>,
}

/// <https://www.w3.org/TR/webdriver-bidi/#session-emulated-maxtouchpoints>
pub struct EmulatedMaxTouchPoints {
    default: Option<usize>,
    user_contexts: IndexMap<(), usize>,
    navigables: IndexMap<BrowsingContextId, usize>,
}

/// <https://www.w3.org/TR/webdriver-bidi/#session-extra-headers>
pub struct ExtraHeaders {
    // TODO: type
    default_headers: Vec<String>,
    user_context_headers: IndexMap<(), Vec<String>>,
    navigable_headers: IndexMap<BrowsingContextId, Vec<String>>,
}

pub struct Collector;

/// <https://www.w3.org/TR/webdriver-bidi/#preload-script-map>
pub struct PreloadScript {
    function_declaration: String,
    arguments: Vec<String>,
    contexts: Option<Vec<String>>,
    sandbox: Option<String>,
    user_contexts: IndexSet<()>,
}
