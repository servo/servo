mod error;
mod messages;
mod modules;
mod server;
mod session;
mod wait;

use std::{
    collections::{HashMap, HashSet, hash_map::Entry},
    str::FromStr,
    thread::{self, JoinHandle},
};

use async_tungstenite::tungstenite::Message as WsMessage;
use crossbeam_channel::{Receiver, RecvError, Sender, select};
use embedder_traits::GenericEmbedderProxy;
use log::warn;
use serde::Deserialize;
use serde_json::Value;
use servo_base::{
    generic_channel::GenericSender,
    id::{BrowsingContextId, WebViewId},
};
use tokio::sync::mpsc::UnboundedSender;
use webdriver_traits::{
    bidi::{
        Command, ErrorCode, ErrorResponse, Event, EventData, ScriptEvent,
        script::{
            self, Message, RealmCreated, RealmDestroyed, RealmDestroyedParameters, RealmInfo,
            Target,
        },
    },
    ids::{ConnectionId, RealmId, ResumeId, SessionId},
    messages::{
        EmbedderToWebDriverMessage, PreloadScriptBody, ScriptToWebDriverMessage, WebDriverMessage,
        WebDriverToConstellationMessage, WebDriverToEmbedderMessage, WebDriverToScriptMessage,
    },
};

use crate::bidi::{
    error::BidiError,
    messages::{ServerToWebDriverMessage, WebDriverToServerMessage},
    modules::{
        CommandHandled,
        script::{Disowned, Evaluated},
    },
    server::WebDriverServer,
    session::Session,
    wait::WaitQueues,
};

pub struct WebDriverBidiThread {
    // senders
    #[allow(unused)]
    embedder_proxy: GenericEmbedderProxy<WebDriverToEmbedderMessage>,
    #[allow(unused)]
    constellation_sender: Sender<WebDriverToConstellationMessage>,
    server_sender: UnboundedSender<WebDriverToServerMessage>,

    // receivers
    embedder_receiver: Receiver<EmbedderToWebDriverMessage>,
    server_receiver: Receiver<ServerToWebDriverMessage>,
    servo_receiver: Receiver<WebDriverMessage>,

    _server_handle: JoinHandle<()>,

    /// A collection of maps of pending events. See [`WaitQueues`] for details.
    wait_queues: WaitQueues,

    /// Connections that are not associated with a session.
    connections: HashSet<ConnectionId>,
    /// Active sessions of a remote end.
    sessions: HashMap<SessionId, Session>,
    /// Cache navigable information in webdriver.
    navigables: HashMap<BrowsingContextId, Navigable>,
    /// Cache realm information in webdriver.
    realms: HashMap<RealmId, Realm>,
}

impl WebDriverBidiThread {
    pub fn start(
        embedder_proxy: GenericEmbedderProxy<WebDriverToEmbedderMessage>,
        embedder_receiver: Receiver<EmbedderToWebDriverMessage>,
        port: u16,
    ) -> (
        Sender<WebDriverMessage>,
        Receiver<WebDriverToConstellationMessage>,
    ) {
        let (c2w_sender, c2w_receiver) = crossbeam_channel::unbounded();
        let (w2c_sender, w2c_receiver) = crossbeam_channel::unbounded();
        thread::Builder::new()
            .name("WebDriverBiDi".into())
            .spawn(move || {
                Self::new(
                    embedder_proxy,
                    embedder_receiver,
                    w2c_sender,
                    c2w_receiver,
                    port,
                )
                .run();
            })
            .expect("Spawning WebDriverBiDi thread failed");
        (c2w_sender, w2c_receiver)
    }

    fn new(
        embedder_proxy: GenericEmbedderProxy<WebDriverToEmbedderMessage>,
        embedder_receiver: Receiver<EmbedderToWebDriverMessage>,
        w2c_sender: Sender<WebDriverToConstellationMessage>,
        c2w_receiver: Receiver<WebDriverMessage>,
        port: u16,
    ) -> Self {
        let (w2s_sender, w2s_receiver) = tokio::sync::mpsc::unbounded_channel();
        let (s2w_sender, s2w_receiver) = crossbeam_channel::unbounded();

        let server_handle = WebDriverServer::start(w2s_receiver, s2w_sender, port);

        Self {
            embedder_proxy,
            server_sender: w2s_sender,
            constellation_sender: w2c_sender,
            embedder_receiver,
            server_receiver: s2w_receiver,
            servo_receiver: c2w_receiver,
            _server_handle: server_handle,
            wait_queues: Default::default(),
            connections: Default::default(),
            sessions: Default::default(),
            navigables: Default::default(),
            realms: Default::default(),
        }
    }

    fn run(&mut self) {
        loop {
            select! {
                recv(self.embedder_receiver) -> msg => self.handle_embedder(msg),
                recv(self.server_receiver) -> msg => self.handle_server(msg),
                recv(self.servo_receiver) -> msg => self.handle_servo(msg),
            }
        }
    }

    fn handle_server(&mut self, msg: Result<ServerToWebDriverMessage, RecvError>) {
        match msg {
            Err(err) => warn!("Receiving message from server failed ({err:?})"),
            Ok(msg) => match msg {
                ServerToWebDriverMessage::Connection(conn_id, session_id) => {
                    match session_id {
                        Some(s) => match SessionId::from_str(&s) {
                            Err(err) => {
                                warn!("Non-UUID session id ({s}) received ({err:?})")
                            },
                            Ok(session_id) => {
                                match self.sessions.get_mut(&session_id) {
                                    None => {
                                        // TODO: should reject in server, but this is impossible
                                        // with `tungstenite` and its derived crates.
                                        warn!("Invalid session id ({session_id}) received");
                                    },
                                    Some(session) => {
                                        session.connections.insert(conn_id);
                                    },
                                }
                            },
                        },
                        None => {
                            // if session id not specified, append to unassociated
                            self.connections.insert(conn_id);
                        },
                    }
                },
                ServerToWebDriverMessage::Message(conn_id, msg) => {
                    self.handle_an_incoming_message(conn_id, msg);
                },
            },
        }
    }

    fn handle_servo(&mut self, msg: Result<WebDriverMessage, RecvError>) {
        match msg {
            Err(err) => warn!("Receiving message from constellation or script failed ({err:?})"),
            Ok(msg) => {
                match msg {
                    WebDriverMessage::FromConstellation(message) => match message {},
                    WebDriverMessage::FromScript(message) => {
                        match message {
                            ScriptToWebDriverMessage::RealmCreated(
                                realm_info,
                                top_level,
                                webview_id,
                                sender,
                            ) => {
                                // update cache
                                let realm_id = *realm_info.realm();
                                self.realms.insert(
                                    realm_id,
                                    Realm {
                                        id: realm_id,
                                        info: realm_info.clone(),
                                        sender,
                                    },
                                );

                                if let RealmInfo::Window(window_realm_info) = &realm_info {
                                    match self.navigables.entry(window_realm_info.context) {
                                        Entry::Occupied(mut occupied) => {
                                            occupied.get_mut().active_realm = realm_id;
                                        },
                                        Entry::Vacant(vacant) => {
                                            vacant.insert(Navigable {
                                                id: window_realm_info.context,
                                                active_realm: realm_id,
                                                top_level,
                                                webview_id,
                                            });
                                        },
                                    }

                                    // RealmCreated indicates that new script thread created,
                                    // full sync preloadscript map to that thread.
                                    let preload_scripts = self
                                        .sessions
                                        .values()
                                        .flat_map(|session| session.preload_script_map.iter())
                                        .filter(|(_key, value)| {
                                            value.navigables.contains(&window_realm_info.context)
                                        })
                                        .map(|(key, value)| {
                                            (
                                                *key,
                                                PreloadScriptBody {
                                                    function_declaration: value
                                                        .function_declaration
                                                        .clone(),
                                                    arguments: value.arguments.clone(),
                                                    sandbox: value.sandbox.clone(),
                                                },
                                            )
                                        })
                                        .collect();
                                    self.send_to_realm(
                                        realm_id,
                                        WebDriverToScriptMessage::AddPreloadScripts(
                                            realm_id,
                                            preload_scripts,
                                        ),
                                    );
                                }

                                // emit event
                                for session in self.sessions.values() {
                                    if session.event_is_enabled("script.realmCreated") {
                                        self.emit_an_event(
                                            session,
                                            Event {
                                                event_data: EventData::Script(
                                                    ScriptEvent::RealmCreated(RealmCreated {
                                                        params: realm_info.clone(),
                                                    }),
                                                ),
                                                extensible: Default::default(),
                                            },
                                        );
                                    }
                                }
                            },
                            ScriptToWebDriverMessage::RealmDestroyed(realm_id) => {
                                self.realms.remove(&realm_id);
                                for session in self.sessions.values() {
                                    if session.event_is_enabled("script.realmCreated") {
                                        self.emit_an_event(
                                            session,
                                            Event {
                                                event_data: EventData::Script(
                                                    ScriptEvent::RealmDestroyed(RealmDestroyed {
                                                        params: RealmDestroyedParameters {
                                                            realm: realm_id,
                                                        },
                                                    }),
                                                ),
                                                extensible: Default::default(),
                                            },
                                        );
                                    }
                                }
                            },
                            ScriptToWebDriverMessage::Disowned(disowned_id) => {
                                self.resume::<Disowned>(disowned_id, ())
                            },
                            ScriptToWebDriverMessage::Evaluated(eval_id, result) => {
                                self.resume::<Evaluated>(eval_id, result)
                            },
                            ScriptToWebDriverMessage::Message(message_body) => {
                                let realm = self.realms.get(&message_body.realm);
                                let event = EventData::Script(ScriptEvent::Message(Message {
                                    params: script::MessageParameters {
                                        channel: message_body.channel,
                                        data: message_body.data,
                                        source: script::Source {
                                            realm: message_body.realm,
                                            context: realm.and_then(Realm::navigable_id),
                                            user_context: realm.and_then(Realm::user_context_id),
                                        },
                                    },
                                }));
                                for session in self.sessions.values() {
                                    if session.event_is_enabled("script.realmCreated") {
                                        self.emit_an_event(
                                            session,
                                            Event {
                                                event_data: event.clone(),
                                                extensible: Default::default(),
                                            },
                                        );
                                    }
                                }
                            },
                        }
                    },
                }
            },
        }
    }

    // TODO: embedder to connection channel is not set yet.
    fn handle_embedder(&mut self, msg: Result<EmbedderToWebDriverMessage, RecvError>) {
        match msg {
            Err(err) => warn!("Receiving message from embedder failed ({err:?})"),
            Ok(msg) => match msg {
                EmbedderToWebDriverMessage::Connection(_connection_id, _session_id) => {},
                EmbedderToWebDriverMessage::Command(_conn_id, _command) => {},
            },
        }
    }

    /// See <https://www.w3.org/TR/webdriver-bidi/#get-a-navigable>.
    fn get_a_navigable(&self, navigable_id: BrowsingContextId) -> Result<&Navigable, BidiError> {
        // Step 1. we do not support null
        match self.navigables.get(&navigable_id) {
            // Step 2. if no naviable, error with "no such frame"
            None => Err(ErrorCode::NoSuchFrame.into()),
            // Step 3. let navigable
            Some(navigable) => Ok(navigable),
        }
        // Step 4. implicit return
    }

    /// See <https://www.w3.org/TR/webdriver-bidi/#get-a-realm-from-a-navigable>.
    pub(crate) fn get_a_realm_from_a_navigable(
        &self,
        navigable_id: BrowsingContextId,
        sandbox: Option<String>,
    ) -> Result<&Realm, BidiError> {
        // Step 1. "get a navigable"
        let navigable = self.get_a_navigable(navigable_id)?;
        match sandbox.as_deref() {
            // Step 2. if sandbox null or empty
            None | Some("") => self
                .realms
                .get(&navigable.active_realm)
                .ok_or(ErrorCode::UnknownError.into()),
            // Sandbox is not supported.
            Some(_) => Err(ErrorCode::UnknownError.into()),
        }
    }

    /// See <https://www.w3.org/TR/webdriver-bidi/#get-a-realm-from-a-target>
    pub(crate) fn get_a_realm_from_a_target(&self, target: Target) -> Result<&Realm, BidiError> {
        match target {
            // Step 1.
            Target::Context(target) => {
                // Step 1.1, 1.2. let "sandbox"
                let sandbox = target.sandbox;
                // Step 1.3.
                self.get_a_realm_from_a_navigable(target.context, sandbox)
            },
            // Step 2.
            Target::Realm(target) => {
                // Step 2.1. skip assert
                // Step 2.2. realm id
                let realm_id = target.realm;
                // Step 2.3. "get a relam"
                self.get_a_realm(realm_id)
            },
        }
        // Step 3. implicit return
    }

    /// See <https://www.w3.org/TR/webdriver-bidi/#get-a-realm>.
    fn get_a_realm(&self, realm_id: RealmId) -> Result<&Realm, BidiError> {
        // Step 1. skip, we do not support null
        match self.realms.get(&realm_id) {
            // Step 2. if no realm, error with "no such frame"
            None => Err(ErrorCode::NoSuchFrame.into()),
            // Step 3. return success
            Some(realm) => Ok(realm),
        }
    }

    /// See <https://www.w3.org/TR/webdriver-bidi/#emit-an-event>.
    fn emit_an_event(&self, session: &Session, body: Event) {
        // Step 1. skip assert since we have strong type
        // Step 2. serialize
        let serialized = serde_json::to_string(&body).expect("Serailizing Event is infallible");
        // Step 3. for each connection
        for connecion in session.connections.iter() {
            // Step 3.1. send message
            _ = self.server_sender.send(WebDriverToServerMessage::Message(
                *connecion,
                WsMessage::Text(serialized.clone().into()),
            ));
        }
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#handle-an-incoming-message>
    fn handle_an_incoming_message(&mut self, connection: ConnectionId, data: WsMessage) {
        // Step 1.
        let WsMessage::Text(data) = data else {
            self.send_an_error_response(connection, None, ErrorCode::InvalidArgument.into());
            return;
        };
        // Step 2. skip assert
        // Step 3. match session
        let session = self
            .sessions
            .values()
            .find(|session| session.connections.contains(&connection))
            .map(|session| session.id);
        // Step 4. parse json
        let parsed = match serde_json::from_str::<Value>(&data) {
            Ok(parsed) => parsed,
            Err(err) => {
                warn!("Parsing JSON value from message failed: {err:?}");
                self.send_an_error_response(connection, None, ErrorCode::InvalidArgument.into());
                return;
            },
        };
        // Step 5.
        // Step 6.
        // Step 6. match remote end definition
        match Command::deserialize(&parsed) {
            // Step 6.1.
            Ok(matched) => {
                // Step 6.2. skip assert
                // Step 6.3.
                let command_id = matched.id;
                // Step 6.4. skip
                // Step 6.5.
                let command = matched.command_data;
                // Step 6.6. check static
                if session.is_none() && !command.is_static() {
                    self.send_an_error_response(
                        connection,
                        Some(matched.id),
                        ErrorCode::InvalidSessionId.into(),
                    );
                    return;
                }
                // Step 6.7.1.
                let handle_id = ResumeId::next();
                let response_id = ResumeId::next();
                self.awaits(
                    handle_id,
                    CommandHandled(connection, command_id, response_id),
                );
                self.handle_command(handle_id, response_id, session, command);

                // See [`CommandHandled`] for following steps
            },
            // Step 7.
            Err(err) => {
                warn!("JSON does not match known definition: {err:?}");
                // Step 7.1.
                let mut command_id = None;
                // Step 7.2. get command "id"
                if let Value::Object(map) = &parsed
                    && let Some(value) = map.get("id")
                    && let Some(uint) = value.as_u64()
                {
                    command_id = Some(uint);
                }
                // Step 7.3.
                let error_code = ErrorCode::InvalidArgument;
                // Step 7.4. set error code given "method"
                // TODO: check all commands names
                // Step 7.5. send error response
                self.send_an_error_response(connection, command_id, error_code.into());
            },
        }
    }

    fn send_an_error_response(
        &mut self,
        connection: ConnectionId,
        command_id: Option<u64>,
        error: BidiError,
    ) {
        // Step 1.
        let error_data = ErrorResponse {
            id: command_id,
            error: error.code,
            message: error.message,
            stack_trace: error.stacktrace,
            extensible: Default::default(),
        };
        // Step 2.
        let response =
            serde_json::to_string(&error_data).expect("Serializing error response is infallible");
        _ = self.server_sender.send(WebDriverToServerMessage::Message(
            connection,
            response.into(),
        ));
    }
}

pub(crate) struct Navigable {
    pub(crate) id: BrowsingContextId,
    pub(crate) active_realm: RealmId,
    pub(crate) top_level: bool,
    pub(crate) webview_id: Option<WebViewId>,
}

pub(crate) struct Realm {
    pub(crate) id: RealmId,
    pub(crate) info: RealmInfo,
    pub(crate) sender: GenericSender<WebDriverToScriptMessage>,
}

impl Realm {
    pub(crate) fn navigable_id(&self) -> Option<BrowsingContextId> {
        match &self.info {
            RealmInfo::Window(info) => Some(info.context),
            _ => None,
        }
    }

    pub(crate) fn user_context_id(&self) -> Option<String> {
        match &self.info {
            RealmInfo::Window(info) => info.user_context.clone(),
            _ => None,
        }
    }
}
