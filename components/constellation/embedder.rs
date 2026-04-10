/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use embedder_traits::{
    InputEventOutcome, JSValue, JavaScriptEvaluationError, JavaScriptEvaluationId,
    MediaSessionEvent, NewWebViewDetails, TraversalId,
};
use servo_base::generic_channel::GenericSender;
use servo_base::id::{PipelineId, WebViewId};
use servo_url::ServoUrl;

/// Messages sent from the `Constellation` to the embedder.
pub enum ConstellationToEmbedderMsg {
    /// Informs the embedder that the constellation has completed shutdown.
    /// Required because the constellation can have pending calls to make
    /// (e.g. SetFrameTree) at the time that we send it an ExitMsg.
    ShutdownComplete,
    /// Report a complete sampled profile
    ReportProfile(Vec<u8>),
    /// All webviews lost focus for keyboard events.
    WebViewBlurred,
    /// A history traversal operation completed.
    HistoryTraversalComplete(WebViewId, TraversalId),
    /// Notifies the embedder about media session events
    /// (i.e. when there is metadata for the active media session, playback state changes...).
    MediaSessionEvent(WebViewId, MediaSessionEvent),
    /// A pipeline panicked. First string is the reason, second one is the backtrace.
    Panic(WebViewId, String, Option<String>),
    /// A webview potentially gained focus for keyboard events.
    /// If the boolean value is false, the webiew could not be focused.
    WebViewFocused(WebViewId, bool),
    /// Inform the embedding layer that a particular `InputEvent` was handled by Servo
    /// and the embedder can continue processing it, if necessary.
    InputEventsHandled(WebViewId, Vec<InputEventOutcome>),
    /// A webview was destroyed.
    WebViewClosed(WebViewId),
    /// Inform the embedding layer that a JavaScript evaluation has
    /// finished with the given result.
    FinishJavaScriptEvaluation(
        JavaScriptEvaluationId,
        Result<JSValue, JavaScriptEvaluationError>,
    ),
    /// Whether or not to allow script to open a new tab/browser
    AllowOpeningWebView(WebViewId, GenericSender<Option<NewWebViewDetails>>),
    /// Whether or not to allow a pipeline to load a url.
    AllowNavigationRequest(WebViewId, PipelineId, ServoUrl),
    /// The history state has changed.
    HistoryChanged(WebViewId, Vec<ServoUrl>, usize),
}
