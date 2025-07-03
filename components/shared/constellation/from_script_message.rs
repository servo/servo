/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Messages send from the ScriptThread to the Constellation.

use std::collections::HashMap;
use std::fmt;

use base::Epoch;
use base::id::{
    BroadcastChannelRouterId, BrowsingContextId, HistoryStateId, MessagePortId,
    MessagePortRouterId, PipelineId, ServiceWorkerId, ServiceWorkerRegistrationId, WebViewId,
};
use canvas_traits::canvas::{CanvasId, CanvasMsg};
use devtools_traits::{DevtoolScriptControlMsg, ScriptToDevtoolsControlMsg, WorkerId};
use embedder_traits::{
    AnimationState, EmbedderMsg, FocusSequenceNumber, JSValue, JavaScriptEvaluationError,
    JavaScriptEvaluationId, MediaSessionEvent, Theme, TouchEventResult, ViewportDetails,
    WebDriverMessageId,
};
use euclid::default::Size2D as UntypedSize2D;
use http::{HeaderMap, Method};
use ipc_channel::Error as IpcError;
use ipc_channel::ipc::{IpcReceiver, IpcSender};
use net_traits::policy_container::PolicyContainer;
use net_traits::request::{Destination, InsecureRequestsPolicy, Referrer, RequestBody};
use net_traits::storage_thread::StorageType;
use net_traits::{CoreResourceMsg, ReferrerPolicy, ResourceThreads};
use profile_traits::mem::MemoryReportResult;
use profile_traits::{mem, time as profile_time};
use serde::{Deserialize, Serialize};
use servo_url::{ImmutableOrigin, ServoUrl};
use strum_macros::IntoStaticStr;
#[cfg(feature = "webgpu")]
use webgpu_traits::{WebGPU, WebGPUAdapterResponse};
use webrender_api::ImageKey;

use crate::structured_data::{BroadcastMsg, StructuredSerializedData};
use crate::{
    LogEntry, MessagePortMsg, PortMessageTask, PortTransferInfo, TraversalDirection, WindowSizeType,
};

/// A Script to Constellation channel.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ScriptToConstellationChan {
    /// Sender for communicating with constellation thread.
    pub sender: IpcSender<(PipelineId, ScriptToConstellationMessage)>,
    /// Used to identify the origin of the message.
    pub pipeline_id: PipelineId,
}

impl ScriptToConstellationChan {
    /// Send ScriptMsg and attach the pipeline_id to the message.
    pub fn send(&self, msg: ScriptToConstellationMessage) -> Result<(), IpcError> {
        self.sender.send((self.pipeline_id, msg))
    }
}

/// The origin where a given load was initiated.
/// Useful for origin checks, for example before evaluation a JS URL.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum LoadOrigin {
    /// A load originating in the constellation.
    Constellation,
    /// A load originating in webdriver.
    WebDriver,
    /// A load originating in script.
    Script(ImmutableOrigin),
}

/// can be passed to `LoadUrl` to load a page with GET/POST
/// parameters or headers
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LoadData {
    /// The origin where the load started.
    pub load_origin: LoadOrigin,
    /// The URL.
    pub url: ServoUrl,
    /// The creator pipeline id if this is an about:blank load.
    pub creator_pipeline_id: Option<PipelineId>,
    /// The method.
    #[serde(
        deserialize_with = "::hyper_serde::deserialize",
        serialize_with = "::hyper_serde::serialize"
    )]
    pub method: Method,
    /// The headers.
    #[serde(
        deserialize_with = "::hyper_serde::deserialize",
        serialize_with = "::hyper_serde::serialize"
    )]
    pub headers: HeaderMap,
    /// The data that will be used as the body of the request.
    pub data: Option<RequestBody>,
    /// The result of evaluating a javascript scheme url.
    pub js_eval_result: Option<JsEvalResult>,
    /// The referrer.
    pub referrer: Referrer,
    /// The referrer policy.
    pub referrer_policy: ReferrerPolicy,
    /// The policy container.
    pub policy_container: Option<PolicyContainer>,

    /// The source to use instead of a network response for a srcdoc document.
    pub srcdoc: String,
    /// The inherited context is Secure, None if not inherited
    pub inherited_secure_context: Option<bool>,
    /// The inherited policy for upgrading insecure requests; None if not inherited.
    pub inherited_insecure_requests_policy: Option<InsecureRequestsPolicy>,
    /// Whether the page's ancestors have potentially trustworthy origin
    pub has_trustworthy_ancestor_origin: bool,
    /// Servo internal: if crash details are present, trigger a crash error page with these details.
    pub crash: Option<String>,
    /// Destination, used for CSP checks
    pub destination: Destination,
}

/// The result of evaluating a javascript scheme url.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum JsEvalResult {
    /// The js evaluation had a non-string result, 204 status code.
    /// <https://html.spec.whatwg.org/multipage/#navigate> 12.11
    NoContent,
    /// The js evaluation had a string result.
    Ok(Vec<u8>),
}

impl LoadData {
    /// Create a new `LoadData` object.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        load_origin: LoadOrigin,
        url: ServoUrl,
        creator_pipeline_id: Option<PipelineId>,
        referrer: Referrer,
        referrer_policy: ReferrerPolicy,
        inherited_secure_context: Option<bool>,
        inherited_insecure_requests_policy: Option<InsecureRequestsPolicy>,
        has_trustworthy_ancestor_origin: bool,
    ) -> LoadData {
        LoadData {
            load_origin,
            url,
            creator_pipeline_id,
            method: Method::GET,
            headers: HeaderMap::new(),
            data: None,
            js_eval_result: None,
            referrer,
            referrer_policy,
            policy_container: None,
            srcdoc: "".to_string(),
            inherited_secure_context,
            crash: None,
            inherited_insecure_requests_policy,
            has_trustworthy_ancestor_origin,
            destination: Destination::Document,
        }
    }
}

/// <https://html.spec.whatwg.org/multipage/#navigation-supporting-concepts:navigationhistorybehavior>
#[derive(Debug, Default, Deserialize, PartialEq, Serialize)]
pub enum NavigationHistoryBehavior {
    /// The default value, which will be converted very early in the navigate algorithm into "push"
    /// or "replace". Usually it becomes "push", but under certain circumstances it becomes
    /// "replace" instead.
    #[default]
    Auto,
    /// A regular navigation which adds a new session history entry, and will clear the forward
    /// session history.
    Push,
    /// A navigation that will replace the active session history entry.
    Replace,
}

/// Entities required to spawn service workers
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ScopeThings {
    /// script resource url
    pub script_url: ServoUrl,
    /// network load origin of the resource
    pub worker_load_origin: WorkerScriptLoadOrigin,
    /// base resources required to create worker global scopes
    pub init: WorkerGlobalScopeInit,
    /// the port to receive devtools message from
    pub devtools_chan: Option<IpcSender<ScriptToDevtoolsControlMsg>>,
    /// service worker id
    pub worker_id: WorkerId,
}

/// Message that gets passed to service worker scope on postMessage
#[derive(Debug, Deserialize, Serialize)]
pub struct DOMMessage {
    /// The origin of the message
    pub origin: ImmutableOrigin,
    /// The payload of the message
    pub data: StructuredSerializedData,
}

/// Channels to allow service worker manager to communicate with constellation and resource thread
#[derive(Deserialize, Serialize)]
pub struct SWManagerSenders {
    /// Sender of messages to the constellation.
    pub swmanager_sender: IpcSender<SWManagerMsg>,
    /// Sender for communicating with resource thread.
    pub resource_sender: IpcSender<CoreResourceMsg>,
    /// Sender of messages to the manager.
    pub own_sender: IpcSender<ServiceWorkerMsg>,
    /// Receiver of messages from the constellation.
    pub receiver: IpcReceiver<ServiceWorkerMsg>,
}

/// Messages sent to Service Worker Manager thread
#[derive(Debug, Deserialize, Serialize)]
#[allow(clippy::large_enum_variant)]
pub enum ServiceWorkerMsg {
    /// Timeout message sent by active service workers
    Timeout(ServoUrl),
    /// Message sent by constellation to forward to a running service worker
    ForwardDOMMessage(DOMMessage, ServoUrl),
    /// <https://w3c.github.io/ServiceWorker/#schedule-job-algorithm>
    ScheduleJob(Job),
    /// Exit the service worker manager
    Exit,
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
/// <https://w3c.github.io/ServiceWorker/#dfn-job-type>
pub enum JobType {
    /// <https://w3c.github.io/ServiceWorker/#register>
    Register,
    /// <https://w3c.github.io/ServiceWorker/#unregister-algorithm>
    Unregister,
    /// <https://w3c.github.io/ServiceWorker/#update-algorithm>
    Update,
}

#[derive(Debug, Deserialize, Serialize)]
/// The kind of error the job promise should be rejected with.
pub enum JobError {
    /// <https://w3c.github.io/ServiceWorker/#reject-job-promise>
    TypeError,
    /// <https://w3c.github.io/ServiceWorker/#reject-job-promise>
    SecurityError,
}

#[derive(Debug, Deserialize, Serialize)]
#[allow(clippy::large_enum_variant)]
/// Messages sent from Job algorithms steps running in the SW manager,
/// in order to resolve or reject the job promise.
pub enum JobResult {
    /// <https://w3c.github.io/ServiceWorker/#reject-job-promise>
    RejectPromise(JobError),
    /// <https://w3c.github.io/ServiceWorker/#resolve-job-promise>
    ResolvePromise(Job, JobResultValue),
}

#[derive(Debug, Deserialize, Serialize)]
/// Jobs are resolved with the help of various values.
pub enum JobResultValue {
    /// Data representing a serviceworker registration.
    Registration {
        /// The Id of the registration.
        id: ServiceWorkerRegistrationId,
        /// The installing worker, if any.
        installing_worker: Option<ServiceWorkerId>,
        /// The waiting worker, if any.
        waiting_worker: Option<ServiceWorkerId>,
        /// The active worker, if any.
        active_worker: Option<ServiceWorkerId>,
    },
}

#[derive(Debug, Deserialize, Serialize)]
/// <https://w3c.github.io/ServiceWorker/#dfn-job>
pub struct Job {
    /// <https://w3c.github.io/ServiceWorker/#dfn-job-type>
    pub job_type: JobType,
    /// <https://w3c.github.io/ServiceWorker/#dfn-job-scope-url>
    pub scope_url: ServoUrl,
    /// <https://w3c.github.io/ServiceWorker/#dfn-job-script-url>
    pub script_url: ServoUrl,
    /// <https://w3c.github.io/ServiceWorker/#dfn-job-client>
    pub client: IpcSender<JobResult>,
    /// <https://w3c.github.io/ServiceWorker/#job-referrer>
    pub referrer: ServoUrl,
    /// Various data needed to process job.
    pub scope_things: Option<ScopeThings>,
}

impl Job {
    /// <https://w3c.github.io/ServiceWorker/#create-job-algorithm>
    pub fn create_job(
        job_type: JobType,
        scope_url: ServoUrl,
        script_url: ServoUrl,
        client: IpcSender<JobResult>,
        referrer: ServoUrl,
        scope_things: Option<ScopeThings>,
    ) -> Job {
        Job {
            job_type,
            scope_url,
            script_url,
            client,
            referrer,
            scope_things,
        }
    }
}

impl PartialEq for Job {
    /// Equality criteria as described in <https://w3c.github.io/ServiceWorker/#dfn-job-equivalent>
    fn eq(&self, other: &Self) -> bool {
        // TODO: match on job type, take worker type and `update_via_cache_mode` into account.
        let same_job = self.job_type == other.job_type;
        if same_job {
            match self.job_type {
                JobType::Register | JobType::Update => {
                    self.scope_url == other.scope_url && self.script_url == other.script_url
                },
                JobType::Unregister => self.scope_url == other.scope_url,
            }
        } else {
            false
        }
    }
}

/// Messages outgoing from the Service Worker Manager thread to constellation
#[derive(Debug, Deserialize, Serialize)]
pub enum SWManagerMsg {
    /// Placeholder to keep the enum,
    /// as it will be needed when implementing
    /// <https://github.com/servo/servo/issues/24660>
    PostMessageToClient,
}

/// Used to determine if a script has any pending asynchronous activity.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub enum DocumentState {
    /// The document has been loaded and is idle.
    Idle,
    /// The document is either loading or waiting on an event.
    Pending,
}

/// This trait allows creating a `ServiceWorkerManager` without depending on the `script`
/// crate.
pub trait ServiceWorkerManagerFactory {
    /// Create a `ServiceWorkerManager`.
    fn create(sw_senders: SWManagerSenders, origin: ImmutableOrigin);
}

/// Whether the sandbox attribute is present for an iframe element
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum IFrameSandboxState {
    /// Sandbox attribute is present
    IFrameSandboxed,
    /// Sandbox attribute is not present
    IFrameUnsandboxed,
}

/// Specifies the information required to load an auxiliary browsing context.
#[derive(Debug, Deserialize, Serialize)]
pub struct AuxiliaryWebViewCreationRequest {
    /// Load data containing the url to load
    pub load_data: LoadData,
    /// The webview that caused this request.
    pub opener_webview_id: WebViewId,
    /// The pipeline opener browsing context.
    pub opener_pipeline_id: PipelineId,
    /// Sender for the constellation’s response to our request.
    pub response_sender: IpcSender<Option<AuxiliaryWebViewCreationResponse>>,
}

/// Constellation’s response to auxiliary browsing context creation requests.
#[derive(Debug, Deserialize, Serialize)]
pub struct AuxiliaryWebViewCreationResponse {
    /// The new webview ID.
    pub new_webview_id: WebViewId,
    /// The new pipeline ID.
    pub new_pipeline_id: PipelineId,
}

/// Specifies the information required to load an iframe.
#[derive(Debug, Deserialize, Serialize)]
pub struct IFrameLoadInfo {
    /// Pipeline ID of the parent of this iframe
    pub parent_pipeline_id: PipelineId,
    /// The ID for this iframe's nested browsing context.
    pub browsing_context_id: BrowsingContextId,
    /// The ID for the top-level ancestor browsing context of this iframe's nested browsing context.
    pub webview_id: WebViewId,
    /// The new pipeline ID that the iframe has generated.
    pub new_pipeline_id: PipelineId,
    ///  Whether this iframe should be considered private
    pub is_private: bool,
    ///  Whether this iframe should be considered secure
    pub inherited_secure_context: Option<bool>,
    /// Whether this load should replace the current entry (reload). If true, the current
    /// entry will be replaced instead of a new entry being added.
    pub history_handling: NavigationHistoryBehavior,
}

/// Specifies the information required to load a URL in an iframe.
#[derive(Debug, Deserialize, Serialize)]
pub struct IFrameLoadInfoWithData {
    /// The information required to load an iframe.
    pub info: IFrameLoadInfo,
    /// Load data containing the url to load
    pub load_data: LoadData,
    /// The old pipeline ID for this iframe, if a page was previously loaded.
    pub old_pipeline_id: Option<PipelineId>,
    /// Sandbox type of this iframe
    pub sandbox: IFrameSandboxState,
    /// The initial viewport size for this iframe.
    pub viewport_details: ViewportDetails,
    /// The [`Theme`] to use within this iframe.
    pub theme: Theme,
}

/// Resources required by workerglobalscopes
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct WorkerGlobalScopeInit {
    /// Chan to a resource thread
    pub resource_threads: ResourceThreads,
    /// Chan to the memory profiler
    pub mem_profiler_chan: mem::ProfilerChan,
    /// Chan to the time profiler
    pub time_profiler_chan: profile_time::ProfilerChan,
    /// To devtools sender
    pub to_devtools_sender: Option<IpcSender<ScriptToDevtoolsControlMsg>>,
    /// From devtools sender
    pub from_devtools_sender: Option<IpcSender<DevtoolScriptControlMsg>>,
    /// Messages to send to constellation
    pub script_to_constellation_chan: ScriptToConstellationChan,
    /// The worker id
    pub worker_id: WorkerId,
    /// The pipeline id
    pub pipeline_id: PipelineId,
    /// The origin
    pub origin: ImmutableOrigin,
    /// The creation URL
    pub creation_url: ServoUrl,
    /// True if secure context
    pub inherited_secure_context: Option<bool>,
}

/// Common entities representing a network load origin
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct WorkerScriptLoadOrigin {
    /// referrer url
    pub referrer_url: Option<ServoUrl>,
    /// the referrer policy which is used
    pub referrer_policy: ReferrerPolicy,
    /// the pipeline id of the entity requesting the load
    pub pipeline_id: PipelineId,
}

/// An iframe sizing operation.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct IFrameSizeMsg {
    /// The child browsing context for this iframe.
    pub browsing_context_id: BrowsingContextId,
    /// The size and scale factor of the iframe.
    pub size: ViewportDetails,
    /// The kind of sizing operation.
    pub type_: WindowSizeType,
}

/// Messages from the script to the constellation.
#[derive(Deserialize, IntoStaticStr, Serialize)]
pub enum ScriptToConstellationMessage {
    /// Request to complete the transfer of a set of ports to a router.
    CompleteMessagePortTransfer(MessagePortRouterId, Vec<MessagePortId>),
    /// The results of attempting to complete the transfer of a batch of ports.
    MessagePortTransferResult(
        /* The router whose transfer of ports succeeded, if any */
        Option<MessagePortRouterId>,
        /* The ids of ports transferred successfully */
        Vec<MessagePortId>,
        /* The ids, and buffers, of ports whose transfer failed */
        HashMap<MessagePortId, PortTransferInfo>,
    ),
    /// A new message-port was created or transferred, with corresponding control-sender.
    NewMessagePort(MessagePortRouterId, MessagePortId),
    /// A global has started managing message-ports
    NewMessagePortRouter(MessagePortRouterId, IpcSender<MessagePortMsg>),
    /// A global has stopped managing message-ports
    RemoveMessagePortRouter(MessagePortRouterId),
    /// A task requires re-routing to an already shipped message-port.
    RerouteMessagePort(MessagePortId, PortMessageTask),
    /// A message-port was shipped, let the entangled port know.
    MessagePortShipped(MessagePortId),
    /// Entangle two message-ports.
    EntanglePorts(MessagePortId, MessagePortId),
    /// Disentangle two message-ports.
    /// The first is the initiator, the second the other port,
    /// unless the message is sent to complete a disentanglement,
    /// in which case the first one is the other port,
    /// and the second is none.
    DisentanglePorts(MessagePortId, Option<MessagePortId>),
    /// A global has started managing broadcast-channels.
    NewBroadcastChannelRouter(
        BroadcastChannelRouterId,
        IpcSender<BroadcastMsg>,
        ImmutableOrigin,
    ),
    /// A global has stopped managing broadcast-channels.
    RemoveBroadcastChannelRouter(BroadcastChannelRouterId, ImmutableOrigin),
    /// A global started managing broadcast channels for a given channel-name.
    NewBroadcastChannelNameInRouter(BroadcastChannelRouterId, String, ImmutableOrigin),
    /// A global stopped managing broadcast channels for a given channel-name.
    RemoveBroadcastChannelNameInRouter(BroadcastChannelRouterId, String, ImmutableOrigin),
    /// Broadcast a message to all same-origin broadcast channels,
    /// excluding the source of the broadcast.
    ScheduleBroadcast(BroadcastChannelRouterId, BroadcastMsg),
    /// Forward a message to the embedder.
    ForwardToEmbedder(EmbedderMsg),
    /// Broadcast a storage event to every same-origin pipeline.
    /// The strings are key, old value and new value.
    BroadcastStorageEvent(
        StorageType,
        ServoUrl,
        Option<String>,
        Option<String>,
        Option<String>,
    ),
    /// Indicates whether this pipeline is currently running animations.
    ChangeRunningAnimationsState(AnimationState),
    /// Requests that a new 2D canvas thread be created. (This is done in the constellation because
    /// 2D canvases may use the GPU and we don't want to give untrusted content access to the GPU.)
    CreateCanvasPaintThread(
        UntypedSize2D<u64>,
        IpcSender<(IpcSender<CanvasMsg>, CanvasId, ImageKey)>,
    ),
    /// Notifies the constellation that this pipeline is requesting focus.
    ///
    /// When this message is sent, the sender pipeline has already its local
    /// focus state updated. The constellation, after receiving this message,
    /// will broadcast messages to other pipelines that are affected by this
    /// focus operation.
    ///
    /// The first field contains the browsing context ID of the container
    /// element if one was focused.
    ///
    /// The second field is a sequence number that the constellation should use
    /// when sending a focus-related message to the sender pipeline next time.
    Focus(Option<BrowsingContextId>, FocusSequenceNumber),
    /// Requests the constellation to focus the specified browsing context.
    FocusRemoteDocument(BrowsingContextId),
    /// Get the top-level browsing context info for a given browsing context.
    GetTopForBrowsingContext(BrowsingContextId, IpcSender<Option<WebViewId>>),
    /// Get the browsing context id of the browsing context in which pipeline is
    /// embedded and the parent pipeline id of that browsing context.
    GetBrowsingContextInfo(
        PipelineId,
        IpcSender<Option<(BrowsingContextId, Option<PipelineId>)>>,
    ),
    /// Get the nth child browsing context ID for a given browsing context, sorted in tree order.
    GetChildBrowsingContextId(
        BrowsingContextId,
        usize,
        IpcSender<Option<BrowsingContextId>>,
    ),
    /// All pending loads are complete, and the `load` event for this pipeline
    /// has been dispatched.
    LoadComplete,
    /// A new load has been requested, with an option to replace the current entry once loaded
    /// instead of adding a new entry.
    LoadUrl(LoadData, NavigationHistoryBehavior),
    /// Abort loading after sending a LoadUrl message.
    AbortLoadUrl,
    /// Post a message to the currently active window of a given browsing context.
    PostMessage {
        /// The target of the posted message.
        target: BrowsingContextId,
        /// The source of the posted message.
        source: PipelineId,
        /// The expected origin of the target.
        target_origin: Option<ImmutableOrigin>,
        /// The source origin of the message.
        /// <https://html.spec.whatwg.org/multipage/#dom-messageevent-origin>
        source_origin: ImmutableOrigin,
        /// The data to be posted.
        data: StructuredSerializedData,
    },
    /// Inform the constellation that a fragment was navigated to and whether or not it was a replacement navigation.
    NavigatedToFragment(ServoUrl, NavigationHistoryBehavior),
    /// HTMLIFrameElement Forward or Back traversal.
    TraverseHistory(TraversalDirection),
    /// Inform the constellation of a pushed history state.
    PushHistoryState(HistoryStateId, ServoUrl),
    /// Inform the constellation of a replaced history state.
    ReplaceHistoryState(HistoryStateId, ServoUrl),
    /// Gets the length of the joint session history from the constellation.
    JointSessionHistoryLength(IpcSender<u32>),
    /// Notification that this iframe should be removed.
    /// Returns a list of pipelines which were closed.
    RemoveIFrame(BrowsingContextId, IpcSender<Vec<PipelineId>>),
    /// Successful response to [crate::ConstellationControlMsg::SetThrottled].
    SetThrottledComplete(bool),
    /// A load has been requested in an IFrame.
    ScriptLoadedURLInIFrame(IFrameLoadInfoWithData),
    /// A load of the initial `about:blank` has been completed in an IFrame.
    ScriptNewIFrame(IFrameLoadInfoWithData),
    /// Script has opened a new auxiliary browsing context.
    CreateAuxiliaryWebView(AuxiliaryWebViewCreationRequest),
    /// Mark a new document as active
    ActivateDocument,
    /// Set the document state for a pipeline (used by screenshot / reftests)
    SetDocumentState(DocumentState),
    /// Update the layout epoch in the constellation (used by screenshot / reftests).
    SetLayoutEpoch(Epoch, IpcSender<bool>),
    /// Update the pipeline Url, which can change after redirections.
    SetFinalUrl(ServoUrl),
    /// Script has handled a touch event, and either prevented or allowed default actions.
    TouchEventProcessed(TouchEventResult),
    /// A log entry, with the top-level browsing context id and thread name
    LogEntry(Option<String>, LogEntry),
    /// Discard the document.
    DiscardDocument,
    /// Discard the browsing context.
    DiscardTopLevelBrowsingContext,
    /// Notifies the constellation that this pipeline has exited.
    PipelineExited,
    /// Send messages from postMessage calls from serviceworker
    /// to constellation for storing in service worker manager
    ForwardDOMMessage(DOMMessage, ServoUrl),
    /// <https://w3c.github.io/ServiceWorker/#schedule-job-algorithm>
    ScheduleJob(Job),
    /// Notifies the constellation about media session events
    /// (i.e. when there is metadata for the active media session, playback state changes...).
    MediaSessionEvent(PipelineId, MediaSessionEvent),
    #[cfg(feature = "webgpu")]
    /// Create a WebGPU Adapter instance
    RequestAdapter(
        IpcSender<WebGPUAdapterResponse>,
        wgpu_core::instance::RequestAdapterOptions,
        wgpu_core::id::AdapterId,
    ),
    #[cfg(feature = "webgpu")]
    /// Get WebGPU channel
    GetWebGPUChan(IpcSender<Option<WebGPU>>),
    /// Notify the constellation of a pipeline's document's title.
    TitleChanged(PipelineId, String),
    /// Notify the constellation that the size of some `<iframe>`s has changed.
    IFrameSizes(Vec<IFrameSizeMsg>),
    /// Request results from the memory reporter.
    ReportMemory(IpcSender<MemoryReportResult>),
    /// Return the result of the evaluated JavaScript with the given [`JavaScriptEvaluationId`].
    FinishJavaScriptEvaluation(
        JavaScriptEvaluationId,
        Result<JSValue, JavaScriptEvaluationError>,
    ),
    /// Notify the completion of a webdriver command.
    WebDriverInputComplete(WebDriverMessageId),
}

impl fmt::Debug for ScriptToConstellationMessage {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let variant_string: &'static str = self.into();
        write!(formatter, "ScriptMsg::{variant_string}")
    }
}
