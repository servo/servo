/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! The high-level interface from script to constellation. Using this abstract interface helps
//! reduce coupling between these two components.

#![allow(clippy::new_without_default)]

use std::cell::Cell;
use std::num::NonZeroU32;
use std::sync::Arc;
use std::time::Duration;
use std::{fmt, mem};

use ipc_channel::ipc::{self, IpcReceiver, IpcSender};
use lazy_static::lazy_static;
use malloc_size_of::malloc_size_of_is_0;
use malloc_size_of_derive::MallocSizeOf;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use size_of_test::size_of_test;
use webrender_api::{ExternalScrollId, PipelineId as WebRenderPipelineId};

macro_rules! namespace_id_method {
    ($func_name:ident, $func_return_data_type:ident, $self:ident, $index_name:ident) => {
        fn $func_name(&mut $self) -> $func_return_data_type {
            $func_return_data_type {
                namespace_id: $self.id,
                index: $index_name($self.next_index()),
            }
        }
    };
}

macro_rules! namespace_id {
    ($id_name:ident, $index_name:ident, $display_prefix:literal) => {
        #[derive(
            Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize,
        )]
        pub struct $index_name(pub NonZeroU32);
        malloc_size_of_is_0!($index_name);

        #[derive(
            Clone, Copy, Deserialize, Eq, Hash, MallocSizeOf, Ord, PartialEq, PartialOrd, Serialize,
        )]
        pub struct $id_name {
            pub namespace_id: PipelineNamespaceId,
            pub index: $index_name,
        }

        impl fmt::Debug for $id_name {
            fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
                let PipelineNamespaceId(namespace_id) = self.namespace_id;
                let $index_name(index) = self.index;
                write!(fmt, "({},{})", namespace_id, index.get())
            }
        }

        impl fmt::Display for $id_name {
            fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
                write!(fmt, "{}{:?}", $display_prefix, self)
            }
        }
    };
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum TraversalDirection {
    Forward(usize),
    Back(usize),
}

#[derive(Debug, Deserialize, Serialize)]
/// Request a pipeline-namespace id from the constellation.
pub struct PipelineNamespaceRequest(pub IpcSender<PipelineNamespaceId>);

/// A per-process installer of pipeline-namespaces.
pub struct PipelineNamespaceInstaller {
    request_sender: Option<IpcSender<PipelineNamespaceRequest>>,
    namespace_sender: IpcSender<PipelineNamespaceId>,
    namespace_receiver: IpcReceiver<PipelineNamespaceId>,
}

impl Default for PipelineNamespaceInstaller {
    fn default() -> Self {
        let (namespace_sender, namespace_receiver) =
            ipc::channel().expect("PipelineNamespaceInstaller ipc channel failure");
        Self {
            request_sender: None,
            namespace_sender,
            namespace_receiver,
        }
    }
}

impl PipelineNamespaceInstaller {
    /// Provide a request sender to send requests to the constellation.
    pub fn set_sender(&mut self, sender: IpcSender<PipelineNamespaceRequest>) {
        self.request_sender = Some(sender);
    }

    /// Install a namespace, requesting a new Id from the constellation.
    pub fn install_namespace(&self) {
        match self.request_sender.as_ref() {
            Some(sender) => {
                let _ = sender.send(PipelineNamespaceRequest(self.namespace_sender.clone()));
                let namespace_id = self
                    .namespace_receiver
                    .recv()
                    .expect("The constellation to make a pipeline namespace id available");
                PipelineNamespace::install(namespace_id);
            },
            None => unreachable!("PipelineNamespaceInstaller should have a request_sender setup"),
        }
    }
}

lazy_static! {
    /// A per-process unique pipeline-namespace-installer.
    /// Accessible via PipelineNamespace.
    ///
    /// Use PipelineNamespace::set_installer_sender to initiate with a sender to the constellation,
    /// when a new process has been created.
    ///
    /// Use PipelineNamespace::fetch_install to install a unique pipeline-namespace from the calling thread.
    static ref PIPELINE_NAMESPACE_INSTALLER: Arc<Mutex<PipelineNamespaceInstaller>> =
        Arc::new(Mutex::new(PipelineNamespaceInstaller::default()));
}

/// Each pipeline ID needs to be unique. However, it also needs to be possible to
/// generate the pipeline ID from an iframe element (this simplifies a lot of other
/// code that makes use of pipeline IDs).
///
/// To achieve this, each pipeline index belongs to a particular namespace. There is
/// a namespace for the constellation thread, and also one for every script thread.
///
/// A namespace can be installed for any other thread in a process
/// where an pipeline-installer has been initialized.
///
/// This allows pipeline IDs to be generated by any of those threads without conflicting
/// with pipeline IDs created by other script threads or the constellation. The
/// constellation is the only code that is responsible for creating new *namespaces*.
/// This ensures that namespaces are always unique, even when using multi-process mode.
///
/// It may help conceptually to think of the namespace ID as an identifier for the
/// thread that created this pipeline ID - however this is really an implementation
/// detail so shouldn't be relied upon in code logic. It's best to think of the
/// pipeline ID as a simple unique identifier that doesn't convey any more information.
#[derive(Clone, Copy)]
pub struct PipelineNamespace {
    id: PipelineNamespaceId,
    index: u32,
}

impl PipelineNamespace {
    /// Install a namespace for a given Id.
    pub fn install(namespace_id: PipelineNamespaceId) {
        PIPELINE_NAMESPACE.with(|tls| {
            assert!(tls.get().is_none());
            tls.set(Some(PipelineNamespace {
                id: namespace_id,
                index: 0,
            }));
        });
    }

    /// Setup the pipeline-namespace-installer, by providing it with a sender to the constellation.
    /// Idempotent in single-process mode.
    pub fn set_installer_sender(sender: IpcSender<PipelineNamespaceRequest>) {
        PIPELINE_NAMESPACE_INSTALLER.lock().set_sender(sender);
    }

    /// Install a namespace in the current thread, without requiring having a namespace Id ready.
    /// Panics if called more than once per thread.
    pub fn auto_install() {
        // Note that holding the lock for the duration of the call is irrelevant to performance,
        // since a thread would have to block on the ipc-response from the constellation,
        // with the constellation already acting as a global lock on namespace ids,
        // and only being able to handle one request at a time.
        //
        // Hence, any other thread attempting to concurrently install a namespace
        // would have to wait for the current call to finish, regardless of the lock held here.
        PIPELINE_NAMESPACE_INSTALLER.lock().install_namespace();
    }

    fn next_index(&mut self) -> NonZeroU32 {
        self.index += 1;
        NonZeroU32::new(self.index).expect("pipeline id index wrapped!")
    }

    namespace_id_method! {next_pipeline_id, PipelineId, self, PipelineIndex}
    namespace_id_method! {next_browsing_context_id, BrowsingContextId, self, BrowsingContextIndex}
    namespace_id_method! {next_history_state_id, HistoryStateId, self, HistoryStateIndex}
    namespace_id_method! {next_message_port_id, MessagePortId, self, MessagePortIndex}
    namespace_id_method! {next_message_port_router_id, MessagePortRouterId, self, MessagePortRouterIndex}
    namespace_id_method! {next_broadcast_channel_router_id, BroadcastChannelRouterId, self, BroadcastChannelRouterIndex}
    namespace_id_method! {next_service_worker_id, ServiceWorkerId, self, ServiceWorkerIndex}
    namespace_id_method! {next_service_worker_registration_id, ServiceWorkerRegistrationId,
    self, ServiceWorkerRegistrationIndex}
    namespace_id_method! {next_blob_id, BlobId, self, BlobIndex}
}

thread_local!(pub static PIPELINE_NAMESPACE: Cell<Option<PipelineNamespace>> = Cell::new(None));

#[derive(
    Clone, Copy, Debug, Deserialize, Eq, Hash, MallocSizeOf, Ord, PartialEq, PartialOrd, Serialize,
)]
pub struct PipelineNamespaceId(pub u32);

namespace_id! {PipelineId, PipelineIndex, "Pipeline"}

size_of_test!(PipelineId, 8);
size_of_test!(Option<PipelineId>, 8);

impl PipelineId {
    pub fn new() -> PipelineId {
        PIPELINE_NAMESPACE.with(|tls| {
            let mut namespace = tls.get().expect("No namespace set for this thread!");
            let new_pipeline_id = namespace.next_pipeline_id();
            tls.set(Some(namespace));
            new_pipeline_id
        })
    }

    pub fn to_webrender(&self) -> WebRenderPipelineId {
        let PipelineNamespaceId(namespace_id) = self.namespace_id;
        let PipelineIndex(index) = self.index;
        WebRenderPipelineId(namespace_id, index.get())
    }

    #[allow(unsafe_code)]
    pub fn from_webrender(pipeline: WebRenderPipelineId) -> PipelineId {
        let WebRenderPipelineId(namespace_id, index) = pipeline;
        unsafe {
            PipelineId {
                namespace_id: PipelineNamespaceId(namespace_id),
                index: PipelineIndex(NonZeroU32::new_unchecked(index)),
            }
        }
    }

    pub fn root_scroll_id(&self) -> webrender_api::ExternalScrollId {
        ExternalScrollId(0, self.to_webrender())
    }
}

namespace_id! {BrowsingContextId, BrowsingContextIndex, "BrowsingContext"}

size_of_test!(BrowsingContextId, 8);
size_of_test!(Option<BrowsingContextId>, 8);

impl BrowsingContextId {
    pub fn new() -> Self {
        PIPELINE_NAMESPACE.with(|tls| {
            let mut namespace = tls.get().expect("No namespace set for this thread!");
            let new_browsing_context_id = namespace.next_browsing_context_id();
            tls.set(Some(namespace));
            new_browsing_context_id
        })
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct BrowsingContextGroupId(pub u32);
impl fmt::Display for BrowsingContextGroupId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BrowsingContextGroup{:?}", self)
    }
}

thread_local!(pub static TOP_LEVEL_BROWSING_CONTEXT_ID: Cell<Option<TopLevelBrowsingContextId>> = Cell::new(None));

#[derive(
    Clone, Copy, Deserialize, Eq, Hash, MallocSizeOf, Ord, PartialEq, PartialOrd, Serialize,
)]
pub struct TopLevelBrowsingContextId(pub BrowsingContextId);
pub type WebViewId = TopLevelBrowsingContextId;

size_of_test!(TopLevelBrowsingContextId, 8);
size_of_test!(Option<TopLevelBrowsingContextId>, 8);

impl fmt::Debug for TopLevelBrowsingContextId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "TopLevel{:?}", self.0)
    }
}

impl fmt::Display for TopLevelBrowsingContextId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "TopLevel{}", self.0)
    }
}

impl TopLevelBrowsingContextId {
    pub fn new() -> TopLevelBrowsingContextId {
        TopLevelBrowsingContextId(BrowsingContextId::new())
    }

    /// Each script and layout thread should have the top-level browsing context id installed,
    /// since it is used by crash reporting.
    pub fn install(id: TopLevelBrowsingContextId) {
        TOP_LEVEL_BROWSING_CONTEXT_ID.with(|tls| tls.set(Some(id)))
    }

    pub fn installed() -> Option<TopLevelBrowsingContextId> {
        TOP_LEVEL_BROWSING_CONTEXT_ID.with(|tls| tls.get())
    }
}

impl From<TopLevelBrowsingContextId> for BrowsingContextId {
    fn from(id: TopLevelBrowsingContextId) -> BrowsingContextId {
        id.0
    }
}

impl PartialEq<TopLevelBrowsingContextId> for BrowsingContextId {
    fn eq(&self, rhs: &TopLevelBrowsingContextId) -> bool {
        self.eq(&rhs.0)
    }
}

impl PartialEq<BrowsingContextId> for TopLevelBrowsingContextId {
    fn eq(&self, rhs: &BrowsingContextId) -> bool {
        self.0.eq(rhs)
    }
}

namespace_id! {MessagePortId, MessagePortIndex, "MessagePort"}

impl MessagePortId {
    pub fn new() -> MessagePortId {
        PIPELINE_NAMESPACE.with(|tls| {
            let mut namespace = tls.get().expect("No namespace set for this thread!");
            let next_message_port_id = namespace.next_message_port_id();
            tls.set(Some(namespace));
            next_message_port_id
        })
    }
}

namespace_id! {MessagePortRouterId, MessagePortRouterIndex, "MessagePortRouter"}

impl MessagePortRouterId {
    pub fn new() -> MessagePortRouterId {
        PIPELINE_NAMESPACE.with(|tls| {
            let mut namespace = tls.get().expect("No namespace set for this thread!");
            let next_message_port_router_id = namespace.next_message_port_router_id();
            tls.set(Some(namespace));
            next_message_port_router_id
        })
    }
}

namespace_id! {BroadcastChannelRouterId, BroadcastChannelRouterIndex, "BroadcastChannelRouter"}

impl BroadcastChannelRouterId {
    pub fn new() -> BroadcastChannelRouterId {
        PIPELINE_NAMESPACE.with(|tls| {
            let mut namespace = tls.get().expect("No namespace set for this thread!");
            let next_broadcast_channel_router_id = namespace.next_broadcast_channel_router_id();
            tls.set(Some(namespace));
            next_broadcast_channel_router_id
        })
    }
}

namespace_id! {ServiceWorkerId, ServiceWorkerIndex, "ServiceWorker"}

impl ServiceWorkerId {
    pub fn new() -> ServiceWorkerId {
        PIPELINE_NAMESPACE.with(|tls| {
            let mut namespace = tls.get().expect("No namespace set for this thread!");
            let next_service_worker_id = namespace.next_service_worker_id();
            tls.set(Some(namespace));
            next_service_worker_id
        })
    }
}

namespace_id! {ServiceWorkerRegistrationId, ServiceWorkerRegistrationIndex, "ServiceWorkerRegistration"}

impl ServiceWorkerRegistrationId {
    pub fn new() -> ServiceWorkerRegistrationId {
        PIPELINE_NAMESPACE.with(|tls| {
            let mut namespace = tls.get().expect("No namespace set for this thread!");
            let next_service_worker_registration_id =
                namespace.next_service_worker_registration_id();
            tls.set(Some(namespace));
            next_service_worker_registration_id
        })
    }
}

namespace_id! {BlobId, BlobIndex, "Blob"}

impl BlobId {
    pub fn new() -> BlobId {
        PIPELINE_NAMESPACE.with(|tls| {
            let mut namespace = tls.get().expect("No namespace set for this thread!");
            let next_blob_id = namespace.next_blob_id();
            tls.set(Some(namespace));
            next_blob_id
        })
    }
}

namespace_id! {HistoryStateId, HistoryStateIndex, "HistoryState"}

impl HistoryStateId {
    pub fn new() -> HistoryStateId {
        PIPELINE_NAMESPACE.with(|tls| {
            let mut namespace = tls.get().expect("No namespace set for this thread!");
            let next_history_state_id = namespace.next_history_state_id();
            tls.set(Some(namespace));
            next_history_state_id
        })
    }
}

// We provide ids just for unit testing.
pub const TEST_NAMESPACE: PipelineNamespaceId = PipelineNamespaceId(1234);
#[allow(unsafe_code)]
pub const TEST_PIPELINE_INDEX: PipelineIndex =
    unsafe { PipelineIndex(NonZeroU32::new_unchecked(5678)) };
pub const TEST_PIPELINE_ID: PipelineId = PipelineId {
    namespace_id: TEST_NAMESPACE,
    index: TEST_PIPELINE_INDEX,
};
#[allow(unsafe_code)]
pub const TEST_BROWSING_CONTEXT_INDEX: BrowsingContextIndex =
    unsafe { BrowsingContextIndex(NonZeroU32::new_unchecked(8765)) };
pub const TEST_BROWSING_CONTEXT_ID: BrowsingContextId = BrowsingContextId {
    namespace_id: TEST_NAMESPACE,
    index: TEST_BROWSING_CONTEXT_INDEX,
};

// Used to specify the kind of input method editor appropriate to edit a field.
// This is a subset of htmlinputelement::InputType because some variants of InputType
// don't make sense in this context.
#[derive(Debug, Deserialize, Serialize)]
pub enum InputMethodType {
    Color,
    Date,
    DatetimeLocal,
    Email,
    Month,
    Number,
    Password,
    Search,
    Tel,
    Text,
    Time,
    Url,
    Week,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
/// The equivalent of script_layout_interface::message::Msg
pub enum LayoutHangAnnotation {
    AddStylesheet,
    RemoveStylesheet,
    SetQuirksMode,
    Reflow,
    CollectReports,
    ExitNow,
    GetCurrentEpoch,
    GetWebFontLoadState,
    CreateLayoutThread,
    SetFinalUrl,
    SetScrollStates,
    UpdateScrollStateFromScript,
    RegisterPaint,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
/// The equivalent of script::script_runtime::ScriptEventCategory
pub enum ScriptHangAnnotation {
    AttachLayout,
    ConstellationMsg,
    DevtoolsMsg,
    DocumentEvent,
    DomEvent,
    FileRead,
    FormPlannedNavigation,
    ImageCacheMsg,
    InputEvent,
    HistoryEvent,
    NetworkEvent,
    Resize,
    ScriptEvent,
    SetScrollState,
    SetViewport,
    StylesheetLoad,
    TimerEvent,
    UpdateReplacedElement,
    WebSocketEvent,
    WorkerEvent,
    WorkletEvent,
    ServiceWorkerEvent,
    EnterFullscreen,
    ExitFullscreen,
    WebVREvent,
    PerformanceTimelineTask,
    PortMessage,
    WebGPUMsg,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum HangAnnotation {
    Layout(LayoutHangAnnotation),
    Script(ScriptHangAnnotation),
}

/// Hang-alerts are sent by the monitor to the constellation.
#[derive(Deserialize, Serialize)]
pub enum HangMonitorAlert {
    /// A component hang has been detected.
    Hang(HangAlert),
    /// Report a completed sampled profile.
    Profile(Vec<u8>),
}

impl fmt::Debug for HangMonitorAlert {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            HangMonitorAlert::Hang(..) => write!(fmt, "Hang"),
            HangMonitorAlert::Profile(..) => write!(fmt, "Profile"),
        }
    }
}

/// Hang-alerts are sent by the monitor to the constellation.
#[derive(Deserialize, Serialize)]
pub enum HangAlert {
    /// Report a transient hang.
    Transient(MonitoredComponentId, HangAnnotation),
    /// Report a permanent hang.
    Permanent(MonitoredComponentId, HangAnnotation, Option<HangProfile>),
}

impl fmt::Debug for HangAlert {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let (annotation, profile) = match self {
            HangAlert::Transient(component_id, annotation) => {
                write!(
                    fmt,
                    "\n The following component is experiencing a transient hang: \n {:?}",
                    component_id
                )?;
                (*annotation, None)
            },
            HangAlert::Permanent(component_id, annotation, profile) => {
                write!(
                    fmt,
                    "\n The following component is experiencing a permanent hang: \n {:?}",
                    component_id
                )?;
                (*annotation, profile.clone())
            },
        };

        write!(fmt, "\n Annotation for the hang:\n{:?}", annotation)?;
        if let Some(profile) = profile {
            write!(fmt, "\n {:?}", profile)?;
        }

        Ok(())
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct HangProfileSymbol {
    pub name: Option<String>,
    pub filename: Option<String>,
    pub lineno: Option<u32>,
}

#[derive(Clone, Deserialize, Serialize)]
/// Info related to the activity of an hanging component.
pub struct HangProfile {
    pub backtrace: Vec<HangProfileSymbol>,
}

impl fmt::Debug for HangProfile {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let hex_width = mem::size_of::<usize>() * 2 + 2;

        write!(fmt, "HangProfile backtrace:")?;

        if self.backtrace.is_empty() {
            write!(fmt, "backtrace failed to resolve")?;
            return Ok(());
        }

        for symbol in self.backtrace.iter() {
            write!(fmt, "\n      {:1$}", "", hex_width)?;

            if let Some(ref name) = symbol.name {
                write!(fmt, " - {}", name)?;
            } else {
                write!(fmt, " - <unknown>")?;
            }

            if let (Some(ref file), Some(ref line)) = (symbol.filename.as_ref(), symbol.lineno) {
                write!(fmt, "\n      {:3$}at {}:{}", "", file, line, hex_width)?;
            }
        }

        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum MonitoredComponentType {
    Script,
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct MonitoredComponentId(pub PipelineId, pub MonitoredComponentType);

/// A handle to register components for hang monitoring,
/// and to receive a means to communicate with the underlying hang monitor worker.
pub trait BackgroundHangMonitorRegister: BackgroundHangMonitorClone + Send {
    /// Register a component for hang monitoring:
    /// to be called from within the thread to be monitored for hangs.
    fn register_component(
        &self,
        component: MonitoredComponentId,
        transient_hang_timeout: Duration,
        permanent_hang_timeout: Duration,
        exit_signal: Option<Box<dyn BackgroundHangMonitorExitSignal>>,
    ) -> Box<dyn BackgroundHangMonitor>;
}

impl Clone for Box<dyn BackgroundHangMonitorRegister> {
    fn clone(&self) -> Box<dyn BackgroundHangMonitorRegister> {
        self.clone_box()
    }
}

pub trait BackgroundHangMonitorClone {
    fn clone_box(&self) -> Box<dyn BackgroundHangMonitorRegister>;
}

/// Proxy methods to communicate with the background hang monitor
pub trait BackgroundHangMonitor {
    /// Notify the start of handling an event.
    fn notify_activity(&self, annotation: HangAnnotation);
    /// Notify the start of waiting for a new event to come in.
    fn notify_wait(&self);
    /// Unregister the component from monitor.
    fn unregister(&self);
}

/// A means for the BHM to signal a monitored component to exit.
/// Useful when the component is hanging, and cannot be notified via the usual way.
/// The component should implement this in a way allowing for the signal to be received when hanging,
/// if at all.
pub trait BackgroundHangMonitorExitSignal: Send {
    /// Called by the BHM, to notify the monitored component to exit.
    fn signal_to_exit(&self);
}

/// Messages to control the sampling profiler.
#[derive(Deserialize, Serialize)]
pub enum BackgroundHangMonitorControlMsg {
    /// Enable the sampler, with a given sampling rate and max total sampling duration.
    EnableSampler(Duration, Duration),
    DisableSampler,
    /// Exit, and propagate the signal to monitored components.
    Exit(IpcSender<()>),
}
