/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::VecDeque;

use embedder_traits::user_contents::UserContentManagerId;
use embedder_traits::{InputEvent, MouseLeftViewportEvent, Theme};
use euclid::Point2D;
use log::{debug, warn};
use rustc_hash::{FxHashMap, FxHashSet};
use script_traits::{ConstellationInputEvent, ScriptThreadMessage};
use servo_base::Epoch;
use servo_base::id::{BrowsingContextId, PipelineId, WebViewId};
use servo_constellation_traits::SessionHistoryTraversalRequest;
use style_traits::CSSPixel;

use crate::browsingcontext::BrowsingContext;
use crate::pipeline::Pipeline;
use crate::session_history::{JointSessionHistory, SessionHistoryChange};

/// The `Constellation`'s view of a `WebView` in the embedding layer. This tracks all of the
/// `Constellation` state for this `WebView`.
pub(crate) struct ConstellationWebView {
    /// The [`WebViewId`] of this [`ConstellationWebView`].
    webview_id: WebViewId,

    /// The [`PipelineId`] of the currently active pipeline at the top level of this WebView.
    pub active_top_level_pipeline_id: Option<PipelineId>,

    /// A counter for changes to [`Self::active_top_level_pipeline_id`].
    pub active_top_level_pipeline_epoch: Epoch,

    /// When a navigation is performed, we do not immediately update
    /// the session history, instead we ask the event loop to begin loading
    /// the new document, and do not update the browsing context until the
    /// document is active. Between starting the load and it activating,
    /// we store a `SessionHistoryChange` object for the navigation in progress.
    pub pending_changes: Vec<SessionHistoryChange>,

    /// The currently focused browsing context in this webview for key events.
    /// The focused pipeline is the current entry of the focused browsing
    /// context.
    pub focused_browsing_context_id: BrowsingContextId,

    /// The [`BrowsingContextId`] of the currently hovered browsing context, to use for
    /// knowing which frame is currently receiving cursor events.
    pub hovered_browsing_context_id: Option<BrowsingContextId>,

    /// The last mouse move point in the coordinate space of the Pipeline that it
    /// happened int.
    pub last_mouse_move_point: Point2D<f32, CSSPixel>,

    /// The joint session history for this webview.
    pub session_history: JointSessionHistory,

    /// <https://html.spec.whatwg.org/multipage/#tn-session-history-traversal-queue>
    ///
    /// A queue of traversals that should be applied sequentially. The next item from
    /// the queue is applied once [`Self::ongoing_history_traversal_request`] has finished.
    pub session_history_traversal_request_queue: VecDeque<SessionHistoryTraversalRequest>,

    /// The currently running session history traversal. This will be completed once all
    /// `Pipeline`s in a traversal become active or their load fails for some other reason.
    pub ongoing_history_traversal_request: Option<OngoingHistoryTraversalRequest>,

    /// The [`UserContentManagerId`] for all pipelines in this `WebView`. This is `Some`
    /// if the embedder has set a `UserContentManager` using the WebViewBuilder API and
    /// it is `None` otherwise.
    pub user_content_manager_id: Option<UserContentManagerId>,

    /// The [`Theme`] that this [`ConstellationWebView`] uses. This is communicated to all
    /// `ScriptThread`s so that they know how to render the contents of a particular `WebView.
    theme: Theme,

    /// Whether accessibility is active for this webview.
    ///
    /// Set by [`crate::Constellation::set_accessibility_active()`], and forwarded to the
    /// webview’s *active* pipelines (of those that represent documents) at any given moment
    /// via [`ScriptThreadMessage::SetAccessibilityActive`] in `set_accessibility_active()`
    /// and [`crate::Constellation::set_frame_tree_for_webview()`].
    pub accessibility_active: bool,
}

impl ConstellationWebView {
    pub(crate) fn new(
        webview_id: WebViewId,
        focused_browsing_context_id: BrowsingContextId,
        user_content_manager_id: Option<UserContentManagerId>,
    ) -> Self {
        Self {
            webview_id,
            user_content_manager_id,
            active_top_level_pipeline_id: None,
            active_top_level_pipeline_epoch: Epoch::default(),
            pending_changes: Default::default(),
            focused_browsing_context_id,
            hovered_browsing_context_id: None,
            last_mouse_move_point: Default::default(),
            session_history: JointSessionHistory::new(),
            session_history_traversal_request_queue: Default::default(),
            ongoing_history_traversal_request: None,
            theme: Theme::Light,
            accessibility_active: false,
        }
    }

    /// Set the [`Theme`] on this [`ConstellationWebView`] returning true if the theme changed.
    pub(crate) fn set_theme(&mut self, new_theme: Theme) -> bool {
        let old_theme = std::mem::replace(&mut self.theme, new_theme);
        old_theme != self.theme
    }

    /// Get the [`Theme`] of this [`ConstellationWebView`].
    pub(crate) fn theme(&self) -> Theme {
        self.theme
    }

    fn target_pipeline_id_for_input_event(
        &self,
        event: &ConstellationInputEvent,
        browsing_contexts: &FxHashMap<BrowsingContextId, BrowsingContext>,
    ) -> Option<PipelineId> {
        if let Some(hit_test_result) = &event.hit_test_result {
            return Some(hit_test_result.pipeline_id);
        }

        // If there's no hit test, send the event to either the hovered or focused browsing context,
        // depending on the event type.
        let browsing_context_id = if matches!(event.event.event, InputEvent::MouseLeftViewport(_)) {
            self.hovered_browsing_context_id
                .unwrap_or(self.focused_browsing_context_id)
        } else {
            self.focused_browsing_context_id
        };

        Some(browsing_contexts.get(&browsing_context_id)?.pipeline_id)
    }

    /// Forward the [`InputEvent`] to this [`ConstellationWebView`]. Returns false if
    /// the event could not be forwarded or true otherwise.
    pub(crate) fn forward_input_event(
        &mut self,
        event: ConstellationInputEvent,
        pipelines: &FxHashMap<PipelineId, Pipeline>,
        browsing_contexts: &FxHashMap<BrowsingContextId, BrowsingContext>,
    ) -> bool {
        let Some(pipeline_id) = self.target_pipeline_id_for_input_event(&event, browsing_contexts)
        else {
            warn!("Unknown pipeline for input event. Ignoring.");
            return false;
        };
        let Some(pipeline) = pipelines.get(&pipeline_id) else {
            warn!("Unknown pipeline id {pipeline_id:?} for input event. Ignoring.");
            return false;
        };

        let mut update_hovered_browsing_context =
            |newly_hovered_browsing_context_id, focus_moving_to_another_iframe: bool| {
                let old_hovered_context_id = std::mem::replace(
                    &mut self.hovered_browsing_context_id,
                    newly_hovered_browsing_context_id,
                );
                if old_hovered_context_id == newly_hovered_browsing_context_id {
                    return;
                }
                let Some(old_hovered_context_id) = old_hovered_context_id else {
                    return;
                };
                let Some(pipeline) = browsing_contexts
                    .get(&old_hovered_context_id)
                    .and_then(|browsing_context| pipelines.get(&browsing_context.pipeline_id))
                else {
                    return;
                };

                let mut synthetic_mouse_leave_event = event.clone();
                synthetic_mouse_leave_event.event.event =
                    InputEvent::MouseLeftViewport(MouseLeftViewportEvent {
                        focus_moving_to_another_iframe,
                    });

                let _ = pipeline
                    .event_loop
                    .send(ScriptThreadMessage::SendInputEvent(
                        self.webview_id,
                        pipeline.id,
                        synthetic_mouse_leave_event,
                    ));
            };

        if let InputEvent::MouseLeftViewport(_) = &event.event.event {
            update_hovered_browsing_context(None, false);
            return true;
        }

        if let InputEvent::MouseMove(_) = &event.event.event {
            update_hovered_browsing_context(Some(pipeline.browsing_context_id), true);
            self.last_mouse_move_point = event
                .hit_test_result
                .as_ref()
                .expect("MouseMove events should always have hit tests.")
                .point_in_viewport;
        }

        let _ = pipeline
            .event_loop
            .send(ScriptThreadMessage::SendInputEvent(
                self.webview_id,
                pipeline.id,
                event,
            ));
        true
    }

    /// If there is an ongoing history traversal request that is waiting on documents to
    /// reload, check to see if none of its pipelines are awaiting activation. If that's the
    /// case unset the ongoing request and return it.
    pub(crate) fn maybe_finish_ongoing_session_history_traversal_request(
        &mut self,
    ) -> Option<SessionHistoryTraversalRequest> {
        let ongoing_history_traversal_request = self.ongoing_history_traversal_request.as_mut()?;

        let pipelines_with_pending_changes = self
            .pending_changes
            .iter()
            .map(|change| change.new_pipeline_id)
            .collect::<FxHashSet<_>>();
        ongoing_history_traversal_request
            .pipelines_awaiting_activation
            .retain(|pipeline_id| pipelines_with_pending_changes.contains(pipeline_id));

        if !ongoing_history_traversal_request
            .pipelines_awaiting_activation
            .is_empty()
        {
            return None;
        }
        Some(
            self.ongoing_history_traversal_request
                .take()
                .expect("Guaranteed above")
                .traversal_request,
        )
    }

    pub(crate) fn has_pending_change(&self) -> bool {
        !self.pending_changes.is_empty()
    }

    pub(crate) fn pipeline_is_pending(&self, pipeline_id: PipelineId) -> bool {
        self.pending_changes
            .iter()
            .any(|pending_change| pending_change.new_pipeline_id == pipeline_id)
    }

    pub(crate) fn add_pending_change(&mut self, change: SessionHistoryChange) {
        debug!(
            "adding pending session history change with {}",
            if change.replace.is_some() {
                "replacement"
            } else {
                "no replacement"
            },
        );
        self.pending_changes.push(change);
    }

    pub(crate) fn remove_pending_change_for_pipeline(
        &mut self,
        pipeline_id: PipelineId,
    ) -> Option<SessionHistoryChange> {
        let pending_index = self
            .pending_changes
            .iter()
            .rposition(|change| change.new_pipeline_id == pipeline_id)?;
        Some(self.pending_changes.swap_remove(pending_index))
    }
}

/// A [`HistoryTraversalRequest`] that is in progress because it is waiting
/// for documents that need reloading.
pub(crate) struct OngoingHistoryTraversalRequest {
    /// The [`HistoryTraversalRequest`] that spawned this series of navigations.
    pub traversal_request: SessionHistoryTraversalRequest,
    /// The ids of all the `Pipeline`s that needed reloading for this traversal.
    /// Multiple pipelines can be traversed if the top-level document contained
    /// `<iframe>`s / browsing contexts. The traversal is only done when all of
    /// the pipelines are ready or have failed to load.
    pub pipelines_awaiting_activation: FxHashSet<PipelineId>,
}
