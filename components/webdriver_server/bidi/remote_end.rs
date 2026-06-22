use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    net::SocketAddr,
    rc::Rc,
};

use async_tungstenite::{
    tokio::accept_async,
    tungstenite::{Error as WsError, Message as WsMessage, Utf8Bytes},
};
use crossbeam_channel::Sender;
use devtools_traits::WorkerId;
use embedder_traits::{EmbedderProxy, EventLoopWaker, GenericEmbedderProxy};
use futures_util::{FutureExt, StreamExt, future};
use log::warn;
use serde::Deserialize;
use serde_json::Value;
use servo_base::{
    generic_channel::GenericSender,
    id::{BrowsingContextId, PainterId, PipelineId, WebViewId},
};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::{Mutex, RwLock, mpsc::UnboundedReceiver, oneshot},
    task,
};
use webdriver_traits::{
    WebDriverMsg, WebDriverToConstellationMsg, WebDriverToEmbedderMsg, WebDriverToScriptMsg,
    bidi::{
        Command, CommandData, CommandResponse, EmptyResult, ErrorCode, ErrorResponse, Event,
        LogEvent, Message, ResultData, SessionCommand, SessionResult,
        script::Realm as RealmId,
        session::{NewParameters, NewResultCapabilities},
    },
};

use crate::bidi::{
    connection::{ConnectionId, ConnectionReceiver, ConnectionSender},
    error::{BidiError, BidiResult},
    session::{Session, SessionId},
    util::new_oneshot_callback,
};

// Since we use single-threaded `LocalRutime`, `RefCell` can be used.
pub(crate) struct RemoteEnd {
    /// An associated list of all sessions that are currently started.
    pub(crate) active_sessions: RefCell<HashMap<SessionId, Session>>,

    /// A set of WebSocket connections not associated with a session.
    pub(crate) unassociated_connections: RefCell<HashSet<ConnectionId>>,

    // the following is the hierarchy of browser.
    pub(crate) client_windows: RefCell<HashMap<PainterId, ClientWindow>>,
    pub(crate) traversables: RefCell<HashMap<WebViewId, Traversable>>,
    pub(crate) navigables: RefCell<HashMap<BrowsingContextId, Navigable>>,
    pub(crate) documents: RefCell<HashMap<PipelineId, Document>>,
    pub(crate) realms: RefCell<HashMap<RealmId, Realm>>,

    /// All the
    pub(crate) connection_senders: RwLock<HashMap<ConnectionId, Mutex<ConnectionSender>>>,

    /// Send message to constellation.
    pub(crate) constellation_sender: Sender<WebDriverToConstellationMsg>,
    pub(crate) script_senders: RefCell<HashMap<PipelineId, GenericSender<WebDriverToScriptMsg>>>,

    /// Receive messages from other components of servo (constellation and script thread).

    /// Send messages and wake embedder.
    embedder_sender: Sender<WebDriverToEmbedderMsg>,
    event_loop_waker: Box<dyn EventLoopWaker>,
}

impl RemoteEnd {
    /// The main loop of a remote end.
    pub(crate) async fn run(
        self: Rc<Self>,
        mut servo_receiver: UnboundedReceiver<WebDriverMsg>,
        listener: TcpListener,
        mut connection_receivers: HashMap<ConnectionId, ConnectionReceiver>,
    ) {
        loop {
            {
                let servo_next = servo_receiver.recv();
                let listener_next = listener.accept();
                let connections_next = future::select_all(
                    connection_receivers
                        .iter_mut()
                        .map(|(conn_id, conn)| conn.next().map(move |msg| (conn_id, msg))),
                )
                .map(|(id_msg, _, _)| id_msg);

                tokio::select! {
                    msg = servo_next => self.clone().handle_servo(msg),
                    stream = listener_next => self.handle_accept(stream, &mut connection_receivers).await,
                    (conn_id, msg) = connections_next => self.clone().handle_connection(*conn_id, msg),
                }
            }

            // TODO: extra step to remove closed connection.
        }
    }

    fn handle_servo(self: Rc<Self>, msg: Option<WebDriverMsg>) {
        let Some(msg) = msg else {
            warn!("Connection from servo closed");
            return;
        };
        match msg {
            WebDriverMsg::FromConstellation(msg) => self.handle_constellation(msg),
            WebDriverMsg::FromScript(msg) => self.handle_script(msg),
        }
    }

    async fn handle_accept(
        &self,
        stream: Result<(TcpStream, SocketAddr), tokio::io::Error>,
        connection_receivers: &mut HashMap<ConnectionId, ConnectionReceiver>,
    ) {
        match stream {
            Ok((tcp_stream, _)) => {
                // TODO: check header
                match accept_async(tcp_stream).await {
                    Ok(ws_stream) => {
                        let conn_id = ConnectionId::next();
                        let (sender, receiver) = ws_stream.split();
                        self.connection_senders
                            .write()
                            .await
                            .insert(conn_id, sender.into());
                        connection_receivers.insert(conn_id, receiver);
                    },
                    Err(err) => {
                        warn!("Accepting websocket connection failed (err: {err:?})");
                    },
                }
            },
            Err(err) => {
                warn!("Accepting new connection failed (err: {:?})", err);
            },
        }
    }

    fn handle_connection(
        self: Rc<Self>,
        connection_id: ConnectionId,
        msg: Option<Result<WsMessage, WsError>>,
    ) {
        match msg {
            Some(msg) => match msg {
                Ok(msg) => {
                    if let WsMessage::Close(_) = msg {
                        self.handle_a_connection_closing(connection_id);
                    }
                    task::spawn_local(self.clone().handle_an_incoming_message(connection_id, msg));
                },
                Err(err) => {
                    warn!(
                        "Receiving message from connection failed (id: {:?}, err: {:?})",
                        connection_id, err
                    );
                },
            },
            None => {
                warn!(
                    "Connection closed without a closing handshake (id: {:?})",
                    connection_id
                );
                self.handle_a_connection_closing(connection_id);
            },
        }
        todo!()
    }

    /// Handle an incoming message from specific bidi connection.
    ///
    /// <https://www.w3.org/TR/webdriver-bidi/#handle-an-incoming-message>
    async fn handle_an_incoming_message(
        self: Rc<Self>,
        connection_id: ConnectionId,
        data: WsMessage,
    ) {
        // Step 1. if not text message, error "invalid argument"
        let WsMessage::Text(data) = data else {
            self.send_an_error_response(connection_id, None, ErrorCode::InvalidArgument)
                .await;
            return;
        };
        // Step 2. skip assert.
        // Step 3. construct session id,
        let session_id = {
            if let Some(session_id) = self
                .active_sessions
                .borrow()
                .values()
                .find(|s| s.is_associated_with_connection(&connection_id))
                .map(|s| s.id)
            {
                Some(session_id)
            } else if self
                .unassociated_connections
                .borrow()
                .contains(&connection_id)
            {
                None
            } else {
                return;
            }
        };
        // Step 4. parse json
        let parsed = match serde_json::from_str::<Value>(&data) {
            Ok(parsed) => parsed,
            Err(err) => {
                warn!("Parsing JSON value from message failed: {err:?}");
                self.send_an_error_response(connection_id, None, ErrorCode::InvalidArgument)
                    .await;
                return;
            },
        };
        // Step 5. check if session is in active session
        if let Some(session_id) = session_id
            && !self.active_sessions.borrow().contains_key(&session_id)
        {
            return;
        }
        // Step 6. match remote end definition
        match Command::deserialize(&parsed) {
            // Step 6.1.
            Ok(matched) => {
                // Step 6.2. skip assert
                // Step 6.3.
                let command_id = matched.id;
                // Step 6.4.
                let method_is_session_new = matches!(
                    matched.command_data,
                    CommandData::SessionCommand(SessionCommand::New(_))
                );
                // Step 6.5.
                let command = matched.command_data;
                // Step 6.6. check static
                if session_id.is_none() && !command.is_static() {
                    self.send_an_error_response(
                        connection_id,
                        Some(command_id),
                        ErrorCode::InvalidSessionId,
                    )
                    .await;
                    return;
                }
                // Step 6.7.1.
                let (msg_sent_tx, msg_sent) = oneshot::channel::<()>();
                let result = self
                    .clone()
                    .handle_command(session_id, command, msg_sent)
                    .await;
                let value = match result {
                    // Step 6.7.2. send error response
                    Err(error) => {
                        self.send_an_error_response(connection_id, Some(command_id), error)
                            .await;
                        return;
                    },
                    // Step 6.7.3.
                    Ok(value) => value,
                };
                // Step 6.7.4. skip assert
                // Step 6.7.5. if session.new, associate conn to session
                if method_is_session_new
                    && let ResultData::SessionResult(SessionResult::NewResult(result)) = &value
                    && let session_id = &result.session_id
                    && let Some(session) = self
                        .active_sessions
                        .borrow_mut()
                        // TODO: codegen should gen uuid
                        .get_mut(&SessionId(session_id.parse().unwrap()))
                {
                    session.connections.insert(connection_id);
                    self.unassociated_connections
                        .borrow_mut()
                        .remove(&connection_id);
                    session.connections.insert(connection_id);
                }
                // Step 6.7.6.
                let response = Message::CommandResponse(Box::new(CommandResponse {
                    id: command_id,
                    result: value,
                    extensible: Default::default(),
                }));
                // Step 6.7.7. serialize
                let serialized = serde_json::to_string(&response)
                    .expect("CommandResponse serialization is infallible");
                // Step 6.7.8. send response message
                match self.connection_senders.read().await.get(&connection_id) {
                    Some(connection) => {
                        if let Err(err) = connection
                            .lock()
                            .await
                            .send(WsMessage::Text(serialized.into()))
                            .await
                        {
                            warn!(
                                "Sending message to connection failde (id: {:?}, err: {:?})",
                                connection_id, err
                            );
                        }
                    },
                    None => {
                        warn!(
                            "Sending command response to an invalid connection (id: {:?})",
                            connection_id
                        );
                    },
                }
                // In addition, notify msg sent to resume `session.end` and `browser.close`
                if let Err(err) = msg_sent_tx.send(()) {
                    warn!("Notifying sending a WebSocket message failed ({err:?})");
                }
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
                let mut error_code = ErrorCode::InvalidArgument;
                // Step 7.4. set error code given "method"
                if let Value::Object(map) = &parsed
                    && let Some(value) = map.get("method")
                    && let Some(str) = value.as_str()
                    && self.set_of_all_command_names().contains(str)
                {
                    error_code = ErrorCode::UnknownCommand;
                }
                // Step 7.5. send error response
                self.send_an_error_response(connection_id, command_id, error_code)
                    .await;
            },
        }
    }

    /// Send an error reponse to the specified bidi connection.
    ///
    /// <https://www.w3.org/TR/webdriver-bidi/#send-an-error-response>
    async fn send_an_error_response(
        self: Rc<Self>,
        connection_id: ConnectionId,
        command_id: Option<u64>,
        error: impl Into<BidiError>,
    ) {
        // Step 1. Construct error data
        let error_data = {
            let error = error.into();
            ErrorResponse {
                id: command_id,
                error: error.code,
                message: error.message,
                stacktrace: error.stacktrace,
                extensible: Default::default(),
            }
        };
        // Step 2. Serialize to text
        let response =
            serde_json::to_string(&error_data).expect("ErrorResponse serialization is infallible");
        // Step 3. Send websocket message
        match self.connection_senders.read().await.get(&connection_id) {
            Some(connection) => {
                if let Err(err) = connection
                    .lock()
                    .await
                    .send(WsMessage::Text(response.into()))
                    .await
                {
                    warn!("Sending error response to ws connection failed: {err:?}");
                }
            },
            None => {
                warn!(
                    "Sending error response to an invalid connection (id: {:?})",
                    connection_id
                );
            },
        }
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#handle-a-connection-closing>
    fn handle_a_connection_closing(&self, connection_id: ConnectionId) {
        // Step 1. remove conn from associated session
        for Session { connections, .. } in self.active_sessions.borrow_mut().values_mut() {
            if connections.remove(&connection_id) {
                break;
            };
        }
        // Step 2. remove conn from unassociated set
        self.unassociated_connections
            .borrow_mut()
            .remove(&connection_id);
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#end-the-session>
    pub(crate) fn end_the_session(&self, session_id: SessionId) {
        // Step 1. remove from active sessions
        self.active_sessions.borrow_mut().remove(&session_id);
        // Step 2. set active flag
        // TODO: send a message
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#cleanup-the-session>
    pub(crate) async fn cleanup_the_session(self: Rc<Self>, session_id: SessionId) {
        // Step 1. close ws connections
        // TODO:
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#close-the-websocket-connections>
    fn close_the_websocket_connections(&self, session_id: SessionId) {
        // TODO:
    }

    /// <https://w3c.github.io/webdriver/#dfn-capabilities-processing>
    pub(crate) fn process_capabilities(
        &self,
        parameters: NewParameters,
        flags: &HashSet<&'static str>,
    ) -> BidiResult<NewResultCapabilities> {
        todo!()
    }

    /// <https://w3c.github.io/webdriver/#dfn-create-a-session>
    pub(crate) fn create_a_session(
        &self,
        capabilities: &NewResultCapabilities,
        flags: &HashSet<&'static str>,
    ) -> BidiResult<Session> {
        todo!()
    }

    pub(crate) async fn close_all_traversables(&self, prompting_to_unload: bool) {
        // TODO: for each webview received before,
        // send close message to constellation and wait for response.
    }

    /// Implementation defined steps to close the browser process.
    pub(crate) fn close_browser(&self) {
        // TODO: send to embedder
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#activate-a-navigable>
    pub(crate) async fn activate_a_navigable(
        self: Rc<Self>,
        navigable_id: BrowsingContextId,
    ) -> BidiResult<EmptyResult> {
        let Some(webview_id) = self
            .navigables
            .borrow()
            .get(&navigable_id)
            .map(|n| n.traversable_id)
        else {
            return Err(ErrorCode::NoSuchFrame.into());
        };
        let (callback, recv) = new_oneshot_callback();
        self.send_to_embedder(WebDriverToEmbedderMsg::Activate(webview_id, callback))?;
        match recv.await {
            Err(err) => {
                warn!("Receiving callback failed ({err:?})");
                Err(ErrorCode::UnknownError.into())
            },
            Ok(msg) => {
                let msg = msg?;
                if msg {
                    Ok(EmptyResult::default())
                } else {
                    Err(ErrorCode::UnsupportedOperation.into())
                }
            },
        }
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#buffer-a-log-event>
    fn buffer_a_log_event(
        &self,
        session_id: SessionId,
        navigable_ids: &[BrowsingContextId],
        event: LogEvent,
    ) {
        // Step 1. get the session's buffer
        let active_sessions = self.active_sessions.borrow();
        let Some(buffer) = active_sessions
            .get(&session_id)
            .map(|s| &s.log_event_buffer)
        else {
            warn!(
                "The session to buffer event is not active (id: {:?})",
                session_id
            );
            return;
        };
        // Step 2-3. skip as we use id directly
        // Step 4.
        for navigable_id in navigable_ids {
            // Step 4.1. record other navigables
            let mut other_navigables = vec![];
            // Step 4.2.
            for other_id in navigable_ids {
                // Step 4.3.
                if other_id != navigable_id {
                    other_navigables.push(*other_id);
                }
            }
            buffer
                .borrow_mut()
                .entry(*navigable_id)
                // Step 4.4. new list if not contained
                .or_default()
                // Step 4.5.
                .push((event.clone(), other_navigables));
        }
    }

    /// <https://w3c.org/TR/webdriver-bidi/#emit-an-event>
    pub(crate) async fn emit_an_event(self: Rc<Self>, session_id: SessionId, body: Event) {
        // Step 1. skip assert
        // Step 2. serialize
        let serialized = serde_json::to_string(&body).expect("Event serialization is infallable");
        let bytes: Utf8Bytes = serialized.into();
        // Step 3. send to each connection
        let session_connections = self
            .active_sessions
            .borrow()
            .get(&session_id)
            .iter()
            .flat_map(|s| s.connections.iter().copied())
            .collect::<Vec<_>>();
        for connection_id in session_connections {
            if let Some(connection) = self.connection_senders.read().await.get(&connection_id)
                && let Err(err) = connection
                    .lock()
                    .await
                    .send(WsMessage::Text(bytes.clone()))
                    .await
            {
                warn!(
                    "Sending event to connection failed (id: {:?}, err: {:?})",
                    connection_id, err
                );
            }
        }
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#get-a-navigable>
    pub(crate) fn get_a_navigable<'a>(
        &'a self,
        navigable_id: BrowsingContextId,
    ) -> BidiResult<Navigable> {
        // Step 1. skip as we do not allow null
        match self.navigables.borrow().get(&navigable_id) {
            // Step 2.
            None => Err(ErrorCode::NoSuchFrame.into()),
            // Step 3.
            Some(navigable) =>
            // Step 4.
            {
                Ok(navigable.clone())
            },
        }
    }

    pub(crate) fn set_of_sessions_for_which_an_event_is_enabled(
        &self,
        event_name: &str,
        navigable_ids: impl Iterator<Item = BrowsingContextId>,
    ) -> HashSet<SessionId> {
        todo!()
    }

    pub(crate) fn send_to_embedder(&self, msg: WebDriverToEmbedderMsg) -> BidiResult<()> {
        if let Err(err) = self.embedder_sender.send(msg) {
            warn!("Error sending WebViewCreate message to embedder ({err:?})");
            return Err(ErrorCode::UnknownError.into());
        }
        self.event_loop_waker.wake();
        Ok(())
    }

    pub(crate) fn send_to_constellation(&self, msg: WebDriverToConstellationMsg) -> BidiResult<()> {
        if let Err(err) = self.constellation_sender.send(msg) {
            warn!("Error sending WebViewCreate message to constellation ({err:?})");
            return Err(ErrorCode::UnknownError.into());
        }
        Ok(())
    }
}

/// The OS client window.
#[derive(Clone, Debug)]
pub struct ClientWindow {
    /// ID of client window itself.
    pub(crate) id: PainterId,
    /// All traversables (tabs) that belongs to this window.
    pub(crate) traversables: Vec<WebViewId>,
}

/// A traversables, which visually corresponds to a tab in OS client window.
#[derive(Clone, Debug)]
pub struct Traversable {
    /// ID of traversable itself.
    pub(crate) id: WebViewId,
    /// The OS client window that contains this.
    pub(crate) window_id: PainterId,
    /// All navigables (tab/iframes) that belongs to this traversable.
    /// The first item represents the top-level navigable (tab).
    pub(crate) navigables: Vec<BrowsingContextId>,
}

/// A navigable, which visually corresponds to a tab or iframe.
#[derive(Clone, Debug)]
pub struct Navigable {
    /// ID of navigable it self.
    pub(crate) id: BrowsingContextId,
    /// The traversable (tab) that contains this.
    pub(crate) traversable_id: WebViewId,
    /// All documents (pipelines) that belongs to this navigable.
    pub(crate) documents: Vec<PipelineId>,
    pub(crate) active_index: usize,
    pub(crate) original_opener: Option<BrowsingContextId>,

    // TODO: should remove this field
    // and query in traversables instead.
    pub(crate) is_top_level_traversable: bool,
    // TODO: parent
}

#[derive(Clone, Debug)]
pub struct Document {
    pub(crate) id: PipelineId,
    pub(crate) navigable_id: BrowsingContextId,
    pub(crate) realms: Vec<RealmId>,
}

#[derive(Clone, Debug)]
pub struct Realm {
    /// Opaque ID of the realm.
    /// Realm is intead indexed by `(PipelineId, Option<WorkerId>)`.
    pub(crate) id: RealmId,
    // TODO: shared worker?
    pub(crate) document_id: PipelineId,
    pub(crate) worker_id: Option<WorkerId>,
}
