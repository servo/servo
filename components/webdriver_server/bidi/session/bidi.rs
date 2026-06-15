use std::sync::Arc;

use indexmap::{IndexMap, IndexSet};
use servo_base::id::BrowsingContextId;
use tokio::sync::RwLock;
use webdriver_traits::bidi::{
    CommandData, EmptyResult, ErrorCode, Event, LogEvent, ResultData,
    script::PreloadScript as PreloadScriptId, session::Subscription as SubscriptionId,
};

use crate::bidi::{
    ActiveSessions,
    connection::Connection,
    session::common::{CommonPart, SessionId, SessionMessage},
};

/// BiDi-specific components of a session.
pub struct BidiPart {
    /// A set of session WebSocket connections associated with this session.
    /// Deviation: we cannot use (ordered) set as we need `iter_mut`.
    /// <https://www.w3.org/TR/webdriver-bidi/#session-websocket-connections>
    pub(crate) connections: Vec<Connection>,

    /// A list of subscriptions for the session.
    /// <https://www.w3.org/TR/webdriver-bidi/#event-subscriptions>
    pub(crate) subscriptions: Vec<Subscription>,

    /// <https://www.w3.org/TR/webdriver-bidi/#event-known-subscription-ids>
    pub(crate) known_subscription_ids: IndexSet<SubscriptionId>,

    /// A map from UUID to preload script.
    /// <https://www.w3.org/TR/webdriver-bidi/#preload-script-map>
    pub(crate) preload_script_map: IndexMap<PreloadScriptId, PreloadScript>,

    /// A map from navigable id to list of log events buffered.
    /// We impose a maximum size of 1000 events per navigable.
    /// <https://www.w3.org/TR/webdriver-bidi/#log-event-buffer>
    pub(crate) log_event_buffer: IndexMap<BrowsingContextId, Vec<LogEvent>>,
}

/// A "subclass" view of BiDi session.
/// With this abstraction, we can follow spec steps more easily.
pub struct BidiSession<'a> {
    pub(crate) id: &'a SessionId,
    pub(crate) common: &'a mut CommonPart,
    pub(crate) bidi: &'a mut BidiPart,
}

impl<'a> BidiSession<'a> {
    /// Remote end steps, the entry point.
    pub(crate) async fn handle_command(
        &mut self,
        command: &CommandData,
    ) -> Result<ResultData, ErrorCode> {
        todo!()
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-session-end>
    async fn handle_session_end(&mut self) -> Result<EmptyResult, ErrorCode> {
        // 1.
        self.end_the_session().await;
        // 3. cleanup should happens after response
        if let Err(e) = self.common.sender.send(SessionMessage::Cleanup) {
            log::warn!("Cleanup message sent failed: {e:?}");
        };
        // 2.
        Ok(EmptyResult {
            extensible: Default::default(),
        })
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#end-the-session>
    async fn end_the_session(&self) {
        // 1.
        self.common
            .remote_end_state
            .active_sessions
            .write()
            .await
            .remove(&self.id);
        // 2. TODO: blocked by webdriver-active flag not implemented
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#cleanup-the-session>
    async fn cleanup_the_session(&mut self) {
        // 1.
        self.close_the_websocket_connections().await;
        // 2. TODO: blocked by user contet not implemented.
        // 3. TODO: network module not implemented
        // 4. TODO: network module not implemented
        // 5. TODO: screencast not implemented
        // 6.
        if self
            .common
            .remote_end_state
            .active_sessions
            .read()
            .await
            .is_empty()
        {
            self.common.remote_end_state.cleanup();
        }
        // 7. SKIP: implementation specific
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#close-the-websocket-connections>
    async fn close_the_websocket_connections(&mut self) {
        // 1.
        for connection in self.bidi.connections.iter_mut() {
            // 1.1.
            if let Err(e) = connection.close(None).await {
                log::warn!("Closing websocket connection failed: {e:?}");
            }
        }
        // result in handle a connection closing
        self.bidi.connections.clear();
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#buffer-a-log-event>
    fn buffer_a_log_event(&mut self, navigable_ids: &[BrowsingContextId], event: LogEvent) {
        // 1.
        let buffer = &mut self.bidi.log_event_buffer;
        // 2-3. SKIP: we can only use naviable id directly
        // 4.
        for navigable_id in navigable_ids {
            // NOTE: the spec self-contradicts here,
            // we choose to follow the `log_event_buffer` type rather than the steps.
            buffer.entry(*navigable_id).or_default().push(event.clone());
        }
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
        for connection in self.bidi.connections.iter_mut() {
            // 3.1.
            connection.send(serialized.clone().into()).await;
        }
    }

    fn connections_mut(&mut self) -> &mut Vec<Connection> {
        &mut self.bidi.connections
    }

    fn connection_mut(&mut self, conn_idx: usize) -> Option<&mut Connection> {
        self.connections_mut().get_mut(conn_idx)
    }

    pub(crate) fn active_session(&self) -> &Arc<RwLock<ActiveSessions>> {
        &self.common.remote_end_state.active_sessions
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

/// <https://www.w3.org/TR/webdriver-bidi/#preload-script-map>
pub struct PreloadScript {
    function_declaration: String,
    arguments: Vec<String>,
    contexts: Option<Vec<String>>,
    sandbox: Option<String>,
    user_contexts: IndexSet<()>,
}
