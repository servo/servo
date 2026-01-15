/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::HashSet;
use std::rc::Rc;

use base::id::{BrowsingContextId, HistoryStateId, PipelineId, WebViewId};
use compositing_traits::{CompositionPipeline, PaintMessage, PaintProxy};
use constellation_traits::{LoadData, ServiceWorkerManagerFactory};
use embedder_traits::{AnimationState, FocusSequenceNumber};
use ipc_channel::Error;
use layout_api::ScriptThreadFactory;
use log::{debug, error, warn};
use script_traits::{
    DiscardBrowsingContext, DocumentActivity, NewPipelineInfo, ScriptThreadMessage,
};
use servo_url::ServoUrl;

use crate::Constellation;
use crate::event_loop::EventLoop;

/// A `Pipeline` is the constellation's view of a `Window`. Each pipeline has an event loop
/// (executed by a script thread). A script thread may be responsible for many pipelines.
pub struct Pipeline {
    /// The ID of the pipeline.
    pub id: PipelineId,

    /// The ID of the browsing context that contains this Pipeline.
    pub browsing_context_id: BrowsingContextId,

    /// The [`WebViewId`] of the `WebView` that contains this Pipeline.
    pub webview_id: WebViewId,

    pub opener: Option<BrowsingContextId>,

    /// The event loop handling this pipeline.
    pub event_loop: Rc<EventLoop>,

    /// A channel to `Paint`.
    pub paint_proxy: PaintProxy,

    /// The most recently loaded URL in this pipeline.
    /// Note that this URL can change, for example if the page navigates
    /// to a hash URL.
    pub url: ServoUrl,

    /// Whether this pipeline is currently running animations. Pipelines that are running
    /// animations cause composites to be continually scheduled.
    pub animation_state: AnimationState,

    /// The child browsing contexts of this pipeline (these are iframes in the document).
    pub children: Vec<BrowsingContextId>,

    /// The Load Data used to create this pipeline.
    pub load_data: LoadData,

    /// The active history state for this pipeline.
    pub history_state_id: Option<HistoryStateId>,

    /// The history states owned by this pipeline.
    pub history_states: HashSet<HistoryStateId>,

    /// Has this pipeline received a notification that it is completely loaded?
    pub completely_loaded: bool,

    /// The title of this pipeline's document.
    pub title: String,

    /// The [`FocusSequenceNumber`] of this [`Pipeline`].
    pub focus_sequence: FocusSequenceNumber,

    /// Whether or not this [`Pipeline`] has an actively loading/loaded. When there is no
    /// active document, the [`Pipeline`] will never be unthrottled. Throttled pipelines
    /// do not update animations and their timers are slowed.
    has_active_document: bool,
}

impl Pipeline {
    /// Possibly starts a script thread, in a new process if requested.
    pub(crate) fn spawn<STF: ScriptThreadFactory, SWF: ServiceWorkerManagerFactory>(
        new_pipeline_info: NewPipelineInfo,
        event_loop: Rc<EventLoop>,
        constellation: &Constellation<STF, SWF>,
        webview_hidden: bool,
    ) -> Result<Self, Error> {
        if let Err(error) = event_loop.send(ScriptThreadMessage::SpawnPipeline(
            new_pipeline_info.clone(),
        )) {
            error!("Could not spawn Pipeline in EventLoop: {error}");
            return Err(error);
        }

        Ok(Self::new_already_spawned(
            new_pipeline_info.new_pipeline_id,
            new_pipeline_info.browsing_context_id,
            new_pipeline_info.webview_id,
            new_pipeline_info.opener,
            event_loop,
            constellation.paint_proxy.clone(),
            webview_hidden,
            new_pipeline_info.load_data,
        ))
    }

    /// Creates a new `Pipeline`, after it has been spawned in its [`EventLoop`].
    #[expect(clippy::too_many_arguments)]
    pub fn new_already_spawned(
        id: PipelineId,
        browsing_context_id: BrowsingContextId,
        webview_id: WebViewId,
        opener: Option<BrowsingContextId>,
        event_loop: Rc<EventLoop>,
        paint_proxy: PaintProxy,
        webview_hidden: bool,
        load_data: LoadData,
    ) -> Self {
        let pipeline = Self {
            id,
            browsing_context_id,
            webview_id,
            opener,
            event_loop,
            paint_proxy,
            url: load_data.url.clone(),
            children: vec![],
            animation_state: AnimationState::NoAnimationsPresent,
            load_data,
            history_state_id: None,
            history_states: HashSet::new(),
            completely_loaded: false,
            title: String::new(),
            focus_sequence: FocusSequenceNumber::default(),
            // Assume that every new Pipeline has an active document until told otherwise.
            has_active_document: true,
        };

        if webview_hidden {
            pipeline.send_throttle_messages(true);
        }

        pipeline
    }

    /// Let the `ScriptThread` for this [`Pipeline`] know that it has exited. If the `ScriptThread` hasn't
    /// panicked and is still alive, it will send a `PipelineExited` message back to the `Constellation`
    /// when it finishes cleaning up.
    pub fn send_exit_message_to_script(&self, discard_bc: DiscardBrowsingContext) {
        debug!("{:?} Sending exit message to script", self.id);

        // Script thread handles shutting down layout, and layout handles shutting down the painter.
        // For now, if the script thread has failed, we give up on clean shutdown.
        if let Err(error) = self.event_loop.send(ScriptThreadMessage::ExitPipeline(
            self.webview_id,
            self.id,
            discard_bc,
        )) {
            warn!("Sending script exit message failed ({error}).");
        }
    }

    /// Notify this pipeline of its activity.
    pub fn set_activity(&self, activity: DocumentActivity) {
        let msg = ScriptThreadMessage::SetDocumentActivity(self.id, activity);
        if let Err(e) = self.event_loop.send(msg) {
            warn!("Sending activity message failed ({}).", e);
        }
    }

    /// `Paint`'s view of a pipeline.
    pub fn to_sendable(&self) -> CompositionPipeline {
        CompositionPipeline {
            id: self.id,
            webview_id: self.webview_id,
        }
    }

    /// Add a new child browsing context.
    pub fn add_child(&mut self, browsing_context_id: BrowsingContextId) {
        self.children.push(browsing_context_id);
    }

    /// Remove a child browsing context.
    pub fn remove_child(&mut self, browsing_context_id: BrowsingContextId) {
        match self
            .children
            .iter()
            .position(|id| *id == browsing_context_id)
        {
            None => {
                warn!(
                    "Pipeline remove child already removed ({:?}).",
                    browsing_context_id
                )
            },
            Some(index) => {
                self.children.remove(index);
            },
        }
    }

    /// Set whether or not this Pipeline has an active document.
    pub(crate) fn set_has_active_document(&mut self, has_active_document: bool) {
        if self.has_active_document == has_active_document {
            return;
        }

        self.has_active_document = has_active_document;

        // If the active document has gone away, throttle the WebView.
        if !self.has_active_document {
            self.send_throttle_messages(true);
        }
    }

    /// Set whether this Pipeline is throttled or unthrottled. If the Pipeline
    /// does not have an active Document, it will not be unthrottled until it does.
    pub(crate) fn send_throttle_messages(&self, throttled: bool) {
        // Never unthrottled Pipelines that do not have an active Document.
        let throttled = !self.has_active_document || throttled;

        if let Err(error) = self.event_loop.send(ScriptThreadMessage::SetThrottled(
            self.webview_id,
            self.id,
            throttled,
        )) {
            warn!("Sending SetThrottled to script failed ({error}).");
        }
        self.paint_proxy.send(PaintMessage::SetThrottled(
            self.webview_id,
            self.id,
            throttled,
        ));
    }
}
