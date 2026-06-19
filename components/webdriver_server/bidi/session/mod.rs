// TODO: doc naming convention
// - handle_xxx: remote end step for command
// - subscribe_xxx: remote end step for subscribe
// - trigger_xxx: trigger event
//
// TODO: doc deviation of async
// we do not rely on the concept of wait queue, reasons:
// 1. many steps implicitly async, but no described with wait queue.

pub(crate) mod bidi;
pub(crate) mod common;
pub(crate) mod proxy;
pub(crate) mod r#static;

use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
};

use async_tungstenite::tungstenite;
use crossbeam_channel::Sender;
use embedder_traits::{EmbedderMsg, GenericEmbedderProxy};
use futures_util::stream::StreamExt;
use net_traits::ResourceThreads;
use tokio::{
    sync::mpsc::{self, UnboundedSender},
    task,
};
use webdriver_traits::{
    WebDriverToConstellationMsg,
    bidi::{CommandData, CommandResponse, ErrorCode, Message as BidiMessage, SessionCommand},
};

use crate::bidi::{
    RemoteEndState,
    connection::{ConnectionOld, ConnectionId},
    session::{
        bidi::{BidiPart, BidiSession},
        common::{CommonPart, SessionId, SessionMessage},
        proxy::SessionProxy,
        r#static::StaticSession,
    },
};

pub(crate) struct Session {
    pub(crate) id: SessionId,
    /// The BiDi flag.
    pub(crate) bidi: bool,
    /// The http flag.
    pub(crate) http: bool,
    // TODO: re-model flags as bitflag
    pub(crate) flags: HashSet<&'static str>,
    /// The session's associated connections.
    pub(crate) connections: HashSet<ConnectionId>,
}

impl Session {
    /// <https://www.w3.org/TR/webdriver-bidi/#associated-with-connection>
    pub(crate) fn is_associated_with_connection(&self, connection_id: &ConnectionId) -> bool {
        self.connections.contains(connection_id)
    }
}

// ==== The old session impl ====

pub enum SessionOldOwning {
    Static {
        common: CommonPart,
        connections: Vec<ConnectionOld>,
    },
    BidiOnly {
        id: SessionId,
        common: CommonPart,
        bidi: BidiPart,
    },
}

impl SessionOldOwning {
    /// Start static session as a tokio local task.
    pub(crate) fn start_static(
        remote_end_state: Rc<RemoteEndState>,
        embedder_proxy: GenericEmbedderProxy<EmbedderMsg>,
        resource_threads: ResourceThreads,
        constellation_sender: Sender<WebDriverToConstellationMsg>,
    ) -> (task::JoinHandle<()>, UnboundedSender<SessionMessage>) {
        let session = Self::new_static(
            remote_end_state,
            embedder_proxy,
            resource_threads,
            constellation_sender,
        );
        let sender = session.session_sender.clone();
        let handle = task::spawn_local(session.run());
        (handle, sender)
    }

    /// Create a new static session.
    /// The only constructor for session is `new_static`.
    /// All other session should be created by the static session.
    fn new_static(
        remote_end_state: Rc<RemoteEndState>,
        embedder_proxy: GenericEmbedderProxy<EmbedderMsg>,
        resource_threads: ResourceThreads,
        constellation_sender: Sender<WebDriverToConstellationMsg>,
    ) -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        let common = CommonPart {
            running: true,
            remote_end_state,
            embedder_proxy,
            resource_threads,
            constellation_sender,
            session_sender: sender,
            session_receiver: receiver,
        };
        Self::Static {
            common,
            connections: Default::default(),
        }
    }

    /// The main event loop of a session task.
    async fn run(mut self) {
        while self.running {
            match &mut self {
                SessionOldOwning::Static {
                    common,
                    connections,
                    ..
                }
                | SessionOldOwning::BidiOnly {
                    common,
                    bidi: BidiPart { connections, .. },
                    ..
                } => {
                    let receiver_next = common.session_receiver.recv();
                    let connections_next =
                        futures_util::future::select_all(connections.iter_mut().map(|c| c.next()));
                    tokio::select! {
                        msg = receiver_next => self.handle_receiver(msg).await,
                        (msg, conn_idx, _) = connections_next => {
                            self.handle_websocket(conn_idx, msg).await
                        },
                    };
                },
            }
        }
    }

    /// View the session as BiDi or static "subclass".
    /// Return `None` when session is classic only otherwise.
    fn as_bidi_or_static<'a>(&'a mut self) -> Option<Result<BidiSession<'a>, StaticSession<'a>>> {
        match self {
            SessionOldOwning::Static {
                common,
                connections,
            } => Some(Err(StaticSession {
                common,
                connections,
            })),
            SessionOldOwning::BidiOnly { id, common, bidi } => {
                Some(Ok(BidiSession { id, common, bidi }))
            },
        }
    }

    /// Create a proxy to self, so that self can receive message from `active_sessions`.
    fn to_proxy(&self) -> SessionProxy {
        let bidi_flag = matches!(self, SessionOldOwning::BidiOnly { .. });
        let sender = self.session_sender.clone();
        SessionProxy { bidi_flag, sender }
    }

    /// Handle messages from receiver.
    /// The message may come from listener, self, or other sessions.
    async fn handle_receiver(&mut self, msg: Option<SessionMessage>) {
        match msg {
            Some(msg) => match msg {
                SessionMessage::Associate(connection) => {
                    self.associate_connection(connection);
                },
                SessionMessage::CleanupSession(sender) => {
                    let success = if let Some(Ok(mut bidi_session)) = self.as_bidi_or_static() {
                        bidi_session.cleanup_the_session().await;
                        true
                    } else {
                        log::warn!("Cannot cleanup a non BiDi session");
                        false
                    };
                    if let Some(sender) = sender {
                        if let Err(e) = sender.send(success) {
                            log::warn!("Sending cleanup callback failed: {e:?}");
                        };
                    }
                },
                SessionMessage::Script(_script) => {
                    // TODO
                },
                SessionMessage::WebDriver(_constellation_to_web_driver_message) => {
                    // TODO
                },
            },
            None => log::warn!("Listener closed."),
        }
    }

    /// The steps for "When a WebSocket message has been received".
    async fn handle_websocket(
        &mut self,
        conn_idx: usize,
        msg: Option<Result<tungstenite::Message, tungstenite::Error>>,
    ) {
        match msg {
            Some(msg) => match msg {
                Ok(msg) => {
                    self.handle_an_incoming_message(conn_idx, msg).await;
                },
                Err(e) => {
                    log::warn!("Receiving WebSocket message error: {e:?}");
                },
            },
            // if the connection is closed
            None => {
                self.handle_a_connection_closing(conn_idx);
            },
        }
    }

    /// Append a connection to a session's connection set.
    fn associate_connection(&mut self, connection: ConnectionOld) {
        if let Some(connections) = self.connections_mut() {
            connections.push(connection);
        } else {
            log::warn!("Session to associate is not static or bidi");
        }
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#handle-a-connection-closing>
    fn handle_a_connection_closing(&mut self, conn_idx: usize) {
        self.connections_mut().unwrap().remove(conn_idx);
    }

    fn connections_mut(&mut self) -> Option<&mut Vec<ConnectionOld>> {
        match self {
            SessionOldOwning::Static { connections, .. } => Some(connections),
            SessionOldOwning::BidiOnly { bidi, .. } => Some(&mut bidi.connections),
        }
    }

    fn connection_mut(&mut self, conn_idx: usize) -> Option<&mut ConnectionOld> {
        self.connections_mut()?.get_mut(conn_idx)
    }

    /// Handlle an incoming WebSocket message.
    /// Deviations:
    /// 1. we use connection idx instead due to single ownership rule.
    /// 2. type and data are represented as Rust enum variant.
    /// <https://www.w3.org/TR/webdriver-bidi/#handle-an-incoming-message>
    async fn handle_an_incoming_message(&mut self, conn_idx: usize, msg: tungstenite::Message) {
        // 1.
        let tungstenite::Message::Text(data) = msg else {
            return;
        };
        // 2. SKIP: Assert
        // 3. use `Result<BidiSession, StaticSession>` instead of `Option<BidiSession>`
        let Some(session) = self.as_bidi_or_static() else {
            return;
        };
        // 4.
        let Ok(parsed) = serde_json::from_str::<webdriver_traits::bidi::Command>(&data) else {
            self.connection_mut(conn_idx)
                .unwrap()
                .send_an_error_response(None, ErrorCode::InvalidArgument)
                .await;
            return;
        };
        // 5. if is Ok(BidiSession) instead of `Some(BidiSession)`
        if let Ok(ref bidi_session) = session
            && bidi_session
                .active_sessions()
                .read()
                .await
                .contains_key(bidi_session.id)
        {
            return;
        }
        // 6.
        // 6.1.
        let matched = &parsed;
        // 6.2. SKIP: Assert
        // 6.3.
        let command_id = matched.id;
        // 6.4. SKIP: method field is represented as enum
        // 6.5.
        let command = &matched.command_data;
        // 6.7.1
        let result = match session {
            // 6.6.
            Err(mut static_session) => match command.try_into() {
                Ok(static_command) => static_session.handle_static_command(static_command).await,
                Err(_) => {
                    static_session
                        .connections
                        .get_mut(conn_idx)
                        .unwrap()
                        .send_an_error_response(Some(command_id), ErrorCode::InvalidSessionId)
                        .await;
                    return;
                },
            },
            Ok(mut bidi_session) => bidi_session.handle_command(command).await,
        };
        // 6.7.2.
        let result = match result {
            Ok(result) => result,
            Err(error_code) => {
                self.connection_mut(conn_idx)
                    .unwrap()
                    .send_an_error_response(Some(command_id), error_code)
                    .await;
                return;
            },
        };
        // 6.7.3.
        let value = result;
        // 6.7.4. SKIP: Assert
        // 6.7.5.
        if matches!(command, CommandData::SessionCommand(SessionCommand::New(_))) {
            // Step 6.7. "in parallel", allowing us to handle it next tick
            // TODO: remove and append
        }
        // 6.7.6.
        let response = BidiMessage::CommandResponse(Box::new(CommandResponse {
            id: command_id,
            result: value,
            extensible: Default::default(),
        }));
        // 6.7.7.
        let serialized =
            serde_json::to_string(&response).expect("Serializing command response failed");
        // 6.7.8.
        if let Err(e) = self
            .connection_mut(conn_idx)
            .unwrap()
            .send(serialized.into())
            .await
        {
            log::warn!("Sending command response failed: {e:?}");
        }
        // 7.
        // 7.1. SKIP: serde parse completely
        // 7.2.
        let command_id = Some(parsed.id);
        // 7.3.
        let error_code = ErrorCode::InvalidArgument;
        // 7.4. TODO: custom module name not implemented
        // 7.5.
        self.connection_mut(conn_idx)
            .unwrap()
            .send_an_error_response(command_id, error_code)
            .await;
    }
}
